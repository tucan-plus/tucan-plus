use std::sync::LazyLock;

use itertools::Either;
use regex::Regex;
use scraper::CaseSensitivity;
use tucan_types::{
    LoggedInHead, LoggedOutHead, LoginResponse, TucanError, VorlesungsverzeichnisUrls,
    vv::ActionRequest,
};

use html_handler::{InElement, InRoot, Root};

// 275 means "Meine Module" is selected in menu
// 311 means "Anmeldung" is selected in menu

pub(crate) static ACTION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&PRGNAME=ACTION&ARGUMENTS=").unwrap()
});

#[must_use]
pub fn html_head_2<'a>(
    html_handler: InElement<'a, InElement<'a, InRoot<'a, Root<'a>>>>,
) -> InElement<'a, InElement<'a, InRoot<'a, Root<'a>>>> {
    html_extractor::html! {
        <script type="text/javascript">
        </script>
        <title>
            "Technische Universität Darmstadt"
        </title>
        <meta http-equiv="X-UA-Compatible" content="IE=EmulateIE9"></meta>
        <meta http-equiv="cache-control" content="no-cache"></meta>
        <meta http-equiv="expires" content="-1"></meta>
        <meta http-equiv="pragma" content="no-cache"></meta>
        <meta http-equiv="Content-Type" content="text/html; charset=utf-8"></meta>
        <meta http-equiv="Content-Script-Type" content="text/javascript"></meta>
        <meta name="viewport" content="width=device-width, initial-scale=1,user-scalable=0"></meta>
        <link href="/css/_default/dl.startpage.css" rel="stylesheet" type="text/css"></link>
        <link href="/css/styles.css" rel="stylesheet" type="text/css"></link>
        <link href="/css/colors.css" rel="stylesheet" type="text/css"></link>
    };
    html_handler
}

#[expect(unreachable_code)]
#[allow(clippy::no_effect_underscore_binding)]
pub fn html_head<'a>(
    html_handler: InElement<'a, InElement<'a, InRoot<'a, Root<'a>>>>,
) -> Result<InElement<'a, InElement<'a, InRoot<'a, Root<'a>>>>, TucanError> {
    html_extractor::html! {
        <title>
            "Technische Universität Darmstadt"
        </title>
        <meta http-equiv="X-UA-Compatible" content="IE=edge"></meta>
        <meta http-equiv="cache-control" content="no-cache"></meta>
        <meta http-equiv="expires" content="-1"></meta>
        <meta http-equiv="pragma" content="no-cache"></meta>
        <meta http-equiv="Content-Type" content="text/html; charset=utf-8"></meta>
        <meta http-equiv="Content-Script-Type" content="text/javascript"></meta>
        <meta name="referrer" content="origin"></meta>
        <meta
            name="keywords"
            content="Datenlotsen,Datenlotsen Informationssysteme GmbH,CampusNet,Campus Management"
        ></meta>
        <link rel="shortcut icon" type="image/x-icon" href="/gfx/tuda/icons/favicon.ico"></link>
        <script src="/js/jquery-3.6.0.min.js" type="text/javascript">
        </script>
        <script src="/js/checkDate.js" type="text/javascript">
        </script>
        <script src="/js/edittext.js" type="text/javascript">
        </script>
        <script src="/js/skripts.js" type="text/javascript">
        </script>
        <script src="/js/x.js" type="text/javascript">
        </script>
        <link
            id="defLayout"
            href="/css/_default/def_layout.css"
            rel="stylesheet"
            type="text/css"
            media="screen"
        ></link>
        <link
            id="defMenu"
            href="/css/_default/def_menu.css"
            rel="stylesheet"
            type="text/css"
            media="screen"
        ></link>
        <link id="defStyles" href="/css/_default/def_styles.css" rel="stylesheet" type="text/css"></link>
        <link
            id="pagePrint"
            href="/css/_default/def_print.css"
            rel="stylesheet"
            type="text/css"
            media="print"
        ></link>
        <link id="pageStyle" href="/css/styles.css" rel="stylesheet" type="text/css"></link>
        <link id="pageColors" href="/css/colors.css" rel="stylesheet" type="text/css" media="screen"></link>
        let _unused = if html_handler.peek().is_none() {
            </head>
            let _unused = if html_handler
                .peek()
                .unwrap()
                .value()
                .as_element()
                .unwrap()
                .has_class(
                    "timeout",
                    scraper::CaseSensitivity::CaseSensitive
                ) {
                <body class="timeout">
                    extern {
                        let _html_handler = html_handler;
                        return Err(TucanError::Timeout);
                    }
                </body>
            } => () else {
                let _unused = if html_handler
                    .peek()
                    .unwrap()
                    .value()
                    .as_element()
                    .unwrap()
                    .has_class(
                        "access_denied",
                        scraper::CaseSensitivity::CaseSensitive
                    ) {
                    <body class="access_denied">
                        extern {
                            let _html_handler = html_handler;
                            return Err(TucanError::AccessDenied);
                        }
                    </body>
                } => () else {
                    extern {
                        panic!();
                    }
                } => ();
            } => ();
            <head>
        } => ();
    }
    Ok(html_handler)
}

