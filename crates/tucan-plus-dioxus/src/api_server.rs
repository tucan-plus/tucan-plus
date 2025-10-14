use fragile::Fragile;
use reqwest::Client;
use tokio::sync::Semaphore;
use tucan_types::{
    CONCURRENCY, LoginRequest, LoginResponse, RevalidationStrategy, SemesterId, Tucan, TucanError,
    coursedetails::{CourseDetailsRequest, CourseDetailsResponse},
    courseresults::ModuleResultsResponse,
    examresults::ExamResultsResponse,
    gradeoverview::{GradeOverviewRequest, GradeOverviewResponse},
    mlsstart::MlsStart,
    moduledetails::{ModuleDetailsRequest, ModuleDetailsResponse},
    mycourses::MyCoursesResponse,
    mydocuments::MyDocumentsResponse,
    myexams::MyExamsResponse,
    mymodules::MyModulesResponse,
    registration::AnmeldungRequest,
    student_result::StudentResultResponse,
    vv::{ActionRequest, Vorlesungsverzeichnis},
};
use url::Url;

pub struct ApiServerTucan {
    client: Client,
    semaphore: Semaphore,
}

impl Default for ApiServerTucan {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiServerTucan {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(
                    "https://github.com/tucan-plus/tucan-plus \
                     6d824ead1b932515a84995cafd92f97c40c53bc5 Moritz.Hedtke@t-online.de",
                )
                .build()
                .unwrap(),
            semaphore: Semaphore::new(CONCURRENCY),
        }
    }
}

impl Tucan for ApiServerTucan {
    async fn login(&self, request: LoginRequest) -> Result<LoginResponse, TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let response = self
            .client
            .post("http://127.0.0.1:8080/api/v1/login")
            .fetch_credentials_include()
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    fn anmeldung(
        &self,
        _login_response: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        request: AnmeldungRequest,
    ) -> impl Future<Output = Result<tucan_types::registration::AnmeldungResponse, TucanError>> + Send
    {
        Fragile::new(async move {
            let _permit = self.semaphore.acquire().await.unwrap();
            let mut url = Url::parse("http://127.0.0.1:8080/api/v1/registration").unwrap();
            url.path_segments_mut().unwrap().push(request.inner());
            let response = self
                .client
                .get(url)
                .fetch_credentials_include()
                .header(
                    "X-Revalidation-Strategy",
                    serde_json::to_string(&revalidation_strategy).unwrap(),
                )
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            Ok(response)
        })
    }

