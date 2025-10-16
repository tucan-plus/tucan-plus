pub mod load_leistungsspiegel;
pub mod load_semesters;

use std::collections::{HashMap, HashSet};

use diesel::Identifiable;
use dioxus::html::FileData;
use dioxus::prelude::*;
use itertools::Itertools;
use log::info;
use tucan_plus_worker::models::{AnmeldungEntry, Semester, State};
use tucan_plus_worker::{
    AnmeldungEntryWithMoveInformation, AnmeldungenEntriesNoSemester, AnmeldungenEntriesPerSemester,
    MyDatabase, RecursiveAnmeldungenRequest, RecursiveAnmeldungenResponse, UpdateAnmeldungEntry,
};
use tucan_types::moduledetails::ModuleDetailsRequest;
use tucan_types::student_result::StudentResultResponse;
use tucan_types::{LoginResponse, RevalidationStrategy, Tucan, TucanError};

use crate::planning::load_leistungsspiegel::load_leistungsspiegel;
use crate::planning::load_semesters::handle_semester;
use crate::{RcTucanType, Route};

#[component]
pub fn Planning(course_of_study: ReadSignal<String>) -> Element {
    let tucan: RcTucanType = use_context();
    let current_session_handle = use_context::<Signal<Option<LoginResponse>>>();

    let student_result: Resource<std::result::Result<StudentResultResponse, TucanError>> =
        use_resource(move || {
            let value = tucan.clone();
            async move {
                value
                    .student_result(
                        &current_session_handle().ok_or(TucanError::LoginRequired)?,
                        RevalidationStrategy::cache(),
                        course_of_study().parse().unwrap_or(0),
                    )
                    .await
            }
        });
    rsx! {
        if let Some(student_result) = student_result.value().with(|value| value.as_ref().map(|inner| inner.as_ref().map_err(|err| err.to_string()).cloned())) {
            match student_result {
                Ok(student_result) => rsx! { PlanningInner {
                    student_result,
                } },
                Err(err) => rsx! {
                    div { class: "container",
                        div {
                            class: "alert alert-danger d-flex align-items-center mt-2",
                            role: "alert",
                            // https://github.com/twbs/icons
                            // The MIT License (MIT)
                            // Copyright (c) 2019-2024 The Bootstrap Authors

                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                class: "bi bi-exclamation-triangle-fill flex-shrink-0 me-2",
                                width: "16",
                                height: "16",
                                view_box: "0 0 16 16",
                                role: "img",
                                "aria-label": "Error:",
                                path { d: "M8.982 1.566a1.13 1.13 0 0 0-1.96 0L.165 13.233c-.457.778.091 1.767.98 1.767h13.713c.889 0 1.438-.99.98-1.767L8.982 1.566zM8 5c.535 0 .954.462.9.995l-.35 3.507a.552.552 0 0 1-1.1 0L7.1 5.995A.905.905 0 0 1 8 5zm.002 6a1 1 0 1 1 0 2 1 1 0 0 1 0-2z" }
                            }
                            div {
                                { err }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub type MyResource = Resource<(
    Option<RecursiveAnmeldungenResponse>,
    Vec<((i32, Semester), Vec<AnmeldungEntryWithMoveInformation>)>,
    Vec<AnmeldungEntryWithMoveInformation>,
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
    let mut sommersemester: Signal<Vec<FileData>> = use_signal(Vec::new);
    let mut wintersemester: Signal<Vec<FileData>> = use_signal(Vec::new);
    let tucan: RcTucanType = use_context();
    let current_session_handle = use_context::<Signal<Option<LoginResponse>>>();
    let mut loading = use_signal(|| false);
    let mut failed: Signal<Vec<AnmeldungEntryWithMoveInformation>> = use_signal(|| Vec::new());
    let mut future: MyResource = {
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
                let no_semester = worker
                    .send_message(AnmeldungenEntriesNoSemester {
                        course_of_study: course_of_study.clone(),
                    })
                    .await;
                let entries: HashMap<_, _> = per_semester
                    .iter()
                    .flat_map(|m| m.1.clone())
                    .chain(no_semester.clone())
                    .map(|m| (m.entry.id, m))
                    .collect();
                failed.with_mut(|failed| {
                    for f in failed {
                        *f = entries[&f.entry.id].clone();
                    }
                });

                (recursive, per_semester, no_semester)
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
                failed.set(
                    load_leistungsspiegel(
                        worker,
                        current_session,
                        tucan,
                        student_result,
                        course_of_study,
                    )
                    .await,
                );

                loading.set(false);
                future.restart();
            }
        }
    };

    let onsubmit = {
        let course_of_study = course_of_study.clone();
        let worker = worker.clone();
        move |evt: Event<FormData>| {
            let course_of_study = course_of_study.clone();
            let worker = worker.clone();
            evt.prevent_default();
            async move {
                loading.set(true);
                handle_semester(
                    &worker,
                    &course_of_study,
                    Semester::Sommersemester,
                    sommersemester,
                )
                .await;
                handle_semester(
                    &worker,
                    &course_of_study,
                    Semester::Wintersemester,
                    wintersemester,
                )
                .await;
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
            if !failed().is_empty() {
                h2 {
                    "Nicht automatisch zuordnenbar"
                }
                AnmeldungenEntries {
                    future,
                    entries: failed()
                }
            }
            if let Some(value) = future.value()() {
                if let Some(value) = value.0 {
                    RegistrationTreeNode {
                        key: "{value:?}",
                        future,
                        value: value
                    }
                }
                for ((i, semester), value) in value.1 {
                    Fragment {
                        key: "{i}{semester}",
                        h2 {
                            "{semester} {i} "
                            span { class: "badge text-bg-secondary", {format!("{} CP", value.iter().filter(|elem| elem.entry.state != State::MaybePlanned).map(|elem| elem.entry.credits).sum::<i32>())} }
                        }
                        AnmeldungenEntries {
                            future,
                            entries: value
                        }
                    }
                }
                Fragment {
                    key: "no-semester",
                    h2 {
                        "Nicht zugeordnet "
                        span { class: "badge text-bg-secondary", {format!("{} CP", value.2.iter().filter(|elem| elem.entry.state != State::MaybePlanned).map(|elem| elem.entry.credits).sum::<i32>())} }
                    }
                    AnmeldungenEntries {
                        future,
                        entries: value.2
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
    entries: ReadSignal<Option<Vec<AnmeldungEntryWithMoveInformation>>>,
) -> Element {
    let worker: MyDatabase = use_context();
    rsx! {
        table {
            class: "table",
            tbody {
                for (key, AnmeldungEntryWithMoveInformation {
                    entry,
                    move_targets
                }) in entries()
                    .iter()
                    .flatten()
                    .map(|entry| (format!("{}{:?}", entry.entry.id, entry.entry.available_semester), entry)) {
                    tr {
                        key: "{key}",
                        td {
                            { entry.id.clone() }
                        }
                        td {
                            if let Some(module_url) = &entry.module_url {
                                Link {
                                    to: Route::ModuleDetails {
                                        module: ModuleDetailsRequest::parse(module_url),
                                    },
                                    { entry.name.clone() }
                                }
                            } else {
                                { entry.name.clone() }
                            }
                        }
                        td {
                            { format!("{:?}", entry.available_semester) }
                            select {
                                class: "form-select",
                                onchange: {
                                    let entry = entry.clone();
                                    let worker = worker.clone();
                                    move |event| {
                                        let mut entry = entry.clone();
                                        let worker = worker.clone();
                                        async move {
                                            let mut new_entry = entry.clone();
                                            new_entry.anmeldung = event.value();
                                            worker.send_message(UpdateAnmeldungEntry { entry, new_entry }).await;
                                            future.restart();
                                        }
                                    }
                                },
                                for move_target in move_targets {
                                    option {
                                        key: "{move_target.1}",
                                        value: "{move_target.1}",
                                        selected: false,
                                        "{move_target.0}"
                                    }
                                }
                            }
                        }
                        td {
                            { entry.credits.to_string() }
                        }
                        td {
                            select {
                                onchange: {
                                    let entry = entry.clone();
                                    let worker = worker.clone();
                                    move |event| {
                                        let mut entry = entry.clone();
                                        let worker = worker.clone();
                                        async move {
                                            let mut new_entry = entry.clone();
                                            new_entry.state = serde_json::from_str(&event.value()).unwrap();
                                            worker.send_message(UpdateAnmeldungEntry { entry, new_entry }).await;
                                            future.restart();
                                        }
                                    }
                                },
                                class: match entry.state {
                                    State::NotPlanned => "form-select bg-secondary",
                                    State::MaybePlanned => "form-select bg-info",
                                    State::Planned => "form-select bg-primary",
                                    State::Done => "form-select bg-success",
                                },
                                option {
                                    value: serde_json::to_string(&State::NotPlanned).unwrap(),
                                    selected: entry.state == State::NotPlanned,
                                    { format!("{:?}", State::NotPlanned) }
                                }
                                option {
                                    value: serde_json::to_string(&State::MaybePlanned).unwrap(),
                                    selected: entry.state == State::MaybePlanned,
                                    { format!("{:?}", State::MaybePlanned) }
                                }
                                option {
                                    value: serde_json::to_string(&State::Planned).unwrap(),
                                    selected: entry.state == State::Planned,
                                    { format!("{:?}", State::Planned) }
                                }
                                option {
                                    value: serde_json::to_string(&State::Done).unwrap(),
                                    selected: entry.state == State::Done,
                                    { format!("{:?}", State::Done) }
                                }
                            }
                            select {
                                class: "form-select",
                                style: "min-width: 15em",
                                onchange: {
                                    let entry = entry.clone();
                                    let worker = worker.clone();
                                    move |event| {
                                        let mut entry = entry.clone();
                                        let worker = worker.clone();
                                        async move {
                                            let mut new_entry = entry.clone();
                                            let (year, semester) = serde_json::from_str(&event.value()).unwrap();
                                            new_entry.year = year;
                                            new_entry.semester = semester;
                                            worker.send_message(UpdateAnmeldungEntry { entry, new_entry }).await;
                                            future.restart();
                                        }
                                    }
                                },
                                option {
                                    key: "none",
                                    value: serde_json::to_string(&(None::<i32>, None::<Semester>)).unwrap(),
                                    selected: entry.semester.is_none() && entry.year.is_none(),
                                    "Choose semester"
                                }
                                for i in 2020..2030 {
                                    option {
                                        key: "sose{i}",
                                        value: serde_json::to_string(&(Some(i), Semester::Sommersemester)).unwrap(),
                                        selected: entry.semester == Some(Semester::Sommersemester)
                                            && entry.year == Some(i),
                                        "Sommersemester {i}"
                                    }
                                    option {
                                        key: "wise{i}",
                                        value: serde_json::to_string(&(Some(i), Semester::Wintersemester)).unwrap(),
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
    let actual_credits = value.actual_credits;
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
                || entries.iter().any(|entry| entry.entry.state != State::NotPlanned) {
                AnmeldungenEntries {
                    future,
                    entries: Some(entries
                        .iter()
                        .filter(|entry| expanded() || entry.entry.state != State::NotPlanned)
                        .cloned()
                        .collect::<Vec<_>>()),
                }
            }
            if expanded() || inner.iter().any(|v| v.has_contents) {
                div {
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
            }
            if value.has_rules {
                p {
                    { "Summe ".to_owned() + &anmeldung.name + ":" }
                    br {
                    }
                    if anmeldung.min_cp != 0 || anmeldung.max_cp.is_some() {
                        span {
                            class: if anmeldung.min_cp <= actual_credits
                                && anmeldung.max_cp.map(|max| actual_credits <= max).unwrap_or(true)
                            {
                                "bg-success"
                            } else {
                                if anmeldung.min_cp <= actual_credits {
                                    "bg-warning"
                                } else {
                                    "bg-danger"
                                }
                            },
                            "CP: "
                            { actual_credits.to_string() }
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
