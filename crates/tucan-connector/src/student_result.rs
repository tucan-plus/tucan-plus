use std::sync::LazyLock;

use crate::head::{footer, html_head, logged_in_head, logged_out_head};
use ego_tree::NodeRef;
use html_handler::{InElement, MyNode, Root, parse_document};
use regex::Regex;
use scraper::CaseSensitivity;
use tucan_types::{
    LeistungsspiegelGrade, LoginResponse, TucanError,
    student_result::{
        CourseOfStudySelection, StudentResultEntry, StudentResultLevel, StudentResultResponse,
        StudentResultRules, StudentResultState,
    },
};

fn get_level(node: &NodeRef<MyNode>) -> i8 {
    node.value()
        .as_element()
        .unwrap()
        .attr("class")
        .map_or(-1, |v| {
            v.trim_start_matches("subhead ")
                .trim_start_matches("level0")
                .parse::<i8>()
                .unwrap()
        })
}

fn part0<T>(
    html_handler: InElement<'_, T>,
    level: i8,
) -> (InElement<'_, T>, Option<(String, Vec<StudentResultEntry>)>) {
    html_extractor::html! {
        let result = if get_level(html_handler.peek().unwrap()) == level {
            <tr class={|l| assert_eq!(l, format!("subhead level0{level}"))}>
                <td colspan="2">
                    level_i
                </td>
                <td style="text-align:center;">
                </td>
                <td>
                </td>
                <td>
                </td>
                <td>
                </td>
                <td>
                </td>
            </tr>
            let entries = while html_handler
                .peek()
                .unwrap()
                .first_child()
                .unwrap()
                .value()
                .as_element()
                .unwrap()
                .has_class("tbdata", CaseSensitivity::CaseSensitive) {
                <tr>
                    <td class="tbdata">
                        id
                    </td>
                    <td class="tbdata">
                        let name_and_resultdetails_url = if html_handler
                            .peek()
                            .unwrap()
                            .value()
                            .is_text() {
                            name
                        } => (name, None::<String>) else {
                            <a name=_name id=_result_id href=resultdetails_url onclick=_onclick>
                                name
                            </a>
                            <script type="text/javascript">
                                _popup_script
                            </script>
                        } => (name, Some(resultdetails_url));
                    </td>
                    <td class="tbdata" style="text-align:right;">
                    </td>
                    <td class="tbdata" style="text-align:right;">
                        let cp = if html_handler.peek().is_some() {
                            cp
                        } => cp;
                    </td>
                    <td class="tbdata" style="text-align:right;">
                        let used_cp = if html_handler.peek().is_some() {
                            used_cp
                        } => used_cp;
                    </td>
                    <td class="tbdata" style="text-align:right;">
                        let grade = if html_handler.peek().is_some() {
                            grade
                        } => grade;
                    </td>
                    <td class="tbdata" style="text-align:center;">
                        <img src=src alt=alt title=state></img>
                    </td>
                </tr>
            } => StudentResultEntry {
                id: if id == "Anerkennung" { None } else { Some(id) },
                name: name_and_resultdetails_url
                    .clone()
                    .either_into::<(String, Option<String>)>()
                    .0,
                resultdetails_url: name_and_resultdetails_url
                    .either_into::<(String, Option<String>)>()
                    .1,
                cp: cp.map(|v| v.trim_end_matches(",0").parse().unwrap()),
                used_cp: used_cp.map(|v| v.trim_end_matches(",0").parse().unwrap()),
                grade: LeistungsspiegelGrade::from((
                    grade.as_deref(),
                    StudentResultState::from((src.as_str(), alt.as_str(), state.as_str()))
                )),
            };
        } => (level_i, entries);
    }
    (html_handler, result)
}

