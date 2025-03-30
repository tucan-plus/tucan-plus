pub mod coursedetails;
pub mod mlsstart;
pub mod moduledetails;
pub mod registration;
pub mod vv;

use axum_core::response::{IntoResponse, Response};
use coursedetails::{CourseDetailsRequest, CourseDetailsResponse};
use mlsstart::MlsStart;
use moduledetails::{ModuleDetailsRequest, ModuleDetailsResponse};
use registration::{AnmeldungRequest, AnmeldungResponse};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vv::{ActionRequest, Vorlesungsverzeichnis};

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct LoginResponse {
    pub id: u64,
    pub cookie_cnsc: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct LoggedInHead {
    pub messages_url: String,
    pub vorlesungsverzeichnis_url: ActionRequest,
    pub vv: VorlesungsverzeichnisUrls,
    pub antraege_url: String,
    pub meine_bewerbung_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct LoggedOutHead {
    pub vorlesungsverzeichnis_url: ActionRequest,
    pub vv: VorlesungsverzeichnisUrls,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct VorlesungsverzeichnisUrls {
    pub lehrveranstaltungssuche_url: String,
    pub vvs: Vec<(String, String)>,
    pub archiv_links: Vec<(String, String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct InstructorImage {
    pub imgsrc: String,
    pub alt: String,
}

#[derive(thiserror::Error, Debug)]
pub enum TucanError {
    #[error("HTTP error {0:?}")]
    Http(#[from] reqwest::Error),
    #[error("IO error {0:?}")]
    Io(#[from] std::io::Error),
    #[error("Tucan session timeout")]
    Timeout,
    #[error("Tucan access denied")]
    AccessDenied,
    #[error("Invalid credentials for TUCaN")]
    InvalidCredentials,
    #[error("Not cached")]
    NotCached,
}

impl IntoResponse for TucanError {
    fn into_response(self) -> Response {
        let body = self.to_string();
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

#[derive(Serialize, Deserialize)]
pub struct RevalidationStrategy {
    /// Try the cache first if age is not larger than `max_age` seconds, then try network. max_age = 0 means never try cache and max_age = i64::MAX means always try cache first.
    pub max_age: i64,
    /// If invalidate_dependents is None, then network is never used but failure is returned.
    pub invalidate_dependents: Option<bool>,
}

impl Default for RevalidationStrategy {
    fn default() -> Self {
        Self { max_age: 0, invalidate_dependents: Some(false) }
    }
}

impl RevalidationStrategy {
    pub fn cache() -> Self {
        Self { max_age: i64::MAX, invalidate_dependents: Some(true) }
    }
}

pub trait Tucan {
    fn login(&self, request: LoginRequest) -> impl std::future::Future<Output = Result<LoginResponse, TucanError>>;

    fn welcome(&self) -> impl std::future::Future<Output = Result<LoggedOutHead, TucanError>>;

    fn after_login(&self, request: &LoginResponse, revalidation_strategy: RevalidationStrategy) -> impl std::future::Future<Output = Result<MlsStart, TucanError>>;

    fn logout(&self, request: &LoginResponse) -> impl std::future::Future<Output = Result<(), TucanError>>;

    fn anmeldung(&self, login_response: LoginResponse, revalidation_strategy: RevalidationStrategy, request: AnmeldungRequest) -> impl std::future::Future<Output = Result<AnmeldungResponse, TucanError>>;

    fn module_details(&self, login_response: &LoginResponse, revalidation_strategy: RevalidationStrategy, request: ModuleDetailsRequest) -> impl std::future::Future<Output = Result<ModuleDetailsResponse, TucanError>>;

    fn course_details(&self, login_response: &LoginResponse, revalidation_strategy: RevalidationStrategy, request: CourseDetailsRequest) -> impl std::future::Future<Output = Result<CourseDetailsResponse, TucanError>>;

    fn vv(&self, login_response: Option<&LoginResponse>, revalidation_strategy: RevalidationStrategy, action: ActionRequest) -> impl std::future::Future<Output = Result<Vorlesungsverzeichnis, TucanError>>;
}
