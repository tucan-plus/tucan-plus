use std::time::Duration;

use dioxus::{html::FileData, prelude::*, web::WebFileExt as _};
use tucan_plus_worker::{ExportDatabaseRequest, ImportDatabaseRequest, MyDatabase};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

#[component]
pub fn ExportDatabase() -> Element {
    let worker: MyDatabase = use_context();
    let database = use_resource(move || {
        let worker = worker.clone();
        async move {
            let value = worker
                .send_message_with_timeout_raw(
                    ExportDatabaseRequest {},
                    Duration::from_secs(10 * 60),
                )
                .await
                .expect("export timed out");
            value
        }
    });
    rsx! {
        if let Some(database) = database() {
            a {
                href: {
                    web_sys::Url::create_object_url_with_blob(database.unchecked_ref()).unwrap()
                },
                download: "tucan-plus.db",
                "Download database"
            }
        }
    }
}

#[component]
pub fn ImportDatabase() -> Element {
    let mut loading = use_signal(|| false);
    let mut success = use_signal(|| false);
    let worker: MyDatabase = use_context();
    let mut file: Signal<Vec<FileData>> = use_signal(Vec::new);
    let onsubmit = {
        move |evt: Event<FormData>| {
            evt.prevent_default();
            let worker = worker.clone();
            async move {
                success.set(false);
                loading.set(true);
                let file = file()[0].get_web_file().unwrap();
                let array_buffer = JsFuture::from(file.array_buffer()).await.unwrap();
                worker
                    .send_message_with_timeout(
                        ImportDatabaseRequest {
                            data: array_buffer.into(),
                        },
                        Duration::from_secs(10 * 60),
                    )
                    .await
                    .expect("import timed out");
                loading.set(false);
                success.set(true);
            }
        }
    };
    rsx! {
        div {
            class: "container mt-3",
        if success() {
            div {
                class: "alert alert-success",
                role: "alert",
                "Database successfully imported"
            }
        }
        form {
                onsubmit: onsubmit,
                class: "mb-3",
        div {
                    class: "mb-3",
                    label {
                        for: "database-file",
                        class: "form-label",
                        "Datenbank importieren"
                    }
                    input {
                        type: "file",
                        class: "form-control",
                        accept: ".db",
                        id: "database-file",
                        required: true,
                        onchange: move |event| {
                            file.set(event.files());
                        },
                    }
                }
                button {
                    disabled: loading(),
                    type: "submit",
                    class: "btn btn-primary",
                    "Datenbank importieren"
                }
            }
        }
    }
}
