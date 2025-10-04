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
use tucan_types::coursedetails::CourseDetailsRequest;
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::registration::{AnmeldungRequest, RegistrationState};
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::{LoginRequest, RevalidationStrategy, Tucan};
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::{LoginResponse, TucanError};

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

    fetcher
        .recursive_anmeldung(
            &tucan,
            &login_response,
            AnmeldungRequest::default(),
            String::new(),
        )
        .await;

    //fetcher.anmeldung_file.flush().await?;
    //fetcher.module_file.flush().await?;
    //fetcher.course_file.flush().await?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
struct Fetcher {
    anmeldung: AtomicU64,
    module: AtomicU64,
    course: AtomicU64,
}

#[cfg(not(target_arch = "wasm32"))]
impl Fetcher {
    pub const fn new() -> Self {
        Self {
            anmeldung: AtomicU64::new(0),
            module: AtomicU64::new(0),
            course: AtomicU64::new(0),
        }
    }

    #[expect(clippy::manual_async_fn)]
    fn recursive_anmeldung<'a, 'b>(
        self: Arc<Self>,
        tucan: &'a TucanConnector,
        login_response: &'b LoginResponse,
        anmeldung_request: AnmeldungRequest,
        path: String,
    ) -> impl Future<Output = ()> + Send + use<'a, 'b> {
        async move {
            //self.anmeldung_file.write_all(anmeldung_request.inner().as_bytes()).await?;
            //self.anmeldung_file.write_all(b"\n").await?;

            //println!("anmeldung {}", anmeldung_request.inner());
            let result = AssertUnwindSafe(async {
                tucan
                    .anmeldung(
                        &login_response.clone(),
                        RevalidationStrategy::cache(),
                        anmeldung_request.clone(),
                    )
                    .await
                    .unwrap()
            })
            .catch_unwind()
            .await;
            let anmeldung_response = match result {
                Err(err) => {
                    eprintln!("failed to fetch anmeldung {anmeldung_request} with error {err:?}");
                    return;
                }
                Ok(value) => value,
            };
            //println!("anmeldung counter: {}",
            // self.anmeldung_counter.load(Ordering::Relaxed));
            self.anmeldung.fetch_add(1, Ordering::Relaxed);

            let results: FuturesUnordered<_> = anmeldung_response
                .submenus
                .iter()
                .map(|entry| {
                    async {
                        self.clone()
                            .recursive_anmeldung(
                                tucan,
                                login_response,
                                entry.1.clone(),
                                path.clone() + " > " + &entry.0,
                            )
                            .await;
                    }
                    .boxed()
                })
                .chain(anmeldung_response.entries.iter().map(|entry| {
                    async {
                        if let Some(module) = &entry.module {
                            if matches!(
                                &module.registration_state,
                                RegistrationState::Registered { unregister_link: _ }
                            ) {
                                eprintln!("registered for {} at {}", module.name, path);
                            }

                            let result = AssertUnwindSafe(async {
                                tucan
                                    .module_details(
                                        login_response,
                                        RevalidationStrategy::cache(),
                                        module.url.clone(),
                                    )
                                    .await
                                    .unwrap()
                            })
                            .catch_unwind()
                            .await;
                            match result {
                                Err(err) => {
                                    eprintln!(
                                        "failed to fetch module {} with error {err:?}",
                                        module.url
                                    );
                                }
                                Ok(module) => if module.registered {},
                            }

                            self.module.fetch_add(1, Ordering::Relaxed);
                        }

                        for course in &entry.courses {
                            let result = AssertUnwindSafe(async {
                                let course_details = tucan
                                    .course_details(
                                        login_response,
                                        RevalidationStrategy::cache(),
                                        CourseDetailsRequest::parse(course.1.url.inner()),
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
                                    course.1.url
                                );
                            }
                            self.course.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    .boxed()
                }))
                .collect();
            results.collect::<Vec<()>>().await;
        }
    }
}


#[cfg(target_arch = "wasm32")]
pub fn main() {}