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

pub async fn login(
    client: &MyClient,
    login_request: &LoginRequest,
) -> Result<LoginResponse, TucanError> {
    static NEXT_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"0; URL=/scripts/mgrqispi\.dll\?APPNAME=CampusNet&PRGNAME=STARTPAGE_DISPATCH&ARGUMENTS=-N(?P<id>\d+),-N000019,-N000000000000000").unwrap()
    });
    assert_ne!(login_request.username, "");
    assert_ne!(login_request.password, "");
    let mut response = client
        .post("https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll")
        .form(&[
            ("usrname", login_request.username.as_str()),
            ("pass", login_request.password.as_str()),
            ("APPNAME", "CampusNet"),
            ("PRGNAME", "LOGINCHECK"),
            (
                "ARGUMENTS",
                "clino,usrname,pass,menuno,menu_type,browser,platform",
            ),
            ("clino", "000000000000001"),
            ("menuno", "000344"),
            ("menu_type", "classic"),
            ("browser", ""),
            ("platform", ""),
        ])
        .send()
        .await?
        .error_for_status()?;
    assert_eq!(
        response.headers_mut().remove("content-type"),
        Some(HeaderValue::from_static("text/html"))
    );
    assert_eq!(
        response.headers_mut().remove("server"),
        Some(HeaderValue::from_static("Microsoft-IIS/10.0"))
    );
    response.headers_mut().remove("mgmiddlewarewaittime");
    assert_eq!(
        response.headers_mut().remove("strict-transport-security"),
        Some(HeaderValue::from_static(
            "max-age=31536000; includeSubDomains"
        ))
    );
    assert_eq!(
        response.headers_mut().remove("x-xss-protection"),
        Some(HeaderValue::from_static("1; mode=block"))
    );
    assert_eq!(
        response.headers_mut().remove("x-frame-options"),
        Some(HeaderValue::from_static("SAMEORIGIN"))
    );
    assert_eq!(
        response.headers_mut().remove("referrer-policy"),
        Some(HeaderValue::from_static("strict-origin"))
    );
    assert_eq!(
        response.headers_mut().remove("x-content-type-options"),
        Some(HeaderValue::from_static("nosniff"))
    );
    assert_eq!(
        response.headers_mut().remove("content-security-policy"),
        Some(HeaderValue::from_static(
            "frame-src https://dsf.tucan.tu-darmstadt.de; frame-ancestors 'self' https://dsf.tucan.tu-darmstadt.de;"
        ))
    );
    response.headers_mut().remove("x-powered-by"); // this header randomly appears and disappears. DO NOT ASK
    assert!(response.headers_mut().remove("date").is_some(),);
    response.headers_mut().remove("content-length");
    let next_url = response.headers_mut().remove("refresh");
    if next_url.is_none() {
        // login failed
        let content = response.text().await?;
        if content
            .contains("Bitte versuchen Sie es erneut. Überprüfen Sie ggf. Ihre Zugangsdaten.") 
        {
            return Err(TucanError::InvalidCredentials);
        }
        if content
            .contains("Aufgrund einer Häufung fehlgeschlagener Einloggversuche ist dieses Benutzerkonto vorübergehend gesperrt.") 
        {
            return Err(TucanError::AccessDenied);
        }
        panic!("{}", content);
    }
    let next_url = next_url.unwrap();
    let next_url = next_url.to_str().unwrap();
    let id = &NEXT_URL_REGEX
        .captures(next_url)
        .expect("english is not supported")["id"];
    let cookie_cnsc = if cfg!(target_arch = "wasm32") {
        String::new()
    } else {
        let cookie_cnsc = response.headers_mut().remove("set-cookie").unwrap();
        cookie_cnsc
            .to_str()
            .unwrap()
            .trim_start_matches("cnsc =")
            .to_owned()
    };
    response.headers_mut().remove("content-encoding");
    response.headers_mut().remove("vary");
    assert_eq!(response.headers().into_iter().collect::<Vec<_>>(), []);
    let content = response.text().await?;
    parse_document(&content);
    Ok(LoginResponse {
        id: id.parse().unwrap(),
        cookie_cnsc,
    })
}
