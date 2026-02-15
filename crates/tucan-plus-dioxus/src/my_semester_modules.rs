use std::{collections::HashSet, str::FromStr};

use dioxus::prelude::*;
use log::warn;
use tucan_types::{
    RevalidationStrategy, SemesterId, Tucan,
    mymodules::{Module, MyModulesResponse},
};

use crate::{RcTucanType, Route, common::use_authenticated_data_loader};

#[component]
pub fn MySemesterModules(semester: ReadSignal<SemesterId>) -> Element {
    let handler = async |tucan: RcTucanType,
                         current_session,
                         revalidation_strategy: RevalidationStrategy,
                         additional: SemesterId| {
        let first = tucan
            .my_modules(&current_session, revalidation_strategy, additional.clone())
            .await?;
        let after = first.semester.iter().skip_while(|e| !e.selected).nth(1);
        if let Some(after) = after {
            let second = tucan
                .my_modules(&current_session, revalidation_strategy, after.value.clone())
                .await?;
            let first_modules: HashSet<Module> = first.modules.iter().cloned().collect();
            let second_modules: HashSet<Module> = second.modules.iter().cloned().collect();
            let diff = first_modules.difference(&second_modules).cloned().collect();
            Ok(MyModulesResponse {
                semester: first.semester,
                modules: diff,
            })
        } else {
            Ok(first)
        }
    };

    let navigator = use_navigator();

    use_authenticated_data_loader(
        handler,
        semester,
        14 * 24 * 60 * 60,
        60 * 60,
        |my_modules: MyModulesResponse, reload| {
            let on_semester_change = {
                Callback::new(move |e: Event<FormData>| {
                    let value = e.value();
                    navigator.push(Route::MySemesterModules {
                        semester: SemesterId::from_str(&value).unwrap(),
                    });
                })
            };
            rsx! {
                div {
                    h1 {
                        {"Meine Semestermodule"}
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
                            my_modules
                                .semester
                                .iter()
                                .map(|semester| {
                                    rsx! {
                                        // TODO while loading here we should already show the newly selected value
                                        option { selected: semester.selected, value: semester.value.inner().clone(), {semester.name.clone()} }
                                    }
                                })
                        }
                    }
                    div { class: "table-responsive",
                        table { class: "table",
                            thead {
                                tr {
                                    th { scope: "col", {"NR"} }
                                    th { scope: "col", {"Name"} }
                                    th { scope: "col", {"Verantwortliche Person"} }
                                    th { scope: "col", {"Credits"} }
                                }
                            }
                            tbody {
                                {
                                    my_modules
                                        .modules
                                        .iter()
                                        .map(|module| {
                                            rsx! {
                                                tr {
                                                    th { scope: "row", {module.nr.clone()} }
                                                    td {
                                                        Link {
                                                            to: Route::ModuleDetails {
                                                                module: module.url.clone(),
                                                            },
                                                            {module.title.clone()}
                                                        }
                                                    }
                                                    td { {module.lecturer.clone()} }
                                                    td { {module.credits.clone().unwrap_or_else(|| "-".to_owned())} }
                                                }
                                            }
                                        })
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}
