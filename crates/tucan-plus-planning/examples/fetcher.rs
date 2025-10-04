#[cfg(not(target_arch = "wasm32"))]
use tucan_connector::TucanConnector;
#[cfg(not(target_arch = "wasm32"))]
use tucan_plus_planning::{compress, recursive_anmeldung};
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::TucanError;
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::registration::AnmeldungRequest;
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::{DynTucan, LoginRequest, RevalidationStrategy, Tucan};

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

    let anmeldung_response = tucan
        .anmeldung(
            &login_response.clone(),
            RevalidationStrategy::cache(),
            AnmeldungRequest::default(),
        )
        .await
        .unwrap();
    for course_of_study in anmeldung_response.studiumsauswahl {
        let result = recursive_anmeldung(
            DynTucan::from_ref(&tucan),
            &login_response,
            course_of_study.value.clone(),
        )
        .await;
        let content = serde_json::to_string(&result).unwrap();
        tokio::fs::write(
            format!(
                "registration{}_{}.json.br",
                course_of_study.value, course_of_study.name
            ),
            &compress(content.as_bytes()).await.unwrap(),
        )
        .await
        .unwrap();
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn main() {}