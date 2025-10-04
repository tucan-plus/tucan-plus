#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod everything {
use std::convert::Infallible;

use axum::{
    Json,
    extract::{FromRequestParts, Path, State},
    http::{StatusCode, request::Parts},
    response::IntoResponse,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use tucan_connector::{TucanConnector, login::login};
use tucan_types::{
    LoginRequest, LoginResponse, RevalidationStrategy, SemesterId, TucanError,
    coursedetails::CourseDetailsRequest,
    examresults::ExamResultsResponse,
    gradeoverview::{GradeOverviewRequest, GradeOverviewResponse},
    moduledetails::ModuleDetailsRequest,
    mycourses::MyCoursesResponse,
    mydocuments::MyDocumentsResponse,
    myexams::MyExamsResponse,
    mymodules::MyModulesResponse,
    registration::{AnmeldungRequest, AnmeldungResponse},
    vv::{ActionRequest, Vorlesungsverzeichnis},
};
use tucan_types::{
    Tucan, coursedetails::CourseDetailsResponse, mlsstart::MlsStart,
    moduledetails::ModuleDetailsResponse,
};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_axum::{router::OpenApiRouter, routes};

// https://docs.rs/utoipa/latest/utoipa/attr.path.html#axum_extras-feature-support-for-axum
// https://github.com/juhaku/utoipa/blob/master/examples/todo-axum/src/main.rs

// http://localhost:3000/swagger-ui/

// http://localhost:3000/api-docs/openapi.json

const TUCAN_PLUS_TAG: &str = "tucan-plus";

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
        tags(
            (name = TUCAN_PLUS_TAG, description = "TUCaN Plus API")
        )
    )]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "cnsc",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("cnsc"))),
            );
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/login",
    tag = TUCAN_PLUS_TAG,
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 500, description = "Some TUCaN error")
    )
)]
pub async fn login_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
    Json(login_request): Json<LoginRequest>,
) -> Result<impl IntoResponse, TucanError> {
    let response = login(&tucan.client, &login_request).await?;

    let jar = jar
        .add(Cookie::build(("id", response.id.to_string())).path("/"))
        .add(Cookie::build(("cnsc", response.cookie_cnsc.to_string())).path("/"));

    Ok((StatusCode::OK, jar, Json(response)).into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/logout",
    tag = TUCAN_PLUS_TAG,
    responses(
        (status = 200, description = "Logout successful", body = ()),
        (status = 500, description = "Some TUCaN error")
    )
)]
pub async fn logout_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
) -> Result<impl IntoResponse, TucanError> {
    let login_response: LoginResponse = LoginResponse {
        id: jar.get("id").unwrap().value().parse().unwrap(),
        cookie_cnsc: jar.get("cnsc").unwrap().value().to_owned(),
    };

    tucan.logout(&login_response).await.unwrap();

    let jar = jar
        .remove(Cookie::build("id").path("/"))
        .remove(Cookie::build("cnsc").path("/"));

    Ok((StatusCode::OK, jar, Json(())).into_response())
}

pub struct RevalidationStrategyW(RevalidationStrategy);

