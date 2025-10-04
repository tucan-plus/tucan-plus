#[cfg(not(target_arch = "wasm32"))]
use std::future::Future;
#[cfg(not(target_arch = "wasm32"))]
use std::panic::AssertUnwindSafe;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(not(target_arch = "wasm32"))]
use futures_util::stream::FuturesUnordered;
#[cfg(not(target_arch = "wasm32"))]
use futures_util::{FutureExt, StreamExt};
#[cfg(not(target_arch = "wasm32"))]
use tucan_connector::TucanConnector;
#[cfg(not(target_arch = "wasm32"))]
use tucan_plus_worker::MyDatabase;
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::vv::ActionRequest;
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::{LoginRequest, RevalidationStrategy, Tucan};
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::{LoginResponse, TucanError};

/*
cargo run --bin vv_fetcher --release | sort > vv.txt
cargo run --bin fetcher --release | sort > anmeldung.txt

*/

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), TucanError> {
    dotenvy::dotenv().unwrap();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}

#[cfg(not(target_arch = "wasm32"))]
async fn async_main() -> Result<(), TucanError> {
    let tucan = TucanConnector::new(MyDatabase::wait_for_worker().await).await?;

    /*let login_response = LoginResponse {
        id: std::env::var("SESSION_ID").unwrap().parse().unwrap(),
        cookie_cnsc: std::env::var("SESSION_KEY").unwrap(),
    };*/

    let login_response = tucan
        .login(LoginRequest {
            username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
            password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing"),
        })
        .await
        .unwrap();

    let fetcher = Arc::new(Fetcher::new());

    let welcome = tucan.welcome().await.unwrap();

    fetcher
        .recursive_vv(
            &tucan,
            &login_response,
            welcome.vorlesungsverzeichnis_url,
            String::new(),
        )
        .await;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
struct Fetcher {
    vv: AtomicU64,
    course: AtomicU64,
}

#[cfg(not(target_arch = "wasm32"))]
impl Fetcher {
    pub const fn new() -> Self {
        Self {
            vv: AtomicU64::new(0),
            course: AtomicU64::new(0),
        }
    }

    #[expect(clippy::manual_async_fn)]
    fn recursive_vv<'a, 'b>(
        self: Arc<Self>,
        tucan: &'a TucanConnector,
        login_response: &'b LoginResponse,
        action: ActionRequest,
        path: String,
    ) -> impl Future<Output = ()> + Send + use<'a, 'b> {
        async move {
            let result = AssertUnwindSafe(async {
                tucan
                    .vv(None, RevalidationStrategy::cache(), action.clone())
                    .await
                    .unwrap()
            })
            .catch_unwind()
            .await;
            let anmeldung_response = match result {
                Err(err) => {
                    eprintln!("failed to fetch vv {action} with error {err:?}");
                    return;
                }
                Ok(value) => value,
            };
            self.vv.fetch_add(1, Ordering::Relaxed);

            let results: FuturesUnordered<_> = anmeldung_response
                .entries
                .iter()
                .map(|entry| {
                    async {
                        self.clone()
                            .recursive_vv(
                                tucan,
                                login_response,
                                entry.1.clone(),
                                path.clone() + " > " + &entry.0,
                            )
                            .await;
                    }
                    .boxed()
                })
                .chain(
                    anmeldung_response
                        .veranstaltungen_or_module
                        .iter()
                        .map(|entry| {
                            async {
                                let result = AssertUnwindSafe(async {
                                    let course_details = tucan
                                        .course_details(
                                            login_response,
                                            RevalidationStrategy::cache(),
                                            entry.coursedetails_url.clone(),
                                        )
                                        .await
                                        .unwrap();
                                    println!("{}: {}", path, course_details.name);
                                })
                                .catch_unwind()
                                .await;
                                if let Err(err) = result {
                                    eprintln!(
                                        "failed to fetch course {} with error {err:?}",
                                        entry.coursedetails_url
                                    );
                                }

                                self.course.fetch_add(1, Ordering::Relaxed);
                            }
                            .boxed()
                        }),
                )
                .collect();
            results.collect::<Vec<()>>().await;
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn main() {}