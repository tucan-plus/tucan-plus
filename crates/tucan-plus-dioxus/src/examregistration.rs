use std::str::FromStr;

use dioxus::prelude::*;
use tucan_types::{
    SemesterId, Tucan,
    examregistration::{ExamRegistrationResponse, ExamRegistrationState},
    myexams::MyExamsResponse,
};

use crate::{RcTucanType, Route, common::use_authenticated_data_loader};

#[component]
pub fn ExamRegistration(semester: ReadSignal<SemesterId>) -> Element {
    let handler = async |tucan: RcTucanType, current_session, revalidation_strategy, additional| {
        tucan
            .exam_registration(&current_session, revalidation_strategy, additional)
            .await
    };

    let navigator = use_navigator();

    use_authenticated_data_loader(
        handler,
        semester,
        60 * 60,
        60,
        |exams: ExamRegistrationResponse, reload| {
            let on_semester_change = {
                Callback::new(move |e: Event<FormData>| {
                    let value = e.value();
                    navigator.push(Route::MyExams {
                        semester: SemesterId::from_str(&value).unwrap(),
                    });
                })
            };
            rsx! {
                div {
                    h1 {
                        {"Prüfungsanmeldung"}
                        {" "}
                        button {
                            onclick: reload,
                            r#type: "button",
                            class: "btn btn-secondary",
                            // https://github.com/twbs/icons
                            // The MIT License (MIT)
                            // Copyright (c) 2019-2024 The Bootstrap Authors

                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "16",
                                height: "16",
                                fill: "currentColor",
                                class: "bi bi-arrow-clockwise",
                                view_box: "0 0 16 16",
                                path {
                                    "fill-rule": "evenodd",
                                    d: "M8 3a5 5 0 1 0 4.546 2.914.5.5 0 0 1 .908-.417A6 6 0 1 1 8 2z",
                                }
                                path { d: "M8 4.466V.534a.25.25 0 0 1 .41-.192l2.36 1.966c.12.1.12.284 0 .384L8.41 4.658A.25.25 0 0 1 8 4.466" }
                            }
                        }
                    }
                    select {
                        onchange: on_semester_change,
                        class: "form-select mb-1",
                        "aria-label": "Select semester",
                        {
                            exams
                                .semester
                                .iter()
                                .map(|semester| {
                                    rsx! {
                                        option { selected: semester.selected, value: semester.value.inner().clone(), {semester.name.clone()} }
                                    }
                                })
                        }
                    }
                    {
                        exams
                            .exam_registrations
                            .iter()
                            .map(|exam| {
                                rsx! {
                                    h4 {
                                        { exam.course_id.clone() } " " { exam.name.clone() } " " { exam.ids.clone() }
                                    }
                                    div { class: "table-responsive",
                                        table { class: "table",
                                            thead {
                                                tr {
                                                    th { scope: "col", "Datum" }
                                                    th { scope: "col", "Prüfungsart" }
                                                    th { scope: "col", "Details" }
                                                    th { scope: "col", "Status" }
                                                }
                                            }
                                            tbody {
                                                {
                                                    exam.registrations.iter().map(|reg| {
                                                        rsx! {
                                                            tr {
                                                                td {
                                                                    { reg.date.clone() }
                                                                }
                                                                td {
                                                                    { reg.pruefungsart.clone() }
                                                                }
                                                                td {
                                                                    a {
                                                                        href: reg.examdetail_url.clone(),
                                                                        "Details"
                                                                    }
                                                                }
                                                                td {
                                                                    match &reg.registration_state {
                                                                        ExamRegistrationState::NotPossible => rsx! { "Nicht anmeldbar" },
                                                                        ExamRegistrationState::ForceSelected => rsx! { "Du musst" },
                                                                        ExamRegistrationState::Registered(href) => rsx! {
                                                                            a {
                                                                                href: href.clone(),
                                                                                "Abmelden"
                                                                            }
                                                                        },
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    })
                                                }
                                            }
                                        }
                                    }
                                }
                            })
                        }

                }
            }
        },
    )
}
