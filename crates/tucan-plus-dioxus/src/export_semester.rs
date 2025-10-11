#[cfg(target_arch = "wasm32")]
use std::time::Duration;
use std::{collections::{HashMap, HashSet}, ops::Add, pin::pin, rc::Rc, sync::atomic::{AtomicUsize, Ordering}};

use dioxus::{html::geometry::euclid::num::Zero, prelude::*};
use futures::{FutureExt as _, StreamExt, stream::BoxStream};
use itertools::Itertools as _;
use num::{BigInt, BigRational, FromPrimitive, One};
use serde::{Deserialize, Serialize};
use time::{Month, macros::offset};
use tokio::io::AsyncWriteExt as _;
use tucan_types::{DynTucan, LoginResponse, RevalidationStrategy, Tucan, TucanError, moduledetails::{ModuleDetailsRequest, ModuleDetailsResponse}, registration::{AnmeldungRequest, AnmeldungResponse}};
use num::ToPrimitive;
use crate::{RcTucanType, compress};



// breath first for progress?
// maybe us a channel?
// atomic for progress?
#[expect(clippy::manual_async_fn)]
pub fn recursive_anmeldung<'a, 'b: 'a>(
    tucan: &'a DynTucan<'static>,
    login_response: &'b LoginResponse,
    factor: BigRational,
    mut atomic_current: SyncSignal<BigRational>,
    mut atomic_total: SyncSignal<BigRational>,
    anmeldung_request: AnmeldungRequest,
) -> BoxStream<'a, AnmeldungResponse> {
    tucan.anmeldung(
        login_response,
        RevalidationStrategy::cache(),
        anmeldung_request.clone(),
    ).into_stream().flat_map(move |element: Result<AnmeldungResponse, TucanError>| {
        let factor = factor.clone();
        let element = element.unwrap();
        if element.submenus.is_empty()  {
            if factor > BigRational::from_f64(0.01).unwrap() {
                let factor = factor.clone();
                atomic_total.with_mut(|total| *total -= factor );
            } else {
                let factor = factor.clone();
                atomic_current.with_mut(|value| {
                    *value += factor;
                })
            }
        }
        futures::stream::once({let element = element.clone(); async move { element.clone() }}).chain(futures::stream::iter(element
            .submenus.clone()
            .into_iter())
            .flat_map_unordered(None, move |entry| {
                recursive_anmeldung(tucan, login_response, factor.clone() / BigRational::from_integer(element.submenus.len().into()), atomic_current, atomic_total, entry.1.clone())
            }))
    }).boxed()
}

#[derive(Serialize, Deserialize)]
pub struct SemesterExportV1 {
    pub anmeldungen: Vec<AnmeldungResponse>,
    pub modules: HashMap<ModuleDetailsRequest, ModuleDetailsResponse>
}

