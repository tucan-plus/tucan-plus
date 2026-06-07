use crate::{
    TucanConnector, TucanError,
    head::{footer, html_head, logged_out_head},
    retryable_get,
};
use html_handler::{Root, parse_document};
use tucan_types::LoggedOutHead;

pub async fn welcome(connector: &TucanConnector) -> Result<LoggedOutHead, TucanError> {
    let (content, ..) = retryable_get(
        connector,
        "https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=EXTERNALPAGES&ARGUMENTS=-N000000000000001,-N000344,-Awelcome",
    )
    .await?;
    let document = parse_document(&content);
    let html_handler = Root::new(document.root());
    let html_handler = html_handler.document_start();
    let html_handler = html_handler.doctype();
    html_extractor::html! {
            <html
                xmlns="http://www.w3.org/1999/xhtml"
                xml:lang="de"
                lang="de"
                xmlns:msdt="uuid:C2F41010-65B3-11d1-A29F-00AA00C14882"
                xmlns:mso="urn:schemas-microsoft-com:office:office"
            >
                <head>
                    use html_head(html_handler)?;
                    <style type="text/css">
                        "PBsLNqyhelKIL09TLRqYsD4XcU0zItzE9RmRIPZhHFo"
                    </style>
                </head>
                <body class="external_pages">
                    let vv = logged_out_head(html_handler);
                    <script type="text/javascript">
                    </script>
                    <meta http-equiv="content-type" content="text/html; charset=windows-1252"></meta>
                    <div
                        id="inhalt"
                        style="padding:0px; width:650px; margin:0px; background-color:#ffffff;"
                    >
                        let _unused = while html_handler.peek().is_some() {
                            let any_child = html_handler.next_any_child();
                        } => any_child;
                    </div>
                </div>
            </div>
        </div>
        use footer(html_handler, 1, 344);
    }
    html_handler.end_document();
    Ok(vv)
}
