use log::info;
use time::{Duration, OffsetDateTime};
use tucant_types::{
    LoginResponse, RevalidationStrategy, TucanError,
    coursedetails::CourseDetailsRequest,
    vv::{ActionRequest, Veranstaltung, Vorlesungsverzeichnis},
};

use crate::{
    COURSEDETAILS_REGEX, TucanConnector, authenticated_retryable_get,
    common::head::{ACTION_REGEX, footer, html_head, logged_in_head, logged_out_head},
    retryable_get,
};
use html_handler::{MyElementRef, MyNode, Root, parse_document};

pub async fn vv(tucan: &TucanConnector, login_response: Option<&LoginResponse>, revalidation_strategy: RevalidationStrategy, request: ActionRequest) -> Result<Vorlesungsverzeichnis, TucanError> {
    let key = format!("unparsed_vv.{}", request.inner());

    let old_content_and_date = tucan.database.get::<(String, OffsetDateTime)>(&key).await;
    if revalidation_strategy.max_age != 0 {
        if let Some((content, date)) = &old_content_and_date {
            info!("{}", OffsetDateTime::now_utc() - *date);
            if OffsetDateTime::now_utc() - *date < Duration::seconds(revalidation_strategy.max_age) {
                return vv_internal(login_response, content);
            }
        }
    }

    let Some(invalidate_dependents) = revalidation_strategy.invalidate_dependents else {
        return Err(TucanError::NotCached);
    };

    let url = format!("https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=ACTION&ARGUMENTS={}", request.inner());
    let (content, date) = if let Some(login_response) = login_response { authenticated_retryable_get(tucan, &url, &login_response.cookie_cnsc).await? } else { retryable_get(tucan, &url).await? };
    let result = vv_internal(login_response, &content)?;
    if invalidate_dependents && old_content_and_date.as_ref().map(|m| &m.0) != Some(&content) {
        // TODO invalidate cached ones?
    }

    tucan.database.put(&key, (content, date)).await;

    Ok(result)
}

#[expect(clippy::too_many_lines)]
fn vv_internal(login_response: Option<&LoginResponse>, content: &str) -> Result<Vorlesungsverzeichnis, TucanError> {
    let document = parse_document(content);
    let html_handler = Root::new(document.root());
    let html_handler = html_handler.document_start();
    let html_handler = html_handler.doctype();
    html_extractor::html! {
            <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="de" lang="de">
                <head>
                    use html_head(html_handler)?;
                    <style type="text/css">
                        _ignore
                    </style>
                    let _ewf = if login_response.is_some() {
                        <style type="text/css">
                            _ignore
                        </style>
                    } => ();
                </head>
                <body class="registration_auditor">
                    use if let Some(login_response) = login_response { logged_in_head(html_handler, login_response.id).0 } else { logged_out_head(html_handler, 334).0 };
                    <script type="text/javascript">
                    </script>
                    <h1>
                        title
                    </h1>
                    <h2>
                        let path = while html_handler.peek().and_then(|e| e.next_sibling()).is_some() {
                            <a href=url>
                                title
                            </a>
                        } => (title, ActionRequest::parse(&ACTION_REGEX.replace(&url, "")));
                        <a href=_garbage_url>
                        </a>
                    </h2>
                    let description = if html_handler.peek().unwrap().value().is_element() && html_handler.peek().unwrap().value().as_element().unwrap().has_class("nb", scraper::CaseSensitivity::CaseSensitive) {
                        <div class="tb nb">
                            let description = while html_handler.peek().is_some() {
                                let any_child = html_handler.next_any_child();
                            } => match any_child.value() {
                                MyNode::Text(text) => text.to_string(),
                                MyNode::Element(_element) => MyElementRef::wrap(any_child).unwrap().html(),
                                _ => panic!(),
                            };
                        </div>
                    } => description;
                    let entries = if html_handler.peek().unwrap().value().as_element().unwrap().name() == "ul" {
                        <ul class="auditRegistrationList" id="auditRegistration_list">
                            let entries = while html_handler.peek().is_some() {
                                <li title=title xss="is-here">
                                    <a class="auditRegNodeLink" href=reg_href>
                                        title
                                    </a>
                                </li>
                            } => (title, ActionRequest::parse(&ACTION_REGEX.replace(&reg_href, "")));
                        </ul>
                    } => entries;
                    let veranstaltungen_or_module = if html_handler.peek().is_some() {
                        <div class="tb">
                            <div class="tbhead">
                                "Veranstaltungen / Module"
                            </div>
                            let veranstaltungen_or_module = if html_handler.peek().unwrap().value().as_element().unwrap().name() == "table" {
                                <table class="nb eventTable">
                                    <tbody>
                                        <tr class="tbsubhead">
                                            <th>
                                            </th>
                                            <th>
                                                "Veranstaltung / Modul"
                                                <br></br>
                                                "Dozenten / Modulverantwortliche"
                                                <br></br>
                                                "Zeitraum"
                                            </th>
                                            <th>
                                            </th>
                                            <th colspan="2">
                                                "Veranstaltungsart"
                                                <br></br>
                                                "Raum"
                                            </th>
                                        </tr>
                                        let veranstaltungen = while html_handler.peek().is_some() {
                                            <tr class="tbdata">
                                                <td>
                                                    let gefaehrdung_schwangere = if html_handler.peek().is_some() {
                                                        <img src="../../gfx/_default/icons/eventIcon.gif" title="Gefährdungspotential für Schwangere"></img>
                                                    } => ();
                                                </td>
                                                <td>
                                                    <a name="eventLink" href=coursedetails_url class="eventTitle">
                                                        title
                                                    </a>
                                                    <br></br>
                                                    let lecturer_name = if html_handler.peek().is_some() {
                                                        lecturer_name
                                                    } => lecturer_name;
                                                    let date_range = if html_handler.peek().is_some() {
                                                        <br></br>
                                                        date_range
                                                    } => date_range;
                                                </td>
                                                <td>
                                                </td>
                                                <td colspan="2">
                                                    course_type
                                                </td>
                                            </tr>
                                        } => Veranstaltung {
                                            title,
                                            coursedetails_url: CourseDetailsRequest::parse(&COURSEDETAILS_REGEX.replace(&coursedetails_url, "")),
                                            lecturer_name,
                                            date_range,
                                            course_type,
                                            gefaehrdung_schwangere: gefaehrdung_schwangere.is_some()
                                        };
                                    </tbody>
                                </table>
                            } => veranstaltungen else {
                                <div class="tbdata" colspan="3">
                                    "Es wurden keine Veranstaltungen gefunden."
                                </div>
                            } => Vec::<Veranstaltung>::new();
                        </div>
                    } => veranstaltungen_or_module.either_into();
                </div>
            </div>
        </div>
    }
    let html_handler = footer(html_handler, login_response.map_or(1, |l| l.id), 326);
    html_handler.end_document();
    Ok(Vorlesungsverzeichnis {
        title,
        entries: entries.unwrap_or_default(),
        path,
        description: description.unwrap_or_default(),
        veranstaltungen_or_module: veranstaltungen_or_module.unwrap_or_default(),
    })
}
