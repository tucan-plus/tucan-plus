use std::panic;

use dioxus::prelude::*;
use tracing::Level;
use tucan_plus_worker::MyDatabase;
use tucan_types::LoginResponse;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "api")]
pub mod api_server;
pub mod common;
pub mod course_details;
pub mod course_results;
pub mod database_management;
pub mod exam_results;
pub mod export_semester;
pub mod gradeoverview;
pub mod login_component;
pub mod logout_component;
pub mod module_details;
pub mod my_courses;
pub mod my_documents;
pub mod my_exams;
pub mod my_modules;
pub mod my_semester_modules;
pub mod navbar;
pub mod navbar_logged_in;
pub mod navbar_logged_out;
pub mod overview;
pub mod planning;
pub mod registration;
pub mod student_result;
pub mod vv;

use crate::export_semester::FetchAnmeldung;
use crate::export_semester::MigrateV0ToV1;
use crate::navbar::Navbar;
use crate::overview::Overview;
use crate::planning::Planning;
use std::ops::Deref;
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::time::Duration;
use tucan_types::DynTucan;
use tucan_types::gradeoverview::GradeOverviewRequest;
use tucan_types::{
    SemesterId, coursedetails::CourseDetailsRequest, moduledetails::ModuleDetailsRequest,
    registration::AnmeldungRequest, vv::ActionRequest,
};

#[used]
pub static BOOTSTRAP_CSS: Asset = asset!(
    "/assets/bootstrap.css",
    AssetOptions::builder().with_hash_suffix(false)
);

#[used]
pub static APP_MANIFEST: Asset = asset!(
    "/assets/manifest.json",
    AssetOptions::builder().with_hash_suffix(false)
);

#[used]
pub static LOGO_SVG: Asset = asset!(
    "/assets/logo.svg",
    AssetOptions::builder().with_hash_suffix(false)
);

#[used]
pub static LOGO_PNG: Asset = asset!(
    "/assets/logo.png",
    AssetOptions::builder().with_hash_suffix(false)
);

#[used]
pub static WORKER_JS: Asset = asset!(
    "/assets/worker.js",
    AssetOptions::builder().with_hash_suffix(false)
);

#[used]
pub static BOOTSTRAP_JS: Asset = asset!(
    "/assets/bootstrap.bundle.min.js",
    AssetOptions::builder().with_hash_suffix(false)
);

#[used]
pub static BOOTSTRAP_PATCH_JS: Asset = asset!(
    "/assets/bootstrap.patch.js",
    AssetOptions::builder().with_hash_suffix(false)
);

#[used]
pub static DEV_JS: Asset = asset!(
    "/assets/dev.js",
    AssetOptions::builder().with_hash_suffix(false)
);

#[used]
pub static DEV_CSS: Asset = asset!(
    "/assets/dev.css",
    AssetOptions::builder().with_hash_suffix(false)
);

#[derive(Copy, Clone)]
pub struct Anonymize(pub bool);

#[cfg(not(any(
    feature = "desktop",
    feature = "mobile",
    feature = "direct",
    feature = "api"
)))]
pub async fn login_response() -> Option<tucan_types::LoginResponse> {
    None
}

#[cfg(any(feature = "desktop", feature = "mobile"))]
pub async fn login_response() -> Option<tucan_types::LoginResponse> {
    #[cfg(feature = "mobile")]
    keyring_core::set_default_store(
        android_native_keyring_store::AndroidStore::from_ndk_context().unwrap(),
    );

    #[cfg(feature = "desktop")]
    keyring_core::set_default_store(dbus_secret_service_keyring_store::Store::new().unwrap());

    let entry = keyring_core::Entry::new("tucan-plus", "session").ok()?;
    Some(serde_json::from_str(&entry.get_password().ok()?).unwrap())
    //println!("My password is '{}'", password);
    //entry.set_password("topS3cr3tP4$$w0rd").ok()?;
    //println!("could set password");
    //None
}

