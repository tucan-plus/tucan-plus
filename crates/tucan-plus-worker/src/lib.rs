use std::collections::HashSet;
#[cfg(target_arch = "wasm32")]
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

#[cfg(not(target_arch = "wasm32"))]
use diesel::r2d2::CustomizeConnection;
use diesel::{prelude::*, upsert::excluded};
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
#[cfg(target_arch = "wasm32")]
use fragile::Fragile;
use itertools::Itertools as _;
use log::info;
#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize, de::DeserializeOwned};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::BroadcastChannel;

use crate::{
    models::{Anmeldung, AnmeldungEntry, CacheEntry, Semester, State},
    schema::{anmeldungen_entries, anmeldungen_plan, cache},
};
use tucan_types::{
    Semesterauswahl, courseresults::ModuleResult, registration::AnmeldungRequest,
    student_result::StudentResultLevel,
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
    pub anmeldung: String, // TODO type safety in database and here
}

impl RequestResponse for AnmeldungChildrenRequest {
    type Response = Vec<Anmeldung>;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        QueryDsl::filter(
            anmeldungen_plan::table,
            anmeldungen_plan::course_of_study
                .eq(&self.course_of_study)
                .and(anmeldungen_plan::parent.eq(&self.anmeldung)),
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
pub struct AnmeldungEntryWithMoveInformation {
    pub entry: AnmeldungEntry,
    pub move_targets: Vec<(String, String)>,
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecursiveAnmeldungenResponse {
    pub has_contents: bool,
    pub has_rules: bool,
    pub actual_credits: i32,
    pub propagated_credits: i32,
    pub modules: usize,
    pub anmeldung: Anmeldung,
    pub results: Vec<Anmeldung>,
    pub entries: Vec<AnmeldungEntryWithMoveInformation>,
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
        anmeldung: anmeldung.url.clone(),
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
        || entries
            .iter()
            .any(|entry| entry.entry.state != State::NotPlanned)
        || inner.iter().any(|v| v.has_contents);
    let actual_credits: i32 = entries
        .iter()
        .filter(|entry| entry.entry.state == State::Done || entry.entry.state == State::Planned)
        .map(|entry| entry.entry.credits)
        .sum::<i32>()
        + inner
            .iter()
            .map(|inner| inner.propagated_credits)
            .sum::<i32>();
    let propagated_credits =
        std::cmp::min(actual_credits, anmeldung.max_cp.unwrap_or(actual_credits));
    let modules: usize = entries
        .iter()
        .filter(|entry| entry.entry.state == State::Done || entry.entry.state == State::Planned)
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
        actual_credits,
        propagated_credits,
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

fn calculate_move_targets(
    connection: &mut SqliteConnection,
    entry: AnmeldungEntry,
) -> AnmeldungEntryWithMoveInformation {
    let mut move_targets = Vec::new();
    let current = anmeldungen_plan::table
        .filter(
            anmeldungen_plan::course_of_study
                .eq(&entry.course_of_study)
                .and(anmeldungen_plan::url.eq(&entry.anmeldung)),
        )
        .select(Anmeldung::as_select())
        .get_result(connection)
        .unwrap();
    move_targets.push((current.name.clone(), current.url.clone()));
    let children = AnmeldungChildrenRequest {
        course_of_study: entry.course_of_study.clone(),
        anmeldung: entry.anmeldung.clone(),
    }
    .execute(connection);
    move_targets.extend(
        children
            .iter()
            .map(|elem| (elem.name.clone(), elem.url.clone())),
    );
    if let Some(parent) = current.parent {
        let parent = anmeldungen_plan::table
            .filter(
                anmeldungen_plan::course_of_study
                    .eq(&entry.course_of_study)
                    .and(anmeldungen_plan::url.eq(&parent)),
            )
            .select(Anmeldung::as_select())
            .get_result(connection)
            .unwrap();
        move_targets.push((parent.name.clone(), parent.url.clone()));
    }
    AnmeldungEntryWithMoveInformation {
        entry,
        move_targets,
    }
}

impl RequestResponse for AnmeldungEntriesRequest {
    type Response = Vec<AnmeldungEntryWithMoveInformation>;

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
        .into_iter()
        .map(|entry| calculate_move_targets(connection, entry))
        .collect_vec()
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
    pub inserts: Vec<AnmeldungEntry>,
}

impl RequestResponse for UpdateAnmeldungEntryRequest {
    type Response = ();

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        for insert in &self.inserts {
            diesel::insert_into(anmeldungen_entries::table)
                .values(insert)
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
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct UpdateAnmeldungEntry {
    pub entry: AnmeldungEntry,
    pub new_entry: AnmeldungEntry,
}

impl RequestResponse for UpdateAnmeldungEntry {
    type Response = ();

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        /*connection.set_instrumentation(
            move |event: diesel::connection::InstrumentationEvent<'_>| {
                info!("{event:?}");
            },
        );*/
        // AsChangeset doesn't update primary keys
        diesel::update(&self.entry)
            .set((
                anmeldungen_entries::course_of_study.eq(&self.new_entry.course_of_study),
                anmeldungen_entries::available_semester.eq(&self.new_entry.available_semester),
                anmeldungen_entries::anmeldung.eq(&self.new_entry.anmeldung),
                anmeldungen_entries::module_url.eq(&self.new_entry.module_url),
                anmeldungen_entries::id.eq(&self.new_entry.id),
                anmeldungen_entries::name.eq(&self.new_entry.name),
                anmeldungen_entries::credits.eq(&self.new_entry.credits),
                anmeldungen_entries::state.eq(&self.new_entry.state),
                anmeldungen_entries::semester.eq(&self.new_entry.semester),
                anmeldungen_entries::year.eq(&self.new_entry.year),
            ))
            .execute(connection)
            .unwrap();
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct AnmeldungenEntriesPerSemester {
    pub course_of_study: String,
}

impl RequestResponse for AnmeldungenEntriesPerSemester {
    type Response = Vec<((i32, Semester), Vec<AnmeldungEntryWithMoveInformation>)>;

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
            .map(|(elem, value)| {
                (
                    elem,
                    value
                        .map(|value| calculate_move_targets(connection, value))
                        .collect_vec(),
                )
            })
            .collect_vec()
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct AnmeldungenEntriesNoSemester {
    pub course_of_study: String,
}

impl RequestResponse for AnmeldungenEntriesNoSemester {
    type Response = Vec<AnmeldungEntryWithMoveInformation>;

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
        .into_iter()
        .map(|value| calculate_move_targets(connection, value))
        .collect_vec()
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct InsertEntrySomewhereBelow {
    pub inserts: Vec<AnmeldungEntry>,
}

impl RequestResponse for InsertEntrySomewhereBelow {
    /// failed ones
    type Response = Vec<AnmeldungEntryWithMoveInformation>;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        let mut failed: Vec<AnmeldungEntryWithMoveInformation> = Vec::new();
        'top_level: for mut entry in self.inserts.clone() {
            // find where the entry is already
            let possible_places = QueryDsl::filter(
                anmeldungen_entries::table,
                anmeldungen_entries::course_of_study
                    .eq(&entry.course_of_study)
                    .and(anmeldungen_entries::id.eq(&entry.id)),
            )
            .select(AnmeldungEntry::as_select())
            .load(connection)
            .unwrap();
            'all: for possible_place in possible_places {
                let mut anmeldung = possible_place.anmeldung.clone();
                while anmeldung != entry.anmeldung {
                    let parent_anmeldung = anmeldungen_plan::table
                        .filter(anmeldungen_plan::url.eq(&anmeldung))
                        .select(Anmeldung::as_select())
                        .get_result(connection)
                        .unwrap()
                        .parent;
                    if let Some(parent_anmeldung) = parent_anmeldung {
                        anmeldung = parent_anmeldung;
                    } else {
                        continue 'all;
                    }
                }
                entry.anmeldung = possible_place.anmeldung;
                diesel::insert_into(anmeldungen_entries::table)
                    .values(entry)
                    .on_conflict((
                        anmeldungen_entries::course_of_study,
                        anmeldungen_entries::anmeldung,
                        anmeldungen_entries::available_semester,
                        anmeldungen_entries::id,
                    ))
                    .do_update()
                    .set((
                        // TODO FIXME this should be cleaner
                        anmeldungen_entries::state.eq(excluded(anmeldungen_entries::state)),
                        (anmeldungen_entries::credits.eq(excluded(anmeldungen_entries::credits))),
                        (anmeldungen_entries::year.eq(excluded(anmeldungen_entries::year))),
                        (anmeldungen_entries::semester.eq(excluded(anmeldungen_entries::semester))),
                    ))
                    .execute(connection)
                    .unwrap();
                continue 'top_level;
            }
            // still insert
            diesel::insert_into(anmeldungen_entries::table)
                .values(&entry)
                .on_conflict((
                    anmeldungen_entries::course_of_study,
                    anmeldungen_entries::anmeldung,
                    anmeldungen_entries::available_semester,
                    anmeldungen_entries::id,
                ))
                .do_update()
                .set((
                    // TODO FIXME this should be cleaner
                    anmeldungen_entries::state.eq(excluded(anmeldungen_entries::state)),
                    (anmeldungen_entries::credits.eq(excluded(anmeldungen_entries::credits))),
                    (anmeldungen_entries::year.eq(excluded(anmeldungen_entries::year))),
                    (anmeldungen_entries::semester.eq(excluded(anmeldungen_entries::semester))),
                ))
                .execute(connection)
                .unwrap();
            failed.push(calculate_move_targets(connection, entry));
        }
        failed
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct SetCpAndModuleCount {
    pub course_of_study: String,
    pub url: Option<String>,
    pub child: StudentResultLevel,
}

impl RequestResponse for SetCpAndModuleCount {
    type Response = String;

    fn execute(&self, connection: &mut SqliteConnection) -> Self::Response {
        diesel::update(QueryDsl::filter(
            anmeldungen_plan::table,
            anmeldungen_plan::course_of_study
                .eq(&self.course_of_study)
                .and(
                    anmeldungen_plan::parent
                        .is(&self.url)
                        .and(anmeldungen_plan::name.eq(&self.child.name.clone().unwrap())),
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

    fn execute(&self, _connection: &mut SqliteConnection) -> Self::Response {
        panic!("should be special cased at caller")
    }
}

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct PingRequest {}

impl RequestResponse for PingRequest {
    type Response = ();

    fn execute(&self, _connection: &mut SqliteConnection) -> Self::Response {}
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
    InsertEntrySomewhereBelow
    SetCpAndModuleCount
    CacheRequest
    StoreCacheRequest
    ExportDatabaseRequest
    UpdateAnmeldungEntry
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
    pub fn wait_for_worker() -> Self {
        use diesel::{
            connection::SimpleConnection as _,
            r2d2::{ConnectionManager, Pool},
        };
        use diesel_migrations::MigrationHarness as _;

        let url = if cfg!(target_os = "android") {
            std::fs::create_dir_all("/data/data/de.selfmade4u.tucanplus/files").unwrap();

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
        _timeout: std::time::Duration,
    ) -> Result<R::Response, ()> {
        Ok(self.send_message(message).await)
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct MyDatabase {
    broadcast_channel: Fragile<BroadcastChannel>,
    pinged: Arc<AtomicBool>,
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
    pub fn wait_for_worker() -> Self {
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
                    // this is a local url which is not correct
                    info!("worker {}", shim_url());
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

        let this = Self {
            broadcast_channel,
            pinged: Arc::new(AtomicBool::new(false)),
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
        self.send_message_with_timeout(message, Duration::from_secs(60))
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
        use std::sync::atomic::Ordering;

        if !self.pinged.load(Ordering::Relaxed) {
            use log::info;
            let mut i = 0;
            while i < 100 && {
                let value = self
                    .send_message_with_timeout_internal::<PingRequest>(
                        PingRequest {},
                        Duration::from_millis(100),
                    )
                    .await;
                if value.is_err() {
                    info!("{value:?}");
                }
                value.is_err()
            } {
                info!("retry ping");
                i += 1;
            }
            if i == 100 {
                panic!("failed to connect to worker in time")
            }
            info!("got pong");
            self.pinged.store(true, Ordering::Relaxed);
        }

        self.send_message_with_timeout_internal(message, timeout)
            .await
            .map_err(|_| ())
    }

    async fn send_message_with_timeout_internal<R: RequestResponse + std::fmt::Debug>(
        &self,
        message: R,
        timeout: Duration,
    ) -> Result<R::Response, String>
    where
        RequestResponseEnum: std::convert::From<R>,
    {
        use rand::distr::{Alphanumeric, SampleString as _};

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
            let value = serde_wasm_bindgen::to_value(&MessageWithId {
                id: id.clone(),
                message: RequestResponseEnum::from(message),
            })
            .unwrap();

            self.broadcast_channel.get().post_message(&value).unwrap();
        }

        let result = Fragile::new(wasm_bindgen_futures::JsFuture::from(promise))
            .await
            .map_err(|error| format!("{error:?}"));
        Ok(serde_wasm_bindgen::from_value(result?).unwrap())
    }
}
