pub mod coursedetails;
pub mod courseprep;
pub mod courseresults;
pub mod enhanced_module_results;
pub mod examresults;
pub mod gradeoverview;
pub mod mlsstart;
pub mod moduledetails;
pub mod mycourses;
pub mod mydocuments;
pub mod myexams;
pub mod mymodules;
pub mod registration;
pub mod student_result;
pub mod vv;

use std::{convert::Infallible, fmt::Display, str::FromStr};

use axum_core::response::{IntoResponse, Response};
use coursedetails::{CourseDetailsRequest, CourseDetailsResponse};
use courseresults::ModuleResultsResponse;
use dynosaur::dynosaur;
use examresults::ExamResultsResponse;
use mlsstart::MlsStart;
use moduledetails::{ModuleDetailsRequest, ModuleDetailsResponse};
use mycourses::MyCoursesResponse;
use mydocuments::MyDocumentsResponse;
use myexams::MyExamsResponse;
use mymodules::MyModulesResponse;
use registration::{AnmeldungRequest, AnmeldungResponse};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use student_result::StudentResultResponse;
use utoipa::ToSchema;
use vv::{ActionRequest, Vorlesungsverzeichnis};

use crate::{
    enhanced_module_results::EnhancedModuleResultsResponse,
    gradeoverview::{GradeOverviewRequest, GradeOverviewResponse},
    student_result::StudentResultState,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum Semester {
    Sommersemester,
    Wintersemester,
}

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
    pub vvs: Vec<(String, ActionRequest)>,
    pub archiv_links: Vec<(String, ActionRequest, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct InstructorImage {
    pub imgsrc: String,
    pub alt: String,
}

#[derive(thiserror::Error, Debug)]
pub enum TucanError {
    #[error("{0}")]
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
    #[error("Login required")]
    LoginRequired,
}

impl IntoResponse for TucanError {
    fn into_response(self) -> Response {
        match self {
            Self::Http(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
            Self::Io(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
            Self::Timeout => (StatusCode::UNAUTHORIZED, "session timeout").into_response(),
            Self::AccessDenied => (StatusCode::FORBIDDEN, "access denied").into_response(),
            Self::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "invalid credentials").into_response()
            }
            Self::NotCached => (StatusCode::NOT_FOUND, "not cached").into_response(),
            Self::LoginRequired => (StatusCode::UNAUTHORIZED, "login required").into_response(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct RevalidationStrategy {
    /// Try the cache first if age is not larger than `max_age` seconds, then
    /// try network. `max_age` = 0 means never try cache and `max_age` =
    /// `i64::MAX` means always try cache first.
    pub max_age: i64,
    /// If `invalidate_dependents` is None, then network is never used but
    /// failure is returned.
    pub invalidate_dependents: Option<bool>,
}

impl Default for RevalidationStrategy {
    fn default() -> Self {
        Self {
            max_age: 0,
            invalidate_dependents: Some(false),
        }
    }
}

impl RevalidationStrategy {
    #[must_use]
    pub const fn cache() -> Self {
        Self {
            max_age: i64::MAX,
            invalidate_dependents: Some(true),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct SemesterId(String);

impl FromStr for SemesterId {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "999" {
            Ok(Self("all".to_owned()))
        } else {
            Ok(Self(s.to_owned()))
        }
    }
}

impl std::fmt::Display for SemesterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SemesterId {
    #[must_use]
    pub fn all() -> Self {
        Self("all".to_owned())
    }

    #[must_use]
    pub fn current() -> Self {
        Self("current".to_owned())
    }

    #[must_use]
    pub const fn inner(&self) -> &String {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Semesterauswahl {
    pub name: String,
    pub value: SemesterId,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum Grade {
    G1_0,
    G1_3,
    G1_7,
    G2_0,
    G2_3,
    G2_7,
    G3_0,
    G3_3,
    G3_7,
    G4_0,
    G5_0,
    B,
    NB,
}

impl Grade {
    #[must_use]
    pub const fn long_text(&self) -> &str {
        match self {
            Self::G1_0 | Self::G1_3 => "sehr gut",
            Self::G1_7 | Self::G2_0 | Self::G2_3 => "gut",
            Self::G2_7 | Self::G3_0 | Self::G3_3 => "befriedigend",
            Self::G3_7 | Self::G4_0 => "ausreichend",
            Self::B => "bestanden",
            Self::G5_0 | Self::NB => "nicht bestanden",
        }
    }
}

impl FromStr for Grade {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "1,0" => Self::G1_0,
            "1,3" => Self::G1_3,
            "1,7" => Self::G1_7,
            "2,0" => Self::G2_0,
            "2,3" => Self::G2_3,
            "2,7" => Self::G2_7,
            "3,0" => Self::G3_0,
            "3,3" => Self::G3_3,
            "3,7" => Self::G3_7,
            "4,0" => Self::G4_0,
            "5,0" => Self::G5_0,
            "b" => Self::B,
            "nb" => Self::NB,
            s => panic!("{}", s),
        })
    }
}

impl Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::G1_0 => write!(f, "1,0"),
            Self::G1_3 => write!(f, "1,3"),
            Self::G1_7 => write!(f, "1,7"),
            Self::G2_0 => write!(f, "2,0"),
            Self::G2_3 => write!(f, "2,3"),
            Self::G2_7 => write!(f, "2,7"),
            Self::G3_0 => write!(f, "3,0"),
            Self::G3_3 => write!(f, "3,3"),
            Self::G3_7 => write!(f, "3,7"),
            Self::G4_0 => write!(f, "4,0"),
            Self::G5_0 => write!(f, "5,0"),
            Self::B => write!(f, "b"),
            Self::NB => write!(f, "nb"),
        }
    }
}

// TODO can this ever store 5,0 or nb? or is it incomplete then?
// maybe when you failed your last attempt?
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum LeistungsspiegelGrade {
    Grade(Grade),
    /// Krankschreibung?
    Unvollständig,
    /// Note zu spät oder Thesis
    Offen,
    /// Validierung
    BestandenOhneNote,
    /// DO NOT ASK, wenn Summe Bereich noch keinen Haken hat, ist das dort
    OffenerBereich,
}

impl From<(Option<&str>, StudentResultState)> for LeistungsspiegelGrade {
    fn from(s: (Option<&str>, StudentResultState)) -> Self {
        match s {
            (Some("unvollständig"), StudentResultState::Unvollstaendig) => Self::Unvollständig,
            (None, StudentResultState::Offen) => Self::Offen,
            (None, StudentResultState::Bestanden) => Self::BestandenOhneNote,
            (None, StudentResultState::OffenerBereich) => Self::OffenerBereich,
            (Some(s), StudentResultState::Bestanden | StudentResultState::NichtBestanden) => {
                Self::Grade(Grade::from_str(s).unwrap())
            }
            _ => panic!("{s:?}"),
        }
    }
}

impl Display for LeistungsspiegelGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unvollständig => write!(f, "unvollständig"),
            Self::Offen => write!(f, "offen"),
            Self::BestandenOhneNote => write!(f, "bestanden ohne Note"),
            Self::OffenerBereich => write!(f, "offener Bereich"),
            Self::Grade(grade) => write!(f, "{grade}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum ExamResultsGrade {
    Grade(Grade),
    /// vermutlich verspätete Note
    NochNichtErbracht,
    Krankschreibung,
}

impl FromStr for ExamResultsGrade {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Noch nicht erbracht" => Self::NochNichtErbracht,
            "Krankschreibung" => Self::Krankschreibung,
            s => Self::Grade(Grade::from_str(s).unwrap()),
        })
    }
}

impl Display for ExamResultsGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NochNichtErbracht => write!(f, "Noch nicht erbracht"),
            Self::Krankschreibung => write!(f, "Krankschreibung"),
            Self::Grade(grade) => write!(f, "{grade}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum ModuleGrade {
    Grade(Grade),
    /// Entweder erst in einem späteren Semester abgeschlossen oder noch gar
    /// nicht abgeschlossen
    NochNichtGesetzt,
    /// Probably only used for Validierung
    BestandenOhneNote,
    // Krankschreibung?
    Unvollständig,
}

impl From<(Option<&str>, Option<&str>)> for ModuleGrade {
    fn from(s: (Option<&str>, Option<&str>)) -> Self {
        match s {
            (Some("noch nicht gesetzt"), None) => Self::NochNichtGesetzt,
            (None, Some("bestanden")) => Self::BestandenOhneNote,
            (Some(s), Some("bestanden")) => Self::Grade(Grade::from_str(s).unwrap()),
            (None, Some("unvollständig")) => Self::Unvollständig,
            _ => panic!("{s:?}"),
        }
    }
}

impl Display for ModuleGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NochNichtGesetzt => write!(f, "noch nicht gesetzt"),
            Self::BestandenOhneNote => write!(f, "bestanden ohne Note"),
            Self::Unvollständig => write!(f, "unvollständig"),
            Self::Grade(grade) => write!(f, "{grade}"),
        }
    }
}

