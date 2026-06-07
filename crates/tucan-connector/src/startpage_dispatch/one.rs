use crate::{TucanConnector, TucanError, head::html_head_2, retryable_get};
use html_handler::{Root, parse_document};

pub async fn startpage_dispatch_1(connector: &TucanConnector) -> Result<(), TucanError> {
    let (content, ..) = retryable_get(
        connector,
        "https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=STARTPAGE_DISPATCH&ARGUMENTS=-N000000000000001",
    )
    .await?;
    let document = parse_document(&content);
    let html_handler = Root::new(document.root());
    let html_handler = html_handler.document_start();
    let html_handler = html_handler.doctype();
    html_extractor::html! {
        <html>
            <head>
                use html_head_2(html_handler);
            </head>
            <body class="redirect">
                <div id="wrapper">
                    <a
                        href="http://http://www.tu-darmstadt.de"
                        title="extern http://www.tu-darmstadt.de"
                    >
                        <img
                            border="0"
                            id="logo"
                            src="/gfx/tuda/logo.png"
                            alt="Logo Technische Universität Darmstadt"
                        ></img>
                    </a>
                    <h2>
                        <a
                            href="/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=EXTERNALPAGES&\
                         ARGUMENTS=-N000000000000001,-N000344,-Awelcome"
                        >
                            "Sie werden zur Startseite weitergeleitet ..."
                        </a>
                    </h2>
                    <a
                        style="text-decoration: underline;"
                        href="/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=EXTERNALPAGES&\
                     ARGUMENTS=-N000000000000001,-N000344,-Awelcome"
                    >
                        "Startseite"
                    </a>
                </div>
                <div id="sessionId" style="display: none;">
                    "000000000000001"
                </div>
                <script>
                    "window.setTimeout(function() {\n\t\t\twindow.location.href = \
                     '/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=EXTERNALPAGES&\
                     ARGUMENTS=-N000000000000001,-N000344,-Awelcome';\n\t\t}, 500);"
                </script>
            </body>
        </html>
    };
    html_handler.end_document();
    Ok(())
}
