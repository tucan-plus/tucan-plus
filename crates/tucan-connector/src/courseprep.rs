// https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=COURSEPREP&ARGUMENTS=-N503526614753137,-N000268,-N0,-N391553606081258,-ACODA,-N393006535520104

// just use the url of another course and change the last number to the last
// number in the coursedetails url seems like there is access control. if you
// are not in a course it does not work. though you could easily register and
// unregister again
use crate::head::{html_head, logged_in_head, logged_out_head};
use html_handler::{Root, parse_document};
use tucan_types::{
    LoginResponse, TucanError,
};
/*
    assert_eq!(request.r#type, CoursePrepType::Course);
    let key = format!(
        "unparsed_course_prep.{}.{}",
        login_response.id, request.course_id
    );

    let url = format!(
        "https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=COURSEPREP&ARGUMENTS=-N{:015},-N000268,{}",
        login_response.id, request
    );
*/

#[allow(clippy::too_many_lines)]
#[allow(unused)]
pub(crate) fn course_prep_internal(
    login_response: &LoginResponse,
    content: &str,
) -> Result<String, TucanError> {
    let document = parse_document(content);
    let html_handler = Root::new(document.root());
    let html_handler = html_handler.document_start();
    let html_handler = html_handler.doctype();
    html_extractor::html! {
        <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="de" lang="de">
            <head>
                use html_head(html_handler)?;
                <style type="text/css">
                    "7eLX2t1Uo0IA6vL9eg7XnOUPvbqtOEwtcyPy3J7000g"
                </style>
            </head>
            <body class="courseprep">
                use if login_response.id == 1 {
                    logged_out_head(html_handler).0
                } else {
                    logged_in_head(html_handler, login_response.id).0
                };
                <script type="text/javascript">
                </script>
                <script language="JavaScript">
                    _trash
                </script>
                <h1>
                    title
                </h1>
                let _kleingruppe = if html_handler
                    .peek()
                    .unwrap()
                    .value()
                    .as_element()
                    .unwrap()
                    .name()
                    == "h2" {
                    <h2>
                        _kleingruppe
                    </h2>
                } => ();
                <p>
                    <span name="appointmentDate">
                        _date
                    </span>
                    <span name="appointmentTimeFrom">
                        _start
                    </span>
                    "-"
                    <span name="appointmentTimeTo">
                        _end
                    </span>
                </p>
                let _raeume = if html_handler
                    .peek()
                    .unwrap()
                    .value()
                    .as_element()
                    .unwrap()
                    .name()
                    == "h2" {
                    <h2>
                        "Räume:"
                    </h2>
                    let _room = if html_handler
                        .peek()
                        .unwrap()
                        .value()
                        .as_element()
                        .unwrap()
                        .name()
                        == "span" {
                        <span name="appoinmentRooms">
                            _room
                        </span>
                    } => () else {
                        let _rooms = while html_handler
                            .peek()
                            .unwrap()
                            .value()
                            .as_element()
                            .unwrap()
                            .name()
                            == "a" {
                            <a name="appoinmentRooms" class="arrow" href=_room_href>
                                _room
                            </a>
                        } => ();
                    } => ();
                    <div style="clear:both;">
                    </div>
                } => ();
                <div class="contentlayoutleft" id="contentlayoutleft">
                    <table class="tb">
                        <tbody>
                            <tr>
                                <td class="tbhead" colspan="2">
                                    "Material zu einzelnen Terminen"
                                </td>
                            </tr>
                            <tr>
                                <td class="tbcontrol" colspan="2">
                                    <a id="Popup_link" href=_message_url class="arrow">
                                        "Neue Nachricht"
    }
    let _ = html_handler;
    Ok(title)
}
