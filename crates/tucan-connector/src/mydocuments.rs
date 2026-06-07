use html_handler::{Root, parse_document};
use tucan_types::{
    LoginResponse, TucanError,
    mydocuments::{Document, MyDocumentsResponse},
};

use crate::head::{footer, html_head, logged_in_head};

pub(crate) fn my_documents_internal(
    login_response: &LoginResponse,
    content: &str,
    _nothing: &(),
) -> Result<MyDocumentsResponse, TucanError> {
    let document = parse_document(content);
    let html_handler = Root::new(document.root());
    let html_handler = html_handler.document_start();
    let html_handler = html_handler.doctype();
    html_extractor::html! {
            <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="de" lang="de">
                <head>
                    use html_head(html_handler)?;
                    <style type="text/css">
                        "41e2ICphaTflKCWDw0-D6hVTXfSC73XVLf9m4PcOeCc"
                    </style>
                    <style type="text/css">
                        "LV-UhM-gCRhIwiPhsdMIXFVizLpnmQ4ZWr5GVZR9YMU"
                    </style>
                </head>
                <body class="createdocument">
                    use logged_in_head(html_handler, login_response.id).0;
                    <script type="text/javascript">
                    </script>
                    <h1>
                        _dokumente_von_name
                    </h1>
                    <form action="/scripts/mgrqispi.dll" name="form1" id="form1" method="post">
                        <div>
                            <input type="hidden" name="templateid" value="1"></input>
                            <input type="hidden" name="status" value="0"></input>
                            <input type="hidden" name="date_from" value=""></input>
                            <input type="hidden" name="date_to" value=""></input>
                            <table class="tb">
                                <tbody>
                                    <tr>
                                        <td class="tbhead">
                                            "Name"
                                        </td>
                                        <td class="tbhead">
                                            "Datum"
                                        </td>
                                        <td class="tbhead">
                                            "Zeit"
                                        </td>
                                        <td class="tbhead">
                                            "Status"
                                        </td>
                                        <td class="tbhead">
                                        </td>
                                    </tr>
                                    let documents = while html_handler.peek().is_some() {
                                        <tr>
                                            <td class="tbdata">
                                                name
                                            </td>
                                            <td class="tbdata">
                                                date
                                            </td>
                                            <td class="tbdata">
                                                time
                                            </td>
                                            <td class="tbdata">
                                            </td>
                                            <td class="tbdata">
                                                <a class="img download" href=url>
                                                    "Download"
                                                </a>
                                            </td>
                                        </tr>
                                    } => Document {
                                        name,
                                        date,
                                        time,
                                        url
                                    };
                                </tbody>
                            </table>
                            <input name="APPNAME" type="hidden" value="CampusNet"></input>
                            <input name="PRGNAME" type="hidden" value="CREATEDOCUMENT"></input>
                            <input
                                name="ARGUMENTS"
                                type="hidden"
                                value="sessionno,menuid,mode,templateid,status,date_from,date_to,documentid"
                            ></input>
                            <input name="sessionno" type="hidden" value=_session_id></input>
                            <input name="menuid" type="hidden" value="000557"></input>
                            <input name="mode" type="hidden" value="1"></input>
                        </div>
                    </form>
                </div>
            </div>
        </div>
        use footer(html_handler, login_response.id, 326);
    }
    html_handler.end_document();
    Ok(MyDocumentsResponse { documents })
}
