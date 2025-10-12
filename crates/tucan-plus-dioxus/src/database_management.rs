use std::time::Duration;

use dioxus::{html::FileData, prelude::*};
use tucan_plus_worker::{ExportDatabaseRequest, ImportDatabaseRequest, MyDatabase};

#[component]
pub fn ExportDatabase() -> Element {
    let worker: MyDatabase = use_context();
    let database = use_resource(move || {
        let worker = worker.clone();
        async move {
            worker
                .send_message_with_timeout(ExportDatabaseRequest, Duration::from_mins(10))
                .await
                .expect("export timed out")
        }
    });
    rsx! {
        if let Some(database) = database() {
            a {
                href: {
                    #[cfg(target_arch = "wasm32")]
                    {
                        // data:text/plain;charset=utf-8,?
                        let blob_properties = web_sys::BlobPropertyBag::new();
                        blob_properties.set_type("octet/stream");
                        let bytes = js_sys::Array::new();
                        bytes.push(&js_sys::Uint8Array::from(&database[..]));
                        let blob =
                            web_sys::Blob::new_with_blob_sequence_and_options(&bytes, &blob_properties).unwrap();
                        web_sys::Url::create_object_url_with_blob(&blob).unwrap()
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    "/todo"
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
    let mut file: Signal<Vec<FileData>> = use_signal(|| Vec::new());
    let onsubmit = {
        move |evt: Event<FormData>| {
            evt.prevent_default();
            let worker = worker.clone();
            async move {
                success.set(false);
                loading.set(true);
                #[cfg(target_arch = "wasm32")]
                crate::sleep(Duration::from_millis(0)).await;
                worker
                    .send_message_with_timeout(
                        ImportDatabaseRequest {
                            data: file()[0].read_bytes().await.unwrap().to_vec(),
                        },
                        Duration::from_mins(10),
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
