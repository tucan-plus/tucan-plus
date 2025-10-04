
#[cfg(not(target_arch = "wasm32"))]
use diesel::r2d2::CustomizeConnection;
use diesel::{prelude::*, upsert::excluded};
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
#[cfg(target_arch = "wasm32")]
use fragile::Fragile;
#[cfg(target_arch = "wasm32")]
use serde::{Serialize, Deserialize, de::DeserializeOwned};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::BroadcastChannel;

use crate::{
    models::{Anmeldung, AnmeldungEntry, CacheEntry, Semester, State},
    schema::{anmeldungen_entries, anmeldungen_plan, cache},
};
use tucan_types::{
    Semesterauswahl,
    courseresults::ModuleResult,
    student_result::{StudentResultLevel, StudentResultResponse},
};

pub mod models;
pub mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[cfg(target_arch = "wasm32")]
pub trait RequestResponse: Serialize + Sized where
    RequestResponseEnum: From<Self>, {
    type Response: DeserializeOwned;
    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response;
}

#[cfg(not(target_arch = "wasm32"))]
pub trait RequestResponse: Sized {
    type Response;
    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response;
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct CacheRequest {
    pub key: String,
}

impl RequestResponse for CacheRequest {
    type Response = Option<CacheEntry>;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        QueryDsl::filter(cache::table, cache::key.eq(&self.key))
            .select(CacheEntry::as_select())
            .get_result(connection)
            .optional()
            .unwrap()
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct StoreCacheRequest(pub CacheEntry);

impl RequestResponse for StoreCacheRequest {
    type Response = ();

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        diesel::insert_into(cache::table)
            .values(&self.0)
            .on_conflict(cache::key)
            .do_update()
            .set(&self.0)
            .execute(connection)
            .unwrap();
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct AnmeldungenRequest {
    pub course_of_study: String,
}

impl RequestResponse for AnmeldungenRequest {
    type Response = Vec<Anmeldung>;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        QueryDsl::filter(
            anmeldungen_plan::table,
            anmeldungen_plan::course_of_study
                .eq(&self.course_of_study)
                .and(anmeldungen_plan::parent.is_null()),
        )
        .select(Anmeldung::as_select())
        .load(connection)
        .unwrap()
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct AnmeldungenRequest2 {
    pub course_of_study: String,
    pub anmeldung: Anmeldung,
}

impl RequestResponse for AnmeldungenRequest2 {
    type Response = Vec<Anmeldung>;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        QueryDsl::filter(
            anmeldungen_plan::table,
            anmeldungen_plan::course_of_study
                .eq(&self.course_of_study)
                .and(anmeldungen_plan::parent.eq(&self.anmeldung.url)),
        )
        .select(Anmeldung::as_select())
        .load(connection)
        .unwrap()
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Fewe {
    pub course_of_study: String,
    pub anmeldung: Anmeldung,
}

impl RequestResponse for Fewe {
    type Response = Vec<AnmeldungEntry>;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        QueryDsl::filter(
            anmeldungen_entries::table,
            anmeldungen_entries::course_of_study
                .eq(&self.course_of_study)
                .and(anmeldungen_entries::anmeldung.eq(&self.anmeldung.url)),
        )
        .select(AnmeldungEntry::as_select())
        .load(connection)
        .unwrap()
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct FEwefweewf {
    pub inserts: Vec<Anmeldung>,
}

impl RequestResponse for FEwefweewf {
    type Response = ();

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        diesel::insert_into(anmeldungen_plan::table)
            .values(&self.inserts)
            .on_conflict((anmeldungen_plan::course_of_study, anmeldungen_plan::url))
            .do_update()
            .set(anmeldungen_plan::parent.eq(excluded(anmeldungen_plan::parent)))
            .execute(connection)
            .unwrap();
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Wlewifhewefwef {
    pub insert: AnmeldungEntry,
}

impl RequestResponse for Wlewifhewefwef {
    type Response = ();

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        diesel::insert_into(anmeldungen_entries::table)
            .values(&self.insert)
            .on_conflict((
                anmeldungen_entries::course_of_study,
                anmeldungen_entries::anmeldung,
                anmeldungen_entries::available_semester,
                anmeldungen_entries::id,
            ))
            .do_update()
            .set((
                // TODO FIXME I think updating does not work
                anmeldungen_entries::state.eq(excluded(anmeldungen_entries::state)),
                (anmeldungen_entries::credits.eq(excluded(anmeldungen_entries::credits))),
            ))
            .execute(connection)
            .unwrap();
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct ChildUrl {
    pub course_of_study: String,
    pub url: String,
    pub name: String,
    pub child: StudentResultLevel,
}

impl RequestResponse for ChildUrl {
    type Response = String;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        diesel::update(QueryDsl::filter(
            anmeldungen_plan::table,
            anmeldungen_plan::course_of_study
                .eq(&self.course_of_study)
                .and(
                    anmeldungen_plan::parent
                        .eq(&self.url)
                        .and(anmeldungen_plan::name.eq(&self.name)),
                ),
        ))
        .set((
            anmeldungen_plan::min_cp.eq(self.child.rules.min_cp as i32),
            anmeldungen_plan::max_cp.eq(self.child.rules.max_cp.map(|v| v as i32)),
            anmeldungen_plan::min_modules.eq(self.child.rules.min_modules as i32),
            anmeldungen_plan::max_modules.eq(self.child.rules.max_modules.map(|v| v as i32)),
        ))
        .returning(anmeldungen_plan::url)
        .get_result(connection)
        .unwrap()
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct UpdateModule {
    pub course_of_study: String,
    pub semester: Semesterauswahl,
    pub module: ModuleResult,
}

impl RequestResponse for UpdateModule {
    type Response = ();

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        diesel::update(anmeldungen_entries::table)
            .filter(
                anmeldungen_entries::course_of_study
                    .eq(&self.course_of_study)
                    .and(
                        anmeldungen_entries::id
                            .eq(&self.module.nr)
                            // TODO FIXME if you can register it at multiple paths
                            // this will otherwise break
                            .and(anmeldungen_entries::state.ne(State::NotPlanned)),
                    ),
            )
            .set((
                anmeldungen_entries::semester.eq(if self.semester.name.starts_with("SoSe ") {
                    Semester::Sommersemester
                } else {
                    Semester::Wintersemester
                }),
                (anmeldungen_entries::year.eq(self.semester.name[5..9].parse::<i32>().unwrap())),
            ))
            .execute(connection)
            .unwrap();
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct UpdateAnmeldungEntry {
    pub entry: AnmeldungEntry
}

impl RequestResponse for UpdateAnmeldungEntry {
    type Response = ();

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        diesel::update(&self.entry)
            .set(&self.entry)
            .execute(connection)
            .unwrap();
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct AnmeldungenEntriesInSemester {
    pub course_of_study: String,
    pub year: i32,
    pub semester: Semester,
}

impl RequestResponse for AnmeldungenEntriesInSemester {
    type Response = Vec<AnmeldungEntry>;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
       QueryDsl::filter(
        anmeldungen_entries::table,
        anmeldungen_entries::course_of_study
            .eq(&self.course_of_study)
            .and(
                anmeldungen_entries::semester
                    .eq(self.semester)
                    .and(anmeldungen_entries::year.eq(self.year))
            ),
    )
    .select(AnmeldungEntry::as_select())
    .load(connection)
    .unwrap()
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct SetStateAndCredits {
    pub inserts: Vec<AnmeldungEntry>,
}

impl RequestResponse for SetStateAndCredits {
    type Response = ();

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        diesel::insert_into(anmeldungen_entries::table)
            .values(&self.inserts)
            .on_conflict((
                anmeldungen_entries::course_of_study,
                anmeldungen_entries::anmeldung,
                anmeldungen_entries::available_semester,
                anmeldungen_entries::id,
            ))
            .do_update()
            .set((
                anmeldungen_entries::state.eq(excluded(anmeldungen_entries::state)),
                (anmeldungen_entries::credits.eq(excluded(anmeldungen_entries::credits))),
            ))
            .execute(connection)
            .unwrap();
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct SetCpAndModuleCount {
    pub course_of_study: String,
    pub name: String,
    pub student_result: StudentResultResponse,
}

impl RequestResponse for SetCpAndModuleCount {
    type Response = String;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        diesel::update(QueryDsl::filter(
            anmeldungen_plan::table,
            anmeldungen_plan::course_of_study
                .eq(&self.course_of_study)
                .and(anmeldungen_plan::name.eq(&self.name)),
        ))
        .set((
            anmeldungen_plan::min_cp.eq(self.student_result.level0.rules.min_cp as i32),
            anmeldungen_plan::max_cp.eq(self.student_result.level0.rules.max_cp.map(|v| v as i32)),
            anmeldungen_plan::min_modules.eq(self.student_result.level0.rules.min_modules as i32),
            anmeldungen_plan::max_modules.eq(self
                .student_result
                .level0
                .rules
                .max_modules
                .map(|v| v as i32)),
        ))
        .returning(anmeldungen_plan::url)
        .get_result(connection)
        .unwrap()
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct ExportDatabaseRequest;

impl RequestResponse for ExportDatabaseRequest {
    type Response = Vec<u8>;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        connection.serialize_database_to_buffer().to_vec()
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Serialize, Deserialize, Debug, derive_more::From)]
pub enum RequestResponseEnum {
    AnmeldungenRequest(AnmeldungenRequest),
    AnmeldungenRequest2(AnmeldungenRequest2),
    Fewe(Fewe),
    FEwefweewf(FEwefweewf),
    Wlewifhewefwef(Wlewifhewefwef),
    ChildUrl(ChildUrl),
    UpdateModule(UpdateModule),
    SetStateAndCredits(SetStateAndCredits),
    SetCpAndModuleCount(SetCpAndModuleCount),
    CacheRequest(CacheRequest),
    StoreCacheRequest(StoreCacheRequest),
    ExportDatabaseRequest(ExportDatabaseRequest),
    UpdateAnmeldungEntry(UpdateAnmeldungEntry),
    AnmeldungenEntriesInSemester(AnmeldungenEntriesInSemester)
}

#[cfg(target_arch = "wasm32")]
impl RequestResponseEnum {
    pub fn execute(&self, connection: &mut SqliteConnection) -> JsValue {
        match self {
            RequestResponseEnum::AnmeldungenRequest(value) => {
                serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
            }
            RequestResponseEnum::AnmeldungenRequest2(value) => {
                serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
            }
            RequestResponseEnum::Fewe(value) => {
                serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
            }
            RequestResponseEnum::FEwefweewf(value) => {
                serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
            }
            RequestResponseEnum::Wlewifhewefwef(value) => {
                serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
            }
            RequestResponseEnum::ChildUrl(value) => {
                serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
            }
            RequestResponseEnum::UpdateModule(value) => {
                serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
            }
            RequestResponseEnum::SetStateAndCredits(value) => {
                serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
            }
            RequestResponseEnum::SetCpAndModuleCount(value) => {
                serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
            }
            RequestResponseEnum::CacheRequest(value) => {
                serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
            }
            RequestResponseEnum::StoreCacheRequest(value) => {
                serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
            }
            RequestResponseEnum::ExportDatabaseRequest(value) => {
                serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
            }
            RequestResponseEnum::UpdateAnmeldungEntry(value) => {
                serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
            }
            RequestResponseEnum::AnmeldungenEntriesInSemester(value) => {
                serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Serialize, Deserialize)]
pub struct MessageWithId {
    pub id: String,
    pub message: RequestResponseEnum,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub struct MyDatabase(diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>);

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
struct ConnectionCustomizer;

#[cfg(not(target_arch = "wasm32"))]
impl<C: diesel::connection::SimpleConnection, E> CustomizeConnection<C, E>
    for ConnectionCustomizer
{
    fn on_acquire(&self, connection: &mut C) -> Result<(), E> {
        connection
            .batch_execute("PRAGMA busy_timeout = 2000;")
            .unwrap();
        connection
            .batch_execute("PRAGMA synchronous = NORMAL;")
            .unwrap();
        Ok(())
    }

    fn on_release(&self, _conn: C) {}
}

#[cfg(not(target_arch = "wasm32"))]
impl MyDatabase {
    pub async fn wait_for_worker() -> Self {
        use diesel::{
            connection::SimpleConnection as _,
            r2d2::{ConnectionManager, Pool},
        };
        use diesel_migrations::MigrationHarness as _;

        let url = if cfg!(target_os = "android") {
            tokio::fs::create_dir_all("/data/data/com.example.TucanPlusDioxus/files")
                .await
                .unwrap();

            "sqlite:///data/data/com.example.TucanPlusDioxus/files/data.db?mode=rwc"
        } else {
            "sqlite://tucan-plus.db?mode=rwc"
        };

        let pool = Pool::builder()
            .connection_customizer(Box::new(ConnectionCustomizer))
            .build(ConnectionManager::<SqliteConnection>::new(url))
            .unwrap();

        let connection = &mut pool.get().unwrap();
        connection
            .batch_execute("PRAGMA journal_mode = WAL;")
            .unwrap();

        connection.run_pending_migrations(MIGRATIONS).unwrap();

        Self(pool)
    }

    pub async fn send_message<R: RequestResponse + std::fmt::Debug>(
        &self,
        value: R,
    ) -> R::Response {
        value.execute(&mut self.0.get().unwrap())
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct MyDatabase {
    broadcast_channel: Fragile<BroadcastChannel>,
}

#[cfg(target_arch = "wasm32")]
impl MyDatabase {
    pub async fn wait_for_worker(worker_js: String) -> Self {
        use js_sys::Promise;
        use wasm_bindgen::{JsCast as _, prelude::Closure};

        let lock_manager = web_sys::window().unwrap().navigator().locks();
        let lock_closure: Closure<dyn Fn(_) -> Promise> = {
            Closure::new(move |_event: web_sys::Lock| {
                let mut cb = |_resolve: js_sys::Function, reject: js_sys::Function| {
                    use web_sys::{WorkerOptions, WorkerType};

                    let options = WorkerOptions::new();
                    options.set_type(WorkerType::Module);
                    let worker = web_sys::Worker::new_with_options(&worker_js, &options).unwrap();
                    let error_closure: Closure<dyn Fn(_)> =
                        Closure::new(move |event: web_sys::ErrorEvent| {
                            use log::info;

                            info!(
                                "error at client {event:?} {:?} {:?}",
                                event.message(),
                                event.error()
                            );

                            reject.call0(&JsValue::undefined()).unwrap();
                        });
                    let error_closure_ref = error_closure.as_ref().clone();
                    worker
                        .add_event_listener_with_callback(
                            "error",
                            error_closure_ref.unchecked_ref(),
                        )
                        .unwrap();
                    error_closure.forget();
                };

                return js_sys::Promise::new(&mut cb);
            })
        };
        let _intentional = lock_manager.request_with_callback("opfs", lock_closure.as_ref().unchecked_ref());
        lock_closure.forget();

        let broadcast_channel = Fragile::new(BroadcastChannel::new("global").unwrap());

        // TODO FIXME add wait for worker to be alive

        Self { broadcast_channel }
    }

    pub async fn send_message<R: RequestResponse + std::fmt::Debug>(
        &self,
        message: R,
    ) -> R::Response
    where
        RequestResponseEnum: std::convert::From<R>,
    {
        use rand::distr::{Alphanumeric, SampleString as _};

        // TODO FIXME add retry

        let id = Alphanumeric.sample_string(&mut rand::rng(), 16);

        let temporary_broadcast_channel = BroadcastChannel::new(&id).unwrap();

        let mut cb = |resolve: js_sys::Function, _reject: js_sys::Function| {
            use wasm_bindgen::{JsCast as _, prelude::Closure};

            let temporary_message_closure: Closure<dyn Fn(_)> = {
                Closure::new(move |event: web_sys::MessageEvent| {
                    resolve.call1(&JsValue::undefined(), &event.data()).unwrap();
                })
            };
            temporary_broadcast_channel
                .add_event_listener_with_callback(
                    "message",
                    temporary_message_closure.as_ref().unchecked_ref(),
                )
                .unwrap();
            temporary_message_closure.forget();
        };

        let promise = js_sys::Promise::new(&mut cb);

        let value = serde_wasm_bindgen::to_value(&MessageWithId {
            id: id.clone(),
            message: RequestResponseEnum::from(message),
        })
        .unwrap();

        self.broadcast_channel.get().post_message(&value).unwrap();

        serde_wasm_bindgen::from_value(wasm_bindgen_futures::JsFuture::from(promise).await.unwrap())
            .unwrap()
    }
}