#[component]
pub fn FetchAnmeldung() -> Element {
    let mut result: Signal<Vec<(String, Vec<u8>)>> = use_signal(Vec::new);
    let tucan: RcTucanType = use_context();
    let current_session_handle = use_context::<Signal<Option<LoginResponse>>>();
    let mut loading = use_signal(|| false);
    let mut progresses = use_signal(Vec::<(SyncSignal<BigRational>, SyncSignal<BigRational>)>::new);

    let onclick = move |_event| {
        let tucan = tucan.clone();
        async move {
            loading.set(true);
            let anmeldung_response = tucan
                .anmeldung(
                    &current_session_handle().unwrap(),
                    RevalidationStrategy::cache(),
                    AnmeldungRequest::default(),
                )
                .await
                .unwrap();
            let datetime = time::OffsetDateTime::now_utc();
            let datetime = datetime.to_offset(offset!(+2));
            let date = datetime.date();
            let registration_sose = Month::March <= date.month() && date.month() <= Month::August;
            let semester = if registration_sose { "sose" } else { "wise" };

            for course_of_study in anmeldung_response.studiumsauswahl {
                log::info!("start");
                let session = current_session_handle().unwrap();
                let atomic_current = use_signal_sync(|| BigRational::zero());
                let atomic_total = use_signal_sync(|| BigRational::one());
                spawn({
                    let mut result = result.clone();
                    let tucan = tucan.clone();
                    let atomic_current = atomic_current.clone();
                    let atomic_total = atomic_total.clone();
                    async move {
                        let mut atomic_current = atomic_current.clone();
                        let atomic_total = atomic_total.clone();
                        let response = recursive_anmeldung(
                            &tucan.0,
                            &session,
                            BigRational::new(BigInt::from(1), BigInt::from(3)),
                            atomic_current,
                            atomic_total,
                            course_of_study.value.clone(),
                        );
                        let response = response.collect::<Vec<AnmeldungResponse>>().await;
                        let modules: HashSet<_> = response.iter().flat_map(|anmeldung| anmeldung.entries.iter()).flat_map(|entry| entry.module.iter()).map(|module| module.url.clone()).collect();
                        let modules_len = 3*modules.len();
                        let module_stream = futures::stream::iter(modules).flat_map_unordered(None, |module_id| {
                            let tucan = tucan.clone();
                            let session = session.clone();
                            let future = async move {
                                let change = BigRational::new(BigInt::from(1), BigInt::from(modules_len));
                                let module = tucan.0.module_details(&session, RevalidationStrategy::cache(), module_id.clone()).await.unwrap();
                                atomic_current.with_mut(|current| *current += change.clone());
                                (module_id, module)
                            };
                            Box::pin(future).into_stream()
                        });
                        let module_response = module_stream.collect().await;

                        log::info!("downloaded done 3");
                        let content = serde_json::to_string(&SemesterExportV1 {
                            anmeldungen: response,
                            modules: module_response
                        }).unwrap();
                        let in_data = content.as_bytes();
                        let mut encoder = async_compression::tokio::write::BrotliEncoder::with_quality(
                            Vec::new(),
                            async_compression::Level::Precise(9),
                        );
                        info!("file chunks: {}", in_data.len()/100/1024);
                        let chunk_part = 3*in_data.len()/100/1024;
                        for chunk in in_data.chunks(100*1024).enumerate() {
                            let change = BigRational::new(BigInt::from(1), BigInt::from(chunk_part));
                            encoder.write_all(chunk.1).await.unwrap(); // hangs, move to worker?
                            info!("{}/{}", chunk.0, in_data.len()/100/1024);
                            atomic_current.with_mut(|current| *current += change.clone());
                            #[cfg(target_arch = "wasm32")]
                            crate::sleep(Duration::from_millis(0)).await;
                        }
                        encoder.shutdown().await.unwrap();
                        let compressed = encoder.into_inner();
                        result.push((
                            format!(
                                "registration{}_{}.{semester}-v1-tucan",
                                course_of_study.value, course_of_study.name
                            ),
                            compressed,
                        ));
                        loading.set(false);
                    }
                });
                progresses.push((atomic_current, atomic_total));
            }
        }
    };

    rsx! {
        div {
            class: "container",
            if loading() {
                div {
                    style: "z-index: 10000",
                    class: "position-fixed top-50 start-50 translate-middle",
                    div {
                        class: "spinner-grow",
                        role: "status",
                        span {
                            class: "visually-hidden",
                            "Loading..."
                        }
                    }
                }
            }
            h1 {
                class: "text-center",
                "Anmeldungsexporte"
            }
            p {
                "Das Laden könnte etwas länger dauern (5-10 Minuten). Außerdem macht es ca. 1500 \
                 Anfragen an TUCaN und benötigt ca. 30MB Datenvolumen."
            }
            button {
                onclick,
                class: "btn btn-primary mb-1",
                disabled: loading(),
                "Exportieren"
            }
            br {
            }
            for entry in result() {
                a {
                    href: {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let blob_properties = web_sys::BlobPropertyBag::new();
                            blob_properties.set_type("octet/stream");
                            let bytes = js_sys::Array::new();
                            bytes.push(&js_sys::Uint8Array::from(&entry.1[..]));
                            let blob =
                                web_sys::Blob::new_with_blob_sequence_and_options(&bytes, &blob_properties)
                                    .unwrap();
                            web_sys::Url::create_object_url_with_blob(&blob).unwrap()
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        "/todo"
                    },
                    download: entry.0.clone(),
                    { format!("Download {}", entry.0.clone()) }
                }
                br {
                }
            }
            for progress in progresses() {
                div {
                class: "progress", role:"progressbar", "aria-label": "Basic example", "aria-valuenow": "25",
                "aria-valuemin": "0", "aria-valuemax": "100",
                        div { class: "progress-bar", style: format!("width: {}%", (progress.0()/progress.1()).to_f64().unwrap()*100.0),
                            { format!("{:.2}%", (progress.0()/progress.1()).to_f64().unwrap()*100.0) }
                        }
                }
            }
        }
    }
}
