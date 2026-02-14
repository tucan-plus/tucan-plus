use std::str::FromStr;

use html_handler::{Root, parse_document};
use scraper::CaseSensitivity;
use tucan_types::{
    LoginResponse, SemesterId, Semesterauswahl, TucanError,
    coursedetails::CourseDetailsRequest,
    examregistration::{
        ExamRegistration, ExamRegistrationCourse, ExamRegistrationResponse, ExamRegistrationState,
    },
    moduledetails::ModuleDetailsRequest,
    myexams::{Exam, MyExamsResponse},
};

use crate::{
    COURSEDETAILS_REGEX,
    head::{footer, html_head, logged_in_head},
    registration::MODULEDETAILS_REGEX,
};

#[expect(clippy::too_many_lines)]
pub(crate) fn exam_registration_internal(
    login_response: &LoginResponse,
    content: &str,
    _nothing: &(),
) -> Result<ExamRegistrationResponse, TucanError> {
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
                        "-ssflzGzRRKnVffWx8j8K20KtkmS7AKd-Cy1Z2bkiyM"
                    </style>
                </head>
                <body class="exam_registration">
                    use logged_in_head(html_handler, login_response.id).0;
                    <script type="text/javascript">
                    </script>
                    <h1>
                        _pruefungen_von_name
                    </h1>
                    <div class="tb">
                        <form id="semesterchange" action="/scripts/mgrqispi.dll" method="post" class="pageElementTop">
                            <div>
                                <div class="tbhead">
                                    "Prüfungen"
                                </div>
                                <div class="tbsubhead">
                                    "Wählen Sie ein Semester"
                                </div>
                                <div class="formRow">
                                    <div class="inputFieldLabel long">
                                        <label for="semester">
                                            "Veranstaltungs-/Modulsemester:"
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
                                <input name="PRGNAME" type="hidden" value="EXAMREGISTRATION"></input>
                                <input name="ARGUMENTS" type="hidden" value="sessionno,menuno,semester,extendedlist"></input>
                                <input name="sessionno" type="hidden" value=_session_id></input>
                                <input name="menuno" type="hidden" value="000318"></input>
                                <input name="extendedlist" type="hidden" value="0"></input>
                            </div>
                        </form>
                        <table class="nb list">
                            <thead>
                                <tr class="tbcontrol">
                                    <td colspan="5">
                                        <a href=_myexams_url class="img">
                                            "Meine Prüfungen"
                                        </a>
                                    </td>
                                </tr>
                                <tr class="tbsubhead">
                                    <td style="width:50px;"><b>"Nr."</b></td>
                                    <td><b>"Veranstaltung/Modul"</b></td>
                                    <td><b>"Prüfung"</b></td>
                                    <td><b>"Datum"</b></td>
                                    <td></td>
                                </tr>
                            </thead>
                            <tbody>
                                let exam_registrations = while html_handler.peek().is_some() {
                                    <tr class="tbsubhead level02">
                                        <td>
                                            course_id
                                        </td>
                                        <td colspan="4">
                                            name
                                            <br></br>
                                            ids
                                        </td>
                                    </tr>
                                    let registrations = while html_handler.peek().is_some() && html_handler.peek().unwrap().value().as_element().unwrap().has_class("tbdata", CaseSensitivity::CaseSensitive) {
                                        <tr class="tbdata">
                                            <td></td>
                                            <td></td>
                                            <td>
                                                <a class="link" href=examdetail_url>
                                                    pruefungsart
                                                </a>
                                            </td>
                                            <td>
                                                date
                                            </td>
                                            <td>
                                                let registration_state = if html_handler.peek().is_some() {
                                                    let examunreg_url = if html_handler.peek().unwrap().value().is_text() {
                                                        "Ausgewählt"
                                                    } => ExamRegistrationState::ForceSelected else {
                                                        <a href=examunreg_url class="img img_arrowLeftRed">
                                                            "Abmelden"
                                                        </a>
                                                    } => ExamRegistrationState::Registered(examunreg_url);
                                                } => examunreg_url.either_into();
                                            </td>
                                        </tr>
                                    } => ExamRegistration { registration_state: registration_state.unwrap_or(ExamRegistrationState::NotPossible) };
                                } => ExamRegistrationCourse { registrations };
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
        use footer(html_handler, login_response.id, 326);
    }
    html_handler.end_document();
    Ok(ExamRegistrationResponse {
        semester,
        exam_registrations,
    })
}

#[test]
fn test_exam_registration_internal() {
    let result = exam_registration_internal(
        &LoginResponse {
            id: 0,
            cookie_cnsc: String::new(),
        },
        include_str!("../test-data/EXAMREGISTRATION.html"),
        &(),
    )
    .unwrap();
    println!("{:?}", result);
}