type InElement5<'a, T> =
    InElement<'a, InElement<'a, InElement<'a, InElement<'a, InElement<'a, T>>>>>;

#[must_use]
pub fn page_start<'a>(
    html_handler: InElement<'a, InElement<'a, InRoot<'a, Root<'a>>>>,
) -> InElement5<'a, InElement<'a, InRoot<'a, Root<'a>>>> {
    html_extractor::html! {
        <div id="Cn-system-desc">
        </div>
        <script type="text/javascript">
            "6f1xplmmi__XaOfSpuTqnwmS3e7kjEtoMr5AEG10Alc"
        </script>
        <div id="acc_pageDescription" class="hidden">
            <a name="keypadDescription" class="hidden">
                "keypadDescription"
            </a>
            "NogHj2hSzz5S9t6TCuQ_Uo6OSW4RuUVHr-y75R1O8lU"
            <a href="#mainNavi" accesskey="1">
                "1 Hauptmenü"
            </a>
            <a href="#mainContent" accesskey="2">
                "2 Inhalt"
            </a>
            <a href="#keypadDescription" accesskey="3">
                "3 Zurück zu dieser Anleitung"
            </a>
        </div>
        <div id="pageContainer" class="pageElementTop">
            <div class="invAnchor">
                <a name="top" class="invAnchor">
                </a>
            </div>
            <div id="pageHead" class="pageElementTop">
                <div id="pageHeadTop" class="pageElementTop">
                    <a href=_imprint_url class="img img_arrowImprint pageElementLeft">
                        "Impressum"
                    </a>
                    <a href=_contact_url class="img img_arrowContact pageElementLeft">
                        "Kontakt"
                    </a>
                    <a href="#" onclick="window.print();" class="img img_arrowPrint pageElementLeft">
                        "Drucken"
                    </a>
                    <a href="#bottom" class="img img_arrowDown pageElementRight">
                        "Zum Ende der Seite"
                    </a>
                </div>
                <div id="pageHeadCenter" class="pageElementTop">
                    <div id="pageHeadLeft" class="pageElementLeft">
                        <a href="http://www.tu-darmstadt.de" title="extern http://www.tu-darmstadt.de">
                            <img
                                id="imagePageHeadLeft"
                                src="/gfx/tuda/logo.gif"
                                alt="Logo Technische Universität Darmstadt"
                            ></img>
                        </a>
                    </div>
                    <div id="pageHeadRight" class="pageElementRight">
                    </div>
                </div>
                <div id="pageHeadBottom_1" class="pageElementTop">
                    <div id="pageHeadControlsLeft" class="pageElementLeft">
                        <a class="img pageHeadLink" href="#" id="extraNav_link1" target="_blank">
                            "Homepage"
                        </a>
                        <a class="img pageHeadLink" href="#" id="extraNav_link2" target="_blank">
                            "standardLink undef"
                        </a>
                    </div>
                    <div id="pageHeadControlsRight" class="pageElementRight">
                        <a class="img" href="#" id="extraNav_link3" target="_blank">
                            "standardLink undef"
                        </a>
                        <a class="img" href="#" id="extraNav_link4" target="_blank">
                            "standardLink undef"
                        </a>
                        <a class="img" href="#" id="extraNav_link5" target="_blank">
                        </a>
                    </div>
                </div>
                <div id="pageHeadBottom_2" class="pageElementTop">
                    <div id="pageHeadBottom_2sub_1" class="pageElementTop">
                    </div>
                    <div id="pageHeadBottom_2sub_2" class="pageElementTop">
                    </div>
                </div>
                <div id="pageTopNavi" class="pageElementTop">
                    <a name="mainNavi" class="hidden">
                    </a>
                    <ul class="nav depth_1 linkItemContainer">
    };
    html_handler
}

