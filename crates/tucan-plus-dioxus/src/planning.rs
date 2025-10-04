pub mod load_leistungsspiegel;
pub mod load_semesters;

use dioxus::html::FileData;
use dioxus::prelude::*;
use futures::StreamExt;
use log::info;
use tucan_plus_worker::models::{Anmeldung, AnmeldungEntry, Semester, State};
use tucan_plus_worker::{AnmeldungenEntriesInSemester, AnmeldungenRequest, AnmeldungenRequest2, Fewe, MyDatabase, UpdateAnmeldungEntry};
use tucan_types::student_result::StudentResultResponse;
use tucan_types::{
    LoginResponse, RevalidationStrategy, Tucan,
};

use crate::planning::load_leistungsspiegel::load_leistungsspiegel;
use crate::planning::load_semesters::handle_semester;
use crate::{RcTucanType, Route};

#[component]
pub fn Planning(course_of_study: ReadSignal<String>) -> Element {
    let tucan: RcTucanType = use_context();
    let current_session_handle = use_context::<Signal<Option<LoginResponse>>>();
    let student_result = use_resource(move || {
        let value = tucan.clone();
        async move {
            // TODO FIXME don't unwrap here
            
            value
                .student_result(
                    &current_session_handle().unwrap(),
                    RevalidationStrategy::cache(),
                    course_of_study().parse().unwrap_or(0),
                )
                .await
                .unwrap()
        }
    });
    rsx! {
        if let Some(student_result) = student_result() {
            PlanningInner {
                student_result,
            }
        }
    }
}

