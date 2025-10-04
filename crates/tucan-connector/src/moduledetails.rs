use itertools::Itertools;
use scraper::CaseSensitivity::CaseSensitive;
use tucan_types::moduledetails::{
    Anmeldefristen, Kurs, KursKategorie, Leistung, Pruefung, Pruefungstermin,
};
use tucan_types::InstructorImage;
use tucan_types::{
    LoginResponse,
    moduledetails::ModuleDetailsResponse,
};

use crate::{
    TucanError,
    head::{footer, html_head, logged_in_head, logged_out_head},
};
use html_handler::{MyElementRef, MyNode, Root, parse_document};

#[expect(clippy::too_many_lines, clippy::cognitive_complexity)]
pub(crate) fn module_details_internal(
    login_response: &LoginResponse,
    content: &str,
    _nothing: &(),
) -> Result<ModuleDetailsResponse, TucanError> {
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
                <body class="moduledetails">
                    use if login_response.id == 1 {
                        logged_out_head(html_handler).0
                    } else {
                        logged_in_head(html_handler, login_response.id).0
                    };
                    <script type="text/javascript">
                    </script>
                    <h1>
                        module_id
                    </h1>
                    <form name="moduleform" id="moduleform" action="/scripts/mgrqispi.dll" method="post">
                        <div class="contentlayoutleft" id="contentlayoutleft">
                            <table class="tb">
                                <caption>
                                    "Moduldetails"
                                </caption>
                                <tbody>
                                    let registered = if html_handler.peek().unwrap().value().as_element().unwrap().attr("class").unwrap() == "tbsubhead" {
                                        <tr class="tbsubhead">
                                            <td colspan="3">
                                                "Sie sind angemeldet!"
                                            </td>
                                        </tr>
                                    } => ();
                                    <tr class="tbcontrol">
                                        <td>
                                        </td>
                                    </tr>
                                    <tr class="tbdata">
                                        <td colspan="3">
                                            <b>
                                                "Modulverantwortliche:"
                                            </b>
                                            <span id="dozenten">
                                                dozenten
                                            </span>
                                            <br></br>
                                            <br></br>
                                            <b>
                                                "Anzeige im Stundenplan:"
                                            </b>
                                            let display_in_timetable = if html_handler.peek().unwrap().value().is_text() {
                                                display_in_timetable
                                            } => display_in_timetable;
                                            <br></br>
                                            <br></br>
                                            <b>
                                                "Dauer:"
                                            </b>
                                            duration
                                            <br></br>
                                            <br></br>
                                            <b>
                                                "Anzahl Wahlkurse:"
                                            </b>
                                            count_elective_courses
                                            <br></br>
                                            <br></br>
                                            <b>
                                                "Credits:"
                                            </b>
                                            let credits = if html_handler.peek().unwrap().value().is_text() {
                                                credits
                                            } => credits;
                                            <br></br>
                                            let abweichende_credits = if html_handler.peek().unwrap().value().is_text() {
                                                "Hinweis: In Ihrer Prüfungsordnung können abweichende Credits festgelegt sein."
                                                <br></br>
                                            } => ();
                                            <br></br>
                                            <b>
                                                "Startsemester:"
                                            </b>
                                            start_semester
                                            <br></br>
                                            <br></br>
                                            let warteliste_percentage = if html_handler.peek().is_some()
                                                && html_handler
                                                    .peek()
                                                    .unwrap()
                                                    .first_child()
                                                    .unwrap()
                                                    .first_child()
                                                    .is_some_and(|v| &**v.value().as_text().unwrap() == "Warteliste:") {
                                                <p>
                                                    <b>
                                                        "Warteliste:"
                                                    </b>
                                                    <input type="checkbox" class="checkBox" checked="checked" disabled="disabled"></input>
                                                </p>
                                                <p>
                                                    <b>
                                                        "Wartelistenquote:"
                                                    </b>
                                                    percentage
                                                </p>
                                            } => percentage;
                                            let description = while html_handler.peek().is_some() {
                                                let child = html_handler.next_any_child();
                                            } => match child.value() {
                                                MyNode::Text(text) => text.to_string(),
                                                MyNode::Element(_element) => MyElementRef::wrap(child).unwrap().html(),
                                                _ => panic!(),
                                            };
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                            <table class="tb rw-table rw-all">
                                <caption>
                                    "Anmeldefristen"
                                </caption>
                                <tbody>
                                    <tr class="tbsubhead rw-hide">
                                        <td>
                                            "Phase"
                                        </td>
                                        <td>
                                            "Block"
                                        </td>
                                        <td>
                                            "Anmeldung von | bis"
                                        </td>
                                        <td>
                                            "Ende Abmeldung"
                                        </td>
                                    </tr>
                                    let anmeldefristen = if html_handler.peek().unwrap().children().nth(1).unwrap().children().next().is_none() {
                                        <tr class="tbdata">
                                            <td class="rw rw-detail-phase">
                                            </td>
                                            <td class="rw rw-detail-block">
                                            </td>
                                            <td class="rw rw-detail-regstart">
                                            </td>
                                            <td class="rw rw-detail-unreg">
                                            </td>
                                        </tr>
                                    } => () else {
                                        <tr class="tbdata">
                                            <td class="rw rw-detail-phase">
                                                anmeldeart
                                            </td>
                                            <td class="rw rw-detail-block">
                                                "Vorlesungszeit"
                                            </td>
                                            <td class="rw rw-detail-regstart">
                                                registration_range
                                            </td>
                                            <td class="rw rw-detail-unreg">
                                                let unregistration_range = if html_handler.peek().is_some() {
                                                    unregistration_range
                                                } => unregistration_range;
                                            </td>
                                        </tr>
                                    } => Anmeldefristen {
                                        anmeldeart,
                                        registration_range,
                                        unregistration_range
                                    };
                                </tbody>
                            </table>
                            let kurskategorien = if html_handler
                                .peek()
                                .unwrap()
                                .first_child()
                                .unwrap()
                                .first_child()
                                .unwrap()
                                .value()
                                .as_text()
                                .is_some_and(|v| &**v == "Kurse") {
                                <table class="tb rw-table rw-all">
                                    <caption>
                                        "Kurse"
                                    </caption>
                                    <tbody>
                                        <tr class="tbsubhead rw-hide">
                                            <td>
                                            </td>
                                            <td>
                                                "Nummer"
                                            </td>
                                            <td>
                                                "Name"
                                            </td>
                                            <td>
                                                "Pflicht"
                                            </td>
                                            <td>
                                                "Semester"
                                            </td>
                                            <td>
                                                "Credits"
                                            </td>
                                            <td>
                                            </td>
                                        </tr>
                                        let kurskategorien = while html_handler.peek().is_some() {
                                            <tr class="tbsubhead">
                                                <td class="rw rw-detail-logo">
                                                </td>
                                                <td class="rw rw-detail-courseno">
                                                    course_no
                                                </td>
                                                <td class="rw rw-detail-name">
                                                    name
                                                </td>
                                                <td class="rw rw-detail-mandatory">
                                                    mandatory
                                                </td>
                                                <td class="rw rw-detail-semester">
                                                    let semester = if html_handler.peek().is_some() {
                                                        semester
                                                    } => semester.parse().unwrap();
                                                </td>
                                                <td class="rw rw-detail-credits">
                                                    credits
                                                </td>
                                                <td>
                                                </td>
                                            </tr>
                                            let kurse = while html_handler.peek().and_then(|e| e.value().as_element()).map(|e| e.has_class("tbdata", CaseSensitive)) == Some(true) {
                                                <tr class="tbdata">
                                                    <td class="tbdata">
                                                        let gefaehrungspotential_schwangere = if html_handler.peek().is_some() {
                                                            <img src="../../gfx/_default/icons/eventIcon.gif" title="Gefährdungspotential für Schwangere"></img>
                                                        } => ();
                                                    </td>
                                                    <td>
                                                        <a name="eventLink" class="link" href=url>
                                                            course_id
                                                        </a>
                                                    </td>
                                                    <td>
                                                        <a name="eventLink" class="link" href={|v| assert_eq!(v, url)}>
                                                            name
                                                        </a>
                                                    </td>
                                                    <td>
                                                    </td>
                                                    <td>
                                                        <a name="eventLink" class="link" href={|v| assert_eq!(v, url)}>
                                                            semester
                                                        </a>
                                                    </td>
                                                    <td>
                                                    </td>
                                                    <td>
                                                    </td>
                                                </tr>
                                            } => Kurs {
                                                name,
                                                course_id,
                                                gefaehrungspotential_schwangere: gefaehrungspotential_schwangere.is_some(),
                                                semester,
                                                url
                                            };
                                        } => KursKategorie {
                                            course_no,
                                            name,
                                            mandatory: if mandatory == "Ja" {
                                                true
                                            } else if mandatory == "Nein" {
                                                false
                                            } else {
                                                panic!("unknown mandatory {mandatory}")
                                            },
                                            semester,
                                            credits: credits.replace(',', ".").parse().expect(&credits),
                                            kurse
                                        };
                                    </tbody>
                                </table>
                            } => kurskategorien;
                            <table class="tb rw-table rw-all" summary="Leistungen">
                                <caption>
                                    "Leistungen"
                                </caption>
                                <thead>
                                    <tr class="tbsubhead rw-hide">
                                        <th scope="col">
                                            "Kurs/Modulabschlussleistungen"
                                        </th>
                                        <th scope="col">
                                            let leistungskombination = if **html_handler.peek().unwrap().value().as_text().unwrap()
                                                == *"Leistungskombination" {
                                                    "Leistungskombination"
                                                </th>
                                                <th scope="col">
                                            } => ();
                                            "Leistungen"
                                        </th>
                                        <th scope="col">
                                            "Bestehenspflicht"
                                        </th>
                                        <th scope="col">
                                            "Gewichtung"
                                        </th>
                                    </tr>
                                </thead>
                                <tbody>
                                    let leistungen = while html_handler.peek().is_some() {
                                        <tr>
                                            <td rowspan=rowspan class="tbsubhead level02_color ">
                                                _modulabschlussleistungen_or_module_id_and_name
                                            </td>
                                            extern {
                                                let mut rowspan: u64 = rowspan.parse().unwrap();
                                            }
                                            let leistungen = if leistungskombination.is_some() {
                                                    <td rowspan="0002" class="level03_color tbborderleft">
                                                        <b>
                                                            name
                                                        </b>
                                                    </td>
                                                    <td colspan="2" class="level03_color alignRight">
                                                        <b>
                                                            "Summe"
                                                        </b>
                                                    </td>
                                                    <td colspan="1" class="level03_color alignRight rw-detail-weight">
                                                        <b>
                                                            weight
                                                        </b>
                                                    </td>
                                                </tr>
                                                <tr class="tbdata">
                                                    <td class="tbborderleft rw rw-detail-reqachieve">
                                                        {|v: String| assert_eq!(name, v)}
                                                    </td>
                                                    <td class="rw rw-detail-compulsory">
                                                        compulsory
                                                    </td>
                                                    <td class="rw rw-detail-weight alignRight">
                                                        {|v: String| assert_eq!(weight, v)}
                                                    </td>
                                                </tr>
                                                let leistungen = while rowspan > 2 {
                                                    <tr>
                                                        <td rowspan="0002" class="level03_color tbborderleft">
                                                            <b>
                                                                name
                                                            </b>
                                                        </td>
                                                        <td colspan="2" class="level03_color alignRight">
                                                            <b>
                                                                "Summe"
                                                            </b>
                                                        </td>
                                                        <td colspan="1" class="level03_color alignRight rw-detail-weight">
                                                            <b>
                                                                weight
                                                            </b>
                                                        </td>
                                                    </tr>
                                                    <tr class="tbdata">
                                                        <td class="tbborderleft rw rw-detail-reqachieve">
                                                            {|v: String| assert_eq!(name, v)}
                                                        </td>
                                                        <td class="rw rw-detail-compulsory">
                                                            compulsory
                                                        </td>
                                                        <td class="rw rw-detail-weight alignRight">
                                                            {|v: String| assert_eq!(weight, v)}
                                                            let weight_more = if html_handler.peek().is_some() {
                                                                <br></br>
                                                                weight_more
                                                            } => weight_more;
                                                        </td>
                                                    </tr>
                                                } => {
                                                    rowspan -= 2;
                                                    Leistung {
                                                        name,
                                                        weight,
                                                        compulsory: if compulsory == "Ja" {
                                                            true
                                                        } else if compulsory == "Nein" {
                                                            false
                                                        } else {
                                                            panic!("unknown compulsory {compulsory}")
                                                        },
                                                        weight_more,
                                                    }
                                                };
                                            } => {
                                                leistungen.insert(
                                                    0,
                                                    Leistung {
                                                        name,
                                                        weight,
                                                        compulsory: if compulsory == "Ja" {
                                                            true
                                                        } else if compulsory == "Nein" {
                                                            false
                                                        } else {
                                                            panic!("unknown compulsory {compulsory}")
                                                        },
                                                        weight_more: None,
                                                    },
                                                );
                                                leistungen
                                            } else {
                                                    <td class="tbborderleft rw rw-detail-reqachieve">
                                                        name
                                                    </td>
                                                    <td class="rw rw-detail-compulsory">
                                                        compulsory
                                                    </td>
                                                    <td class="rw rw-detail-weight alignRight">
                                                        weight
                                                    </td>
                                                </tr>
                                                let leistungen = while rowspan > 1 {
                                                    <tr class="tbdata">
                                                        <td class="tbborderleft rw rw-detail-reqachieve">
                                                            name
                                                        </td>
                                                        <td class="rw rw-detail-compulsory">
                                                            compulsory
                                                        </td>
                                                        <td class="rw rw-detail-weight alignRight">
                                                            weight
                                                        </td>
                                                    </tr>
                                                } => {
                                                    rowspan -= 1;
                                                    Leistung {
                                                        name,
                                                        weight,
                                                        compulsory: if compulsory == "Ja" {
                                                            true
                                                        } else if compulsory == "Nein" {
                                                            false
                                                        } else {
                                                            panic!("unknown compulsory {compulsory}")
                                                        },
                                                        weight_more: None,
                                                    }
                                                };
                                            } => {
                                                leistungen.insert(
                                                    0,
                                                    Leistung {
                                                        name,
                                                        weight,
                                                        compulsory: if compulsory == "Ja" {
                                                            true
                                                        } else if compulsory == "Nein" {
                                                            false
                                                        } else {
                                                            panic!("unknown compulsory {compulsory}")
                                                        },
                                                        weight_more: None,
                                                    },
                                                );
                                                leistungen
                                            };
                                    } => leistungen.either_into::<Vec<Leistung>>();
                                </tbody>
                            </table>
                            let pruefungen = if html_handler.peek().is_some() {
                                <table class="tb rw-table rw-all" summary="Modulabschlussprüfungen">
                                    <caption>
                                        "Modulabschlussprüfungen"
                                    </caption>
                                    <thead>
                                        <tr class="tbsubhead rw-hide">
                                            <th scope="col">
                                                let leistungskombination = if **html_handler.peek().unwrap().value().as_text().unwrap()
                                                    == *"Leistungskombination" {
                                                        "Leistungskombination"
                                                    </th>
                                                    <th scope="col">
                                                } => ();
                                                "Prüfung"
                                            </th>
                                            <th scope="col">
                                                "Datum"
                                            </th>
                                            <th scope="col">
                                                "Lehrende"
                                            </th>
                                            <th scope="col">
                                                "Bestehenspflicht"
                                            </th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        let pruefungen = while html_handler.peek().is_some() {
                                            let pruefung = if leistungskombination.is_some() {
                                                <tr class="tbdata">
                                                    <td rowspan=rowspan class="level03_color rw rw-detail-combination ">
                                                        <b>
                                                            name
                                                        </b>
                                                    </td>
                                                    <td class="tbborderleft rw rw-detail-exam">
                                                        subname
                                                    </td>
                                                    <td class="rw rw-detail-date">
                                                        date
                                                    </td>
                                                    <td class="rw rw-detail-instructors">
                                                        examiner
                                                    </td>
                                                    <td class="rw rw-detail-compulsory">
                                                        compulsory
                                                    </td>
                                                </tr>
                                                extern {
                                                    let mut rowspan: u64 = rowspan.parse().unwrap();
                                                }
                                                let termine = while rowspan > 1 {
                                                    <tr class="tbdata">
                                                        <td class="tbborderleft rw rw-detail-exam">
                                                            subname
                                                        </td>
                                                        <td class="rw rw-detail-date">
                                                            date
                                                        </td>
                                                        <td class="rw rw-detail-instructors">
                                                            examiner
                                                        </td>
                                                        <td class="rw rw-detail-compulsory">
                                                            {|v: String| assert_eq!(compulsory, v)}
                                                        </td>
                                                    </tr>
                                                } => {
                                                    rowspan -= 1;
                                                    Pruefungstermin { date, examiner, subname }
                                                };
                                            } => {
                                                termine.insert(0, Pruefungstermin { date, examiner, subname });
                                                Pruefung {
                                                    compulsory: if compulsory == "Ja" {
                                                        true
                                                    } else if compulsory == "Nein" {
                                                        false
                                                    } else {
                                                        panic!("unknown compulsory {compulsory}")
                                                    },
                                                    name,
                                                    termine,
                                                }
                                            } else {
                                                <tr class="tbdata">
                                                    <td class="tbborderleft rw rw-detail-exam">
                                                        name
                                                    </td>
                                                    <td class="rw rw-detail-date">
                                                        date
                                                    </td>
                                                    <td class="rw rw-detail-instructors">
                                                        examiner
                                                    </td>
                                                    <td class="rw rw-detail-compulsory">
                                                        compulsory
                                                    </td>
                                                </tr>
                                            } => Pruefung {
                                                name: name.clone(),
                                                compulsory: if compulsory == "Ja" {
                                                    true
                                                } else if compulsory == "Nein" {
                                                    false
                                                } else {
                                                    panic!("unknown compulsory {compulsory}")
                                                },
                                                termine: vec![Pruefungstermin { date, examiner, subname: name }]
                                            };
                                        } => pruefung.either_into();
                                    </tbody>
                                </table>
                            } => pruefungen;
                        </div>
                        <div class="contentlayoutright" id="contentlayoutright">
                            let modulverantwortliche = if html_handler.peek().is_some() {
                                <table class="tb_contentright">
                                    <caption>
                                        "Modulverantwortliche"
                                    </caption>
                                    <tbody>
                                        let modulverantwortliche = while html_handler.peek().is_some() {
                                            let bild = if html_handler.peek().unwrap().value().as_element().unwrap().attrs.is_empty() {
                                                <tr>
                                                    <td class="tbdata_nob" style="text-align:center;padding-top:10px;padding-left:0px;">
                                                        <img src=imgsrc width="120" height="160" border="0" alt=alt></img>
                                                    </td>
                                                </tr>
                                            } => InstructorImage { alt, imgsrc };
                                            <tr class="tbdata">
                                                <td style="text-align:center;">
                                                    name
                                                </td>
                                            </tr>
                                        } => (name, bild);
                                    </tbody>
                                </table>
                            } => modulverantwortliche;
                        </div>
                        <br style="clear:both;"></br>
                    </form>
                </div>
            </div>
        </div>
    };
    let html_handler = footer(html_handler, login_response.id, 311);
    html_handler.end_document();
    let modulverantwortliche = modulverantwortliche.unwrap_or_default();
    if modulverantwortliche.is_empty() {
        assert_eq!(dozenten, "N.N.");
    } else {
        assert_eq!(
            dozenten.split("; ").sorted().collect::<Vec<_>>(),
            modulverantwortliche
                .iter()
                .map(|m| &m.0)
                .sorted()
                .collect::<Vec<_>>()
        );
    }
    Ok(ModuleDetailsResponse {
        module_id,
        registered: registered.is_some(),
        count_elective_courses,
        credits: credits.map(|credits| credits.trim_end_matches(",0").parse().expect(&credits)),
        description,
        display_in_timetable,
        duration,
        abweichende_credits: abweichende_credits.is_some(),
        start_semester,
        anmeldefristen: anmeldefristen.right(),
        kurskategorien: kurskategorien.unwrap_or_default(),
        modulverantwortliche,
        leistungen: leistungen.into_iter().flatten().collect(),
        pruefungen: pruefungen.unwrap_or_default(),
        warteliste_percentage,
    })
}
