use std::str::FromStr;

use html_handler::{Root, parse_document};
use tucan_types::{
    LoginResponse, SemesterId, Semesterauswahl, TucanError,
    moduledetails::ModuleDetailsRequest,
    mymodules::{Module, MyModulesResponse},
};

use crate::{
    head::{footer, html_head, logged_in_head},
    registration::MODULEDETAILS_REGEX,
};

#[expect(clippy::too_many_lines)]
pub(crate) fn my_modules_internal(
    login_response: &LoginResponse,
    content: &str,
    _nothing: &(),
) -> Result<MyModulesResponse, TucanError> {
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
                        "ez4igVpcXJnoZyie_yy-b7wKrGd2q4L-BvRmEYSDi2k"
                    </style>
                </head>
                <body class="mymodules">
                    use logged_in_head(html_handler, login_response.id).0;
                    <script type="text/javascript">
                    </script>
                    <h1>
                        _module_von_name
                    </h1>
                    <div class="tb">
                        <form id="semesterchange" action="/scripts/mgrqispi.dll" method="post" class="pageElementTop">
                            <div>
                                <div class="tbhead">
                                    "Modul"
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
                                        <input name="Refresh" type="submit" value="Aktualisieren" class="img img_arrowReload refresh"></input>
                                    </div>
                                </div>
                                <input name="APPNAME" type="hidden" value="CampusNet"></input>
                                <input name="PRGNAME" type="hidden" value="MYMODULES"></input>
                                <input name="ARGUMENTS" type="hidden" value="sessionno,menuno,semester"></input>
                                <input name="sessionno" type="hidden" value=_session_id></input>
                                <input name="menuno" type="hidden" value="000275"></input>
                            </div>
                        </form>
                        <table class="nb list rw-table rw-all">
                            <thead>
                                <tr class="tbsubhead rw-hide">
                                    <th>
                                    </th>
                                    <th id="Nr.">
                                        "Nr."
                                    </th>
                                    <th id="Name">
                                        "Name"
                                    </th>
                                    <th id="Modulverantwortliche">
                                        "Modulverantwortliche"
                                    </th>
                                    <th id="Credits">
                                        "Credits"
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                let modules = while html_handler.peek().is_some() {
                                    <tr class="tbdata ">
                                        <td class="rw rw-mod-logo">
                                        </td>
                                        <td headers="Nr." class="rw rw-mod-no">
                                            module_nr
                                        </td>
                                        <td headers="Name" class="rw rw-mod-name">
                                            <a class="link" href=moduledetails_url>
                                                module_title
                                            </a>
                                        </td>
                                        <td headers="Modulverantwortliche" class="rw rw-mod-prof">
                                            lecturer
                                        </td>
                                        <td headers="Credits" class="rw rw-mod-credits" style="text-align:left">
                                            let credits = if html_handler.peek().is_some() {
                                                credits
                                            } => credits;
                                        </td>
                                    </tr>
                                } => Module {
                                    nr: module_nr,
                                    title: module_title,
                                    url: ModuleDetailsRequest::parse(MODULEDETAILS_REGEX.replace(&moduledetails_url, "").split_once(",-A").unwrap().0),
                                    lecturer,
                                    credits
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
    Ok(MyModulesResponse { semester, modules })
}
