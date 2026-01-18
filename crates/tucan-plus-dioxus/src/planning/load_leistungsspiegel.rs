use std::{collections::HashMap, sync::LazyLock};

use log::info;
use tucan_plus_worker::{
    AnmeldungEntryWithMoveInformation, InsertEntrySomewhereBelow, MyDatabase, SetCpAndModuleCount,
    models::{AnmeldungEntry, Semester, State},
};
use tucan_types::{
    LeistungsspiegelGrade, LoginResponse, RevalidationStrategy, SemesterId, Tucan as _,
    courseresults::ModuleResult,
    enhanced_module_results::EnhancedModuleResult,
    mymodules::Module,
    student_result::{StudentResultLevel, StudentResultResponse, StudentResultRules},
};

use crate::RcTucanType;

static PATCHES: LazyLock<HashMap<&str, StudentResultLevel>> = LazyLock::new(|| {
    HashMap::from([(
        "M.Sc. Informatik (2023)",
        StudentResultLevel {
            name: Some("Vertiefungen, Wahlbereiche und Studium Generale".to_string()),
            entries: Vec::new(),
            sum_cp: None,
            sum_used_cp: None,
            state: None,
            rules: StudentResultRules {
                min_cp: 90,
                max_cp: Some(90),
                min_modules: 0,
                max_modules: None,
            },
            children: vec![StudentResultLevel {
                name: Some("Vertiefungen".to_string()),
                entries: Vec::new(),
                sum_cp: None,
                sum_used_cp: None,
                state: None,
                rules: StudentResultRules {
                    min_cp: 66,
                    max_cp: Some(72),
                    min_modules: 0,
                    max_modules: None,
                },
                children: vec![StudentResultLevel {
                    name: Some("Individuelle Vertiefung".to_string()),
                    entries: Vec::new(),
                    sum_cp: None,
                    sum_used_cp: None,
                    state: None,
                    rules: StudentResultRules {
                        min_cp: 66,
                        max_cp: Some(72),
                        min_modules: 0,
                        max_modules: None,
                    },
                    children: vec![StudentResultLevel {
                        name: Some("Wahlbereich Studienbegleitende Leistungen".to_string()),
                        entries: Vec::new(),
                        sum_cp: None,
                        sum_used_cp: None,
                        state: None,
                        rules: StudentResultRules {
                            min_cp: 9,
                            max_cp: Some(18),
                            min_modules: 0,
                            max_modules: None,
                        },
                        children: vec![
                            StudentResultLevel {
                                name: Some("Seminare".to_string()),
                                entries: Vec::new(),
                                sum_cp: None,
                                sum_used_cp: None,
                                state: None,
                                rules: StudentResultRules {
                                    min_cp: 3,
                                    max_cp: Some(12),
                                    min_modules: 1,
                                    max_modules: None,
                                },
                                children: vec![],
                            },
                            StudentResultLevel {
                                name: Some("Praktikum in der Lehre".to_string()),
                                entries: Vec::new(),
                                sum_cp: None,
                                sum_used_cp: None,
                                state: None,
                                rules: StudentResultRules {
                                    min_cp: 0,
                                    max_cp: Some(5),
                                    min_modules: 0,
                                    max_modules: Some(1),
                                },
                                children: vec![],
                            },
                            StudentResultLevel {
                                name: Some("Praktika, Projektpraktika, ähnliche LV".to_string()),
                                entries: Vec::new(),
                                sum_cp: None,
                                sum_used_cp: None,
                                state: None,
                                rules: StudentResultRules {
                                    min_cp: 6,
                                    max_cp: Some(15),
                                    min_modules: 1,
                                    max_modules: None,
                                },
                                children: vec![],
                            },
                        ],
                    }],
                }],
            }],
        },
    )])
});

#[must_use]
pub async fn recursive_update(
    worker: MyDatabase,
    course_of_study: &str,
    modules: &HashMap<String, EnhancedModuleResult>,
    url: Option<String>,
    level: StudentResultLevel,
) -> Vec<AnmeldungEntryWithMoveInformation> {
    let mut failed = Vec::new();
    let this_url = worker
        .send_message(SetCpAndModuleCount {
            course_of_study: course_of_study.to_string(),
            url: url.clone(),
            child: level.clone(),
        })
        .await;
    for child in level.children {
        failed.extend(
            Box::pin(recursive_update(
                worker.clone(),
                course_of_study,
                modules,
                Some(this_url.clone()),
                child,
            ))
            .await,
        );
    }
    let inserts: Vec<_> = level
        .entries
        .iter()
        .map(|entry| {
            let module_result = entry
                .id
                .as_ref()
                .and_then(|nr| modules.get(nr));
             AnmeldungEntry {
            course_of_study: course_of_study.to_owned(),
            available_semester: module_result
                .map(|m| m.semester.clone())
                .unwrap_or(tucan_types::Semester::Wintersemester)
                .into(),
            anmeldung: this_url.clone(),
            module_url: entry
                .id
                .as_ref()
                .and_then(|nr| modules.get(nr))
                .and_then(|module| module.url.clone())
                .map(|m| m.inner().to_owned()),
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
            year: module_result.map(|m| m.year),
            semester: module_result.map(|m| m.semester.into()),
        }})
        .collect();
    failed.extend(
        worker
            .send_message(InsertEntrySomewhereBelow { inserts })
            .await,
    );
    failed
}

#[must_use]
pub async fn load_leistungsspiegel(
    worker: MyDatabase,
    current_session: LoginResponse,
    tucan: RcTucanType,
    mut student_result: StudentResultResponse,
    course_of_study: String,
) -> Vec<AnmeldungEntryWithMoveInformation> {
    // top level anmeldung has name "M.Sc. Informatik (2023)"
    // top level leistungsspiegel has "Informatik"

    let name = student_result
        .course_of_study
        .iter()
        .find(|e| e.selected)
        .unwrap()
        .name
        .to_owned();

    student_result.level0.name = Some(name.clone());

    let module_results: HashMap<String, EnhancedModuleResult> = tucan
        .enhanced_module_results(
            &current_session,
            RevalidationStrategy::cache(),
            SemesterId::all(),
        )
        .await
        .unwrap()
        .results
        .into_iter()
        .map(|result| (result.nr.clone(), result))
        .collect();

    let mut failed: Vec<AnmeldungEntryWithMoveInformation> = Vec::new();

    // load patches
    if let Some(patch) = PATCHES.get(name.as_str()) {
        let this_url = worker
            .send_message(SetCpAndModuleCount {
                course_of_study: course_of_study.to_string(),
                url: None,
                child: student_result.level0.clone(),
            })
            .await;
        failed.extend(
            recursive_update(
                worker.clone(),
                &course_of_study,
                &module_results,
                Some(this_url),
                patch.clone(),
            )
            .await,
        );
    }

    // load leistungsspiegel hierarchy
    failed.extend(
        recursive_update(
            worker.clone(),
            &course_of_study,
            &module_results,
            None,
            student_result.level0,
        )
        .await,
    );

    failed
}
