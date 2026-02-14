use dioxus::prelude::*;
use tucan_types::{LoginResponse, Tucan};

use crate::RcTucanType;

#[component]
pub fn LogoutComponent() -> Element {
    let tucan: RcTucanType = use_context();

    let current_session_handle = use_context::<Signal<Option<LoginResponse>>>();

    // https://github.com/DioxusLabs/dioxus/issues/4303
    let on_submit = move |e: FormEvent| {
        e.prevent_default();
        let mut current_session_handle = current_session_handle;
        let tucan = tucan.clone();

        spawn(async move {
            if let Some(current_session) = current_session_handle() {
                tucan.logout(&current_session).await.unwrap();

                current_session_handle.set(None);
            }
        });
    };

    rsx! {
        form {
            onsubmit: on_submit,
            class: "d-flex",
            button {
                id: "logout-button",
                class: "btn btn-outline-success",
                r#type: "submit",
                { "Logout" }
            }
        }
    }
}