#[cfg(feature = "direct")]
pub async fn login_response() -> Option<tucan_types::LoginResponse> {
    let session_id = web_extensions::cookies::get(web_extensions::cookies::CookieDetails {
        name: "id".to_owned(),
        partition_key: None,
        store_id: None,
        url: "https://www.tucan.tu-darmstadt.de/scripts".to_owned(),
    })
    .await?
    .value;

    let cnsc = web_extensions::cookies::get(web_extensions::cookies::CookieDetails {
        name: "cnsc".to_owned(),
        url: "https://www.tucan.tu-darmstadt.de/scripts".to_owned(),
        partition_key: None,
        store_id: None,
    })
    .await?
    .value;

    Some(tucan_types::LoginResponse {
        id: session_id.parse().unwrap(),
        cookie_cnsc: cnsc,
    })
}

#[cfg(feature = "api")]
pub async fn login_response() -> Option<tucan_types::LoginResponse> {
    use wasm_bindgen::JsCast;
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let html_document = document.dyn_into::<web_sys::HtmlDocument>().unwrap();
    let cookie = html_document.cookie().unwrap();

    Some(tucan_types::LoginResponse {
        id: cookie::Cookie::split_parse(&cookie)
            .find_map(|cookie| {
                let cookie = cookie.unwrap();
                if cookie.name() == "id" {
                    Some(cookie.value().to_string())
                } else {
                    None
                }
            })?
            .parse()
            .unwrap(),
        cookie_cnsc: cookie::Cookie::split_parse(&cookie).find_map(|cookie| {
            let cookie = cookie.unwrap();
            if cookie.name() == "cnsc" {
                Some(cookie.value().to_string())
            } else {
                None
            }
        })?,
    })
}
use crate::course_details::CourseDetails;
use crate::course_results::CourseResults;
use crate::database_management::ExportDatabase;
use crate::database_management::ImportDatabase;
use crate::exam_results::ExamResults;
use crate::gradeoverview::GradeOverview;
use crate::module_details::ModuleDetails;
use crate::my_courses::MyCourses;
use crate::my_documents::MyDocuments;
use crate::my_exams::MyExams;
use crate::my_modules::MyModules;
use crate::my_semester_modules::MySemesterModules;
use crate::registration::Registration;
use crate::student_result::StudentResult;
use crate::vv::Vorlesungsverzeichnis;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Root {},
    #[route("/:..route")]
    NotFound { route: Vec<String> },
    #[route("/module-details/:module")]
    ModuleDetails { module: ModuleDetailsRequest },
    #[route("/course-details/:course")]
    CourseDetails { course: CourseDetailsRequest },
    #[route("/registration/:registration")]
    Registration { registration: AnmeldungRequest },
    #[route("/overview")]
    Overview {},
    #[route("/vv/:vv")]
    Vorlesungsverzeichnis { vv: ActionRequest },
    #[route("/my-modules/:semester")]
    MyModules { semester: SemesterId },
    #[route("/my-semester-modules/:semester")]
    MySemesterModules { semester: SemesterId },
    #[route("/my-courses/:semester")]
    MyCourses { semester: SemesterId },
    #[route("/my-exams/:semester")]
    MyExams { semester: SemesterId },
    #[route("/exam-results/:semester")]
    ExamResults { semester: SemesterId },
    #[route("/course-results/:semester")]
    CourseResults { semester: SemesterId },
    #[route("/my-documents")]
    MyDocuments {},
    #[route("/student-result/:course_of_study")]
    StudentResult { course_of_study: String },
    #[route("/gradeoverview/:gradeoverview")]
    GradeOverview { gradeoverview: GradeOverviewRequest },
    #[route("/fetch-anmeldung")]
    FetchAnmeldung {},
    #[route("/planning/:course_of_study")]
    Planning { course_of_study: String },
    #[route("/export-database")]
    ExportDatabase {},
    #[route("/import-database")]
    ImportDatabase {},
    #[route("/migrate-v0-to-v1")]
    MigrateV0ToV1 {},
}

#[component]
pub fn NotFound(route: Vec<String>) -> Element {
    rsx! {
        h1 {
            "Page not found"
        }
    }
}

