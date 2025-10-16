use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    ModuleGrade, Semester, Semesterauswahl, gradeoverview::GradeOverviewRequest,
    moduledetails::ModuleDetailsRequest,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GPA {
    pub semester: Semesterauswahl,
    pub course_of_study: String,
    pub average_grade: String,
    pub sum_credits: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EnhancedModuleResultsResponse {
    pub semester: Vec<Semesterauswahl>,
    pub results: Vec<EnhancedModuleResult>,
    pub gpas: Vec<GPA>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EnhancedModuleResult {
    pub year: i32,
    pub semester: Semester,
    pub url: ModuleDetailsRequest,
    pub nr: String,
    pub name: String,
    pub lecturer: String,
    pub grade: ModuleGrade,
    pub credits: String,
    pub pruefungen_url: Option<String>,
    pub average_url: Option<GradeOverviewRequest>,
}
