use std::collections::HashMap;

use dioxus::{hooks::use_context, html::FileData, signals::Signal};
use futures::StreamExt as _;
use log::warn;
use tucan_plus_worker::{
    InsertOrUpdateAnmeldungenRequest, MyDatabase, UpdateAnmeldungEntryRequest,
    models::{Anmeldung, AnmeldungEntry, Semester, State},
};
use tucan_types::{
    CONCURRENCY, LoginResponse, RevalidationStrategy, Tucan as _,
    moduledetails::{ModuleDetailsRequest, ModuleDetailsResponse},
    registration::AnmeldungResponse,
};

use crate::{RcTucanType, decompress, export_semester::SemesterExportV1};

pub async fn handle_semester(
    worker: &MyDatabase,
    course_of_study: &str,
    tucan: RcTucanType,
    login_response: &LoginResponse,
    semester: Semester,
    file_names: Signal<Vec<FileData>>,
) {
    for file in file_names() {
        let decompressed = decompress(&file.read_bytes().await.unwrap()).await.unwrap();
        let mut result: SemesterExportV1 =
            serde_json::from_reader(decompressed.as_slice()).unwrap();
        result.anmeldungen.sort_by_key(|e| e.path.len());
        let inserts: Vec<_> = result
            .anmeldungen
            .iter()
            .map(|e| Anmeldung {
                course_of_study: course_of_study.to_owned(),
                url: e.path.last().unwrap().1.inner().to_owned(),
                name: e.path.last().unwrap().0.clone(),
                parent: e
                    .path
                    .len()
                    .checked_sub(2)
                    .map(|v| e.path[v].1.inner().to_owned()),
                min_cp: 0,
                max_cp: None,
                min_modules: 0,
                max_modules: None,
            })
            .collect();
        worker
            .send_message(InsertOrUpdateAnmeldungenRequest { inserts })
            .await;
        let inserts: Vec<AnmeldungEntry> = futures::stream::iter(result.anmeldungen.iter())
            .flat_map(|anmeldung| {
                futures::stream::iter(anmeldung.entries.iter().filter(|entry| {
                    if entry.module.is_none() {
                        warn!("entry with no module {entry:?}");
                        return false;
                    }
                    true
                }))
                .map(
                    async |entry: &tucan_types::registration::AnmeldungEntry| {
                        let module_id = entry.module.as_ref().unwrap().url.clone();
                        let credits = result.modules[&module_id].credits;
                        let credits = if let Some(credits) = credits {
                            credits
                        } else {
                            warn!("module with no credits {:?}", entry.module);
                            0
                        };
                        AnmeldungEntry {
                            course_of_study: course_of_study.to_owned(),
                            available_semester: semester,
                            anmeldung: anmeldung.path.last().unwrap().1.inner().to_owned(),
                            module_url: Some(entry.module.as_ref().unwrap().url.inner().to_owned()),
                            id: entry.module.as_ref().unwrap().id.clone(),
                            name: entry.module.as_ref().unwrap().name.clone(),
                            credits: credits.try_into().unwrap(),
                            state: State::NotPlanned,
                            year: None,
                            semester: None,
                        }
                    },
                )
            })
            .buffer_unordered(CONCURRENCY)
            .collect()
            .await;
        // prevent too many variable error, TODO maybe batching
        for insert in inserts {
            worker
                .send_message(UpdateAnmeldungEntryRequest { insert })
                .await;
        }
    }
}