pub const CONCURRENCY: usize = 10;

#[dynosaur(pub DynTucan = dyn(box) Tucan)]
pub trait Tucan: Send + Sync {
    fn login(
        &self,
        request: LoginRequest,
    ) -> impl std::future::Future<Output = Result<LoginResponse, TucanError>>;

    fn welcome(&self) -> impl std::future::Future<Output = Result<LoggedOutHead, TucanError>>;

    fn after_login(
        &self,
        request: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
    ) -> impl std::future::Future<Output = Result<MlsStart, TucanError>>;

    fn logout(
        &self,
        request: &LoginResponse,
    ) -> impl std::future::Future<Output = Result<(), TucanError>>;

    fn my_modules(
        &self,
        request: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        semester: SemesterId,
    ) -> impl std::future::Future<Output = Result<MyModulesResponse, TucanError>>;

    fn my_courses(
        &self,
        request: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        semester: SemesterId,
    ) -> impl std::future::Future<Output = Result<MyCoursesResponse, TucanError>>;

    fn my_exams(
        &self,
        request: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        semester: SemesterId,
    ) -> impl std::future::Future<Output = Result<MyExamsResponse, TucanError>>;

    fn exam_results(
        &self,
        request: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        semester: SemesterId,
    ) -> impl std::future::Future<Output = Result<ExamResultsResponse, TucanError>>;