fn parse_rules(rules: &[String]) -> StudentResultRules {
    static RULES_1: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^Es sind mindestens +(?P<min>\d+),0 Credits einzubringen. Die Ergebnisse von maximal +(?P<max>\d+),0 Credits gehen in die Notenberechnung ein.$").unwrap()
    });
    static RULES_2: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^Maximal +(?P<max>\d+),0 Credits gehen in die Notenberechnung ein.$").unwrap()
    });
    static RULES_3: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^In diesem Bereich sind +(?P<eq>\d+),0 Credits einzubringen.$").unwrap()
    });
    static RULES_4: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^Erforderliche Credits für Abschluss: +(?P<eq>\d+),0$").unwrap()
    });
    static RULES_5: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^Es sind mindestens +(?P<min>\d+),0 Credits einzubringen.$").unwrap()
    });
    static RULES_6: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^In diesem Bereich sind mindestens[[:space:]]+(?P<min>\d+)[[:space:]]+und maximal[[:space:]]+(?P<max>\d+) Module zu belegen.$").unwrap()
    });
    let mut result = StudentResultRules {
        min_cp: 0,
        max_cp: None,
        min_modules: 0,
        max_modules: None,
    };
    for rule in rules {
        if let Some(c) = RULES_1.captures(rule) {
            result.min_cp = c["min"].parse().unwrap();
            result.max_cp = Some(c["max"].parse().unwrap());
        } else if let Some(c) = RULES_2.captures(rule) {
            result.max_cp = Some(c["max"].parse().unwrap());
        } else if let Some(c) = RULES_3.captures(rule) {
            result.min_cp = c["eq"].parse().unwrap();
            result.max_cp = Some(c["eq"].parse().unwrap());
        } else if let Some(c) = RULES_4.captures(rule) {
            result.min_cp = c["eq"].parse().unwrap();
            result.max_cp = Some(c["eq"].parse().unwrap());
        } else if let Some(c) = RULES_5.captures(rule) {
            result.min_cp = c["min"].parse().unwrap();
        } else if let Some(c) = RULES_6.captures(rule) {
            result.min_modules = c["min"].parse().unwrap();
            let max: u64 = c["max"].parse().unwrap();
            if max != 0 {
                result.max_modules = Some(max);
            }
        } else {
            panic!("{}", rule);
        }
    }
    result
}

fn part1<T>(
    html_handler: InElement<'_, T>,
    level: i8,
    name: Option<(String, Vec<StudentResultEntry>)>,
    children: Vec<StudentResultLevel>,
) -> (InElement<'_, T>, StudentResultLevel) {
    html_extractor::html! {
        let optional = if html_handler
            .peek()
            .unwrap()
            .value()
            .as_element()
            .unwrap()
            .attrs
            .is_empty()
            && get_level(
                &html_handler.peek().unwrap().first_child().unwrap()
            ) == level {
            <tr>
                <td colspan="2" class={|v| assert_eq!(v, format!("level0{level}"))}>
                    _summe
                </td>
                let sum_cp_and_used_cp = if html_handler
                    .peek()
                    .unwrap()
                    .value()
                    .as_element()
                    .unwrap()
                    .attr("colspan")
                    .is_some() {
                    <td
                        colspan="4"
                        class={|v| assert_eq!(v, format!("level0{level}"))}
                        style="text-align:left;white-space:nowrap;"
                    >
                        _summe_wird_erst_berechnet_wenn_der_bereich_abgeschlossen_ist
                    </td>
                } => (None, None) else {
                    <td class={|v| assert_eq!(v, format!("level0{level}"))}>
                    </td>
                    <td
                        class={|v| assert_eq!(v, format!("level0{level}"))}
                        style="text-align:right;white-space:nowrap;"
                    >
                        let sum_cp = if html_handler.peek().is_some() {
                            sum_cp
                        } => sum_cp;
                    </td>
                    <td
                        class={|v| assert_eq!(v, format!("level0{level}"))}
                        style="text-align:right;white-space:nowrap;"
                    >
                        let sum_used_cp = if html_handler.peek().is_some() {
                            sum_used_cp
                        } => sum_used_cp;
                    </td>
                    <td class={|v| assert_eq!(v, format!("level0{level}"))} style="text-align:right;">
                    </td>
                } => (sum_cp, sum_used_cp);
                <td class={|v| assert_eq!(v, format!("level0{level}"))} style="text-align:center;">
                    <img src=src alt=alt title=state></img>
                </td>
            </tr>
            let rules = while html_handler.peek().is_some()
                && html_handler
                    .peek()
                    .unwrap()
                    .first_child()
                    .unwrap()
                    .value()
                    .as_element()
                    .unwrap()
                    .has_class(&format!("level0{level}"), CaseSensitivity::CaseSensitive) {
                <tr>
                    <td colspan="   7" class={|v| assert_eq!(v, format!("level0{level}"))}>
                        rule
                    </td>
                </tr>
            } => rule;
        } => {
            let (sum_cp, sum_used_cp) = sum_cp_and_used_cp.either_into();
            (
                sum_cp,
                sum_used_cp,
                StudentResultState::from((src.as_str(), alt.as_str(), state.as_str())),
                rules,
            )
        };
    }
    (
        html_handler,
        StudentResultLevel {
            name: name.as_ref().map(|n| n.0.clone()),
            entries: name.map(|n| n.1).unwrap_or_default(),
            sum_cp: optional
                .clone()
                .and_then(|o| o.0)
                .map(|v| v.trim_end_matches(",0").parse().unwrap()),
            sum_used_cp: optional
                .clone()
                .and_then(|o| o.1)
                .map(|v| v.trim_end_matches(",0").parse().unwrap()),
            state: optional.clone().map(|o| o.2),
            rules: parse_rules(&optional.map(|o| o.3).unwrap_or_default()),
            children,
        },
    )
}