#[must_use]
pub fn vv_something<'a>(
    html_handler: InElement5<'a, InElement<'a, InElement<'a, InRoot<'a, Root<'a>>>>>,
) -> (
    InElement5<'a, InElement<'a, InElement<'a, InRoot<'a, Root<'a>>>>>,
    VorlesungsverzeichnisUrls,
) {
    // these link ids are incrementing so they are different if used from different
    // contexts. could in theory be calculated based on some starting number
    html_extractor::html! {
        <ul class="nav depth_2 linkItemContainer">
            <li class="intern depth_2 linkItem " title="Lehrveranstaltungssuche" id=_id>
                <a class=_class href=lehrveranstaltungssuche_url>
                    "Lehrveranstaltungssuche"
                </a>
            </li>
            <li class="intern depth_2 linkItem " title="Raumsuche" id=_id>
                <a class=_class href=_raumsuche_url>
                    "Raumsuche"
                </a>
            </li>
            let vvs = while !html_handler
                .peek()
                .unwrap()
                .value()
                .as_element()
                .unwrap()
                .has_class("branchLinkItem", CaseSensitivity::CaseSensitive) {
                <li class="intern depth_2 linkItem " title=_title id=_linkclass>
                    <a class=_linkclass href=url>
                        title
                    </a>
                </li>
            } => (title, ActionRequest::parse(&ACTION_REGEX.replace(&url, "")));
            <li class="tree depth_2 linkItem branchLinkItem " title="Archiv" id=_linkclass>
                <a class=_linkclass href=_url>
                    "Archiv"
                </a>
                <ul class="nav depth_3 linkItemContainer">
                    let archiv_links = while html_handler.peek().is_some()
                        && html_handler
                            .peek()
                            .unwrap()
                            .value()
                            .as_element()
                            .unwrap()
                            .attr("title")
                            .unwrap()
                            != "Altsysteme" {
                        <li class="intern depth_3 linkItem " title=title id=_linkclass>
                            <a class=_linkclass href=url>
                                text
                            </a>
                        </li>
                    } => (
                        title,
                        ActionRequest::parse(&ACTION_REGEX.replace(&url, "")),
                        text
                    );
                    let _altsysteme = if html_handler.peek().is_some() {
                        <li class="intern depth_3 linkItem " title="Altsysteme" id=_linkclass>
                            <a class=_linkclass href=_url>
                                "Altsysteme"
                            </a>
                        </li>
                    } => ();
                </ul>
            </li>
        </ul>
    };
    (
        html_handler,
        VorlesungsverzeichnisUrls {
            lehrveranstaltungssuche_url,
            vvs,
            archiv_links,
        },
    )
}

#[must_use]
pub fn logged_in_head<'a>(
    html_handler: InElement<'a, InElement<'a, InRoot<'a, Root<'a>>>>,
    id: u64,
) -> (
    InElement5<'a, InElement<'a, InRoot<'a, Root<'a>>>>,
    LoggedInHead,
) {
    assert_ne!(id, 1);
    html_extractor::html! {
        use page_start(html_handler);
        let result = logged_in_head_internal(html_handler, id);
    }
    (html_handler, result)
}