impl<S> FromRequestParts<S> for RevalidationStrategyW
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.headers.get("X-Revalidation-Strategy").map_or_else(
            || Ok(Self(RevalidationStrategy::default())),
            |user_agent| {
                Ok(Self(
                    serde_json::from_str(user_agent.to_str().unwrap()).unwrap(),
                ))
            },
        )
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/registration/{registration}",
    tag = TUCAN_PLUS_TAG,
    params(("registration" = AnmeldungRequest, Path)),
    responses(
        (status = 200, description = "Successful", body = AnmeldungResponse),
        (status = 500, description = "Some TUCaN error")
    )
)]
pub async fn registration_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
    Path(registration): Path<String>,
    revalidation_strategy: RevalidationStrategyW,
) -> Result<impl IntoResponse, TucanError> {
    let login_response: LoginResponse = LoginResponse {
        id: jar.get("id").unwrap().value().parse().unwrap(),
        cookie_cnsc: jar.get("cnsc").unwrap().value().to_owned(),
    };

    let response = tucan
        .anmeldung(
            &login_response,
            revalidation_strategy.0,
            AnmeldungRequest::parse(&registration),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/vv/{vv}",
    tag = TUCAN_PLUS_TAG,
    params(("vv" = ActionRequest, Path)),
    responses(
        (status = 200, description = "Successful", body = Vorlesungsverzeichnis),
        (status = 500, description = "Some TUCaN error")
    )
)]
pub async fn vv_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
    Path(vv): Path<String>,
    revalidation_strategy: RevalidationStrategyW,
) -> Result<impl IntoResponse, TucanError> {
    let login_response = match (jar.get("id"), jar.get("cnsc")) {
        (Some(id), Some(cnsc)) => Some(LoginResponse {
            id: id.value().parse().unwrap(),
            cookie_cnsc: cnsc.value().to_owned(),
        }),
        _ => None,
    };

    let response = tucan
        .vv(
            login_response.as_ref(),
            revalidation_strategy.0,
            ActionRequest::parse(&vv),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/module-details/{module}",
    tag = TUCAN_PLUS_TAG,
    params(("module" = ModuleDetailsRequest, Path)),
    responses(
        (status = 200, description = "Successful", body = ModuleDetailsResponse),
        (status = 500, description = "Some TUCaN error")
    )
)]
pub async fn module_details_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
    Path(module): Path<String>,
    revalidation_strategy: RevalidationStrategyW,
) -> Result<impl IntoResponse, TucanError> {
    let login_response: LoginResponse = LoginResponse {
        id: jar.get("id").unwrap().value().parse().unwrap(),
        cookie_cnsc: jar.get("cnsc").unwrap().value().to_owned(),
    };

    let response = tucan
        .module_details(
            &login_response,
            revalidation_strategy.0,
            ModuleDetailsRequest::parse(&module),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/course-details/{course}",
    tag = TUCAN_PLUS_TAG,
    params(("course" = CourseDetailsRequest, Path)),
    responses(
        (status = 200, description = "Successful", body = CourseDetailsResponse),
        (status = 500, description = "Some TUCaN error")
    )
)]
pub async fn course_details_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
    Path(course): Path<String>,
    revalidation_strategy: RevalidationStrategyW,
) -> Result<impl IntoResponse, TucanError> {
    let login_response: LoginResponse = LoginResponse {
        id: jar.get("id").unwrap().value().parse().unwrap(),
        cookie_cnsc: jar.get("cnsc").unwrap().value().to_owned(),
    };

    let response = tucan
        .course_details(
            &login_response,
            revalidation_strategy.0,
            CourseDetailsRequest::parse(&course),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/after-login",
    tag = TUCAN_PLUS_TAG,
    responses(
        (status = 200, description = "Successful", body = MlsStart),
        (status = 500, description = "Some TUCaN error")
    )
)]
pub async fn after_login_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
    revalidation_strategy: RevalidationStrategyW,
) -> Result<impl IntoResponse, TucanError> {
    let login_response: LoginResponse = LoginResponse {
        id: jar.get("id").unwrap().value().parse().unwrap(),
        cookie_cnsc: jar.get("cnsc").unwrap().value().to_owned(),
    };

    let response = tucan
        .after_login(&login_response, revalidation_strategy.0)
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/my-modules/{semester}",
    tag = TUCAN_PLUS_TAG,
    params(("semester" = SemesterId, Path)),
    responses(
        (status = 200, description = "Successful", body = MyModulesResponse),
        (status = 500, description = "Some TUCaN error")
    )
)]
pub async fn my_modules_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
    revalidation_strategy: RevalidationStrategyW,
    semester: Path<SemesterId>,
) -> Result<impl IntoResponse, TucanError> {
    let login_response: LoginResponse = LoginResponse {
        id: jar.get("id").unwrap().value().parse().unwrap(),
        cookie_cnsc: jar.get("cnsc").unwrap().value().to_owned(),
    };

    let response = tucan
        .my_modules(&login_response, revalidation_strategy.0, semester.0)
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/my-courses/{semester}",
    tag = TUCAN_PLUS_TAG,
    params(("semester" = SemesterId, Path)),
    responses(
        (status = 200, description = "Successful", body = MyCoursesResponse),
        (status = 500, description = "Some TUCaN error")
    )
)]
pub async fn my_courses_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
    revalidation_strategy: RevalidationStrategyW,
    semester: Path<SemesterId>,
) -> Result<impl IntoResponse, TucanError> {
    let login_response: LoginResponse = LoginResponse {
        id: jar.get("id").unwrap().value().parse().unwrap(),
        cookie_cnsc: jar.get("cnsc").unwrap().value().to_owned(),
    };

    let response = tucan
        .my_courses(&login_response, revalidation_strategy.0, semester.0)
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/my-exams/{semester}",
    tag = TUCAN_PLUS_TAG,
    params(("semester" = SemesterId, Path)),
    responses(
        (status = 200, description = "Successful", body = MyExamsResponse),
        (status = 500, description = "Some TUCaN error")
    )
)]
pub async fn my_exams_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
    revalidation_strategy: RevalidationStrategyW,
    semester: Path<SemesterId>,
) -> Result<impl IntoResponse, TucanError> {
    let login_response: LoginResponse = LoginResponse {
        id: jar.get("id").unwrap().value().parse().unwrap(),
        cookie_cnsc: jar.get("cnsc").unwrap().value().to_owned(),
    };

    let response = tucan
        .my_exams(&login_response, revalidation_strategy.0, semester.0)
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/exam-results/{semester}",
    tag = TUCAN_PLUS_TAG,
    params(("semester" = SemesterId, Path)),
    responses(
        (status = 200, description = "Successful", body = ExamResultsResponse),
        (status = 500, description = "Some TUCaN error")
    )
)]
pub async fn exam_results_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
    revalidation_strategy: RevalidationStrategyW,
    semester: Path<SemesterId>,
) -> Result<impl IntoResponse, TucanError> {
    let login_response: LoginResponse = LoginResponse {
        id: jar.get("id").unwrap().value().parse().unwrap(),
        cookie_cnsc: jar.get("cnsc").unwrap().value().to_owned(),
    };

    let response = tucan
        .exam_results(&login_response, revalidation_strategy.0, semester.0)
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/course-results/{semester}",
    tag = TUCAN_PLUS_TAG,
    params(("semester" = SemesterId, Path)),
    responses(
        (status = 200, description = "Successful", body = ExamResultsResponse),
        (status = 500, description = "Some TUCaN error")
    )
)]
pub async fn course_results_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
    revalidation_strategy: RevalidationStrategyW,
    semester: Path<SemesterId>,
) -> Result<impl IntoResponse, TucanError> {
    let login_response: LoginResponse = LoginResponse {
        id: jar.get("id").unwrap().value().parse().unwrap(),
        cookie_cnsc: jar.get("cnsc").unwrap().value().to_owned(),
    };

    let response = tucan
        .course_results(&login_response, revalidation_strategy.0, semester.0)
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/my-documents",
    tag = TUCAN_PLUS_TAG,
    responses(
        (status = 200, description = "Successful", body = MyDocumentsResponse),
        (status = 500, description = "Some TUCaN error")
    )
)]
pub async fn my_documents_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
    revalidation_strategy: RevalidationStrategyW,
) -> Result<impl IntoResponse, TucanError> {
    let login_response: LoginResponse = LoginResponse {
        id: jar.get("id").unwrap().value().parse().unwrap(),
        cookie_cnsc: jar.get("cnsc").unwrap().value().to_owned(),
    };

    let response = tucan
        .my_documents(&login_response, revalidation_strategy.0)
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/student-result/{course-of-study}",
    tag = TUCAN_PLUS_TAG,
    params(("course-of-study" = String, Path)),
    responses(
        (status = 200, description = "Successful", body = ExamResultsResponse),
        (status = 500, description = "Some TUCaN error")
    )
)]
/// set course-of-study to default to get the default one
pub async fn student_result_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
    revalidation_strategy: RevalidationStrategyW,
    course_of_study: Path<String>,
) -> Result<impl IntoResponse, TucanError> {
    let login_response: LoginResponse = LoginResponse {
        id: jar.get("id").unwrap().value().parse().unwrap(),
        cookie_cnsc: jar.get("cnsc").unwrap().value().to_owned(),
    };

    let response = tucan
        .student_result(
            &login_response,
            revalidation_strategy.0,
            if course_of_study.0 == "default" {
                0
            } else {
                course_of_study.0.parse().unwrap()
            },
        )
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/gradeoverview/{gradeoverview}",
    tag = TUCAN_PLUS_TAG,
    params(("gradeoverview" = GradeOverviewRequest, Path)),
    responses(
        (status = 200, description = "Successful", body = GradeOverviewResponse),
        (status = 500, description = "Some TUCaN error")
    )
)]
pub async fn gradeoverview_endpoint(
    State(tucan): State<TucanConnector>,
    jar: CookieJar,
    Path(gradeoverview): Path<GradeOverviewRequest>,
    revalidation_strategy: RevalidationStrategyW,
) -> Result<impl IntoResponse, TucanError> {
    let login_response: LoginResponse = LoginResponse {
        id: jar.get("id").unwrap().value().parse().unwrap(),
        cookie_cnsc: jar.get("cnsc").unwrap().value().to_owned(),
    };

    let response = tucan
        .gradeoverview(&login_response, revalidation_strategy.0, gradeoverview)
        .await?;

    Ok((StatusCode::OK, Json(response)).into_response())
}

pub fn router() -> OpenApiRouter<TucanConnector> {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(login_endpoint))
        .routes(routes!(logout_endpoint))
        .routes(routes!(registration_endpoint))
        .routes(routes!(vv_endpoint))
        .routes(routes!(module_details_endpoint))
        .routes(routes!(course_details_endpoint))
        .routes(routes!(after_login_endpoint))
        .routes(routes!(my_modules_endpoint))
        .routes(routes!(my_courses_endpoint))
        .routes(routes!(my_exams_endpoint))
        .routes(routes!(exam_results_endpoint))
        .routes(routes!(course_results_endpoint))
        .routes(routes!(my_documents_endpoint))
        .routes(routes!(student_result_endpoint))
        .routes(routes!(gradeoverview_endpoint))
}

}

#[cfg(not(target_arch = "wasm32"))]
pub use everything::router;