#[cfg(not(target_arch = "wasm32"))]
use std::future::Future;
#[cfg(not(target_arch = "wasm32"))]
use std::panic::AssertUnwindSafe;
#[cfg(not(target_arch = "wasm32"))]
use std::pin::pin;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(not(target_arch = "wasm32"))]
use std::u64;

#[cfg(not(target_arch = "wasm32"))]
use futures_util::stream::{self, FuturesUnordered};
#[cfg(not(target_arch = "wasm32"))]
use futures_util::{FutureExt, Stream, StreamExt};
#[cfg(not(target_arch = "wasm32"))]
use itertools::Itertools;
#[cfg(not(target_arch = "wasm32"))]
use tucan_connector::TucanConnector;
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::coursedetails::CourseDetailsRequest;
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::moduledetails::ModuleDetailsResponse;
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::registration::{AnmeldungModule, AnmeldungRequest, RegistrationState};
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::student_result::{StudentResultEntry, StudentResultLevel, StudentResultState};
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::{LeistungsspiegelGrade, LoginRequest, RevalidationStrategy, Tucan};
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
fn validate(errors: &mut Vec<String>, level: &StudentResultLevel) -> (u64, u64) {
    let mut cp = 0;
    let mut modules = 0;
    for level in &level.children {
        let inner = validate(errors, &level);
        cp += inner.0;
        modules += inner.1;
    }
    for entry in &level.entries {
        if let Some(module_cp) = entry.cp {
            cp += module_cp;
        }
        modules += 1;
    }
    if cp > level.rules.max_cp.unwrap_or(u64::MAX) || cp < level.rules.min_cp {
        errors.push(format!(
            "invalid cp {} <= {} <= {:?} for {:?}",
            level.rules.min_cp, cp, level.rules.max_cp, level.name
        ))
    }
    if modules > level.rules.max_modules.unwrap_or(u64::MAX) || modules < level.rules.min_modules {
        errors.push(format!(
            "invalid module count {} <= {} <= {:?} for {:?}",
            level.rules.min_modules, modules, level.rules.max_modules, level.name
        ))
    }
    (cp, modules)
}

#[cfg(not(target_arch = "wasm32"))]
async fn async_main() -> Result<(), TucanError> {
    use tucan_plus_worker::MyDatabase;

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

    let course_of_studies = tucan
        .student_result(&login_response, RevalidationStrategy::cache(), 0)
        .await
        .unwrap();
    let bachelor = course_of_studies
        .course_of_study
        .iter()
        .find(|v| v.name == "B.Sc. Informatik (2015)")
        .unwrap()
        .value;
    let student_result = tucan
        .student_result(&login_response, RevalidationStrategy::cache(), bachelor)
        .await
        .unwrap();
    println!("{:#?}", student_result);

    let mut errors = Vec::new();
    validate(&mut errors, &student_result.level0);
    println!("{:#?}", errors);

    let master = course_of_studies
        .course_of_study
        .iter()
        .find(|v| v.name == "M.Sc. Informatik (2023)")
        .unwrap()
        .value;
    let mut student_result = tucan
        .student_result(&login_response, RevalidationStrategy::cache(), master)
        .await
        .unwrap();
    println!("{:#?}", student_result);

    let fetcher = Arc::new(Fetcher::new());

    let mut stream = pin!(fetcher.recursive_anmeldung(
        tucan,
        login_response,
        AnmeldungRequest::default(),
        Vec::new(),
    ));
    while let Some((module, path, module_details)) = stream.next().await {
        let mut level = &mut student_result.level0;
        for element in path {
            if let Some(inner) = level
                .children
                .iter_mut()
                .find(|child| child.name == Some(element.clone()))
            {
                level = inner;
            } else {
                break;
            }
        }
        level.entries.push(StudentResultEntry {
            id: Some(module.id),
            name: module.name,
            resultdetails_url: None,
            cp: module_details.credits,
            used_cp: module_details.credits,
            grade: LeistungsspiegelGrade::Offen,
        });
    }

    println!("{:#?}", student_result);

    let mut errors = Vec::new();
    validate(&mut errors, &student_result.level0);
    println!("{}: {:#?}", errors.len(), errors);

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
struct Fetcher {}

#[cfg(not(target_arch = "wasm32"))]
impl Fetcher {
    pub const fn new() -> Self {
        Self {}
    }

    fn recursive_anmeldung(
        self: Arc<Self>,
        tucan: TucanConnector,
        login_response: LoginResponse,
        anmeldung_request: AnmeldungRequest,
        path: Vec<String>,
    ) -> impl Stream<Item = (AnmeldungModule, Vec<String>, ModuleDetailsResponse)> + Send {
        let stream = {
            let tucan = tucan.clone();
            let login_response = login_response.clone();
            async move {
                tucan
                    .anmeldung(
                        &login_response.clone(),
                        RevalidationStrategy::cache(),
                        anmeldung_request.clone(),
                    )
                    .await
                    .unwrap()
            }
        }
        .into_stream();
        stream.flat_map(move |anmeldung_response| {
            let path = path.clone();
            anmeldung_response
                .entries
                .into_iter()
                .filter_map({
                    let path = path.clone();
                    move |entry| {
                        entry.module.as_ref().and_then(|module| {
                            if matches!(
                                &module.registration_state,
                                RegistrationState::Registered { unregister_link: _ }
                            ) {
                                Some((module.clone(), path.clone()))
                            } else {
                                None
                            }
                        })
                    }
                })
                .map(|(module, path)| {
                    let tucan = tucan.clone();
                    let login_response = login_response.clone();
                    async move {
                        let module_details = tucan
                            .module_details(
                                &login_response,
                                RevalidationStrategy::cache(),
                                module.url.clone(),
                            )
                            .await
                            .unwrap();
                        (module, path, module_details)
                    }
                })
                .collect::<FuturesUnordered<_>>()
                .chain(
                    stream::iter(
                        anmeldung_response
                            .submenus
                            .into_iter()
                            .filter(|entry| entry.0 != "Zusätzliche Leistungen"),
                    )
                    .flat_map({
                        let path = path.clone();
                        let self_clone = self.clone();
                        let tucan = tucan.clone();
                        let login_response = login_response.clone();
                        move |entry| {
                            self_clone
                                .clone()
                                .recursive_anmeldung(
                                    tucan.clone(),
                                    login_response.clone(),
                                    entry.1.clone(),
                                    {
                                        let mut path = path.clone();
                                        path.push(entry.0);
                                        path
                                    },
                                )
                                .boxed()
                        }
                    }),
                )
        })
    }
}

#[cfg(target_arch = "wasm32")]
pub fn main() {}