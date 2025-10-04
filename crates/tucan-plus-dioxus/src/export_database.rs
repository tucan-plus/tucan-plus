
use dioxus::prelude::*;
use js_sys::{Array, Uint8Array};
use tucan_plus_worker::{ExportDatabaseRequest, MyDatabase};
use web_sys::{Blob, Url};

async fn export_db() -> Vec<u8> {
    let worker: MyDatabase = use_context();
    worker.send_message(ExportDatabaseRequest).await
}

#[component]
pub fn ExportDatabase() -> Element {
    let connection = use_resource(move || async move { export_db().await });
    rsx! {
        if let Some(database) = connection() {
            a {
                href: {
                    let blob_properties = web_sys::BlobPropertyBag::new();
                    blob_properties.set_type("octet/stream");
                    let bytes = Array::new();
                    bytes.push(&Uint8Array::from(&database[..]));
                    let blob =
                        Blob::new_with_blob_sequence_and_options(&bytes, &blob_properties).unwrap();
                    Url::create_object_url_with_blob(&blob).unwrap()
                },
                download: "tucan-plus.db",
                "Download database"
            }
        }
    }
}