#[component]
pub fn PlanningInner(student_result: StudentResultResponse) -> Element {
    let worker: MyDatabase = use_context();
    let course_of_study = student_result
        .course_of_study
        .iter()
        .find(|e| e.selected)
        .unwrap()
        .value
        .to_string();
    let navigator = use_navigator();
    let mut sommersemester: Signal<Vec<FileData>> = use_signal(|| Vec::new());
    let mut wintersemester: Signal<Vec<FileData>> = use_signal(|| Vec::new());
    let tucan: RcTucanType = use_context();
    let current_session_handle = use_context::<Signal<Option<LoginResponse>>>();
    let mut loading = use_signal(|| false);
    let mut future = {
        let course_of_study = course_of_study.clone();
        let worker = worker.clone();
        use_resource(move || {
            let course_of_study = course_of_study.clone();
            let worker = worker.clone();
            async move {
                // TODO FIXME I think based on course of study we can create an
                // anmeldung_request and then this here is not special cased any more?
                let result = worker
                    .send_message(AnmeldungenRequest {
                        course_of_study: course_of_study.clone(),
                    })
                    .await;
                futures::stream::iter(result.into_iter())
                    .then(async |anmeldung| {
                        prep_planning(&course_of_study, anmeldung).await.element
                    })
                    .collect::<Vec<Element>>()
                    .await
            }
        })
    };
    let load_leistungsspiegel = {
        let tucan = tucan.clone();
        let student_result = student_result.clone();
        let course_of_study = course_of_study.clone();
        move |_event: dioxus::prelude::Event<MouseData>| {
            let current_session_handle = current_session_handle;
            let tucan = tucan.clone();
            let student_result = student_result.clone();
            let course_of_study = course_of_study.clone();
            async move {
                loading.set(true);

                let current_session = current_session_handle().unwrap();
                load_leistungsspiegel(current_session, tucan, student_result, course_of_study)
                    .await;

                info!("updated");
                loading.set(false);
                future.restart();
            }
        }
    };

    let tucan = tucan.clone();
    let onsubmit = {
        let course_of_study = course_of_study.clone();
        move |evt: Event<FormData>| {
            let tucan = tucan.clone();
            let course_of_study = course_of_study.clone();
            evt.prevent_default();
            async move {
                loading.set(true);
                handle_semester(
                    &course_of_study,
                    tucan.clone(),
                    &current_session_handle().unwrap(),
                    Semester::Sommersemester,
                    sommersemester,
                )
                .await;
                handle_semester(
                    &course_of_study,
                    tucan.clone(),
                    &current_session_handle().unwrap(),
                    Semester::Wintersemester,
                    wintersemester,
                )
                .await;
                info!("done");
                loading.set(false);
                future.restart();
            }
        }
    };

    rsx! {
        div {
            class: "container",
            if loading() {
                div {
                    style: "z-index: 10000",
                    class: "position-fixed top-50 start-50 translate-middle",
                    div {
                        class: "spinner-grow",
                        role: "status",
                        span {
                            class: "visually-hidden",
                            "Loading..."
                        }
                    }
                }
            }
            h2 {
                class: "text-center",
                "Semesterplanung"
            }
            select {
                onchange: move |event: Event<FormData>| {
                    navigator.push(Route::Planning {
                        course_of_study: event.value(),
                    });
                },
                class: "form-select mb-1",
                "aria-label": "Select course of study",
                {
                    student_result
                        .course_of_study
                        .iter()
                        .map(|course_of_study| {
                            let value = course_of_study.value;
                            rsx! {
                                option {
                                    key: "{value}",
                                    selected: course_of_study.selected,
                                    value: course_of_study.value,
                                    { course_of_study.name.clone() }
                                }
                            }
                        })
                }
            }
            form {
                onsubmit: onsubmit,
                class: "mb-3",
                div {
                    class: "mb-3",
                    label {
                        for: "sommersemester-file",
                        class: "form-label",
                        "Sommersemester"
                    }
                    input {
                        type: "file",
                        class: "form-control",
                        id: "sommersemester-file",
                        onchange: move |event| {
                            sommersemester.set(event.files());
                        },
                    }
                }
                div {
                    class: "mb-3",
                    label {
                        for: "wintersemester-file",
                        class: "form-label",
                        "Wintersemester"
                    }
                    input {
                        type: "file",
                        class: "form-control",
                        id: "wintersemester-file",
                        onchange: move |event| {
                            wintersemester.set(event.files());
                        },
                    }
                }
                button {
                    disabled: loading(),
                    type: "submit",
                    class: "btn btn-primary",
                    "Planung starten"
                }
            }
            button {
                onclick: load_leistungsspiegel,
                disabled: loading(),
                type: "button",
                class: "btn btn-primary mb-3",
                "Leistungsspiegel laden (nach Laden der Semester)"
            }
            if let Some(value) = future() {
                for entry in value {
                    { entry }
                }
            }
        }
        for i in 2020..2030 {
            Fragment {
                key: "{i}",
                h2 {
                    "Sommersemester {i}"
                }
                AnmeldungenEntries {
                    future,
                    entries: {
                    let worker = worker.clone();
                    let course_of_study = course_of_study.clone();
                        use_resource(move || {
                    let worker = worker.clone();
                    let course_of_study = course_of_study.clone();
                    async move {worker.send_message(AnmeldungenEntriesInSemester { course_of_study, year: i, semester: Semester::Sommersemester }).await}
                    }).value()
                },
                }
                h2 {
                    "Wintersemester {i}"
                }
                AnmeldungenEntries {
                    future,
                    entries: {
                    let worker = worker.clone();
                    let course_of_study = course_of_study.clone();
                        use_resource(move || {
                    let worker = worker.clone();
                    let course_of_study = course_of_study.clone();
                    async move {worker.send_message(AnmeldungenEntriesInSemester { course_of_study, year: i, semester: Semester::Wintersemester }).await}
                    }).value()
                },
                }
            }
        }
    }
}

pub struct PrepPlanningReturn {
    has_contents: bool,
    credits: i32,
    modules: usize,
    element: Element,
}

pub struct YearAndSemester(pub u32, pub Semester);

pub enum PlanningState {
    NotPlanned,
    MaybePlanned(Option<YearAndSemester>),
    Planned(Option<YearAndSemester>),
    Done(Option<YearAndSemester>),
}

