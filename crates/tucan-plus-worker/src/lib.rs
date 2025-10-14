use std::collections::HashSet;
#[cfg(target_arch = "wasm32")]
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use diesel::r2d2::CustomizeConnection;
use diesel::{prelude::*, upsert::excluded};
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
#[cfg(target_arch = "wasm32")]
use fragile::Fragile;
use itertools::Itertools as _;
#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize, de::DeserializeOwned};
#[cfg(target_arch = "wasm32")]
use tokio::sync::OnceCell;
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
    registration::AnmeldungRequest,
    student_result::{StudentResultLevel, StudentResultResponse},
};

pub mod models;
pub mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[cfg(target_arch = "wasm32")]
pub trait RequestResponse: Serialize + Sized
where
    RequestResponseEnum: From<Self>,
{
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
pub struct AnmeldungenRootRequest {
    pub course_of_study: String,
}

impl RequestResponse for AnmeldungenRootRequest {
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
pub struct AnmeldungChildrenRequest {
    pub course_of_study: String,
    pub anmeldung: Anmeldung,
}

impl RequestResponse for AnmeldungChildrenRequest {
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
pub struct RecursiveAnmeldungenRequest {
    pub course_of_study: String,
    pub expanded: HashSet<AnmeldungRequest>,
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecursiveAnmeldungenResponse {
    pub has_contents: bool,
    pub has_rules: bool,
    pub credits: i32,
    pub modules: usize,
    pub anmeldung: Anmeldung,
    pub results: Vec<Anmeldung>,
    pub entries: Vec<AnmeldungEntry>,
    pub inner: Vec<RecursiveAnmeldungenResponse>,
}

fn prep_planning(
    connection: &mut SqliteConnection,
    course_of_study: &str,
    expanded: &HashSet<AnmeldungRequest>,
    anmeldung: Anmeldung, // ahh this needs to be a signal?
) -> RecursiveAnmeldungenResponse {
    let results = AnmeldungChildrenRequest {
        course_of_study: course_of_study.to_owned(),
        anmeldung: anmeldung.clone(),
    }
    .execute(connection);
    let entries = AnmeldungEntriesRequest {
        course_of_study: course_of_study.to_owned(),
        anmeldung: anmeldung.clone(),
    }
    .execute(connection);
    let inner: Vec<RecursiveAnmeldungenResponse> = results
        .iter()
        .map(|result| prep_planning(connection, course_of_study, expanded, result.clone()))
        .collect();
    let has_rules = anmeldung.min_cp != 0
        || anmeldung.max_cp.is_some()
        || anmeldung.min_modules != 0
        || anmeldung.max_modules.is_some();
    let has_contents = expanded.contains(&AnmeldungRequest::parse(&anmeldung.url))
        || has_rules
        || entries.iter().any(|entry| entry.state != State::NotPlanned)
        || inner.iter().any(|v| v.has_contents);
    let cp: i32 = entries
        .iter()
        .filter(|entry| entry.state == State::Done || entry.state == State::Planned)
        .map(|entry| entry.credits)
        .sum::<i32>()
        + inner.iter().map(|inner| inner.credits).sum::<i32>();
    let credits = std::cmp::min(cp, anmeldung.max_cp.unwrap_or(cp));
    let modules: usize = entries
        .iter()
        .filter(|entry| entry.state == State::Done || entry.state == State::Planned)
        .count()
        + inner.iter().map(|inner| inner.modules).sum::<usize>();
    RecursiveAnmeldungenResponse {
        anmeldung,
        results,
        entries,
        inner,
        has_contents,
        has_rules,
        modules,
        credits,
    }
}

impl RequestResponse for RecursiveAnmeldungenRequest {
    type Response = Option<RecursiveAnmeldungenResponse>;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        let root = AnmeldungenRootRequest {
            course_of_study: self.course_of_study.clone(),
        }
        .execute(connection);
        assert!(root.len() <= 1);
        if root.is_empty() {
            return None;
        }
        Some(prep_planning(
            connection,
            &self.course_of_study,
            &self.expanded,
            root[0].clone(),
        ))
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct AnmeldungEntriesRequest {
    pub course_of_study: String,
    pub anmeldung: Anmeldung,
}

impl RequestResponse for AnmeldungEntriesRequest {
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
pub struct InsertOrUpdateAnmeldungenRequest {
    pub inserts: Vec<Anmeldung>,
}

impl RequestResponse for InsertOrUpdateAnmeldungenRequest {
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
pub struct UpdateAnmeldungEntryRequest {
    pub insert: AnmeldungEntry,
}

impl RequestResponse for UpdateAnmeldungEntryRequest {
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
pub struct UpdateModuleYearAndSemester {
    pub course_of_study: String,
    pub semester: Semesterauswahl,
    pub module: ModuleResult,
}

impl RequestResponse for UpdateModuleYearAndSemester {
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
    pub entry: AnmeldungEntry,
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
                        .and(anmeldungen_entries::year.eq(self.year)),
                ),
        )
        .select(AnmeldungEntry::as_select())
        .load(connection)
        .unwrap()
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct AnmeldungenEntriesPerSemester {
    pub course_of_study: String,
}

impl RequestResponse for AnmeldungenEntriesPerSemester {
    type Response = Vec<((i32, Semester), Vec<AnmeldungEntry>)>;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        let result = QueryDsl::filter(
            anmeldungen_entries::table,
            anmeldungen_entries::course_of_study
                .eq(&self.course_of_study)
                .and(anmeldungen_entries::year.is_not_null())
                .and(anmeldungen_entries::semester.is_not_null()),
        )
        .order_by((anmeldungen_entries::year, anmeldungen_entries::semester))
        .select(AnmeldungEntry::as_select())
        .load(connection)
        .unwrap();
        result
            .into_iter()
            .chunk_by(|elem| (elem.year.unwrap(), elem.semester.unwrap()))
            .into_iter()
            .map(|(elem, value)| (elem, value.collect_vec()))
            .collect_vec()
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct AnmeldungenEntriesNoSemester {
    pub course_of_study: String,
}

impl RequestResponse for AnmeldungenEntriesNoSemester {
    type Response = Vec<AnmeldungEntry>;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        QueryDsl::filter(
            anmeldungen_entries::table,
            anmeldungen_entries::course_of_study
                .eq(&self.course_of_study)
                .and(anmeldungen_entries::state.ne(State::NotPlanned))
                .and(
                    anmeldungen_entries::year
                        .is_null()
                        .or(anmeldungen_entries::semester.is_null()),
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
pub struct ExportDatabaseRequest {}

impl RequestResponse for ExportDatabaseRequest {
    type Response = Vec<u8>;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        connection.serialize_database_to_buffer().to_vec()
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct ImportDatabaseRequest {
    pub data: Vec<u8>,
}

impl RequestResponse for ImportDatabaseRequest {
    type Response = ();

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        panic!("should be special cased at caller")
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct PingRequest {}

impl RequestResponse for PingRequest {
    type Response = ();

    fn execute(&self, _connection: &mut SqliteConnection) -> Self::Response {
        ()
    }
}

macro_rules! request_response_enum {
    ($($struct: ident)*) => {
        #[cfg(target_arch = "wasm32")]
        #[derive(Serialize, Deserialize, Debug, derive_more::From)]
        pub enum RequestResponseEnum {
            $($struct($struct)),*
        }

        #[cfg(target_arch = "wasm32")]
        impl RequestResponseEnum {
            pub fn execute(&self, connection: &mut SqliteConnection) -> JsValue {
                match self {
                    $(RequestResponseEnum::$struct(value) => {
                        serde_wasm_bindgen::to_value(&value.execute(connection)).unwrap()
                    })*
                }
            }
        }
    };
}

request_response_enum!(
    AnmeldungenRootRequest
    AnmeldungChildrenRequest
    AnmeldungEntriesRequest
    InsertOrUpdateAnmeldungenRequest
    UpdateAnmeldungEntryRequest
    ChildUrl
    UpdateModuleYearAndSemester
    SetStateAndCredits
    SetCpAndModuleCount
    CacheRequest
    StoreCacheRequest
    ExportDatabaseRequest
    UpdateAnmeldungEntry
    AnmeldungenEntriesInSemester
    PingRequest
    ImportDatabaseRequest
    RecursiveAnmeldungenRequest
    AnmeldungenEntriesPerSemester
    AnmeldungenEntriesNoSemester
);

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
            tokio::fs::create_dir_all("/data/data/de.selfmade4u.tucanplus/files")
                .await
                .unwrap();

            "file:/data/data/de.selfmade4u.tucanplus/files/data.db?mode=rwc"
        } else {
            "file:tucan-plus.db?mode=rwc"
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

    pub async fn send_message_with_timeout<R: RequestResponse + std::fmt::Debug>(
        &self,
        message: R,
        timeout: std::time::Duration,
    ) -> Result<R::Response, ()> {
        Ok(self.send_message(message).await)
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct MyDatabase {
    broadcast_channel: Fragile<BroadcastChannel>,
    pinged: OnceCell<()>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    // Getters can only be declared on classes, so we need a fake type to declare it
    // on.
    #[wasm_bindgen]
    type meta;

    #[wasm_bindgen(js_namespace = import, static_method_of = meta, getter)]
    fn url() -> String;
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn shim_url() -> String {
    meta::url()
}

#[cfg(target_arch = "wasm32")]
impl MyDatabase {
    pub async fn wait_for_worker() -> Self {
        use js_sys::Promise;
        use log::info;
        use wasm_bindgen::{JsCast as _, prelude::Closure};

        let lock_manager = web_sys::window().unwrap().navigator().locks();
        let lock_closure: Closure<dyn Fn(_) -> Promise> = {
            Closure::new(move |_event: web_sys::Lock| {
                let mut cb = |_resolve: js_sys::Function, reject: js_sys::Function| {
                    use web_sys::{WorkerOptions, WorkerType};

                    let options = WorkerOptions::new();
                    options.set_type(WorkerType::Module);
                    let worker = web_sys::Worker::new_with_options(&shim_url(), &options).unwrap();
                    let error_closure: Closure<dyn Fn(_)> =
                        Closure::new(move |event: web_sys::Event| {
                            use log::info;

                            info!("error at client {event:?}",);

                            reject.call1(&JsValue::undefined(), &event).unwrap();
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
        let _intentional =
            lock_manager.request_with_callback("opfs", lock_closure.as_ref().unchecked_ref());
        lock_closure.forget();

        let broadcast_channel = Fragile::new(BroadcastChannel::new("global").unwrap());

        // TODO FIXME add wait for worker to be aliv

        let this = Self {
            broadcast_channel,
            pinged: OnceCell::new(),
        };

        this
    }

    pub async fn send_message<R: RequestResponse + std::fmt::Debug>(
        &self,
        message: R,
    ) -> R::Response
    where
        RequestResponseEnum: std::convert::From<R>,
    {
        self.send_message_with_timeout(message, Duration::from_secs(10))
            .await
            .expect("timed out")
    }

    pub async fn send_message_with_timeout<R: RequestResponse + std::fmt::Debug>(
        &self,
        message: R,
        timeout: Duration,
    ) -> Result<R::Response, ()>
    where
        RequestResponseEnum: std::convert::From<R>,
    {
        self.pinged
            .get_or_init(|| async {
                use log::info;
                let mut i = 0;
                while i < 100
                    && self
                        .send_message_with_timeout_internal::<PingRequest>(
                            PingRequest {},
                            Duration::from_millis(100),
                        )
                        .await
                        .is_err()
                {
                    info!("retry ping");
                    i += 1;
                }
                if i == 100 {
                    panic!("failed to connect to worker in time")
                }
                info!("got pong");
            })
            .await;

        self.send_message_with_timeout_internal(message, timeout)
            .await
    }

    async fn send_message_with_timeout_internal<R: RequestResponse + std::fmt::Debug>(
        &self,
        message: R,
        timeout: Duration,
    ) -> Result<R::Response, ()>
    where
        RequestResponseEnum: std::convert::From<R>,
    {
        use rand::distr::{Alphanumeric, SampleString as _};

        // TODO FIXME add retry

        let id = Alphanumeric.sample_string(&mut rand::rng(), 16);

        let temporary_broadcast_channel = Fragile::new(BroadcastChannel::new(&id).unwrap());

        let mut cb = |resolve: js_sys::Function, reject: js_sys::Function| {
            use wasm_bindgen::{JsCast as _, prelude::Closure};

            let temporary_message_closure: Closure<dyn Fn(_)> = {
                Closure::new(move |event: web_sys::MessageEvent| {
                    resolve.call1(&JsValue::undefined(), &event.data()).unwrap();
                })
            };
            temporary_broadcast_channel
                .get()
                .add_event_listener_with_callback(
                    "message",
                    temporary_message_closure.as_ref().unchecked_ref(),
                )
                .unwrap();
            temporary_message_closure.forget();

            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    &reject,
                    timeout.as_millis().try_into().unwrap(),
                )
                .unwrap();
        };

        let promise = js_sys::Promise::new(&mut cb);

        {
            use log::info;

            let value = serde_wasm_bindgen::to_value(&MessageWithId {
                id: id.clone(),
                message: RequestResponseEnum::from(message),
            })
            .unwrap();

            self.broadcast_channel.get().post_message(&value).unwrap();

            info!("send a message to worker");
        }

        let result = Fragile::new(wasm_bindgen_futures::JsFuture::from(promise))
            .await
            .map_err(|_| ());
        Ok(serde_wasm_bindgen::from_value(result?).unwrap())
    }
}
