use std::str::FromStr;

use html_handler::{Root, parse_document};
use tucan_types::{
    LoginResponse, ModuleGrade, SemesterId, Semesterauswahl, TucanError,
    courseresults::{ModuleResult, ModuleResultsResponse},
    gradeoverview::GradeOverviewRequest,
};

use crate::{
    gradeoverview::GRADEOVERVIEW_REGEX,
    head::{footer, html_head, logged_in_head},
};

#[expect(clippy::too_many_lines)]
pub(crate) fn course_results_internal(
    login_response: &LoginResponse,
    content: &str,
    _nothing: &(),
) -> Result<ModuleResultsResponse, TucanError> {
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
                        "Fh9QXGwM_sXrM0QKenGB9RZNLE7wpBMHS188Im7J1M4"
                    </style>
                    <style type="text/css">
                        "9_MbCqyCOkddcwSY33R_7gqnd6subPQ-km7hoA0s4xo"
                    </style>
                </head>
                <body class="course_results">
                    use logged_in_head(html_handler, login_response.id).0;
                    <script type="text/javascript">
                    </script>
                    <h1>
                        _modulnoten_semester_fuer_name
                    </h1>
                    <div class="tb">
                        <form id="semesterchange" action="/scripts/mgrqispi.dll" method="post" class="pageElementTop">
                            <div>
                                <div class="tbhead">
                                </div>
                                <div class="tbsubhead">
                                    "Wählen Sie ein Semester"
                                </div>
                                <div class="formRow">
                                    <div class="inputFieldLabel long">
                                        <label for="semester">
                                            "Semester:"
                                        </label>
                                        <select id="semester" name="semester" onchange=_onchange class="tabledata">
                                            let semester = while html_handler.peek().is_some() {
                                                let option = if html_handler.peek().unwrap().value().as_element().unwrap().attr("selected").is_some() {
                                                    <option value=value selected="selected">
                                                        name
                                                    </option>
                                                } => Semesterauswahl {
                                                    name,
                                                    value: SemesterId::from_str(&value).unwrap(),
                                                    selected: true
                                                } else {
                                                    <option value=value>
                                                        name
                                                    </option>
                                                } => Semesterauswahl {
                                                    name,
                                                    value: SemesterId::from_str(&value).unwrap(),
                                                    selected: false
                                                };
                                            } => option.either_into();
                                        </select>
                                        <input name="Refresh" type="submit" value="Aktualisieren" class="img img_arrowReload"></input>
                                    </div>
                                </div>
                                <input name="APPNAME" type="hidden" value="CampusNet"></input>
                                <input name="PRGNAME" type="hidden" value="COURSERESULTS"></input>
                                <input name="ARGUMENTS" type="hidden" value="sessionno,menuno,semester"></input>
                                <input name="sessionno" type="hidden" value=_sessionno></input>
                                <input name="menuno" type="hidden" value="000324"></input>
                            </div>
                        </form>
                        <table class="nb list">
                            <thead>
                                <tr>
                                    <td class="tbsubhead">
                                        "Nr."
                                    </td>
                                    <td class="tbsubhead">
                                        "Kursname"
                                    </td>
                                    <td class="tbsubhead">
                                        "Endnote"
                                    </td>
                                    <td class="tbsubhead">
                                        "Credits"
                                    </td>
                                    <td class="tbsubhead">
                                        "Status"
                                    </td>
                                    <td class="tbsubhead" colspan="2">
                                    </td>
                                </tr>
                            </thead>
                            <tbody>
                                let results = while html_handler.peek().unwrap().first_child().unwrap().value().as_element().unwrap().name() == "td" {
                                    <tr>
                                        <td class="tbdata">
                                            nr
                                        </td>
                                        <td class="tbdata">
                                            name
                                        </td>
                                        <td class="tbdata_numeric" style="vertical-align:top;">
                                            let grade = if html_handler.peek().is_some() {
                                                grade
                                            } => grade;
                                        </td>
                                        <td class="tbdata_numeric">
                                            credits
                                        </td>
                                        <td class="tbdata">
                                            let status = if html_handler.peek().is_some() {
                                                status
                                            } => status;
                                        </td>
                                        <td class="tbdata" style="vertical-align:top;">
                                            let pruefungen_url = if html_handler.peek().is_some() {
                                                <a id=_some_id href=pruefungen_url>
                                                    "Prüfungen"
                                                </a>
                                                <script type="text/javascript">
                                                    _script
                                                </script>
                                            } => pruefungen_url;
                                        </td>
                                        <td class="tbdata">
                                            let average_url = if html_handler.peek().is_some() {
                                                <a id=_some_id href=average_url class="link" title="Notenspiegel">
                                                    <b>
                                                        "Ø"
                                                    </b>
                                                </a>
                                                <script type="text/javascript">
                                                    _script
                                                </script>
                                            } => GradeOverviewRequest::parse(&GRADEOVERVIEW_REGEX.replace(&average_url, ""));
                                        </td>
                                    </tr>
                                } => ModuleResult {
                                    nr,
                                    name,
                                    grade: ModuleGrade::from((grade.as_deref(), status.as_deref())),
                                    credits,
                                    pruefungen_url,
                                    average_url
                                };
                                let gpas = while html_handler.peek().is_some() {
                                    <tr>
                                        <th colspan="2">
                                            course_of_study
                                        </th>
                                        <th class="tbdata">
                                            average_grade
                                        </th>
                                        <th>
                                            sum_credits
                                        </th>
                                        <th class="tbdata" colspan="4">
                                        </th>
                                    </tr>
                                } => (course_of_study, average_grade, sum_credits);
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
        use footer(html_handler, login_response.id, 326);
    }
    html_handler.end_document();
    Ok(ModuleResultsResponse {
        semester,
        results,
        gpas,
    })
}
