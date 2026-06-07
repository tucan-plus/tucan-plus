use tucan_types::LoginResponse;

use crate::{
    TucanConnector, TucanError, authenticated_retryable_get,
    head::{html_head, logged_in_head},
};
use html_handler::{Root, parse_document};

#[expect(clippy::too_many_lines)]
pub async fn veranstaltungen(
    connector: &TucanConnector,
    login_response: LoginResponse,
) -> Result<(), TucanError> {
    let (content, ..) = authenticated_retryable_get(
        connector,
        &format!(
            "https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=EXTERNALPAGES&ARGUMENTS=-N{:015},-N000273,-Astudveranst%2Ehtml",
            login_response.id
        ),
        &login_response.cookie_cnsc,
    )
    .await?;
    let document = parse_document(&content);
    let html_handler = Root::new(document.root());
    let html_handler = html_handler.document_start();
    let html_handler = html_handler.doctype();
    html_extractor::html! {
        <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="de" lang="de">
            <head>
                use html_head(html_handler)?;
                <style type="text/css">
                    "Z8Nk5s0HqiFiRYeqc3zP-bPxIN31ePraM-bbLg_KfNQ"
                </style>
            </head>
            <body class="external_pages">
                use logged_in_head(html_handler, login_response.id).0;
                <script type="text/javascript">
                </script>
                <div id="inhalt">
                    <h1>
                        "Das Menü Veranstaltungen"
                    </h1>
                    <div style="padding:0px; width:650px; margin:0px; background-color:#ffffff;">
                        <p>
                            "In diesem Bereich können Sie sich zu Modulen und Lehrveranstaltungen anmelden."
                        </p>
                        <p>
                            <strong>
                                "Benötigen Sie Hilfe im Umgang mit TUCaN?"
                                <a href="https://www.tu-darmstadt.de/studieren/studierende_tu/studienorganisation_und_tucan/hilfe_und_faq/index.de.jsp" target="_blank">
                                    "Hier"
                                </a>
                                "finden Sie zahlreiche Anleitungen und FAQs."
                            </strong>
                        </p>
                        <p>
                        </p>
                        <h3>
                            "Meine Module / Meine Veranstaltungen"
                        </h3>
                        <p style="line-height: 140%;">
                            "Im Untermenü"
                            <em>
                                "Meine Module"
                            </em>
                            "sehen Sie Module, die Sie im Laufe Ihres Studiums belegt haben. Im Untermenü"
                            <em>
                                "Meine Veranstaltungen"
                            </em>
                            "sehen Sie, in welchem Semester Sie welche Veranstaltungen belegt haben."
                        </p>
                        <br></br>
                        <h3>
                            "Meine Wahlbereiche"
                        </h3>
                        <p style="line-height: 140%;">
                            "Sofern Ihre Prüfungsordnung Nebenfächer, Schwerpunkte oder Wahlbereiche vorsieht, legen Sie diese im Untermenü"
                            <em>
                                "Meine Wahlbereiche"
                            </em>
                            "fest. Bitte halten Sie die Regeln Ihrer  Prüfungsordnung ein."
                        </p>
                        <br></br>
                        <h3>
                            "Anmeldung"
                        </h3>
                        <p style="line-height: 140%;">
                            "Im Untermenü"
                            <em>
                                "Anmeldung"
                            </em>
                            "melden Sie sich zu Modulen und Lehrveranstaltungen an."
                        </p>
                        <br></br>
                        <h3>
                            "Mein aktueller Anmeldestatus"
                        </h3>
                        <p style="line-height: 140%;">
                            "Im Untermenü"
                            <em>
                                "Mein aktueller Anmeldestatus"
                            </em>
                            "sehen Sie, zu welchen Veranstaltungen Sie sich im aktuellen Semester angemeldet haben. Hier erfahren Sie, welche Anmeldungen akzeptiert, welche abgelehnt wurden."
                        </p>
                        <p>
                        </p>
                    </div>
                    <div style="padding:10px; width:650px; border:thin solid #f8f9ed; margin:0px; background-color:#f8f9ed;">
                        <h1>
                            "Wichtige Hinweise zur Anmeldung"
                        </h1>
                        <p style="line-height: 140%;">
                            "Die Anmeldephase für die meisten Lehrveranstaltungen beginnt mit der Publikation des Vorlesungsverzeichnisses im"
                        </p>
                        <ul>
                            <li style="line-height: 140%;">
                                "Sommersemester: 1. Werktag im März"
                            </li>
                            <li style="line-height: 140%;">
                                "Wintersemester: 1. Werktag im September"
                            </li>
                        </ul>
                        "Die Fristen können individuell abweichen – bitte beachten Sie die Terminierungen Ihres Fachbereichs! Die jeweilige Anmeldephase ist in TUCaN bei jeder Lehrveranstaltung \
                         individuell aufgeführt."
                        <p>
                        </p>
                        <p style="line-height: 140%;">
                            "Bei Veranstaltungen mit Teilnahmebeschränkung wird in manchen Studiengängen die Anmeldung für den Zeitraum der Gruppeneinteilung ausgesetzt."
                        </p>
                        <p style="line-height: 140%;">
                            <strong>
                                "Bitte beachten Sie:"
                            </strong>
                        </p>
                        <ul>
                            <li style="line-height: 140%;">
                                "Wenn Sie sich  zu einer Lehrveranstaltung anmelden, melden Sie sich damit nicht automatisch  zur Prüfung an. \nZu Prüfungen melden Sie sich im Menü"
                                <em>
                                    "Prüfungen"
                                </em>
                                "an."
                            </li>
                        </ul>
                        <ul>
                            <li style="line-height: 140%;">
                                "Nur wenn Sie  zum Modul bzw. der Veranstaltung angemeldet sind, können Sie sich zur  dazugehörigen Prüfung anmelden."
                            </li>
                        </ul>
                        <ul>
                            <li style="line-height: 140%;">
                                "Einige Prüfungsordnungen sehen  vor, dass bestimmte Module und Veranstaltungen nur im Nebenfach/ im Schwerpunkt/ in Ihrer Vertiefung  belegt werden können. Für Sie \
                                 bedeutet  das: Sie müssen Ihr Nebenfach/ Ihren Schwerpunkt/ Ihre Vertiefung festlegen,  bevor Sie sich zu diesen Modulen und Lehrveranstaltungen anmelden können. \
                                 Bitte beachten Sie die Fristen, in denen Sie sich Ihr Nebenfach/ Ihren Schwerpunkt/ Ihren Wahlbereich aussuchen  können. Falls für Ihren Studiengang Fristen \
                                 festgelegt sind, stehen diese in  Klammern hinter den Titeln der Wahlbereiche."
                            </li>
                        </ul>
                        <p>
                        </p>
                        <p style="line-height: 140%;">
                            "FAQ zu Problemen bei der Anmeldung finden Sie"
                            <a href="https://www.tu-darmstadt.de/studieren/studierende_tu/studienorganisation_und_tucan/hilfe_und_faq/index.de.jsp" target="_blank">
                                "hier"
                            </a>
                            ". Bei  weiteren Fragen hilft Ihnen"
                            <a href="https://www.tu-darmstadt.de/studienbueros" target="_blank">
                                "Ihr Studienbüro"
                            </a>
                            "."
                        </p>
    };
    let _ = html_handler;
    Ok(())
}
