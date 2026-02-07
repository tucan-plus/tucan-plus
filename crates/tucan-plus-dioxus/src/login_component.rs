use dioxus::prelude::*;
use tucan_types::{LoginRequest, LoginResponse, Tucan};

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
                    web_extensions::cookies::set(web_extensions::cookies::SetCookieDetails {
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
            web_extensions::cookies::set(web_extensions::cookies::SetCookieDetails {
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
            web_extensions::cookies::set(web_extensions::cookies::SetCookieDetails {
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

            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsCast as _;

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
            }

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
        a {
            class: "btn btn-primary",
            role: "button",
            href: "https://dsf.tucan.tu-darmstadt.de/IdentityServer/External/Challenge?provider=dfnshib&returnUrl=%2FIdentityServer%2Fconnect%2Fauthorize%2Fcallback%3Fclient_id%3DClassicWeb%26scope%3Dopenid%2520DSF%2520email%26response_mode%3Dquery%26response_type%3Dcode%26ui_locales%3Dde%26redirect_uri%3Dhttps%253A%252F%252Fwww.tucan.tu-darmstadt.de%252Fscripts%252Fmgrqispi.dll%253FAPPNAME%253DCampusNet%2526PRGNAME%253DLOGINCHECK%2526ARGUMENTS%253D-N000000000000001,ids_mode%2526ids_mode%253DY",
            "Login"
        }
    }
}
