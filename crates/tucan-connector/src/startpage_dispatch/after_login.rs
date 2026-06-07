use tucan_types::LoginResponse;

use crate::{TucanConnector, TucanError, authenticated_retryable_get, head::html_head_2};
use html_handler::{Root, parse_document};

pub async fn redirect_after_login(
    connector: &TucanConnector,
    login_response: LoginResponse,
) -> Result<(), TucanError> {
    let (content, ..) = authenticated_retryable_get(
        connector,
        &format!(
            "https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=STARTPAGE_DISPATCH&ARGUMENTS=-N{},-N000019,-N000000000000000",
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
                        <a href=_href_link_1>
                            "Sie werden zur Startseite weitergeleitet ..."
                        </a>
                    </h2>
                    <a style="text-decoration: underline;" href=_href_link_2>
                        "Startseite"
                    </a>
                </div>
                <div id="sessionId" style="display: none;">
                    _session_id
                </div>
                <script>
                    _script_contents
                </script>
    };
    let _ = html_handler;
    Ok(())
}
