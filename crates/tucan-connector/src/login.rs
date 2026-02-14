use std::sync::LazyLock;

use html_handler::parse_document;
use regex::Regex;
use reqwest::header::HeaderValue;
use tucan_types::{LoginRequest, LoginResponse};

use crate::{MyClient, TucanConnector, TucanError, authenticated_retryable_get};

pub async fn logout(
    connector: &TucanConnector,
    login_response: &LoginResponse,
) -> Result<(), TucanError> {
    let _content = authenticated_retryable_get(
        connector,
        &format!(
            "https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=LOGOUT&ARGUMENTS=-N{:015},-N001",
            login_response.id
        ),
        &login_response.cookie_cnsc,
    )
    .await?;
    Ok(())
}
