use std::{convert::Infallible, fmt::Display, str::FromStr, sync::LazyLock};

use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{coursedetails::CourseDetailsRequest, moduledetails::ModuleDetailsRequest};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, Hash)]
pub struct AnmeldungRequest(String);

impl FromStr for AnmeldungRequest {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

impl Display for AnmeldungRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AnmeldungRequest {
    #[must_use]
    pub fn parse(input: &str) -> Self {
        if input.is_empty() || input == "-A" {
            Self("-A".to_owned())
        } else {
            // just_registered_module_id will be added but 0 for just registering a course
            static REGISTRATION_DETAILS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
                Regex::new(r"^-N(?P<n1>\d+),-N0,-N(?P<n3>\d+),-N(?P<n4>\d+)(,-N(?P<just_registered_module_id>\d+,-N0))?$").unwrap()
            });
            let c = &REGISTRATION_DETAILS_REGEX.captures(input).expect(input);
            Self(format!("-N{},-N0,-N{},-N{}", &c["n1"], &c["n3"], &c["n4"],))
        }
    }

    #[must_use]
    pub const fn inner(&self) -> &str {
        self.0.as_str()
    }
}

impl Default for AnmeldungRequest {
    fn default() -> Self {
        Self::parse("")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnmeldungResponse {
    pub path: Vec<(String, AnmeldungRequest)>,
    pub submenus: Vec<(String, AnmeldungRequest)>,
    pub entries: Vec<AnmeldungEntry>,
    pub additional_information: Vec<String>,
    pub studiumsauswahl: Vec<Studiumsauswahl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnmeldungEntry {
    pub module: Option<AnmeldungModule>,
    pub courses: Vec<(Option<AnmeldungExam>, AnmeldungCourse)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum RegistrationState {
    /// I think already completed
    Unknown,
    /// Only possible in other course of study
    NotPossible,
    Registered {
        unregister_link: String,
    },
    NotRegistered {
        register_link: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnmeldungModule {
    pub url: ModuleDetailsRequest,
    pub id: String,
    pub name: String,
    pub lecturer: Option<String>,
    pub date: Option<String>,
    pub limit_and_size: Option<String>,
    pub registration_state: RegistrationState,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnmeldungExam {
    pub name: String,
    pub typ: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnmeldungCourse {
    pub url: CourseDetailsRequest,
    pub id: String,
    pub name: String,
    pub lecturers: Option<String>,
    pub begin_and_end: Option<String>,
    pub registration_until: Option<String>,
    pub limit_and_size: Option<String>,
    pub registration_button_link: RegistrationState,
    pub gefaehrdung_schwangere: bool,
    pub location_or_additional_info: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Studiumsauswahl {
    pub name: String,
    pub value: AnmeldungRequest,
    pub selected: bool,
}
