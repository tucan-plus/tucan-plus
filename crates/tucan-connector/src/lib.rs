use std::{sync::LazyLock, time::Duration};

use coursedetails::index::course_details;
use externalpages::welcome::welcome;
use key_value_database::Database;
use login::{login, logout};
use mlsstart::start_page::after_login;
use moduledetails::index::module_details;
use mycourses::mycourses;
use mymodules::mymodules;
use regex::Regex;
use registration::index::anmeldung;
use reqwest::header;
use time::{OffsetDateTime, format_description::well_known::Rfc2822};
use tokio::{sync::Semaphore, time::sleep};
use tucant_types::{
    RevalidationStrategy, Tucan, TucanError,
    mlsstart::MlsStart,
    mycourses::MyCoursesResponse,
    mymodules::MyModulesResponse,
    vv::{ActionRequest, Vorlesungsverzeichnis},
};
use vv::vv;

pub mod common;
pub mod coursedetails;
pub mod courseresults;
pub mod examresults;
pub mod externalpages;
pub mod login;
pub mod mlsstart;
pub mod moduledetails;
pub mod mycourses;
pub mod mydocuments;
pub mod myexams;
pub mod mymodules;
pub mod registration;
pub mod root;
pub mod startpage_dispatch;
pub mod vv;

static COURSEDETAILS_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&PRGNAME=COURSEDETAILS&ARGUMENTS=-N\\d+,-N\\d+,").unwrap());

type MyClient = reqwest::Client;

pub struct TucanConnector {
    pub client: MyClient,
    pub database: Database,
    semaphore: Semaphore,
}

/// `TUCaN` being unreliable is a feature
pub async fn retryable_get(connector: &TucanConnector, url: &str) -> Result<(String, OffsetDateTime), TucanError> {
    let mut i = 0;
    loop {
        let permit = connector.semaphore.acquire().await.unwrap();
        let response = connector.client.get(url).send().await?.error_for_status()?;
        let date = &response.headers()["Date"];
        let date = OffsetDateTime::parse(date.to_str().unwrap(), &Rfc2822).unwrap();
        let result = response.text().await;
        drop(permit);
        if i == 4 {
            return Ok((result?, date));
        }
        match result {
            Ok(value) => return Ok((value, date)),
            Err(err) => println!("ignoring error: {err}"),
        }
        sleep(Duration::from_secs(2u64.pow(i))).await;
        i += 1;
    }
}

pub async fn authenticated_retryable_get(connector: &TucanConnector, url: &str, cookie_cnsc: &str) -> Result<(String, OffsetDateTime), TucanError> {
    let mut i = 0;
    loop {
        let permit = connector.semaphore.acquire().await.unwrap();
        let response = connector.client.get(url).header("Cookie", format!("cnsc={cookie_cnsc}")).send().await?.error_for_status()?;
        let date = &response.headers()["Date"];
        let date = OffsetDateTime::parse(date.to_str().unwrap(), &Rfc2822).unwrap();
        let result = response.text().await;
        drop(permit);
        if i == 4 {
            return Ok((result?, date));
        }
        match result {
            Ok(value) => return Ok((value, date)),
            Err(err) => println!("ignoring error: {err}"),
        }
        sleep(Duration::from_secs(2u64.pow(i))).await;
        i += 1;
    }
}

impl TucanConnector {
    pub async fn new() -> Result<Self, TucanError> {
        let mut headers = header::HeaderMap::new();
        headers.insert("Accept-Language", header::HeaderValue::from_static("de-DE,de;q=0.5"));
        let client = reqwest::Client::builder().default_headers(headers).user_agent("https://github.com/tucant/tucant d8167c8 Moritz.Hedtke@t-online.de").build().unwrap();
        Ok(Self { client, database: Database::new().await, semaphore: Semaphore::new(10) })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new_test() -> Result<Self, TucanError> {
        let mut headers = header::HeaderMap::new();
        headers.insert("Accept-Language", header::HeaderValue::from_static("de-DE,de;q=0.5"));
        let client = reqwest::Client::builder().default_headers(headers).user_agent("https://github.com/tucant/tucant d8167c8 Moritz.Hedtke@t-online.de").build().unwrap();
        Ok(Self { client, database: Database::new_test().await, semaphore: Semaphore::new(10) })
    }
}

impl Tucan for TucanConnector {
    async fn login(&self, request: tucant_types::LoginRequest) -> Result<tucant_types::LoginResponse, TucanError> {
        login(&self.client, &request).await
    }

    async fn welcome(&self) -> Result<tucant_types::LoggedOutHead, TucanError> {
        welcome(self).await
    }

