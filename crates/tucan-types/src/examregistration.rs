use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    Semesterauswahl, coursedetails::CourseDetailsRequest, moduledetails::ModuleDetailsRequest,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExamRegistrationResponse {
    pub semester: Vec<Semesterauswahl>,
    //pub exams: Vec<Exam>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExamRegistration {}
