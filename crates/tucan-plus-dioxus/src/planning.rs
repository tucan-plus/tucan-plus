pub mod load_leistungsspiegel;
pub mod load_semesters;

use std::collections::HashSet;

use dioxus::html::FileData;
use dioxus::prelude::*;
use futures::StreamExt;
use log::info;
use tucan_plus_worker::models::{Anmeldung, AnmeldungEntry, Semester, State};
use tucan_plus_worker::{
    AnmeldungChildrenRequest, AnmeldungEntriesRequest, AnmeldungenEntriesInSemester,
    AnmeldungenEntriesPerSemester, AnmeldungenRootRequest, MyDatabase, RecursiveAnmeldungenRequest,
    RecursiveAnmeldungenResponse, UpdateAnmeldungEntry,
};
use tucan_types::student_result::StudentResultResponse;
use tucan_types::{LoginResponse, RevalidationStrategy, Tucan};

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

pub type MyResource = Resource<(
    RecursiveAnmeldungenResponse,
    Vec<((i32, Semester), Vec<AnmeldungEntry>)>,
)>;

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
                let recursive = worker
                    .send_message(RecursiveAnmeldungenRequest {
                        course_of_study: course_of_study.clone(),
                        expanded: HashSet::new(),
                    })
                    .await;
                let per_semester = worker
                    .send_message(AnmeldungenEntriesPerSemester {
                        course_of_study: course_of_study.clone(),
                    })
                    .await;
                (recursive, per_semester)
            }
        })
    };
    let load_leistungsspiegel = {
        let tucan = tucan.clone();
        let student_result = student_result.clone();
        let course_of_study = course_of_study.clone();
        let worker = worker.clone();
        move |_event: dioxus::prelude::Event<MouseData>| {
            let current_session_handle = current_session_handle;
            let tucan = tucan.clone();
            let student_result = student_result.clone();
            let course_of_study = course_of_study.clone();
            let worker = worker.clone();
            async move {
                loading.set(true);

                let current_session = current_session_handle().unwrap();
                load_leistungsspiegel(
                    worker,
                    current_session,
                    tucan,
                    student_result,
                    course_of_study,
                )
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
        let worker = worker.clone();
        move |evt: Event<FormData>| {
            let tucan = tucan.clone();
            let course_of_study = course_of_study.clone();
            let worker = worker.clone();
            evt.prevent_default();
            async move {
                loading.set(true);
                handle_semester(
                    &worker,
                    &course_of_study,
                    tucan.clone(),
                    &current_session_handle().unwrap(),
                    Semester::Sommersemester,
                    sommersemester,
                )
                .await;
                handle_semester(
                    &worker,
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
                        accept: ".sose-v1-tucan",
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
                        accept: ".wise-v1-tucan",
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
            if let Some(value) = future.value()() {
                RegistrationTreeNode {
                    future,
                    value: value.0
                }
                for ((i, semester), value) in value.1 {
                    Fragment {
                        key: "{i}{semester}",
                        h2 {
                            "{semester} {i}"
                        }
                        AnmeldungenEntries {
                            future,
                            entries: value
                        }
                    }
                }
            }
        }
    }
}

pub struct YearAndSemester(pub u32, pub Semester);

pub enum PlanningState {
    NotPlanned,
    MaybePlanned(Option<YearAndSemester>),
    Planned(Option<YearAndSemester>),
    Done(Option<YearAndSemester>),
}

#[component]
fn AnmeldungenEntries(
    future: MyResource,
    entries: ReadSignal<Option<Vec<AnmeldungEntry>>>,
) -> Element {
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

pub struct PrepPlanningReturn {}

#[component]
fn RegistrationTreeNode(future: MyResource, value: RecursiveAnmeldungenResponse) -> Element {
    let anmeldung = value.anmeldung;
    let entries = value.entries;
    let inner = value.inner;
    let cp = value.credits;
    let modules = value.modules;
    let mut expanded = use_signal(|| false); // TODO FIXME
    rsx! {
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
                    future,
                    entries: Some(entries
                        .iter()
                        .filter(|entry| expanded() || entry.state != State::NotPlanned)
                        .cloned()
                        .collect::<Vec<_>>()),
                }
            }
            if expanded() || inner.iter().any(|v| v.has_contents) {
                for (key, value) in value.results
                    .iter()
                    .zip(inner.into_iter())
                    .filter(|(_, value)| expanded() || value.has_contents)
                    .map(|(key, value)| (&key.url, value)) {
                    div {
                        key: "{key}",
                        RegistrationTreeNode { future, value }
                    }
                }
            }
            if value.has_rules {
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
    }
}