    async fn after_login(&self, request: &tucant_types::LoginResponse, revalidation_strategy: RevalidationStrategy) -> Result<MlsStart, TucanError> {
        after_login(self, request, revalidation_strategy).await
    }

    async fn logout(&self, request: &tucant_types::LoginResponse) -> Result<(), TucanError> {
        logout(self, request).await
    }

    async fn my_modules(&self, request: &tucant_types::LoginResponse, revalidation_strategy: RevalidationStrategy) -> Result<MyModulesResponse, TucanError> {
        mymodules(self, request, revalidation_strategy).await
    }

    async fn my_courses(&self, request: &tucant_types::LoginResponse, revalidation_strategy: RevalidationStrategy) -> Result<MyCoursesResponse, TucanError> {
        mycourses(self, request, revalidation_strategy).await
    }

    async fn anmeldung(&self, login_response: tucant_types::LoginResponse, revalidation_strategy: RevalidationStrategy, request: tucant_types::registration::AnmeldungRequest) -> Result<tucant_types::registration::AnmeldungResponse, TucanError> {
        anmeldung(self, &login_response, revalidation_strategy, request).await
    }

    async fn module_details(&self, login_response: &tucant_types::LoginResponse, revalidation_strategy: RevalidationStrategy, request: tucant_types::moduledetails::ModuleDetailsRequest) -> Result<tucant_types::moduledetails::ModuleDetailsResponse, TucanError> {
        module_details(self, login_response, revalidation_strategy, request).await
    }

    async fn course_details(&self, login_response: &tucant_types::LoginResponse, revalidation_strategy: RevalidationStrategy, request: tucant_types::coursedetails::CourseDetailsRequest) -> Result<tucant_types::coursedetails::CourseDetailsResponse, TucanError> {
        course_details(self, login_response, revalidation_strategy, request).await
    }

    async fn vv(&self, login_response: Option<&tucant_types::LoginResponse>, revalidation_strategy: RevalidationStrategy, action: ActionRequest) -> Result<Vorlesungsverzeichnis, TucanError> {
        vv(self, login_response, revalidation_strategy, action).await
    }
}

#[cfg(test)]
mod tests {
    use tucant_types::{LoginRequest, LoginResponse, RevalidationStrategy, TucanError, coursedetails::CourseDetailsRequest, moduledetails::ModuleDetailsRequest};

    use crate::{Tucan, TucanConnector, externalpages::welcome::welcome, login::login, root::root, startpage_dispatch::one::startpage_dispatch_1};

    #[tokio::test]
    pub async fn login_incorrect() {
        let tucan = TucanConnector::new_test().await.unwrap();
        assert!(matches!(login(&tucan.client, &LoginRequest { username: "not_found".to_owned(), password: "not_correct".to_owned() },).await, Err(TucanError::InvalidCredentials)));
    }

    #[tokio::test]
    pub async fn test_root_page() {
        let tucan = TucanConnector::new_test().await.unwrap();
        root(&tucan).await.unwrap();
    }

    /// /
    /// redirects to
    /// /scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=STARTPAGE_DISPATCH&ARGUMENTS=-N000000000000001
    #[tokio::test]
    pub async fn test_startpage_dispatch_1() {
        let tucan = TucanConnector::new_test().await.unwrap();
        startpage_dispatch_1(&tucan).await.unwrap();
    }

    /// /scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=STARTPAGE_DISPATCH&ARGUMENTS=-N000000000000001
    /// redirects to
    /// /scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=EXTERNALPAGES&ARGUMENTS=-N000000000000001,-N000344,-Awelcome
    #[tokio::test]
    pub async fn test_welcome() {
        let tucan = TucanConnector::new_test().await.unwrap();
        welcome(&tucan).await.unwrap();
    }

    #[tokio::test]
    pub async fn module_keine_leistungskombination() {
        let tucan = TucanConnector::new_test().await.unwrap();
        let _result = tucan.module_details(&LoginResponse { id: 1, cookie_cnsc: String::new() }, RevalidationStrategy::default(), ModuleDetailsRequest::parse("-N383723477792938")).await.unwrap();
    }

    #[tokio::test]
    pub async fn module_leistungskombination() {
        let tucan = TucanConnector::new_test().await.unwrap();
        let _result = tucan.module_details(&LoginResponse { id: 1, cookie_cnsc: String::new() }, RevalidationStrategy::default(), ModuleDetailsRequest::parse("-N374884241922478")).await.unwrap();
    }

    #[tokio::test]
    pub async fn course_1() {
        let tucan = TucanConnector::new_test().await.unwrap();
        let _result = tucan.course_details(&LoginResponse { id: 1, cookie_cnsc: String::new() }, RevalidationStrategy::default(), CourseDetailsRequest::parse("-N0,-N389955196599934,-N389955196524935,-N0,-N0,-N3")).await.unwrap();
    }

