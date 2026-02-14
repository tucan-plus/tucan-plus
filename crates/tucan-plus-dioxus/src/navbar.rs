use dioxus::prelude::*;
use tucan_types::{LoginResponse, RevalidationStrategy, Tucan};

use crate::{
    LOGO_SVG, RcTucanType, Route, common::handle_error, logout_component::LogoutComponent,
    navbar_logged_in::NavbarLoggedIn, navbar_logged_out::NavbarLoggedOut,
};

#[component]
pub fn Navbar() -> Element {
    let tucan: RcTucanType = use_context();

    let current_session = use_context::<Signal<Option<LoginResponse>>>();

    let data = use_resource(move || {
        let tucan = tucan.clone();
        async move {
            if let Some(the_current_session) = current_session() {
                match tucan
                    .after_login(&the_current_session, RevalidationStrategy::cache())
                    .await
                {
                    Ok(response) => Ok(Some(response)),
                    Err(error) => handle_error(current_session, error, true).await,
                }
            } else {
                Ok(None)
            }
        }
    });

    rsx! {
        nav {
            class: "navbar navbar-expand-xl bg-body-tertiary",
            div {
                class: "container-fluid",
                Link {
                    to: Route::Root { },
                    class: "navbar-brand",
                    img {
                        src: LOGO_SVG,
                        height: 24,
                        alt: "TUCaN Plus",
                    }
                }
                button {
                    aria_controls: "navbarSupportedContent",
                    aria_expanded: "false",
                    aria_label: "Toggle navigation",
                    class: "navbar-toggler",
                    "data-bs-target": "#navbarSupportedContent",
                    "data-bs-toggle": "collapse",
                    r#type: "button",
                    span {
                        class: "navbar-toggler-icon",
                    }
                }
                div {
                    class: "collapse navbar-collapse",
                    id: "navbarSupportedContent",
                    ul {
                        class: "navbar-nav me-auto mb-2 mb-xl-0",
                        match (current_session(), data()) {
                            (Some(current_session), Some(Ok(data))) => {
                                rsx! {
                                    NavbarLoggedIn {
                                        current_session,
                                        data,
                                    }
                                }
                            }
                            _ => {
                                rsx! {
                                    NavbarLoggedOut {
                                    }
                                }
                            }
                        }
                    }
                    if let Some(_current_session) = current_session() {
                        LogoutComponent {
                        }
                    } else {
                        a {
                            id: "login-button",
                            class: "btn btn-primary",
                            role: "button",
                            href: "https://dsf.tucan.tu-darmstadt.de/IdentityServer/External/Challenge?provider=dfnshib&returnUrl=%2FIdentityServer%2Fconnect%2Fauthorize%2Fcallback%3Fclient_id%3DClassicWeb%26scope%3Dopenid%2520DSF%2520email%26response_mode%3Dquery%26response_type%3Dcode%26ui_locales%3Dde%26redirect_uri%3Dhttps%253A%252F%252Fwww.tucan.tu-darmstadt.de%252Fscripts%252Fmgrqispi.dll%253FAPPNAME%253DCampusNet%2526PRGNAME%253DLOGINCHECK%2526ARGUMENTS%253D-N000000000000001,ids_mode%2526ids_mode%253DY",
                            "Login"
                        }
                    }
                }
            }
        }
        SuspenseBoundary {
            fallback: |_| rsx! { span { "Loading..." } },
            Outlet::<Route> {
            }
        }
    }
}