    async fn module_details(
        &self,
        _login_response: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        request: ModuleDetailsRequest,
    ) -> Result<ModuleDetailsResponse, TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let mut url = Url::parse("http://127.0.0.1:8080/api/v1/module-details").unwrap();
        url.path_segments_mut().unwrap().push(request.inner());
        let response = self
            .client
            .get(url)
            .fetch_credentials_include()
            .header(
                "X-Revalidation-Strategy",
                serde_json::to_string(&revalidation_strategy).unwrap(),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    async fn course_details(
        &self,
        _login_response: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        request: CourseDetailsRequest,
    ) -> Result<CourseDetailsResponse, TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let mut url = Url::parse("http://127.0.0.1:8080/api/v1/course-details").unwrap();
        url.path_segments_mut().unwrap().push(request.inner());
        let response = self
            .client
            .get(url)
            .fetch_credentials_include()
            .header(
                "X-Revalidation-Strategy",
                serde_json::to_string(&revalidation_strategy).unwrap(),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    async fn logout(&self, _request: &LoginResponse) -> Result<(), TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        self.client
            .post("http://127.0.0.1:8080/api/v1/logout")
            .fetch_credentials_include()
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn after_login(
        &self,
        _request: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
    ) -> Result<MlsStart, TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let url = Url::parse("http://127.0.0.1:8080/api/v1/after-login").unwrap();
        let response = self
            .client
            .get(url)
            .fetch_credentials_include()
            .header(
                "X-Revalidation-Strategy",
                serde_json::to_string(&revalidation_strategy).unwrap(),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    async fn my_modules(
        &self,
        _request: &tucan_types::LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        semester: SemesterId,
    ) -> Result<MyModulesResponse, TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let url = Url::parse(&format!(
            "http://127.0.0.1:8080/api/v1/my-modules/{}",
            semester.inner()
        ))
        .unwrap();
        let response = self
            .client
            .get(url)
            .fetch_credentials_include()
            .header(
                "X-Revalidation-Strategy",
                serde_json::to_string(&revalidation_strategy).unwrap(),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    async fn my_courses(
        &self,
        _request: &tucan_types::LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        semester: SemesterId,
    ) -> Result<MyCoursesResponse, TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let url = Url::parse(&format!(
            "http://127.0.0.1:8080/api/v1/my-courses/{}",
            semester.inner()
        ))
        .unwrap();
        let response = self
            .client
            .get(url)
            .fetch_credentials_include()
            .header(
                "X-Revalidation-Strategy",
                serde_json::to_string(&revalidation_strategy).unwrap(),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    async fn my_exams(
        &self,
        _request: &tucan_types::LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        semester: SemesterId,
    ) -> Result<MyExamsResponse, TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let url = Url::parse(&format!(
            "http://127.0.0.1:8080/api/v1/my-exams/{}",
            semester.inner()
        ))
        .unwrap();
        let response = self
            .client
            .get(url)
            .fetch_credentials_include()
            .header(
                "X-Revalidation-Strategy",
                serde_json::to_string(&revalidation_strategy).unwrap(),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    async fn exam_results(
        &self,
        _request: &tucan_types::LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        semester: SemesterId,
    ) -> Result<ExamResultsResponse, TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let url = Url::parse(&format!(
            "http://127.0.0.1:8080/api/v1/exam-results/{}",
            semester.inner()
        ))
        .unwrap();
        let response = self
            .client
            .get(url)
            .fetch_credentials_include()
            .header(
                "X-Revalidation-Strategy",
                serde_json::to_string(&revalidation_strategy).unwrap(),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    async fn course_results(
        &self,
        _request: &tucan_types::LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        semester: SemesterId,
    ) -> Result<ModuleResultsResponse, TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let url = Url::parse(&format!(
            "http://127.0.0.1:8080/api/v1/course-results/{}",
            semester.inner()
        ))
        .unwrap();
        let response = self
            .client
            .get(url)
            .fetch_credentials_include()
            .header(
                "X-Revalidation-Strategy",
                serde_json::to_string(&revalidation_strategy).unwrap(),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    async fn my_documents(
        &self,
        _request: &tucan_types::LoginResponse,
        revalidation_strategy: RevalidationStrategy,
    ) -> Result<MyDocumentsResponse, TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let url = Url::parse("http://127.0.0.1:8080/api/v1/my-documents").unwrap();
        let response = self
            .client
            .get(url)
            .fetch_credentials_include()
            .header(
                "X-Revalidation-Strategy",
                serde_json::to_string(&revalidation_strategy).unwrap(),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    async fn vv(
        &self,
        _login_response: Option<&LoginResponse>,
        revalidation_strategy: RevalidationStrategy,
        action: ActionRequest,
    ) -> Result<Vorlesungsverzeichnis, TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let mut url = Url::parse("http://127.0.0.1:8080/api/v1/vv").unwrap();
        url.path_segments_mut().unwrap().push(action.inner());
        let response = self
            .client
            .get(url)
            .fetch_credentials_include()
            .header(
                "X-Revalidation-Strategy",
                serde_json::to_string(&revalidation_strategy).unwrap(),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    async fn student_result(
        &self,
        _login_response: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        course_of_study: u64,
    ) -> Result<StudentResultResponse, TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let mut url = Url::parse("http://127.0.0.1:8080/api/v1/student-result").unwrap();
        url.path_segments_mut()
            .unwrap()
            .push(&course_of_study.to_string());
        let response = self
            .client
            .get(url)
            .fetch_credentials_include()
            .header(
                "X-Revalidation-Strategy",
                serde_json::to_string(&revalidation_strategy).unwrap(),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }

    async fn welcome(&self) -> Result<tucan_types::LoggedOutHead, TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        todo!()
    }

    async fn gradeoverview(
        &self,
        _login_response: &LoginResponse,
        revalidation_strategy: RevalidationStrategy,
        gradeoverview: GradeOverviewRequest,
    ) -> Result<GradeOverviewResponse, TucanError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let url = Url::parse(&format!(
            "http://127.0.0.1:8080/api/v1/gradeoverview/{}",
            gradeoverview.inner()
        ))
        .unwrap();
        let response = self
            .client
            .get(url)
            .fetch_credentials_include()
            .header(
                "X-Revalidation-Strategy",
                serde_json::to_string(&revalidation_strategy).unwrap(),
            )
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response)
    }
}
