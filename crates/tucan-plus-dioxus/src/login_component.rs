use dioxus::prelude::*;
use tucan_types::{LoginRequest, LoginResponse, Tucan};
use wasm_bindgen::JsCast as _;

use crate::{Anonymize, RcTucanType};

#[component]
pub fn LoginComponent() -> Element {
    tracing::error!("login component");
    let tucan: RcTucanType = use_context();

    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut error_message = use_signal(|| None);
    let mut loading = use_signal(|| false);

    let mut current_session = use_context::<Signal<Option<LoginResponse>>>();

    let anonymize = use_context::<Anonymize>().0;

    let on_submit = move |e: FormEvent| {
        e.prevent_default();
        let tucan = tucan.clone();
        spawn(async move {
            let tucan = tucan.clone();

            let password_string = password();
            password.set("".to_owned());

            loading.set(true);
            match tucan
                .login(LoginRequest {
                    username: username(),
                    password: password_string,
                })
                .await
            {
                Ok(response) => {
                    #[cfg(feature = "direct")]
                    web_extensions_sys::chrome()
                        .cookies()
                        .set(web_extensions_sys::SetCookieDetails {
                            name: Some("id".to_owned()),
                            partition_key: None,
                            store_id: None,
                            url: "https://www.tucan.tu-darmstadt.de".to_owned(),
                            domain: None,
                            path: Some("/scripts".to_owned()),
                            value: Some(response.id.to_string()),
                            expiration_date: None,
                            http_only: None,
                            secure: Some(true),
                            same_site: None,
                        })
                        .await;

                    #[cfg(any(feature = "desktop", feature = "mobile"))]
                    keyring_core::Entry::new("tucan-plus", "session")
                        .unwrap()
                        .set_password(&serde_json::to_string(&response).unwrap())
                        .unwrap();
                    #[cfg(any(feature = "desktop", feature = "mobile"))]
                    tracing::error!("saving password to keyring");

                    current_session.set(Some(response.clone()));
                    error_message.set(None);
                }
                Err(e) => {
                    tracing::error!("{e}");
                    error_message.set(Some(e.to_string()));
                }
            };
            loading.set(false);
        });
    };
    let _set_fake_session = move |_event: Event<MouseData>| {
        async move {
            // TODO deduplicate
            #[cfg(feature = "direct")]
            web_extensions_sys::chrome()
                .cookies()
                .set(web_extensions_sys::SetCookieDetails {
                    name: Some("id".to_owned()),
                    partition_key: None,
                    store_id: None,
                    url: "https://www.tucan.tu-darmstadt.de".to_owned(),
                    domain: None,
                    path: Some("/scripts".to_owned()),
                    value: Some("544780631865356".to_owned()),
                    expiration_date: None,
                    http_only: None,
                    secure: Some(true),
                    same_site: None,
                })
                .await;

            #[cfg(feature = "direct")]
            web_extensions_sys::chrome()
                .cookies()
                .set(web_extensions_sys::SetCookieDetails {
                    name: Some("cnsc".to_owned()),
                    partition_key: None,
                    store_id: None,
                    url: "https://www.tucan.tu-darmstadt.de".to_owned(),
                    domain: None,
                    path: Some("/scripts".to_owned()),
                    value: Some("84BC747762F472B5A7507EB9F5CE2330".to_owned()),
                    expiration_date: None,
                    http_only: None,
                    secure: Some(true),
                    same_site: None,
                })
                .await;

            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let html_document = document.dyn_into::<web_sys::HtmlDocument>().unwrap();

            // this probably is a long timeout, it seems like tucan will at some point
            // forget that a session is a timeout
            html_document
                .set_cookie("id=544780631865356; Path=/")
                .unwrap();
            html_document
                .set_cookie("cnsc=84BC747762F472B5A7507EB9F5CE2330; Path=/")
                .unwrap();

            current_session.set(Some(LoginResponse {
                id: 544780631865356,
                cookie_cnsc: "84BC747762F472B5A7507EB9F5CE2330".to_string(),
            }));
            error_message.set(None);
        }
    };

    let is_invalid = if error_message().is_some() {
        "is-invalid"
    } else {
        ""
    };
    rsx! {
        form {
            onsubmit: on_submit,
            class: "d-flex",
            input {
                id: "login-username",
                value: "{username}",
                oninput: move |event| username.set(event.value()),
                required: true,
                class: "align-self-start form-control me-2",
                r#type: if anonymize { "password" } else { "username" },
                placeholder: "TU-ID",
                "aria-label": "TU-ID",
                autocomplete: "current-username",
                disabled: loading(),
            }
            div {
                class: "align-self-start input-group has-validation",
                input {
                    id: "login-password",
                    value: "{password}",
                    oninput: move |event| password.set(event.value()),
                    required: true,
                    class: "form-control me-2 {is_invalid}",
                    r#type: "password",
                    placeholder: "Password",
                    "aria-label": "Password",
                    "aria-describedby": "password-feedback",
                    autocomplete: "current-password",
                    disabled: loading(),
                }
                if let Some(error_message) = error_message() {
                    div {
                        id: "password-feedback",
                        class: "invalid-feedback",
                        "{error_message}"
                    }
                }
            }
            button {
                class: "align-self-start btn btn-outline-success",
                r#type: "submit",
                id: "login-button",
                disabled: loading(),
                "Login"
            }
        }
    }
}