#[component]
fn AnmeldungenEntries(future: Resource<Vec<Element>>, entries: ReadSignal<Option<Vec<AnmeldungEntry>>>) -> Element {
    let worker: MyDatabase = use_context();
    rsx! {
        table {
            class: "table",
            tbody {
                for (key, entry) in entries()
                    .iter()
                    .flatten()
                    .map(|entry| (format!("{}{:?}", entry.id, entry.available_semester), entry)) {
                    tr {
                        key: "{key}",
                        td {
                            { entry.id.clone() }
                        }
                        td {
                            { entry.name.clone() }
                        }
                        td {
                            { format!("{:?}", entry.available_semester) }
                        }
                        td {
                            { entry.credits.to_string() }
                        }
                        td {
                            select {
                                class: match entry.state {
                                    State::NotPlanned => "form-select bg-secondary",
                                    State::Planned => "form-select bg-primary",
                                    State::Done => "form-select bg-success",
                                },
                                option {
                                    onclick: {
                                        let entry = entry.clone();
                                        let worker = worker.clone();
                                        move |event| {
                                            let mut entry = entry.clone();
                                            let worker = worker.clone();
                                            async move {
                                                event.prevent_default();
                                                entry.state = State::NotPlanned;
                                                worker.send_message(UpdateAnmeldungEntry { entry }).await;
                                                future.restart();
                                            }
                                        }
                                    },
                                    selected: entry.state == State::NotPlanned,
                                    { format!("{:?}", State::NotPlanned) }
                                }
                                option {
                                    onclick: {
                                        let entry = entry.clone();
                                        let worker = worker.clone();
                                        move |event| {
                                            let mut entry = entry.clone();
                                            let worker = worker.clone();
                                            async move {
                                                event.prevent_default();
                                                entry.state = State::Planned;
                                                worker.send_message(UpdateAnmeldungEntry { entry }).await;
                                                future.restart();
                                            }
                                        }
                                    },
                                    selected: entry.state == State::Planned,
                                    { format!("{:?}", State::Planned) }
                                }
                                option {
                                    onclick: {
                                        let entry = entry.clone();
                                        let worker = worker.clone();
                                        move |event| {
                                            let mut entry = entry.clone();
                                            let worker = worker.clone();
                                            async move {
                                                event.prevent_default();
                                                entry.state = State::Done;
                                                worker.send_message(UpdateAnmeldungEntry { entry }).await;
                                                future.restart();
                                            }
                                        }
                                    },
                                    selected: entry.state == State::Done,
                                    { format!("{:?}", State::Done) }
                                }
                            }
                            select {
                                class: "form-select",
                                style: "min-width: 15em",
                                option {
                                    key: "",
                                    value: "",
                                    onclick: {
                                        let entry = entry.clone();
                                        let worker = worker.clone();
                                        move |event| {
                                            let mut entry = entry.clone();
                                            let worker = worker.clone();
                                            async move {
                                                event.prevent_default();
                                                entry.semester = None;
                                                entry.year = None;
                                                worker.send_message(UpdateAnmeldungEntry { entry }).await;
                                                future.restart();
                                            }
                                        }
                                    },
                                    selected: entry.semester.is_none() && entry.year.is_none(),
                                    "Choose semester"
                                }
                                for i in 2020..2030 {
                                    option {
                                        key: "sose{i}",
                                        onclick: {
                                            let entry = entry.clone();
                                            let worker = worker.clone();
                                            move |event| {
                                                let mut entry = entry.clone();
                                                let worker = worker.clone();
                                                async move {
                                                    event.prevent_default();
                                                    entry.semester = Some(Semester::Sommersemester);
                                                    entry.year = Some(i);
                                                    worker.send_message(UpdateAnmeldungEntry { entry }).await;
                                                    future.restart();
                                                }
                                            }
                                        },
                                        selected: entry.semester == Some(Semester::Sommersemester)
                                            && entry.year == Some(i),
                                        "Sommersemester {i}"
                                    }
                                    option {
                                        key: "wise{i}",
                                        onclick: {
                                            let entry = entry.clone();
                                            let worker = worker.clone();
                                            move |event| {
                                                let mut entry = entry.clone();
                                                let worker = worker.clone();
                                                async move {
                                                    event.prevent_default();
                                                    entry.semester = Some(Semester::Wintersemester);
                                                    entry.year = Some(i);
                                                    worker.send_message(UpdateAnmeldungEntry { entry }).await;
                                                    future.restart();
                                                }
                                            }
                                        },
                                        selected: entry.semester == Some(Semester::Wintersemester)
                                            && entry.year == Some(i),
                                        "Wintersemester {i}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn prep_planning(
    course_of_study: &str,
    anmeldung: Anmeldung, // ahh this needs to be a signal?
) -> PrepPlanningReturn {
    let worker: MyDatabase = use_context();
    let results = worker
        .send_message(AnmeldungenRequest2 {
            course_of_study: course_of_study.to_owned(),
            anmeldung: anmeldung.clone(),
        })
        .await;
    let entries = worker
        .send_message(Fewe {
            course_of_study: course_of_study.to_owned(),
            anmeldung: anmeldung.clone(),
        })
        .await;
    let inner: Vec<PrepPlanningReturn> = futures::stream::iter(results.iter())
        .then(async |result| Box::pin(prep_planning(course_of_study, result.clone())).await)
        .collect()
        .await;
    let has_rules = anmeldung.min_cp != 0
        || anmeldung.max_cp.is_some()
        || anmeldung.min_modules != 0
        || anmeldung.max_modules.is_some();
    let mut expanded = use_signal(|| false);
    let interesting = expanded()
        || has_rules
        || entries.iter().any(|entry| entry.state != State::NotPlanned)
        || inner.iter().any(|v| v.has_contents);
    let cp: i32 = entries
        .iter()
        .filter(|entry| entry.state == State::Done || entry.state == State::Planned)
        .map(|entry| entry.credits)
        .sum::<i32>()
        + inner.iter().map(|inner| inner.credits).sum::<i32>();
    let used_cp = std::cmp::min(cp, anmeldung.max_cp.unwrap_or(cp));
    let modules: usize = entries
        .iter()
        .filter(|entry| entry.state == State::Done || entry.state == State::Planned)
        .count()
        + inner.iter().map(|inner| inner.modules).sum::<usize>();
    PrepPlanningReturn {
        has_contents: interesting,
        credits: used_cp,
        modules,
        element: rsx! {
            div {
                class: "h3",
                { anmeldung.name.clone() }
                " "
                button {
                    type: "button",
                    class: "btn btn-secondary",
                    onclick: move |_| {
                        expanded.toggle();
                    },
                    { if expanded() { "-" } else { "+" } }
                }
            }
            div {
                class: "ms-2 ps-2",
                style: "border-left: 1px solid #ccc;",
                if (!entries.is_empty() && expanded())
                    || entries.iter().any(|entry| entry.state != State::NotPlanned) {
                    AnmeldungenEntries {
                        future: use_resource(|| async { Vec::new() }),
                        entries: ReadSignal::new(use_signal(|| Some(entries
                            .iter()
                            .filter(|entry| expanded() || entry.state != State::NotPlanned)
                            .cloned()
                            .collect::<Vec<_>>()))),
                    }
                }
                if expanded() || inner.iter().any(|v| v.has_contents) {
                    for (key, value) in results
                        .iter()
                        .zip(inner.into_iter())
                        .filter(|(_, value)| expanded() || value.has_contents)
                        .map(|(key, value)| (&key.url, value)) {
                        div {
                            key: "{key}",
                            { value.element }
                        }
                    }
                }
                if has_rules {
                    p {
                        { "Summe ".to_owned() + &anmeldung.name + ":" }
                        br {
                        }
                        if anmeldung.min_cp != 0 || anmeldung.max_cp.is_some() {
                            span {
                                class: if anmeldung.min_cp <= cp
                                    && anmeldung.max_cp.map(|max| cp <= max).unwrap_or(true)
                                {
                                    "bg-success"
                                } else {
                                    if anmeldung.min_cp <= cp {
                                        "bg-warning"
                                    } else {
                                        "bg-danger"
                                    }
                                },
                                "CP: "
                                { cp.to_string() }
                                " / "
                                { anmeldung.min_cp.to_string() }
                                " - "
                                {
                                    anmeldung
                                        .max_cp
                                        .map(|v| v.to_string())
                                        .unwrap_or("*".to_string())
                                }
                            }
                        }
                        if (anmeldung.min_cp != 0 || anmeldung.max_cp.is_some())
                            && (anmeldung.min_modules != 0 || anmeldung.max_modules.is_some()) {
                            br {
                            }
                        }
                        if anmeldung.min_modules != 0 || anmeldung.max_modules.is_some() {
                            span {
                                class: if anmeldung.min_modules <= modules.try_into().unwrap()
                                    && anmeldung
                                        .max_modules
                                        .map(|max| modules <= max.try_into().unwrap())
                                        .unwrap_or(true)
                                {
                                    "bg-success"
                                } else {
                                    "bg-danger"
                                },
                                "Module: "
                                { modules.to_string() }
                                " / "
                                { anmeldung.min_modules.to_string() }
                                {
                                    anmeldung.max_modules.map(|max_modules| {
                                        " - ".to_string() + &max_modules.to_string()
                                    })
                                }
                            }
                        }
                    }
                }
            }
        },
    }
}
