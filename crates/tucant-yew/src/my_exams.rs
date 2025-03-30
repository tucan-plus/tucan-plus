use std::ops::Deref;

use log::info;
use tucant_types::{LoginResponse, RevalidationStrategy, Tucan, coursedetails::CourseDetailsRequest};
use wasm_bindgen_futures::spawn_local;
use yew::{Callback, Html, HtmlResult, MouseEvent, Properties, UseStateHandle, function_component, html, use_context, use_effect, use_effect_with, use_state};

use crate::RcTucanType;

#[function_component(MyExams)]
pub fn my_exams<TucanType: Tucan + 'static>() -> HtmlResult {
    let tucan: RcTucanType<TucanType> = use_context().expect("no ctx found");

    let data = use_state(|| Ok(None));
    let loading = use_state(|| false);
    let current_session_handle = use_context::<UseStateHandle<Option<LoginResponse>>>().expect("no ctx found");
    {
        let data = data.clone();
        let loading = loading.clone();
        let current_session_handle = current_session_handle.clone();
        let tucan = tucan.clone();
        use_effect_with((), move |()| {
            if let Some(current_session) = (*current_session_handle).to_owned() {
                loading.set(true);
                let data = data.clone();
                let tucan = tucan.clone();
                spawn_local(async move {
                    match tucan.0.my_exams(&current_session, RevalidationStrategy { max_age: 14 * 24 * 60 * 60, invalidate_dependents: Some(true) }).await {
                        Ok(response) => {
                            data.set(Ok(Some(response)));
                            loading.set(false);

                            match tucan.0.my_exams(&current_session, RevalidationStrategy { max_age: 4 * 24 * 60 * 60, invalidate_dependents: Some(true) }).await {
                                Ok(response) => data.set(Ok(Some(response))),
                                Err(error) => {
                                    info!("ignoring error when refetching: {}", error)
                                }
                            }
                        }
                        Err(error) => {
                            data.set(Err(error.to_string()));
                            loading.set(false);
                        }
                    }
                })
            } else {
                data.set(Err("Not logged in".to_owned()));
            }
        });
    }

    let reload = {
        let current_session = current_session_handle.clone();
        let data = data.clone();
        let loading = loading.clone();
        let current_session = current_session.clone();
        let tucan = tucan.clone();
        Callback::from(move |e: MouseEvent| {
            if let Some(current_session) = (*current_session).to_owned() {
                loading.set(true);
                let data = data.clone();
                let tucan = tucan.clone();
                let loading = loading.clone();
                spawn_local(async move {
                    match tucan.0.my_exams(&current_session, RevalidationStrategy { max_age: 0, invalidate_dependents: Some(true) }).await {
                        Ok(response) => {
                            data.set(Ok(Some(response)));
                            loading.set(false);
                        }
                        Err(error) => {
                            data.set(Err(error.to_string()));
                            loading.set(false);
                        }
                    }
                })
            } else {
                data.set(Err("Not logged in".to_owned()));
            }
        })
    };

    let data = match data.deref() {
        Ok(data) => data,
        Err(error) => {
            return Ok(html! {
                <div class="container">
                    <div class="alert alert-danger d-flex align-items-center mt-2" role="alert">
                        // https://github.com/twbs/icons
                        // The MIT License (MIT)
                        // Copyright (c) 2019-2024 The Bootstrap Authors
                        <svg xmlns="http://www.w3.org/2000/svg" class="bi bi-exclamation-triangle-fill flex-shrink-0 me-2" width="16" height="16" viewBox="0 0 16 16" role="img" aria-label="Error:">
                            <path d="M8.982 1.566a1.13 1.13 0 0 0-1.96 0L.165 13.233c-.457.778.091 1.767.98 1.767h13.713c.889 0 1.438-.99.98-1.767L8.982 1.566zM8 5c.535 0 .954.462.9.995l-.35 3.507a.552.552 0 0 1-1.1 0L7.1 5.995A.905.905 0 0 1 8 5zm.002 6a1 1 0 1 1 0 2 1 1 0 0 1 0-2z" />
                        </svg>
                        <div>{ error }</div>
                    </div>
                </div>
            });
        }
    };

    Ok(html! {
        <div class="container">
            if *loading {
                <div style="z-index: 10000" class="position-fixed top-50 start-50 translate-middle">
                    <div class="spinner-grow" role="status">
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                </div>
            }

            if let Some(exams) = data {
                    <div>

                    <h1>
                        { "Prüfungen" }
                        {" "}<button onclick={reload} type="button" class="btn btn-light">
                        // https://github.com/twbs/icons
                        // The MIT License (MIT)
                        // Copyright (c) 2019-2024 The Bootstrap Authors
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-arrow-clockwise" viewBox="0 0 16 16">
                            <path fill-rule="evenodd" d="M8 3a5 5 0 1 0 4.546 2.914.5.5 0 0 1 .908-.417A6 6 0 1 1 8 2z"/>
                            <path d="M8 4.466V.534a.25.25 0 0 1 .41-.192l2.36 1.966c.12.1.12.284 0 .384L8.41 4.658A.25.25 0 0 1 8 4.466"/>
                        </svg>
                    </button>
                    </h1>

                    <table class="table">
                    <thead>
                    <tr>
                        <th scope="col">{"NR"}</th>
                        <th scope="col">{"Name"}</th>
                        <th scope="col">{"Prüfungsart"}</th>
                        <th scope="col">{"Termin"}</th>
                    </tr>
                    </thead>
                    <tbody>
                    {
                        exams.exams.iter().map(|exam| {
                            html!{
                                <tr>
                                    <th scope="row">{&exam.id}</th>
                                    <td>{&exam.name}</td>
                                    <td>{&exam.pruefungsart}</td>
                                    <td>{&exam.date}</td>
                                </tr>
                            }
                        }).collect::<Html>()
                    }
                    </tbody>
                    </table>

                    </div>
                }
        </div>
    })
}
