use crate::{
    TucanConnector, TucanError,
    common::head::{footer, html_head, logged_out_head},
    retryable_get,
};
use html_handler::{Root, parse_document};
use tucant_types::LoggedOutHead;

#[expect(clippy::too_many_lines)]
pub async fn welcome(connector: &TucanConnector) -> Result<LoggedOutHead, TucanError> {
    let (content, ..) = retryable_get(connector, "https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=EXTERNALPAGES&ARGUMENTS=-N000000000000001,-N000344,-Awelcome").await?;
    let document = parse_document(&content);
    let html_handler = Root::new(document.root());
    let html_handler = html_handler.document_start();
    let html_handler = html_handler.doctype();
    html_extractor::html! {
            <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="de" lang="de" xmlns:msdt="uuid:C2F41010-65B3-11d1-A29F-00AA00C14882" xmlns:mso="urn:schemas-microsoft-com:office:office">
                <head>
                    use html_head(html_handler)?;
                    <style type="text/css">
                        "PBsLNqyhelKIL09TLRqYsD4XcU0zItzE9RmRIPZhHFo"
                    </style>
                </head>
                <body class="external_pages">
                    let vv = logged_out_head(html_handler, 344);
                    <script type="text/javascript">
                    </script>
                    <meta http-equiv="content-type" content="text/html; charset=windows-1252"></meta>
                    <div id="inhalt" style="padding:0px; width:650px; margin:0px; background-color:#ffffff;">
                        <h1>
                            "Herzlich willkommen bei TUCaN, dem Campus-Management-System der"
                            <br></br>
                            "TU Darmstadt!"
                        </h1>
                        <br></br>
                        <p style="line-height: 140%;">
                            <strong>
                                "Studierende, Lehrende, Stellvertretungen und Mitarbeitende der TU Darmstadt"
                            </strong>
                            <br></br>
                            "melden sich mit ihrer TU-ID an, um das System zu nutzen."
                        </p>
                        <ul>
                            <li>
                                <a href="https://www.tu-darmstadt.de/studieren/studierende_tu/studienorganisation_und_tucan/index.de.jsp" target="_blank">
                                    "FAQ für Studierende"
                                </a>
                            </li>
                            <li>
                                <a href="https://www.intern.tu-darmstadt.de/dez_ii/campusmanagement/cm_tucan/infos_fuer_lehrende/index.de.jsp" target="_blank">
                                    "FAQ für Lehrende"
                                </a>
                            </li>
                        </ul>
                        <p style="line-height: 40%;">
                        </p>
                        <p style="line-height: 140%;">
                            <strong>
                                "Bewerber:innen und Gasthörer:innen"
                            </strong>
                            <br></br>
                            "legen sich zunächst ein"
                            <a href="https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=EXTERNALPAGES&ARGUMENTS=-N000000000000001,-N000410,-Atucan%5Faccount%2Ehtml">
                                "TUCaN-Account"
                            </a>
                            "an,\n um ihre Zugangsdaten zu erhalten und melden sich anschließend mit \ndiesen Zugangsdaten an, bis sie ihre endgültige TU-ID erhalten."
                        </p>
                        <ul>
                            <li>
                                <a href="https://www.tu-darmstadt.de/studieren/studieninteressierte/bewerbung_zulassung_tu/online_bewerbung/index.de.jsp" target="_blank">
                                    "FAQ für Bewerber:innen"
                                </a>
                            </li>
                            <li>
                                <a href="https://www.tu-darmstadt.de/gasthoerer" target="_blank">
                                    "FAQ für Gasthörer:innen"
                                </a>
                            </li>
                        </ul>
                        <p style="line-height: 40%;">
                        </p>
                        <p style="line-height: 140%;">
                            <strong>
                                "Promovierende zur Registrierung / Einschreibung"
                            </strong>
                            <br></br>
                            "beachten bitte die Informationen auf den"
                            <a href="http://www.tu-darmstadt.de/promotion-registrierung" target="_blank">
                                "Webseiten"
                            </a>
                            "."
                        </p>
                        <p style="line-height: 40%;">
                        </p>
                        <div style="padding:10px; width:650px; border:thin solid grey; margin:0px; background-color:#f8f9ed;">
                            <p style="line-height: 140%;">
                                <strong>
                                    "Aktuelles: Fristen zur Prüfungsanmeldung in TUCaN für das Wintersemester 2024/2025"
                                </strong>
                            </p>
                            <p style="line-height: 140%;">
                                "Die Anmeldezeit zu Prüfungen im WiSe 2024/2025 hat in der Regel am 15. November 2024 begonnen."
                                <br></br>
                                "In vielen Studiengängen endet die Anmeldefrist am 15. Dezember 2024 - bitte informieren Sie sich rechtzeitig! Ihre Anmeldung nehmen Sie im TUCaN-Webportal im Bereich"
                                <i>
                                    "Prüfungen"
                                </i>
                                "unter"
                                <i>
                                    "Meine Prüfungen / Anmeldung zu Prüfungen"
                                </i>
                                "vor."
                            </p>
                            <p style="line-height: 140%;">
                                "Fachbereiche können darüber hinaus individuelle Fristen festlegen. Die An- und Abmeldefristen entnehmen Sie bitte den"
                                <a href="http://www.tu-darmstadt.de/tucan-pruefungsdetails" target="_blank">
                                    "Prüfungsdetails"
                                </a>
                                "in TUCaN."
                            </p>
                            "→"
                            <a href="https://www.tu-darmstadt.de/studieren/studierende_tu/studienorganisation_und_tucan/hilfe_und_faq/index.de.jsp" target="_blank">
                                "Hilfe & FAQ zur Prüfungsanmeldung"
                            </a>
                            <br></br>
                            <p>
                            </p>
                            <br></br>
                        </div>
                        <p>
                        </p>
                        "→"
                        <a href="https://www.tu-darmstadt.de/studieren/studierende_tu/studienorganisation_und_tucan/hilfe_und_faq/artikel_details_de_en_37312.de.jsp" target="_blank">
                            "TUCaN Wartungszeit: Dienstag um 6 - 9 Uhr"
                        </a>
                        <br></br>
                        <br></br>
                        "→"
                        <a href="https://www.tu-darmstadt.de/studieren/studierende_tu/studienorganisation_und_tucan/hilfe_und_faq/artikel_details_de_en_344192.de.jsp" target="_blank">
                            "Hinweise zum Datenschutz"
                        </a>
                        <p>
                        </p>
                        <title>
                        </title>
                    </div>
                </div>
            </div>
        </div>
        use footer(html_handler, 1, 344);
    }
    html_handler.end_document();
    Ok(vv)
}
