use std::sync::LazyLock;

use regex::Regex;
use scraper::CaseSensitivity;
use tucan_types::{
    LoginResponse,
    coursedetails::CourseDetailsRequest,
    moduledetails::ModuleDetailsRequest,
    registration::{
        AnmeldungCourse, AnmeldungEntry, AnmeldungExam, AnmeldungModule, AnmeldungRequest,
        AnmeldungResponse, RegistrationState, Studiumsauswahl,
    },
};

use crate::{
    COURSEDETAILS_REGEX, TucanError,
    head::{footer, html_head, logged_in_head},
};
use html_handler::{InElement, InRoot, MyElementRef, MyNode, Root, parse_document};

pub static MODULEDETAILS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&PRGNAME=MODULEDETAILS&ARGUMENTS=-N\\d+,-N\\d+,",
    )
    .unwrap()
});

fn is_tbsubhead(
    html_handler: &InElement<
        '_,
        InElement<
            '_,
            InElement<
                '_,
                InElement<
                    '_,
                    InElement<
                        '_,
                        InElement<'_, InElement<'_, InElement<'_, InRoot<'_, Root<'_>>>>>,
                    >,
                >,
            >,
        >,
    >,
) -> bool {
    html_handler
        .peek()
        .unwrap()
        .children()
        .next()
        .unwrap()
        .value()
        .as_element()
        .unwrap()
        .has_class("tbsubhead", scraper::CaseSensitivity::CaseSensitive)
}

