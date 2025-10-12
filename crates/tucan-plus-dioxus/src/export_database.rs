
use std::time::Duration;

use dioxus::prelude::*;
use tucan_plus_worker::{ExportDatabaseRequest, MyDatabase};

async fn export_db() -> Vec<u8> {
    let worker: MyDatabase = use_context();
    worker.send_message_with_timeout(ExportDatabaseRequest, Duration::from_mins(10)).await.expect("export timed out")
}

#[component]
pub fn ExportDatabase() -> Element {
    let connection = use_resource(move || async move { export_db().await });
    rsx! {
        if let Some(database) = connection() {
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
