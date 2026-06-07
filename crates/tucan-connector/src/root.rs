use crate::{TucanConnector, TucanError, retryable_get};
use html_handler::{Root, parse_document};

pub async fn root(connector: &TucanConnector) -> Result<(), TucanError> {
    let (content, ..) = retryable_get(connector, "https://www.tucan.tu-darmstadt.de/").await?;
    let document = parse_document(&content);
    let html_handler = Root::new(document.root());
    let html_handler = html_handler.document_start();
    let html_handler = html_handler.doctype();
    html_extractor::html! {
        <html>
            <head>
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
                <link rel="shortcut icon" type="image/x-icon" href="/gfx/tuda/icons/favicon.ico"></link>
                <link
                    rel="apple-touch-icon"
                    href="/gfx/tuda/icons/iphone_touch_icon.png"
                    type="image/gif"
                ></link>
                <meta
                    http-equiv="refresh"
                    content="0; URL=/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=STARTPAGE_DISPATCH&\
                 ARGUMENTS=-N000000000000001"
                ></meta>
                <link href="/css/_default/dl.startpage.css" rel="stylesheet" type="text/css"></link>
                <script type="text/javascript" src="/js/mobile_master/jquery.js">
                </script>
                <script type="text/javascript" src="/js/mobile_master/onmediaquery.min.js">
                </script>
            </head>
            <body>
                <div id="wrapper">
                    <a href="http://www.tu-darmstadt.de" title="extern http://www.tu-darmstadt.de">
                        <img
                            border="0"
                            id="logo"
                            src="/gfx/tuda/logo.png"
                            alt="Logo Technische Universität Darmstadt"
                        ></img>
                    </a>
                    <ul id="langMenu">
                        <li>
                            <a
                                class="img img_LangGerman"
                                href="/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=STARTPAGE_DISPATCH&\
                             ARGUMENTS=-N000000000000001"
                            >
                                "de"
                            </a>
                        </li>
                        <li>
                            <a
                                class="img img_LangEnglish"
                                href="/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=STARTPAGE_DISPATCH&\
                             ARGUMENTS=-N000000000000002"
                            >
                                "en"
                            </a>
                        </li>
                    </ul>
                </div>
            </body>
        </html>
    };
    html_handler.end_document();
    Ok(())
}