#[component]
pub fn Root() -> Element {
    rsx! {
        div {
            class: "container",
            h1 {
                { "Willkommen bei TUCaN Plus!" }
            }
            p {
                { "Du kannst gerne die " }
                a {
                    href: "https://github.com/tucan-plus/tucan-plus?tab=readme-ov-file#installation",
                    target: "_blank",
                    { "Browsererweiterung installieren" }
                }
                { ", falls Du diese noch nicht verwendest." }
            }
            p {
                { "Der Quellcode dieses Projekts ist unter der AGPL-3.0 Lizenz auf " }
                a {
                    href: "https://github.com/tucan-plus/tucan-plus/",
                    target: "_blank",
                    { "GitHub" }
                }
                { " verfügbar." }
            }
            p {
                { "Du kannst Dir deine " }
                Link {
                    to: Route::Registration {
                        registration: AnmeldungRequest::default(),
                    },
                    { "anmeldbaren Module ansehen" }
                }
                { "." }
            }
            p {
                "Version "
                { git_version::git_version!() }
            }
        }
    }
}

pub struct MyRc<T: ?Sized>(pub Arc<T>);

impl<T: ?Sized> MyRc<T> {
    pub fn new(value: Arc<T>) -> Self {
        Self(value)
    }
}

impl<T: ?Sized> Clone for MyRc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> PartialEq for MyRc<T> {
    fn eq(&self, other: &MyRc<T>) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: ?Sized> Deref for MyRc<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type RcTucanType = MyRc<DynTucan<'static>>;

#[cfg(target_arch = "wasm32")]
pub async fn sleep(duration: Duration) {
    let mut cb = |resolve: js_sys::Function, _reject: js_sys::Function| {
        use wasm_bindgen::JsCast as _;

        let global = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();
        global
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                duration.as_millis().try_into().unwrap(),
            )
            .unwrap();
    };

    let p = js_sys::Promise::new(&mut cb);

    wasm_bindgen_futures::JsFuture::from(p).await.unwrap();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(msg: String);

    fn alert(s: &str);

    type Error;

    #[wasm_bindgen(constructor)]
    fn new() -> Error;

    #[wasm_bindgen(structural, method, getter)]
    fn stack(error: &Error) -> String;
}

// https://github.com/tauri-apps/wry
// https://github.com/tauri-apps/tao/blob/5ac00b57ad3f5c5c7135dde626cb90bc1ad469dc/src/platform_impl/android/ndk_glue.rs#L236

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(main))]
#[cfg_attr(not(target_arch = "wasm32"), tokio::main)]
pub async fn main() {
    // From https://github.com/rustwasm/console_error_panic_hook, licensed under MIT and Apache 2.0
    #[cfg(target_arch = "wasm32")]
    panic::set_hook(Box::new(|info| {
        let mut msg = "Version: ".to_string();
        msg.push_str(git_version::git_version!());
        msg.push('\n');
        msg.push_str(&info.to_string());
        msg.push_str("\n\nStack:\n\n");
        let e = Error::new();
        let stack = e.stack();
        msg.push_str(&stack);
        msg.push_str("\n\n");
        error(msg.clone());
        if web_sys::window().is_some() {
            alert(msg.as_str());
        }
    }));
    #[cfg(target_arch = "wasm32")]
    console_log::init().unwrap();

    dioxus::logger::init(Level::INFO).expect("logger failed to init");

    tracing::info!("tracing works");
    log::info!("logging works");

    #[cfg(target_arch = "wasm32")]
    if web_sys::window().is_some() {
        frontend_main().await
    } else {
        worker_main().await
    }

    #[cfg(not(target_arch = "wasm32"))]
    frontend_main().await
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(feature = "wasm-split", wasm_split::wasm_split(worker))]
async fn worker_main() {
    use std::cell::RefCell;

    use diesel::{Connection as _, SqliteConnection};
    use diesel_migrations::MigrationHarness as _;
    use tucan_plus_worker::MIGRATIONS;
    use wasm_bindgen::{JsCast as _, JsValue, prelude::Closure};
    use web_sys::{BroadcastChannel, MessageEvent};

    let global = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();

    let util = sqlite_wasm_vfs::sahpool::install::<sqlite_wasm_rs::WasmOsCallback>(
        &sqlite_wasm_vfs::sahpool::OpfsSAHPoolCfg::default(),
        true,
    )
    .await
    .unwrap();

    let mut connection = SqliteConnection::establish("file:tucan-plus.db?mode=rwc").unwrap();

    connection.run_pending_migrations(MIGRATIONS).unwrap();

    let connection = RefCell::new(connection);

    let broadcast_channel = BroadcastChannel::new("global").unwrap();

    let closure: Closure<dyn Fn(MessageEvent)> = Closure::new(move |event: MessageEvent| {
        use log::info;
        use tucan_plus_worker::MessageWithId;

        let value: MessageWithId = serde_wasm_bindgen::from_value(event.data()).unwrap();

        let result = if let tucan_plus_worker::RequestResponseEnum::ImportDatabaseRequest(import) =
            value.message
        {
            let old_connection =
                connection.replace(SqliteConnection::establish(":memory:").unwrap());
            drop(old_connection);
            util.delete_db("tucan-plus.db").unwrap();
            util.import_db("tucan-plus.db", &import.data).unwrap();
            connection.replace(SqliteConnection::establish("file:tucan-plus.db?mode=rwc").unwrap());
            connection
                .borrow_mut()
                .run_pending_migrations(MIGRATIONS)
                .unwrap();
            JsValue::null()
        } else {
            value.message.execute(&mut connection.borrow_mut())
        };

        let temporary_broadcast_channel = BroadcastChannel::new(&value.id).unwrap();

        temporary_broadcast_channel.post_message(&result).unwrap();
    });
    broadcast_channel
        .add_event_listener_with_callback("message", closure.as_ref().unchecked_ref())
        .unwrap();

    //util.export_db("tucan-plus.db").unwrap();
    closure.forget();

    global.post_message(&JsValue::from_str("ready")).unwrap();
}

