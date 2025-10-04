use std::str::FromStr;

use html_handler::{Root, parse_document};
use tucan_types::{
    LoginResponse, SemesterId, Semesterauswahl, TucanError,
    coursedetails::CourseDetailsRequest,
    moduledetails::ModuleDetailsRequest,
    myexams::{Exam, MyExamsResponse},
};

use crate::{
    COURSEDETAILS_REGEX,
    head::{footer, html_head, logged_in_head},
    registration::MODULEDETAILS_REGEX,
};

#[expect(clippy::too_many_lines)]
pub(crate) fn my_exams_internal(
    login_response: &LoginResponse,
    content: &str,
    _nothing: &(),
) -> Result<MyExamsResponse, TucanError> {
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
                <body class="myexams">
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
                                <input name="PRGNAME" type="hidden" value="MYEXAMS"></input>
                                <input name="ARGUMENTS" type="hidden" value="sessionno,menuno,semester"></input>
                                <input name="sessionno" type="hidden" value=_session_id></input>
                                <input name="menuno" type="hidden" value="000318"></input>
                            </div>
                        </form>
                        <table class="nb list">
                            <thead>
                                <tr class="tbcontrol">
                                    <td colspan="5">
                                        <a href=_examregistration_url class="arrow">
                                            "Anmeldung zu Prüfungen"
                                        </a>
                                    </td>
                                </tr>
                                <tr>
                                    <th scope="col" id="Nr.">
                                        "Nr."
                                    </th>
                                    <th scope="col" id="Course_event_module">
                                        "Veranstaltung/Modul"
                                    </th>
                                    <th scope="col" id="Name">
                                        "Name"
                                    </th>
                                    <th scope="col" id="Date">
                                        "Datum"
                                    </th>
                                    <th>
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                let exams = while html_handler.peek().is_some() {
                                    <tr>
                                        <td class="tbdata">
                                            course_id
                                        </td>
                                        <td class="tbdata">
                                            let res = if html_handler.peek().unwrap().value().as_element().unwrap().attr("name").is_some() {
                                                <a class="link" name="eventLink" href=coursedetails_url>
                                                    name
                                                </a>
                                                <br></br>
                                                tuple_of_courses
                                            } => (
                                                name,
                                                Some(CourseDetailsRequest::parse(&COURSEDETAILS_REGEX.replace(&coursedetails_url, ""))),
                                                None,
                                                Some(tuple_of_courses)
                                            ) else {
                                                <a class="link" href=moduledetails_url>
                                                    name
                                                </a>
                                                let _thesis = if html_handler.peek().is_some() {
                                                    <br></br>
                                                    <b>
                                                        "Thema:"
                                                    </b>
                                                    _topic
                                                    <br></br>
                                                    let _submitted = if html_handler.peek().is_some() {
                                                        _submitted_date
                                                        <br></br>
                                                    } => ();
                                                } => ();
                                            } => (name, None, Some(ModuleDetailsRequest::parse(&MODULEDETAILS_REGEX.replace(&moduledetails_url, ""))), None);
                                        </td>
                                        <td class="tbdata">
                                            <a class="link" href=examdetail_url>
                                                pruefungsart
                                            </a>
                                        </td>
                                        <td class="tbdata">
                                            let date_and_courseprep = if html_handler.peek().unwrap().value().is_text() {
                                                date
                                            } => (date, None) else {
                                                <a class="link" href=courseprep_url>
                                                    date
                                                </a>
                                            } => (date, Some(courseprep_url));
                                        </td>
                                        <td class="tbdata">
                                            let examunreg_url = if html_handler.peek().unwrap().value().is_text() {
                                                "Ausgewählt"
                                            } => () else {
                                                <a href=examunreg_url class="img img_arrowLeftRed">
                                                    "Abmelden"
                                                </a>
                                            } => examunreg_url;
                                        </td>
                                    </tr>
                                } => Exam {
                                    id: course_id,
                                    name: res.clone().either_into::<(String, Option<CourseDetailsRequest>, Option<ModuleDetailsRequest>, Option<String>)>().0,
                                    coursedetails_url: res.clone().either_into::<(String, Option<CourseDetailsRequest>, Option<ModuleDetailsRequest>, Option<String>)>().1,
                                    moduledetails_url: res.clone().either_into::<(String, Option<CourseDetailsRequest>, Option<ModuleDetailsRequest>, Option<String>)>().2,
                                    tuple_of_courses: res.either_into::<(String, Option<CourseDetailsRequest>, Option<ModuleDetailsRequest>, Option<String>)>().3,
                                    examdetail_url,
                                    pruefungsart,
                                    date: date_and_courseprep.clone().either_into::<(String, Option<String>)>().0,
                                    courseprep_url: date_and_courseprep.either_into::<(String, Option<String>)>().1,
                                    examunreg_url: examunreg_url.right(),
                                };
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
        use footer(html_handler, login_response.id, 326);
    }
    html_handler.end_document();
    Ok(MyExamsResponse { semester, exams })
}