    #[tokio::test]
    pub async fn course_2() {
        let tucan = TucanConnector::new_test().await.unwrap();
        let _result = tucan.course_details(&LoginResponse { id: 1, cookie_cnsc: String::new() }, RevalidationStrategy::default(), CourseDetailsRequest::parse("-N0,-N389955196291846,-N389955196210847,-N0,-N0,-N3")).await.unwrap();
    }

    #[tokio::test]
    pub async fn course_3() {
        let tucan = TucanConnector::new_test().await.unwrap();
        let _result = tucan.course_details(&LoginResponse { id: 1, cookie_cnsc: String::new() }, RevalidationStrategy::default(), CourseDetailsRequest::parse("-N0,-N389947398808423,-N389947398839424,-N0,-N0,-N3")).await.unwrap();
    }

    #[tokio::test]
    pub async fn course_4() {
        let tucan = TucanConnector::new_test().await.unwrap();
        let _result = tucan.course_details(&LoginResponse { id: 1, cookie_cnsc: String::new() }, RevalidationStrategy::default(), CourseDetailsRequest::parse("-N0,-N389043269698095,-N389043269646096,-N0,-N0,-N3")).await.unwrap();
    }

    #[tokio::test]
    pub async fn course_5() {
        let tucan = TucanConnector::new_test().await.unwrap();
        let _result = tucan.course_details(&LoginResponse { id: 1, cookie_cnsc: String::new() }, RevalidationStrategy::default(), CourseDetailsRequest::parse("-N0,-N392125895008100,-N392125895040101,-N0,-N0,-N3")).await.unwrap();
    }
}

#[cfg(all(test, not(feature = "authenticated_tests")))]
mod authenticated_tests {

    #[test]
    #[ignore = "feature authenticated_tests disabled"]
    pub const fn authenticated_tests() {}
}

#[cfg(all(test, feature = "authenticated_tests"))]
mod authenticated_tests {
    use tucant_types::{LoginRequest, RevalidationStrategy, registration::AnmeldungRequest};

    use crate::{Tucan, TucanConnector, courseresults::courseresults, examresults::examresults, login::login, mlsstart::start_page::after_login, mycourses::mycourses, mydocuments::mydocuments, myexams::myexams, mymodules::mymodules, registration::index::anmeldung, startpage_dispatch::after_login::redirect_after_login};

    #[tokio::test]
    pub async fn test_login() {
        dotenvy::dotenv().unwrap();
        let tucan = TucanConnector::new_test().await.unwrap();
        assert!(
            (login(
                &tucan.client,
                &LoginRequest {
                    username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
                    password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing")
                },
            )
            .await)
                .is_ok()
        );
    }

    #[tokio::test]
    pub async fn test_redirect_after_login() {
        dotenvy::dotenv().unwrap();
        let tucan = TucanConnector::new_test().await.unwrap();
        let login_response = login(
            &tucan.client,
            &LoginRequest {
                username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
                password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing"),
            },
        )
        .await
        .unwrap();
        redirect_after_login(&tucan, login_response).await.unwrap();
    }

    #[tokio::test]
    pub async fn test_mlsstart() {
        dotenvy::dotenv().unwrap();
        let tucan = TucanConnector::new_test().await.unwrap();
        let login_response = login(
            &tucan.client,
            &LoginRequest {
                username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
                password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing"),
            },
        )
        .await
        .unwrap();
        after_login(&tucan, &login_response, RevalidationStrategy::default()).await.unwrap();
    }

    #[tokio::test]
    pub async fn test_registration() {
        dotenvy::dotenv().unwrap();
        let tucan = TucanConnector::new_test().await.unwrap();
        let login_response = login(
            &tucan.client,
            &LoginRequest {
                username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
                password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing"),
            },
        )
        .await
        .unwrap();
        let _response = anmeldung(&tucan, &login_response, RevalidationStrategy::default(), AnmeldungRequest::default()).await.unwrap();
    }

    #[tokio::test]
    pub async fn vv_top_level() {
        dotenvy::dotenv().unwrap();
        let tucan = TucanConnector::new_test().await.unwrap();
        let login_response = login(
            &tucan.client,
            &LoginRequest {
                username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
                password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing"),
            },
        )
        .await
        .unwrap();
        let action = tucan.after_login(&login_response, RevalidationStrategy::default()).await.unwrap().logged_in_head.vorlesungsverzeichnis_url;
        let _result = tucan.vv(Some(&login_response), RevalidationStrategy::default(), action).await.unwrap();
    }

