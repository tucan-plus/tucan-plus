use std::sync::LazyLock;

use regex::Regex;
use tucan_types::{
    LoginResponse,
    gradeoverview::{GradeOverviewResponse, Grades},
};

use crate::{
    TucanError,
    head::{footer, html_head, logged_in_head},
};
use html_handler::{Root, parse_document};

pub static GRADEOVERVIEW_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&PRGNAME=GRADEOVERVIEW&ARGUMENTS=-N\\d+,-N\\d+,",
    )
    .unwrap()
});

#[expect(clippy::too_many_lines)]
pub(crate) fn gradeoverview_internal(
    login_response: &LoginResponse,
    content: &str,
    _nothing: &(),
) -> Result<GradeOverviewResponse, TucanError> {
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
                        "rg0QEneqjn5GhiURVuEXyYt07X4xXCeM1lgTu-0-5Ak"
                    </style>
                </head>
                <body class="students_grades_diagramm_BFW">
                    let _head = logged_in_head(html_handler, login_response.id);
                    <script type="text/javascript">
                    </script>
                    <h1>
                        _notenspiegel
                    </h1>
                    <h2>
                        module_and_semester
                    </h2>
                    let modulangebot = if html_handler
                        .peek()
                        .unwrap()
                        .value()
                        .as_element()
                        .unwrap()
                        .name()
                        == "table" {
                        <table class="tb">
                            <tbody>
                                <tr>
                                    <td class="tbhead">
                                        "Kontext"
                                    </td>
                                </tr>
                                <tr>
                                    <td class="tbdata">
                                        modulangebot
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    } => modulangebot;
                    <h2>
                        let studienleistung = if html_handler
                            .peek()
                            .is_some() {
                            studienleistung
                        } => studienleistung;
                    </h2>
                    <div class="tb">
                        <div class="tbhead">
                        </div>
                        <div class="tbcontrol">
                            <a href=_examresults_url class="img img_arrowLeft prev">
                                "Zurück"
                            </a>
                        </div>
                        let maybe_grades = if html_handler
                            .peek()
                            .unwrap()
                            .value()
                            .as_element()
                            .unwrap()
                            .name()
                            == "table" {
                            <table class="nb">
                                <tbody>
                                    <tr>
                                        <td class="tbsubhead">
                                            "Noten"
                                        </td>
                                        let names = while html_handler.peek().is_some() {
                                            <td class="tbsubhead">
                                                name
                                            </td>
                                        } => name;
                                    </tr>
                                    <tr>
                                        <td class="tbdata">
                                            "Anzahl"
                                        </td>
                                        let values = while html_handler.peek().is_some() {
                                            <td class="tbdata">
                                                value
                                            </td>
                                        } => if value == "---" {
                                            0
                                        } else {
                                            value.parse().expect(&value)
                                        };
                                    </tr>
                                </tbody>
                            </table>
                            let infos = while html_handler.peek().is_some() {
                                <div class="tbdata">
                                    info
                                </div>
                            } => info;
                        } => Grades {
                            columns: names.into_iter().zip(values).collect(),
                            infos
                        } else {
                            <div class="tbdata">
                                "noch nicht gesetzt"
                            </div>
                        } => ();
                    </div>
                </div>
            </div>
        </div>
    };
    let html_handler = footer(html_handler, login_response.id, 19);
    html_handler.end_document();
    Ok(GradeOverviewResponse {
        module_and_semester,
        modulangebot,
        studienleistung,
        maybe_grades: maybe_grades.left(),
    })
}