#[expect(clippy::too_many_lines)]
pub(crate) fn student_result_internal(
    login_response: &LoginResponse,
    content: &str,
    _nothing: &(),
) -> Result<StudentResultResponse, TucanError> {
    let document = parse_document(content);
    let html_handler = Root::new(document.root());
    let html_handler = html_handler.document_start();
    let html_handler = html_handler.doctype();
    html_extractor::html! {
            <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="de" lang="de">
                <head>
                    use html_head(html_handler)?;
                    <style type="text/css">
                        "hmeJiQNKqsf_yG6nmm6z0mPHuZmNXFlumNxu52NwnGY"
                    </style>
                    <style type="text/css">
                        "UfX2y86jUlyyH7pO6ezOiKqPWGK7fDWbsbA23Rfw7Ak"
                    </style>
                </head>
                <body class="students_results">
                    use if login_response.id == 1 {
                        logged_out_head(html_handler).0
                    } else {
                        logged_in_head(html_handler, login_response.id).0
                    };
                    <script type="text/javascript">
                    </script>
                    <h1>
                        _leistungsspiegel_von
                    </h1>
                    <div class="tb">
                        <form id="students_results" action="/scripts/mgrqispi.dll" method="post">
                            <div>
                                <div class="tbhead">
                                    selected_course_of_study
                                </div>
                                let course_of_study = if html_handler.peek().is_some() {
                                    <div class="tbcontrol">
                                        <div class="inputFieldLabel">
                                            <label for="study">
                                                "Studium:"
                                            </label>
                                            <select name="study" id="study" onchange="reloadpage.submitForm(this.form.id);" class="tabledata pageElementLeft">
                                                let course_of_study = while html_handler.peek().is_some() {
                                                    let course_of_study = if html_handler
                                                        .peek()
                                                        .unwrap()
                                                        .value()
                                                        .as_element()
                                                        .unwrap()
                                                        .attr("selected")
                                                        .is_some() {
                                                        <option value=value selected="selected">
                                                            name
                                                        </option>
                                                    } => CourseOfStudySelection {
                                                        name,
                                                        value: value.parse().unwrap(),
                                                        selected: true
                                                    } else {
                                                        <option value=value>
                                                            name
                                                        </option>
                                                    } => CourseOfStudySelection {
                                                        name,
                                                        value: value.parse().unwrap(),
                                                        selected: false
                                                    };
                                                } => course_of_study.either_into::<CourseOfStudySelection>();
                                            </select>
                                            <input id="Refresh" name="Refresh" type="submit" value="Aktualisieren" class="img img_arrowReload pageElementLeft update"></input>
                                        </div>
                                    </div>
                                    <input name="APPNAME" type="hidden" value="CampusNet"></input>
                                    <input name="PRGNAME" type="hidden" value="STUDENT_RESULT"></input>
                                    <input name="ARGUMENTS" type="hidden" value="sessionno,menuno,mode, semester,student,study,changestudy,section"></input>
                                    <input name="sessionno" type="hidden" value=_session_id></input>
                                    <input name="menuno" type="hidden" value="000316"></input>
                                    <input name="resulttype" type="hidden" value="0"></input>
                                    <input name="semester" type="hidden" value="0"></input>
                                    <input name="student" type="hidden" value="000000000000000"></input>
                                    <input name="changestudy" type="hidden" value="1"></input>
                                    <input name="section" type="hidden" value="000000000000000"></input>
                                } => course_of_study;
                            </div>
                        </form>
                        <table class="nb list students_results">
                            <thead>
                                <tr class="tbsubhead">
                                    <th colspan="2">
                                    </th>
                                    <th style="text-align:center;">
                                        "Datum"
                                    </th>
                                    <th style="text-align:right;">
                                        "Credits"
                                    </th>
                                    <th style="text-align:right;">
                                        "Angerechnet"
                                    </th>
                                    <th class="tbsubhead" style="text-align:right;">
                                        "Note"
                                    </th>
                                    <th class="tbsubhead" style="text-align:center;">
                                        "Status"
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                let level0_title = part0(html_handler, 0);
                                let level1 = while get_level(html_handler.peek().unwrap()) >= 1 {
                                    let level1_title = part0(html_handler, 1);
                                    let level2 = while get_level(html_handler.peek().unwrap()) >= 2 {
                                        let level2_title = part0(html_handler, 2);
                                        let level3 = while get_level(html_handler.peek().unwrap()) >= 3 {
                                            let level3_title = part0(html_handler, 3);
                                            let level4 = while get_level(html_handler.peek().unwrap()) >= 4 {
                                                let level4_title = part0(html_handler, 4);
                                                let level5 = while get_level(html_handler.peek().unwrap()) >= 5 {
                                                    let level5_title = part0(html_handler, 5);
                                                    let level6 = while get_level(html_handler.peek().unwrap()) >= 6 {
                                                        let level6_title = part0(html_handler, 6);
                                                        let level6_contents = part1(html_handler, 6, level6_title, Vec::new());
                                                    } => level6_contents;
                                                    let level5_contents = part1(html_handler, 5, level5_title, level6);
                                                } => level5_contents;
                                                let level4_contents = part1(html_handler, 4, level4_title, level5);
                                            } => level4_contents;
                                            let level3_contents = part1(html_handler, 3, level3_title, level4);
                                        } => level3_contents;
                                        let level2_contents = part1(html_handler, 2, level2_title, level3);
                                    } => level2_contents;
                                    let level1_contents = part1(html_handler, 1, level1_title, level2);
                                } => level1_contents;
                                let level0_contents = part1(html_handler, 0, level0_title, level1);
                            </tbody>
                        </table>
                        <table class="nb list students_results">
                            <tbody>
                                <tr>
                                    <th class="tbsubhead" style="text-align:left;">
                                        "Gesamt-GPA"
                                    </th>
                                    <th class="tbsubhead" style="text-align:right;">
                                        total_gpa
                                    </th>
                                </tr>
                                <tr>
                                    <th class="tbsubhead" style="text-align:left;">
                                        "Hauptfach-GPA"
                                    </th>
                                    <th class="tbsubhead" style="text-align:right;">
                                        main_gpa
                                    </th>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    }
    let html_handler = footer(html_handler, login_response.id, 311);
    html_handler.end_document();

    Ok(StudentResultResponse {
        course_of_study: course_of_study.unwrap_or_else(|| {
            vec![CourseOfStudySelection {
                name: selected_course_of_study,
                selected: true,
                value: 0,
            }]
        }),
        level0: level0_contents,
        total_gpa,
        main_gpa,
    })
}