#[cfg_attr(feature = "wasm-split", wasm_split::wasm_split(frontend))]
async fn frontend_main() {
    let worker = MyDatabase::wait_for_worker(); // maybe move this before the wasm-split point?

    let anonymize = {
        #[cfg(feature = "direct")]
        {
            // TODO we need to update this when you update the value in the extension
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &"anonymize".into(), &false.into()).unwrap();
            let storage = web_extensions_sys::chrome().storage().sync();
            let result = storage.get(&obj).await.unwrap();
            js_sys::Reflect::get(&result, &"anonymize".into())
                .unwrap()
                .as_bool()
                .unwrap()
        }
        #[cfg(not(feature = "direct"))]
        false
    };

    // Does not work in Firefox extensions
    // web_sys::window().unwrap().navigator().service_worker().register(&
    // SERVICE_WORKER_JS.to_string());

    let launcher = dioxus::LaunchBuilder::new();

    let launcher = launcher.with_context(worker.clone());

    #[cfg(feature = "web")]
    let launcher = launcher.with_cfg(
        dioxus::web::Config::new().history(std::rc::Rc::new(dioxus::web::HashHistory::new(false))),
    );

    // TODO FIXME also use this for web and here we should have access to the asset
    // paths?
    #[cfg(feature = "desktop")]
    let launcher = launcher.with_cfg(
        dioxus::desktop::Config::new()
            .with_custom_index(include_str!("../index.html").replace("{base_path}", ".")),
    );

    #[cfg(feature = "mobile")]
    let launcher = launcher.with_cfg(
        dioxus::mobile::Config::new()
            .with_custom_index(include_str!("../index.html").replace("{base_path}", ".")),
    );

    let login_response = login_response().await;
    let launcher = launcher.with_context(login_response);

    #[cfg(feature = "api")]
    let launcher = launcher.with_context(RcTucanType::new(tucan_types::DynTucan::new_arc(
        api_server::ApiServerTucan::new(),
    )));

    #[cfg(any(feature = "direct", feature = "desktop", feature = "mobile"))]
    let launcher = launcher.with_context(RcTucanType::new(tucan_types::DynTucan::new_arc(
        tucan_connector::TucanConnector::new(worker).await.unwrap(),
    )));

    let launcher = launcher.with_context(Anonymize(anonymize));
    launcher.launch(App);
}

#[component]
fn App() -> Element {
    let login_response: Option<LoginResponse> = use_context();
    let login_response = use_signal(|| login_response);
    provide_context(login_response);
    rsx! {
        Router::<Route> {
        }
        script {
            src: "http://127.0.0.1:8080/assets/bootstrap.bundle.min.js",
        }
        script {
            src: "http://127.0.0.1:8080/assets/bootstrap.patch.js",
        }
    }
}
