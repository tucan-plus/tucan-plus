use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    Semesterauswahl, coursedetails::CourseDetailsRequest, moduledetails::ModuleDetailsRequest,
    registration::RegistrationState,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExamRegistrationResponse {
    pub semester: Vec<Semesterauswahl>,
    pub exam_registrations: Vec<ExamRegistrationCourse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ExamRegistrationState {
    NotPossible,
    ForceSelected,
    Registered(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExamRegistration {
    pub registration_state: ExamRegistrationState,
    pub date: String,
    pub pruefungsart: String,
    pub examdetail_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExamRegistrationCourse {
    pub registrations: Vec<ExamRegistration>,
    pub course_id: String,
    pub name: String,
    pub ids: String,
}