#[expect(clippy::too_many_lines, clippy::cognitive_complexity)]
pub(crate) fn anmeldung_internal(
    login_response: &LoginResponse,
    content: &str,
    _nothing: &(),
) -> Result<AnmeldungResponse, TucanError> {
    static REGISTRATION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&PRGNAME=REGISTRATION&ARGUMENTS=-N\\d+,\
             -N000311,",
        )
        .unwrap()
    });
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^\p{Alphabetic}{2}, \d{1,2}\. \p{Alphabetic}{3}\. \d{4} \[\d\d:\d\d\] - \p{Alphabetic}{2}, \d{1,2}\. \p{Alphabetic}{3}\. \d{4} \[\d\d:\d\d\]$").unwrap()
    });
    let document = parse_document(content);
    let html_handler = Root::new(document.root());
    let html_handler = html_handler.document_start();
    let html_handler = html_handler.doctype();
    html_extractor::html! {
            <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="de" lang="de">
                <head>
                    use html_head(html_handler)?;
                    <style type="text/css">
                        "lbOQfuwTSH1NQfB9sjkC-_xOS0UGzyKBoNNl8bXs_FE"
                    </style>
                    <style type="text/css">
                        "qZ_1IiJLIcPvkbl6wYm5QbasBhsSKdRw5fl6vVyINxY"
                    </style>
                </head>
                <body class="registration">
                    use logged_in_head(html_handler, login_response.id).0;
                    <script type="text/javascript">
                    </script>
                    <h1>
                        "Anmeldung zu Modulen und Veranstaltungen"
                    </h1>
                    let studiumsauswahl = if html_handler.peek().unwrap().value().as_element().unwrap().name() == "form" {
                        <form id="registration" action="/scripts/mgrqispi.dll" method="post">
                            <table class="tbcoursestatus rw-table rw-all">
                                <tbody>
                                    <tr>
                                        <td class="tbhead" colspan="100%">
                                            "Weitere Studien"
                                        </td>
                                    </tr>
                                    <tr>
                                        <td class="tbcontrol" colspan="100%">
                                            <div class="inputFieldLabel">
                                                <label for="study">
                                                    "Studium:"
                                                </label>
                                                <select name="study" id="study" onchange="reloadpage.submitForm(this.form.id);" class="pageElementLeft">
                                                    let studiumsauswahl = while html_handler.peek().is_some() {
                                                        let studiumsauswahl = if html_handler
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
                                                        } => Studiumsauswahl {
                                                            name,
                                                            value: AnmeldungRequest::parse(&format!("-N{value},-N0,-N0,-N0")),
                                                            selected: true
                                                        } else {
                                                            <option value=value>
                                                                name
                                                            </option>
                                                        } => Studiumsauswahl {
                                                            name,
                                                            value: AnmeldungRequest::parse(&format!("-N{value},-N0,-N0,-N0")),
                                                            selected: false
                                                        };
                                                    } => studiumsauswahl.either_into();
                                                </select>
                                                <input name="Aktualisieren" type="submit" value="Aktualisieren" class="img img_arrowReload pageElementLeft"></input>
                                            </div>
                                            <input name="APPNAME" type="hidden" value="CampusNet"></input>
                                            <input name="PRGNAME" type="hidden" value="REGISTRATION"></input>
                                            <input name="ARGUMENTS" type="hidden" value="sessionno,menuno,study,changestudy,parent1,parent2"></input>
                                            <input
                                                name="sessionno"
                                                type="hidden"
                                                value={|v: String| {
                                                    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^\\d+$").unwrap());
                                                    assert!(REGEX.is_match(&v), "{v}");
                                                }}
                                            ></input>
                                            <input name="menuno" type="hidden" value="000311"></input>
                                            <input name="pa rent1" type="hidden" value="000000000000000"></input>
                                            <input name="parent2" type="hidden" value="000000000000000"></input>
                                            <input name="changestudy" type="hidden" value="1"></input>
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                        </form>
                    } => studiumsauswahl;
                    <h2>
                        <a href=registration_url>
                            study
                        </a>
                        let path = while html_handler.peek().is_some() {
                            ">"
                            <a href=url>
                                let any_child = if html_handler.peek().is_some() {
                                    let any_child = html_handler.next_any_child();
                                } => any_child;
                            </a>
                        } => match any_child.map(|c| c.value()) {
                            Some(MyNode::Text(text)) => {
                                let url = REGISTRATION_REGEX.replace(&url, "");
                                Some((text.to_string(), AnmeldungRequest::parse(&url)))
                            }
                            None => None,
                            _ => panic!(),
                        };
                        extern {
                            let registration_url = REGISTRATION_REGEX.replace(&registration_url, "");
                            path.insert(0, Some((study, AnmeldungRequest::parse(&registration_url))));
                        }
                    </h2>
                    let submenus = if html_handler.peek().and_then(|e| e.value().as_element()).is_some_and(|e| e.name() == "ul") {
                        <ul>
                            let submenus = while html_handler.peek().is_some() {
                                <li>
                                    <a href=url>
                                        item
                                    </a>
                                </li>
                            } => (item, AnmeldungRequest::parse(&REGISTRATION_REGEX.replace(&url, "")));
                        </ul>
                    } => submenus;
                    let additional_information = while html_handler.peek().unwrap().next_sibling().is_some()
                        && html_handler
                            .peek()
                            .and_then(ego_tree::NodeRef::next_sibling)
                            .and_then(|e| e.value().as_element())
                            .is_none_or(|e| !e.has_class("tbcoursestatus", CaseSensitivity::CaseSensitive)) {
                        let child = html_handler.next_any_child();
                    } => if let MyNode::Element(_element) = child.value() {
                        Some(MyElementRef::wrap(child).unwrap().html())
                    } else {
                        panic!()
                    };
                    <br></br>
                    let anmeldung_entries = if html_handler.peek().is_some() {
                        <table class="tbcoursestatus rw-table rw-all">
                            <tbody>
                                <tr>
                                    <td class="tbhead" colspan="100%">
                                        "Anmeldung zu Modulen und Veranstaltungen"
                                    </td>
                                </tr>
                                <tr>
                                    let anmeldung_entries = if html_handler.peek().unwrap().value().as_element().unwrap().attr("class").unwrap() == "tbdata" {
                                                    <td class="tbdata" colspan="4">
                                                        "Keine Module oder Veranstaltungen zur Anmeldung gefunden"
                                                    </td>
                                                </tr>
                                            </tbody>
                                        </table>
                                    } => () else {
                                                    <td class="tbsubhead">
                                                    </td>
                                                    <td class="tbsubhead">
                                                        "Veranstaltung"
                                                        <br></br>
                                                        "Dozenten"
                                                        <br></br>
                                                        "Zeitraum"
                                                        <br></br>
                                                        "Anmeldegruppe"
                                                        <br></br>
                                                        "Standort"
                                                    </td>
                                                    <td class="tbsubhead">
                                                        "Anmeld. bis"
                                                        <br></br>
                                                        "Max.Teiln.|Anm."
                                                    </td>
                                                    <td class="tbsubhead">
                                                    </td>
                                                </tr>
                                                let anmeldung_entries = while html_handler.peek().is_some() {
                                                    let module = if html_handler.peek().is_some() && is_tbsubhead(&html_handler) {
                                                        <tr>
                                                            <td class="tbsubhead">
                                                            </td>
                                                            <td class="tbsubhead dl-inner">
                                                                <p>
                                                                    <strong>
                                                                        <a href=module_url>
                                                                            module_id
                                                                            <span class="eventTitle">
                                                                                module_name
                                                                            </span>
                                                                        </a>
                                                                    </strong>
                                                                </p>
                                                                <p>
                                                                    lecturer
                                                                </p>
                                                            </td>
                                                            <td class="tbsubhead">
                                                                let date = if html_handler.peek().unwrap().value().is_text() {
                                                                    date
                                                                } => date;
                                                                <br></br>
                                                                let limit_and_size = if html_handler.peek().is_some() {
                                                                    limit_and_size
                                                                } => limit_and_size;
                                                            </td>
                                                            <td class="tbsubhead rw-qbf">
                                                                let registration_state = if html_handler.peek().is_some() {
                                                                    let registered = if html_handler
                                                                        .peek()
                                                                        .unwrap()
                                                                        .value()
                                                                        .as_element()
                                                                        .unwrap()
                                                                        .attr("class")
                                                                        .unwrap()
                                                                        == "img noFloat register" {
                                                                        <a href=registration_button_link class="img noFloat register">
                                                                            "Anmelden"
                                                                        </a>
                                                                    } => RegistrationState::NotRegistered {
                                                                        register_link: registration_button_link
                                                                    } else {
                                                                        <a href=registration_button_link class="img img_arrowLeftRed noFLoat unregister">
                                                                            "Abmelden"
                                                                        </a>
                                                                    } => RegistrationState::Registered {
                                                                        unregister_link: registration_button_link
                                                                    };
                                                                } => registered.either_into::<RegistrationState>() else {
                                                                } => RegistrationState::Unknown;
                                                            </td>
                                                        </tr>
                                                    } => {
                                                        let module_url = MODULEDETAILS_REGEX.replace(&module_url, "");
                                                        let module_url = module_url.split_once(",-A").unwrap().0;

                                                        AnmeldungModule {
                                                            url: ModuleDetailsRequest::parse(module_url),
                                                            id: module_id,
                                                            name: module_name,
                                                            lecturer: if lecturer == "N.N." { None } else { Some(lecturer) },
                                                            date,
                                                            limit_and_size,
                                                            registration_state: registration_state.either_into(),
                                                        }
                                                    };
                                                    let courses = while html_handler.peek().is_some()
                                                        && !html_handler
                                                            .peek()
                                                            .unwrap()
                                                            .children()
                                                            .next()
                                                            .unwrap()
                                                            .value()
                                                            .as_element()
                                                            .unwrap()
                                                            .has_class("tbsubhead", scraper::CaseSensitivity::CaseSensitive) {
                                                        let exam = if html_handler
                                                            .peek()
                                                            .unwrap()
                                                            .children()
                                                            .nth(1)
                                                            .unwrap()
                                                            .value()
                                                            .as_element()
                                                            .unwrap()
                                                            .attr("class")
                                                            .unwrap()
                                                            == "tbdata" {
                                                            <tr>
                                                                <td class="tbdata">
                                                                </td>
                                                                <td class="tbdata">
                                                                    exam_name
                                                                    let exam_type = if html_handler.peek().is_some() {
                                                                        <br></br>
                                                                        exam_type
                                                                    } => exam_type;
                                                                </td>
                                                                <td class="tbdata">
                                                                </td>
                                                                <td class="tbdata">
                                                                </td>
                                                            </tr>
                                                        } => AnmeldungExam { name: exam_name, typ: exam_type };
                                                        <tr>
                                                            <td class="tbdata">
                                                                let gefaehrdung_schwangere = if html_handler.peek().is_some() {
                                                                    <img src="../../gfx/_default/icons/eventIcon.gif" title="Gefährdungspotential für Schwangere"></img>
                                                                } => ();
                                                            </td>
                                                            <td class="tbdata dl-inner">
                                                                <p>
                                                                    <strong>
                                                                        <a href=course_url name="eventLink">
                                                                            course_id
                                                                            <span class="eventTitle">
                                                                                course_name
                                                                            </span>
                                                                        </a>
                                                                    </strong>
                                                                </p>
                                                                <p>
                                                                    let lecturers = if html_handler.peek().is_some()
                                                                        && !RE.is_match(
                                                                            html_handler.peek().unwrap().value().as_text().unwrap()
                                                                        ) {
                                                                            lecturers
                                                                        </p>
                                                                        <p>
                                                                    } => lecturers;
                                                                    let begin_and_end = if html_handler.peek().is_some() {
                                                                            begin_and_end
                                                                        </p>
                                                                        <p>
                                                                    } => begin_and_end;
                                                                    let location_or_additional_info = if html_handler
                                                                        .peek()
                                                                        .is_some() {
                                                                            let location_or_additional_info = html_handler.next_any_child();
                                                                        </p>
                                                                    } => match location_or_additional_info.value() {
                                                                        MyNode::Text(text) => text.to_string(),
                                                                        MyNode::Element(_element) => MyElementRef::wrap(location_or_additional_info).unwrap().html(),
                                                                        _ => panic!(),
                                                                    } else {
                                                                        </p>
                                                                    } => ();
                                                                let location = if html_handler.peek().is_some() {
                                                                    <p>
                                                                        let location = if html_handler.peek().is_some() {
                                                                            location
                                                                        } => location;
                                                                    </p>
                                                                } => location;
                                                            </td>
                                                            <td class="tbdata">
                                                                let registration_until = if html_handler
                                                                    .peek()
                                                                    .unwrap()
                                                                    .value()
                                                                    .is_text() {
                                                                    registration_until
                                                                } => registration_until;
                                                                <br></br>
                                                                let limit_and_size = if html_handler.peek().is_some() {
                                                                    limit_and_size
                                                                } => limit_and_size;
                                                            </td>
                                                            <td class="tbdata rw-qbf">
                                                                let registration_button_link = if html_handler.peek().is_some() {
                                                                    let registration_button_link = if html_handler.peek().unwrap().value().is_text() {
                                                                        _anmeldung_nur_in_anderem_studiengang_möglich
                                                                    } => RegistrationState::NotPossible else {
                                                                        let registration_button_link = if html_handler
                                                                            .peek()
                                                                            .unwrap()
                                                                            .value()
                                                                            .as_element()
                                                                            .unwrap()
                                                                            .attr("class")
                                                                            .unwrap()
                                                                            == "img noFLoat register" {
                                                                            <a href=registration_button_link class="img noFLoat register">
                                                                                "Anmelden"
                                                                            </a>
                                                                        } => RegistrationState::NotRegistered {
                                                                            register_link: registration_button_link
                                                                        } else {
                                                                            <a href=registration_button_link class="img img_arrowLeftRed noFLoat unregister">
                                                                                "Abmelden"
                                                                            </a>
                                                                        } => RegistrationState::Registered {
                                                                            unregister_link: registration_button_link
                                                                        };
                                                                    } => registration_button_link.either_into::<RegistrationState>();
                                                                } => registration_button_link.either_into::<RegistrationState>() else {
                                                                } => RegistrationState::Unknown;
                                                            </td>
                                                        </tr>
                                                    } => {
                                                        let course_url = COURSEDETAILS_REGEX.replace(&course_url, "");
                                                        let course_url = course_url.split_once(",-A").unwrap().0;
                                                        let course = AnmeldungCourse {
                                                            gefaehrdung_schwangere: gefaehrdung_schwangere.is_some(),
                                                            url: CourseDetailsRequest::parse(course_url),
                                                            id: course_id,
                                                            name: course_name,
                                                            lecturers,
                                                            begin_and_end,
                                                            registration_until,
                                                            limit_and_size,
                                                            registration_button_link: registration_button_link.either_into(),
                                                            location_or_additional_info: location_or_additional_info.left(),
                                                            location: location.flatten(),
                                                        };
                                                        (exam, course)
                                                    };
                                                } => AnmeldungEntry { module, courses };
                                            </tbody>
                                        </table>
                                    } => anmeldung_entries;
                    } => anmeldung_entries.right().unwrap_or_default();
                </div>
            </div>
        </div>
    };
    let html_handler = footer(html_handler, login_response.id, 311);
    html_handler.end_document();
    let path: Vec<(String, AnmeldungRequest)> = path.into_iter().flatten().collect();
    Ok(AnmeldungResponse {
        studiumsauswahl: studiumsauswahl.unwrap_or_else(|| {
            vec![Studiumsauswahl {
                name: path.first().unwrap().0.clone(),
                value: path.first().unwrap().1.clone(),
                selected: true,
            }]
        }),
        path,
        submenus: submenus.unwrap_or_default(),
        entries: anmeldung_entries.unwrap_or_default(),
        additional_information: additional_information.into_iter().flatten().collect(),
    })
}
