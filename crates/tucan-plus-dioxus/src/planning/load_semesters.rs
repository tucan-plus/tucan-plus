use dioxus::{hooks::use_context, html::FileData, signals::Signal};
use futures::StreamExt as _;
use tucan_plus_planning::decompress;
use tucan_plus_worker::{
    FEwefweewf, MyDatabase, Wlewifhewefwef,
    models::{Anmeldung, AnmeldungEntry, Semester, State},
};
use tucan_types::{
    CONCURRENCY, LoginResponse, RevalidationStrategy, Tucan as _, registration::AnmeldungResponse,
};

use crate::RcTucanType;

pub async fn handle_semester(
    course_of_study: &str,
    tucan: RcTucanType,
    login_response: &LoginResponse,
    semester: Semester,
    file_names: Signal<Vec<FileData>>,
) {
    let worker: MyDatabase = use_context();
    for file in file_names() {
        let decompressed = decompress(&file.read_bytes().await.unwrap()).await.unwrap();
        let mut result: Vec<AnmeldungResponse> =
            serde_json::from_reader(decompressed.as_slice()).unwrap();
        result.sort_by_key(|e| e.path.len());
        let inserts: Vec<_> = result
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
        worker.send_message(FEwefweewf { inserts }).await;
        let inserts: Vec<AnmeldungEntry> = futures::stream::iter(result.iter())
            .flat_map(|anmeldung| {
                futures::stream::iter(anmeldung.entries.iter()).map(async |entry| AnmeldungEntry {
                    course_of_study: course_of_study.to_owned(),
                    available_semester: semester,
                    anmeldung: anmeldung.path.last().unwrap().1.inner().to_owned(),
                    module_url: entry.module.as_ref().unwrap().url.inner().to_owned(),
                    id: entry.module.as_ref().unwrap().id.clone(),
                    name: entry.module.as_ref().unwrap().name.clone(),
                    credits: tucan
                        .module_details(
                            login_response,
                            RevalidationStrategy::cache(),
                            entry.module.as_ref().unwrap().url.clone(),
                        )
                        .await
                        .unwrap()
                        .credits
                        .unwrap_or_default()
                        .try_into()
                        .unwrap(),
                    state: State::NotPlanned,
                    year: None,
                    semester: None,
                })
            })
            .buffer_unordered(CONCURRENCY)
            .collect()
            .await;
        // prevent too many variable error, TODO maybe batching
        for insert in inserts {
            worker.send_message(Wlewifhewefwef { insert }).await;
        }
    }
}