#[expect(clippy::too_many_lines)]
#[must_use]
fn logged_in_head_internal<'a>(
    html_handler: InElement5<'a, InElement<'a, InRoot<'a, Root<'a>>>>,
    id: u64,
) -> (
    InElement5<'a, InElement<'a, InRoot<'a, Root<'a>>>>,
    LoggedInHead,
) {
    assert_ne!(id, 1);
    html_extractor::html! {
                    <li class="tree depth_1 linkItem branchLinkItem " title="Aktuelles" id="link000019">
                        <a class="depth_1 link000019 navLink branchLink " href=_aktuelles_url>
                            "Aktuelles"
                        </a>
                        <ul class="nav depth_2 linkItemContainer">
                            <li class="intern depth_2 linkItem " title="Nachrichten" id="link000299">
                                <a class="depth_2 link000299 navLink " href=messages_url>
                                    "Nachrichten"
                                </a>
                            </li>
                        </ul>
                    </li>
                    <li class="tree depth_1 linkItem branchLinkItem " title="VV" id="link000326">
                        <a
                            class="depth_1 link000326 navLink branchLink "
                            href=vorlesungsverzeichnis_url
                        >
                            "VV"
                        </a>
                        let vv = vv_something(html_handler);
                    </li>
                    <li
                        class="tree depth_1 linkItem branchLinkItem "
                        title="Stundenplan"
                        id="link000268"
                    >
                        <a
                            class="depth_1 link000268 navLink branchLink "
                            href={|v: String| {
                                static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                    Regex::new(
                                        "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                         PRGNAME=SCHEDULER&ARGUMENTS=-N\\d+,-N000268,-A,-A,-N1$",
                                    )
                                    .unwrap()
                                });
                                assert!(REGEX.is_match(&v), "{v}");
                            }}
                        >
                            "Stundenplan"
                        </a>
                        <ul class="nav depth_2 linkItemContainer">
                            <li class="intern depth_2 linkItem " title="Tagesansicht" id="link000269">
                                <a
                                    class="depth_2 link000269 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=SCHEDULER&ARGUMENTS=-N\\d+,-N000269,-A,\
                                                 -A,-N0$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Tagesansicht"
                                </a>
                            </li>
                            <li class="intern depth_2 linkItem " title="Wochenansicht" id="link000270">
                                <a
                                    class="depth_2 link000270 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=SCHEDULER&ARGUMENTS=-N\\d+,-N000270,-A,\
                                                 -A,-N1$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Wochenansicht"
                                </a>
                            </li>
                            <li class="intern depth_2 linkItem " title="Monatsansicht" id="link000271">
                                <a
                                    class="depth_2 link000271 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=MONTH&ARGUMENTS=-N\\d+,-N000271,-A$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Monatsansicht"
                                </a>
                            </li>
                            <li class="intern depth_2 linkItem " title="Export" id="link000272">
                                <a
                                    class="depth_2 link000272 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=SCHEDULER_EXPORT&ARGUMENTS=-N\\d+,\
                                                 -N000272,$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Export"
                                </a>
                            </li>
                        </ul>
                    </li>
                    <li
                        class="tree depth_1 linkItem branchLinkItem "
                        title="Veranstaltungen"
                        id="link000273"
                    >
                        <a
                            class="depth_1 link000273 navLink branchLink "
                            href={|v: String| {
                                static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                    Regex::new(
                                        "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                         PRGNAME=EXTERNALPAGES&ARGUMENTS=-N\\d+,-N000273,\
                                         -Astudveranst%2Ehtml$",
                                    )
                                    .unwrap()
                                });
                                assert!(REGEX.is_match(&v), "{v}");
                            }}
                        >
                            "Veranstaltungen"
                        </a>
                        <ul class="nav depth_2 linkItemContainer">
                            <li class="intern depth_2 linkItem " title="Meine Module" id="link000275">
                                <a
                                    class="depth_2 link000275 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=MYMODULES&ARGUMENTS=-N\\d+,-N000275,$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Meine Module"
                                </a>
                            </li>
                            <li
                                class="intern depth_2 linkItem "
                                title="Meine Veranstaltungen"
                                id="link000274"
                            >
                                <a
                                    class="depth_2 link000274 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=PROFCOURSES&ARGUMENTS=-N\\d+,-N000274,$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Meine Veranstaltungen"
                                </a>
                            </li>
                            <li
                                class="intern depth_2 linkItem "
                                title="Meine Wahlbereiche"
                                id="link000307"
                            >
                                <a
                                    class="depth_2 link000307 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=STUDENTCHOICECOURSES&ARGUMENTS=-N\\d+,\
                                                 -N000307,$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Meine Wahlbereiche"
                                </a>
                            </li>
                            <li class="intern depth_2 linkItem " title="Anmeldung" id="link000311">
                                <a
                                    class="depth_2 link000311 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=REGISTRATION&ARGUMENTS=-N\\d+,-N000311,\
                                                 -A$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Anmeldung"
                                </a>
                            </li>
                            <li
                                class="intern depth_2 linkItem "
                                title="Mein aktueller Anmeldestatus"
                                id="link000308"
                            >
                                <a
                                    class="depth_2 link000308 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=MYREGISTRATIONS&ARGUMENTS=-N\\d+,\
                                                 -N000308,-N000000000000000$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Mein aktueller Anmeldestatus"
                                </a>
                            </li>
                        </ul>
                    </li>
                    <li class="tree depth_1 linkItem branchLinkItem " title="Prüfungen" id="link000280">
                        <a
                            class="depth_1 link000280 navLink branchLink "
                            href={|v: String| {
                                static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                    Regex::new(
                                        "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                         PRGNAME=EXTERNALPAGES&ARGUMENTS=-N\\d+,-N000280,\
                                         -Astudpruefungen%2Ehtml$",
                                    )
                                    .unwrap()
                                });
                                assert!(REGEX.is_match(&v), "{v}");
                            }}
                        >
                            "Prüfungen"
                        </a>
                        <ul class="nav depth_2 linkItemContainer">
                            <li
                                class="intern depth_2 linkItem "
                                title="Meine Prüfungen"
                                id="link000318"
                            >
                                <a
                                    class="depth_2 link000318 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=MYEXAMS&ARGUMENTS=-N\\d+,-N000318,$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Meine Prüfungen"
                                </a>
                            </li>
                            <li
                                class="tree depth_2 linkItem branchLinkItem "
                                title="Mein Prüfungsplan"
                                id="link000389"
                            >
                                <a
                                    class="depth_2 link000389 navLink branchLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=SCPCHOICE&ARGUMENTS=-N\\d+,-N000389,$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Mein Prüfungsplan"
                                </a>
                                <ul class="nav depth_3 linkItemContainer">
                                    <li
                                        class="intern depth_3 linkItem "
                                        title="Wichtige Hinweise"
                                        id="link000391"
                                    >
                                        <a
                                            class="depth_3 link000391 navLink "
                                            href={|v: String| {
                                                static REGEX: LazyLock<Regex> =
                                                    LazyLock::new(|| {
                                                        Regex::new(
                                                            "^/scripts/mgrqispi.dll\\?\
                                                             APPNAME=CampusNet&\
                                                             PRGNAME=EXTERNALPAGES&ARGUMENTS=-N\\\
                                                             d+,-N000391,-Astudplan%2Ehtml$",
                                                        )
                                                        .unwrap()
                                                    });
                                                assert!(REGEX.is_match(&v), "{v}");
                                            }}
                                        >
                                            "Wichtige Hinweise"
                                        </a>
                                    </li>
                                </ul>
                            </li>
                            <li
                                class="tree depth_2 linkItem branchLinkItem "
                                title="Semesterergebnisse"
                                id="link000323"
                            >
                                <a
                                    class="depth_2 link000323 navLink branchLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=EXTERNALPAGES&ARGUMENTS=-N\\d+,-N000323,\
                                                 -Astudergebnis%2Ehtml$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Semesterergebnisse"
                                </a>
                                <ul class="nav depth_3 linkItemContainer">
                                    <li
                                        class="intern depth_3 linkItem "
                                        title="Modulergebnisse"
                                        id="link000324"
                                    >
                                        <a
                                            class="depth_3 link000324 navLink "
                                            href={|v: String| {
                                                static REGEX: LazyLock<Regex> =
                                                    LazyLock::new(|| {
                                                        Regex::new(
                                                            "^/scripts/mgrqispi.dll\\?\
                                                             APPNAME=CampusNet&\
                                                             PRGNAME=COURSERESULTS&ARGUMENTS=-N\\\
                                                             d+,-N000324,$",
                                                        )
                                                        .unwrap()
                                                    });
                                                assert!(REGEX.is_match(&v), "{v}");
                                            }}
                                        >
                                            "Modulergebnisse"
                                        </a>
                                    </li>
                                    <li
                                        class="intern depth_3 linkItem "
                                        title="Prüfungsergebnisse"
                                        id="link000325"
                                    >
                                        <a
                                            class="depth_3 link000325 navLink "
                                            href={|v: String| {
                                                static REGEX: LazyLock<Regex> =
                                                    LazyLock::new(|| {
                                                        Regex::new(
                                                            "^/scripts/mgrqispi.dll\\?\
                                                             APPNAME=CampusNet&\
                                                             PRGNAME=EXAMRESULTS&ARGUMENTS=-N\\d+,\
                                                             -N000325,$",
                                                        )
                                                        .unwrap()
                                                    });
                                                assert!(REGEX.is_match(&v), "{v}");
                                            }}
                                        >
                                            "Prüfungsergebnisse"
                                        </a>
                                    </li>
                                </ul>
                            </li>
                            <li
                                class="intern depth_2 linkItem "
                                title="Leistungsspiegel"
                                id="link000316"
                            >
                                <a
                                    class="depth_2 link000316 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=STUDENT_RESULT&ARGUMENTS=-N\\d+,-N000316,\
                                                 -N0,-N000000000000000,-N000000000000000,\
                                                 -N000000000000000,-N0,-N000000000000000$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Leistungsspiegel"
                                </a>
                            </li>
                        </ul>
                    </li>
                    <li class="tree depth_1 linkItem branchLinkItem " title="Service" id="link000337">
                        <a
                            class="depth_1 link000337 navLink branchLink "
                            href={|v: String| {
                                static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                    Regex::new(
                                        "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                         PRGNAME=EXTERNALPAGES&ARGUMENTS=-N\\d+,-N000337,\
                                         -Aservice%2Ehtml$",
                                    )
                                    .unwrap()
                                });
                                assert!(REGEX.is_match(&v), "{v}");
                            }}
                        >
                            "Service"
                        </a>
                        <ul class="nav depth_2 linkItemContainer">
                            <li
                                class="intern depth_2 linkItem "
                                title="Persönliche Daten"
                                id="link000339"
                            >
                                <a
                                    class="depth_2 link000339 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=PERSADDRESS&ARGUMENTS=-N\\d+,-N000339,-A$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Persönliche Daten"
                                </a>
                            </li>
                            <li
                                class="intern depth_2 linkItem "
                                title="Meine Dokumente"
                                id="link000557"
                            >
                                <a
                                    class="depth_2 link000557 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=CREATEDOCUMENT&ARGUMENTS=-N\\d+,-N000557,\
                                                 $",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Meine Dokumente"
                                </a>
                            </li>
                            <li class="intern depth_2 linkItem " title="Anträge" id="link000600">
                                <a class="depth_2 link000600 navLink " href=antraege_url>
                                    "Anträge"
                                </a>
                            </li>
                            <li class="intern depth_2 linkItem " title="Sperren" id="link000652">
                                <a
                                    class="depth_2 link000652 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=HOLDINFO&ARGUMENTS=-N\\d+,-N000652,$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Sperren"
                                </a>
                            </li>
                        </ul>
                    </li>
                    <li class="tree depth_1 linkItem branchLinkItem " title="Bewerbung" id="link000441">
                        <a
                            class="depth_1 link000441 navLink branchLink "
                            href={|v: String| {
                                static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                    Regex::new(
                                        "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                         PRGNAME=EXTERNALPAGES&ARGUMENTS=-N\\d+,-N000441,\
                                         -Abewerbung$",
                                    )
                                    .unwrap()
                                });
                                assert!(REGEX.is_match(&v), "{v}");
                            }}
                        >
                            "Bewerbung"
                        </a>
                        <ul class="nav depth_2 linkItemContainer">
                            <li
                                class="intern depth_2 linkItem "
                                title="Herzlich Willkommen"
                                id="link000442"
                            >
                                <a
                                    class="depth_2 link000442 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=EXTERNALPAGES&ARGUMENTS=-N\\d+,-N000442,\
                                                 -Abewerbung$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Herzlich Willkommen"
                                </a>
                            </li>
                            <li
                                class="intern depth_2 linkItem "
                                title="Meine Bewerbung"
                                id="link000443"
                            >
                                <a class="depth_2 link000443 navLink " href=meine_bewerbung_url>
                                    "Meine Bewerbung"
                                </a>
                            </li>
                            <li
                                class="intern depth_2 linkItem "
                                title="Meine Dokumente"
                                id="link000444"
                            >
                                <a
                                    class="depth_2 link000444 navLink "
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                                 PRGNAME=CREATEDOCUMENT&ARGUMENTS=-N\\d+,-N000444,\
                                                 $",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                >
                                    "Meine Dokumente"
                                </a>
                            </li>
                        </ul>
                    </li>
                    <li class="intern depth_1 linkItem " title="Hilfe" id="link000340">
                        <a
                            class="depth_1 link000340 navLink "
                            href={|v: String| {
                                static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                    Regex::new(
                                        "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&\
                                         PRGNAME=EXTERNALPAGES&ARGUMENTS=-N\\d+,-N000340,-Ahilfe%\
                                         2Ehtml$",
                                    )
                                    .unwrap()
                                });
                                assert!(REGEX.is_match(&v), "{v}");
                            }}
                        >
                            "Hilfe"
                        </a>
                    </li>
                </ul>
            </div>
            <div id="pageHeadBottom_3" class="pageElementTop">
                <div id="pageHeadSwitchLang" class="pageElementRight">
                    <a href=_wef class="img img_LangEnglish pageElementLeft" title="English">
                        "English"
                    </a>
                    <a
                        href={|v: String| {
                            static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                Regex::new(
                                    "^/scripts/mgrqispi.dll\\?APPNAME=CampusNet&PRGNAME=LOGOUT&\
                                     ARGUMENTS=-N\\d+,-N001$",
                                )
                                .unwrap()
                            });
                            assert!(REGEX.is_match(&v), "{v}");
                        }}
                        id="logoutButton"
                        class="img img_arrowLogout logout"
                        title="Abmelden"
                    >
                        "Abmelden"
                    </a>
                </div>
            </div>
        </div>
        <div id="pageContentContainer" class="pageElementTop">
            <div id="pageLeft" class="pageElementLeft">
                <div id="pageLeftTop">
                </div>
            </div>
            <div id="pageContent" class="pageElementLeft">
                <div id="featureBanner">
                </div>
                <a name="mainContent" class="hidden">
                </a>
                <div id="pageContentTop" class="pageElementTop">
                    <div id="loginData">
                        <span class="loginDataLoggedAs">
                            <b>
                                "Sie sind angemeldet als"
                                <span class="colon">
                                    ":"
                                </span>
                            </b>
                        </span>
                        <span class="loginDataName" id="loginDataName">
                            <b>
                                "Name"
                                <span class="colon">
                                    ":"
                                </span>
                            </b>
                            _full_name
                        </span>
                        <span class="loginDataDate">
                            <b>
                                "am"
                                <span class="colon">
                                    ":"
                                </span>
                            </b>
                            _date
                        </span>
                        <span class="loginDataTime">
                            <b>
                                "um"
                                <span class="colon time_colon">
                                    ":"
                                </span>
                            </b>
                            _time
                        </span>
                    </div>
                </div>
                <div id="contentSpacer_IE" class="pageElementTop">
    };
    (
        html_handler,
        LoggedInHead {
            messages_url,
            vorlesungsverzeichnis_url: ActionRequest::parse(
                &ACTION_REGEX.replace(&vorlesungsverzeichnis_url, ""),
            ),
            vv,
            antraege_url,
            meine_bewerbung_url,
        },
    )
}