    fn course_results(
        &self,
        request: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        semester: SemesterId,
    ) -> impl std::future::Future<Output = Result<ModuleResultsResponse, TucanError>>;

    fn my_documents(
        &self,
        request: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
    ) -> impl std::future::Future<Output = Result<MyDocumentsResponse, TucanError>>;

    fn anmeldung(
        &self,
        login_response: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        request: AnmeldungRequest,
    ) -> impl std::future::Future<Output = Result<AnmeldungResponse, TucanError>> + Send;

    fn module_details(
        &self,
        login_response: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        request: ModuleDetailsRequest,
    ) -> impl std::future::Future<Output = Result<ModuleDetailsResponse, TucanError>>;

    fn course_details(
        &self,
        login_response: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        request: CourseDetailsRequest,
    ) -> impl std::future::Future<Output = Result<CourseDetailsResponse, TucanError>>;

    fn vv(
        &self,
        login_response: Option<&LoginResponse>,
        revalidation_strategy: RevalidationStrategy,
        action: ActionRequest,
    ) -> impl std::future::Future<Output = Result<Vorlesungsverzeichnis, TucanError>>;

    fn student_result(
        &self,
        login_response: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        course_of_study: u64,
    ) -> impl std::future::Future<Output = Result<StudentResultResponse, TucanError>>;

    fn gradeoverview(
        &self,
        login_response: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        gradeoverview: GradeOverviewRequest,
    ) -> impl std::future::Future<Output = Result<GradeOverviewResponse, TucanError>>;

    fn enhanced_module_results(
        &self,
        login_response: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        semester: SemesterId,
    ) -> impl std::future::Future<Output = Result<EnhancedModuleResultsResponse, TucanError>> {
        async move {
            // SemesterId::all() will internally loop
            if semester == SemesterId::all() {
                let all_semesters = self
                    .course_results(login_response, revalidation_strategy, SemesterId::current())
                    .await?;
                for semester in all_semesters.semester {
                    let module_results_in_semester = self
                        .course_results(login_response, revalidation_strategy, semester.value)
                        .await?;
                }
            } else {
                let module_results = self
                    .course_results(login_response, revalidation_strategy, semester.clone())
                    .await?;
            }
            let modules = self
                .my_modules(login_response, revalidation_strategy, semester)
                .await?;
            // now we need to combine them
            Ok(todo!())
        }
    }
}
