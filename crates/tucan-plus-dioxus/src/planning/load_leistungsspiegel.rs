use dioxus::hooks::use_context;
use log::info;
use tucan_plus_worker::{
    ChildUrl, MyDatabase, SetCpAndModuleCount, SetStateAndCredits, UpdateModule,
    models::{AnmeldungEntry, Semester, State},
};
use tucan_types::{
    LeistungsspiegelGrade, LoginResponse, RevalidationStrategy, SemesterId, Tucan as _,
    student_result::{StudentResultLevel, StudentResultResponse},
};

use crate::RcTucanType;

pub async fn recursive_update(
    worker: MyDatabase,
    course_of_study: &str,
    url: String,
    level: StudentResultLevel,
) {
    for child in level.children {
        let name = child.name.as_ref().unwrap();
        let child_url = worker
            .send_message(ChildUrl {
                course_of_study: course_of_study.to_string(),
                url: url.clone(),
                name: name.clone(),
                child: child.clone(),
            })
            .await;
        info!("updated");
        Box::pin(recursive_update(
            worker.clone(),
            course_of_study,
            child_url,
            child,
        ))
        .await;
    }
    let inserts: Vec<_> = level
        .entries
        .iter()
        .map(|entry| AnmeldungEntry {
            course_of_study: course_of_study.to_owned(),
            available_semester: Semester::Sommersemester, // TODO FIXME
            anmeldung: url.clone(),
            module_url: "TODO".to_owned(), // TODO FIXME
            id: entry.id.as_ref().unwrap_or(&entry.name).to_owned(), /* TODO FIXME, use two columns
                                            * and both as primary key */
            credits: i32::try_from(entry.used_cp.unwrap_or_else(|| {
                if level.name.as_deref() == Some("Masterarbeit") {
                    30
                } else {
                    0
                }
            }))
            .unwrap(),
            name: entry.name.clone(),
            state: if matches!(
                entry.grade,
                LeistungsspiegelGrade::Grade(_) | LeistungsspiegelGrade::BestandenOhneNote
            ) {
                State::Done
            } else {
                State::Planned
            },
            year: None,
            semester: None,
        })
        .collect();
    worker.send_message(SetStateAndCredits { inserts }).await;
}

pub async fn load_leistungsspiegel(
    current_session: LoginResponse,
    tucan: RcTucanType,
    student_result: StudentResultResponse,
    course_of_study: String,
) {
    let worker: MyDatabase = use_context();

    // top level anmeldung has name "M.Sc. Informatik (2023)"
    // top level leistungsspiegel has "Informatik"

    let name = student_result
        .course_of_study
        .iter()
        .find(|e| e.selected)
        .unwrap()
        .name
        .to_owned();
    let the_url = worker
        .send_message(SetCpAndModuleCount {
            course_of_study: course_of_study.clone(),
            name,
            student_result: student_result.clone(),
        })
        .await;

    recursive_update(
        worker.clone(),
        &course_of_study,
        the_url,
        student_result.level0,
    )
    .await;

    let semesters = tucan
        .course_results(
            &current_session,
            RevalidationStrategy::cache(),
            SemesterId::current(),
        )
        .await
        .unwrap();
    for semester in semesters.semester {
        let result = tucan
            .course_results(
                &current_session,
                RevalidationStrategy::cache(),
                semester.value.clone(),
            )
            .await
            .unwrap();
        for module in result.results {
            worker
                .send_message(UpdateModule {
                    course_of_study: course_of_study.clone(),
                    semester: semester.clone(),
                    module,
                })
                .await;
        }
    }
}