#[must_use]
pub fn logged_out_head<'a>(
    html_handler: InElement<'a, InElement<'a, InRoot<'a, Root<'a>>>>,
) -> (
    InElement5<'a, InElement<'a, InRoot<'a, Root<'a>>>>,
    LoggedOutHead,
) {
    html_extractor::html! {
        use page_start(html_handler);
        let result = logged_out_head_internal(html_handler);
    }
    (html_handler, result)
}

#[must_use]
fn logged_out_head_internal<'a>(
    html_handler: InElement5<'a, InElement<'a, InRoot<'a, Root<'a>>>>,
) -> (
    InElement5<'a, InElement<'a, InRoot<'a, Root<'a>>>>,
    LoggedOutHead,
) {
    html_extractor::html! {
                    <li class="intern depth_1 linkItem " title="Startseite" id="link000344">
                        <a
                            class="depth_1 link000344 navLink "
                            href="/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=EXTERNALPAGES&\
                         ARGUMENTS=-N000000000000001,-N000344,-Awelcome"
                        >
                            "Startseite"
                        </a>
                    </li>
                    <li
                        class="tree depth_1 linkItem branchLinkItem "
                        title="Vorlesungsverzeichnis (VV)"
                        id="link000334"
                    >
                        <a
                            class="depth_1 link000334 navLink branchLink "
                            href=vorlesungsverzeichnis_url
                        >
                            "Vorlesungsverzeichnis (VV)"
                        </a>
                        let vv = vv_something(html_handler);
                    </li>
                    <li
                        class="tree depth_1 linkItem branchLinkItem "
                        title="TUCaN-Account"
                        id="link000410"
                    >
                        <a
                            class="depth_1 link000410 navLink branchLink "
                            href="/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=EXTERNALPAGES&\
                         ARGUMENTS=-N000000000000001,-N000410,-Atucan%5Faccount%2Ehtml"
                        >
                            "TUCaN-Account"
                        </a>
                        <ul class="nav depth_2 linkItemContainer">
                            <li
                                class="intern depth_2 linkItem "
                                title="Account anlegen"
                                id="link000425"
                            >
                                <a
                                    class="depth_2 link000425 navLink "
                                    href="/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=CREATEACCOUNT&\
                                 ARGUMENTS=-N000000000000001,-N000425,"
                                >
                                    "Account anlegen"
                                </a>
                            </li>
                            <li
                                class="intern depth_2 linkItem "
                                title="Passwort vergessen (nur für Bewerber/innen!)"
                                id="link000426"
                            >
                                <a
                                    class="depth_2 link000426 navLink "
                                    href="/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=LOSTPASS&\
                                 ARGUMENTS=-N000000000000001,-N000426,-A"
                                >
                                    "Passwort vergessen (nur für Bewerber/innen!)"
                                </a>
                            </li>
                        </ul>
                    </li>
                    <li class="intern depth_1 linkItem " title="Hilfe" id="link000340">
                        <a
                            class="depth_1 link000340 navLink "
                            href="/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=EXTERNALPAGES&\
                         ARGUMENTS=-N000000000000001,-N000340,-Ahilfe%2Ehtml"
                        >
                            "Hilfe"
                        </a>
                    </li>
                </ul>
            </div>
            <div id="pageHeadBottom_3" class="pageElementTop">
                <div id="pageHeadSwitchLang" class="pageElementRight">
                    <a
                        href="/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=CHANGELANGUAGE&\
                     ARGUMENTS=-N000000000000002,-N002"
                        class="img img_LangEnglish pageElementLeft"
                        title="English"
                    >
                        "English"
                    </a>
                </div>
                <form
                    name="cn_loginForm"
                    id="cn_loginForm"
                    action="/scripts/mgrqispi.dll"
                    method="post"
                    class="pageElementRight"
                >
                    <div>
                        <fieldset id="fieldSet_login">
                            <legend>
                                "Anmeldung"
                            </legend>
                            <div class="formRow nb">
                                <div class="inputFieldLabel">
                                    <label for="field_user">
                                        "TU-ID:"
                                    </label>
                                    <input
                                        type="text"
                                        id="field_user"
                                        name="usrname"
                                        size="15"
                                        class="login"
                                        maxlength="255"
                                        accesskey="n"
                                        autofocus=""
                                    ></input>
                                </div>
                                <div class="inputFieldLabel">
                                    <label for="field_pass">
                                        "Passwort:"
                                    </label>
                                    <input
                                        type="password"
                                        id="field_pass"
                                        name="pass"
                                        value=""
                                        size="15"
                                        class="login"
                                        maxlength="255"
                                        accesskey="p"
                                    ></input>
                                </div>
                            </div>
                        </fieldset>
                        <input
                            class="img img_arrowSubmit login_btn"
                            type="submit"
                            id="logIn_btn"
                            value="Anmelden"
                            onclick="return checkform('cn_loginForm','usrname:TU-ID,pass:Passwort','\
                         000000000000001');"
                        ></input>
                        <input name="APPNAME" type="hidden" value="CampusNet"></input>
                        <input name="PRGNAME" type="hidden" value="LOGINCHECK"></input>
                        <input
                            name="ARGUMENTS"
                            type="hidden"
                            value="clino,usrname,pass,menuno,menu_type,browser,platform"
                        ></input>
                        <input name="clino" type="hidden" value="000000000000001"></input>
                        <input name="menuno" type="hidden" value=_value></input>
                        <input name="menu_type" type="hidden" value="classic"></input>
                        <input name="browser" type="hidden" value=""></input>
                        <input name="platform" type="hidden" value=""></input>
                    </div>
                </form>
            </div>
        </div>
        <div id="pageContentContainer" class="pageElementTop">
            <div id="pageLeft" class="pageElementLeft">
                <div id="pageLeftTop">
                </div>
            </div>
            <div id="pageContent" class="pageElementLeft">
                <div id="featureBanner">
                </div>
                <a name="mainContent" class="hidden">
                </a>
                <div id="pageContentTop" class="pageElementTop">
                </div>
                <div id="contentSpacer_IE" class="pageElementTop">
    }
    (
        html_handler,
        LoggedOutHead {
            vorlesungsverzeichnis_url: ActionRequest::parse(
                &ACTION_REGEX.replace(&vorlesungsverzeichnis_url, ""),
            ),
            vv,
        },
    )
}