    #[tokio::test]
    pub async fn vv_first_level() {
        dotenvy::dotenv().unwrap();
        let tucan = TucanConnector::new_test().await.unwrap();
        let login_response = login(
            &tucan.client,
            &LoginRequest {
                username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
                password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing"),
            },
        )
        .await
        .unwrap();
        let action = tucan.after_login(&login_response, RevalidationStrategy::default()).await.unwrap().logged_in_head.vorlesungsverzeichnis_url;
        let result = tucan.vv(Some(&login_response), RevalidationStrategy::default(), action).await.unwrap().entries[0].clone().1;
        let _result = tucan.vv(Some(&login_response), RevalidationStrategy::default(), result).await.unwrap();
    }

    #[tokio::test]
    pub async fn vv_first_level_4_courses() {
        dotenvy::dotenv().unwrap();
        let tucan = TucanConnector::new_test().await.unwrap();
        let login_response = login(
            &tucan.client,
            &LoginRequest {
                username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
                password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing"),
            },
        )
        .await
        .unwrap();
        let action = tucan.after_login(&login_response, RevalidationStrategy::default()).await.unwrap().logged_in_head.vorlesungsverzeichnis_url;
        let result = tucan.vv(Some(&login_response), RevalidationStrategy::default(), action).await.unwrap().entries[4].clone().1;
        let _result = tucan.vv(Some(&login_response), RevalidationStrategy::default(), result).await.unwrap();
    }

    #[tokio::test]
    pub async fn vv_first_level_all() {
        dotenvy::dotenv().unwrap();
        let tucan = TucanConnector::new_test().await.unwrap();
        let login_response = login(
            &tucan.client,
            &LoginRequest {
                username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
                password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing"),
            },
        )
        .await
        .unwrap();
        let action = tucan.after_login(&login_response, RevalidationStrategy::default()).await.unwrap().logged_in_head.vorlesungsverzeichnis_url;
        for (_title, action) in tucan.vv(Some(&login_response), RevalidationStrategy::default(), action).await.unwrap().entries {
            println!("{action}");
            let _result = tucan.vv(Some(&login_response), RevalidationStrategy::default(), action).await.unwrap();
        }
    }

    #[tokio::test]
    pub async fn test_mymodules() {
        dotenvy::dotenv().unwrap();
        let tucan = TucanConnector::new_test().await.unwrap();
        let login_response = login(
            &tucan.client,
            &LoginRequest {
                username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
                password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing"),
            },
        )
        .await
        .unwrap();
        tucan.my_modules(&login_response, RevalidationStrategy::default()).await.unwrap();
    }

    #[tokio::test]
    pub async fn test_mycourses() {
        dotenvy::dotenv().unwrap();
        let tucan = TucanConnector::new_test().await.unwrap();
        let login_response = login(
            &tucan.client,
            &LoginRequest {
                username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
                password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing"),
            },
        )
        .await
        .unwrap();
        mycourses(&tucan, &login_response, RevalidationStrategy::default()).await.unwrap();
    }

    #[tokio::test]
    pub async fn test_myexams() {
        dotenvy::dotenv().unwrap();
        let tucan = TucanConnector::new_test().await.unwrap();
        let login_response = login(
            &tucan.client,
            &LoginRequest {
                username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
                password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing"),
            },
        )
        .await
        .unwrap();
        myexams(&tucan, &login_response).await.unwrap();
    }

    #[tokio::test]
    pub async fn test_courseresults() {
        dotenvy::dotenv().unwrap();
        let tucan = TucanConnector::new_test().await.unwrap();
        let login_response = login(
            &tucan.client,
            &LoginRequest {
                username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
                password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing"),
            },
        )
        .await
        .unwrap();
        courseresults(&tucan, &login_response).await.unwrap();
    }

    #[tokio::test]
    pub async fn test_examresults() {
        dotenvy::dotenv().unwrap();
        let tucan = TucanConnector::new_test().await.unwrap();
        let login_response = login(
            &tucan.client,
            &LoginRequest {
                username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
                password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing"),
            },
        )
        .await
        .unwrap();
        examresults(&tucan, &login_response).await.unwrap();
    }

    #[tokio::test]
    pub async fn test_mydocuments() {
        dotenvy::dotenv().unwrap();
        let tucan = TucanConnector::new_test().await.unwrap();
        let login_response = login(
            &tucan.client,
            &LoginRequest {
                username: std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing"),
                password: std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing"),
            },
        )
        .await
        .unwrap();
        mydocuments(&tucan, &login_response).await.unwrap();
    }
}