#[must_use]
pub fn logged_in_or_out_head<'a>(
    html_handler: InElement<'a, InElement<'a, InRoot<'a, Root<'a>>>>,
    login_response: Option<&LoginResponse>,
) -> (
    InElement5<'a, InElement<'a, InRoot<'a, Root<'a>>>>,
    Either<LoggedOutHead, LoggedInHead>,
) {
    html_extractor::html! {
        use page_start(html_handler);
        let result = if html_handler
            .peek()
            .unwrap()
            .value()
            .as_element()
            .unwrap()
            .attr("title")
            == Some("Startseite") {
            let a = logged_out_head_internal(html_handler);
        } => a else {
            let b = logged_in_head_internal(html_handler, login_response.unwrap().id);
        } => b;
    }
    (html_handler, result)
}

#[must_use]
pub fn footer<'a>(
    html_handler: InElement<'a, InElement<'a, InElement<'a, InRoot<'a, Root<'a>>>>>,
    _id: u64,
    _subid: u64,
) -> InRoot<'a, Root<'a>> {
    html_extractor::html! {
                    <div id="pageFoot" class="pageElementTop">
                        <div id="pageFootControls" class="pageElementTop">
                            <div id="pageFootControlsLeft">
                                <a
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^\\?APPNAME=CampusNet&PRGNAME=EXTERNALPAGES&\
                                                 ARGUMENTS=-N\\d+,-N\\d+,-Aimprint$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                    class="img img_arrowImprint pageElementLeft"
                                    id="pageFootControl_imp"
                                >
                                    "Impressum"
                                </a>
                                <a
                                    href={|v: String| {
                                        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                                            Regex::new(
                                                "^\\?APPNAME=CampusNet&PRGNAME=EXTERNALPAGES&\
                                                 ARGUMENTS=-N\\d+,-N\\d+,-Acontact$",
                                            )
                                            .unwrap()
                                        });
                                        assert!(REGEX.is_match(&v), "{v}");
                                    }}
                                    class="img img_arrowContact pageElementLeft"
                                    id="pageFootControl_con"
                                >
                                    "Kontakt"
                                </a>
                                <a
                                    href="#"
                                    onclick="window.print();"
                                    class="img img_arrowPrint pageElementLeft"
                                    id="pageFootControl_pri"
                                >
                                    "Drucken"
                                </a>
                            </div>
                            <div id="pageFootControlsRight">
                                <a
                                    href="#top"
                                    class="img img_arrowUp pageElementRight"
                                    id="pageFootControl_up"
                                >
                                </a>
                            </div>
                        </div>
                    </div>
                </div>
                <div id="IEdiv">
                </div>
                <div class="invAnchor">
                    <a name="bottom" class="invAnchor">
                    </a>
                </div>
            </body>
        </html>
    };
    html_handler
}
