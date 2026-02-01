use rustenium::{
    browsers::{ChromeCapabilities, ChromeConfig, create_chrome_browser},
    css,
    nodes::Node,
    rustenium_bidi_commands::{
        CommandData, ResultData, WebExtensionCommand, WebExtensionResult,
        browsing_context::types::ReadinessState,
        web_extension::{
            commands::{Install, InstallParameters, WebExtensionInstallMethod},
            types::{ExtensionData, ExtensionPath, PathEnum},
        },
    },
};
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn open_browser() {
    let mut browser = create_chrome_browser(Some(ChromeConfig {
        chrome_executable_path: Some("chromium-browser".to_string()),
        driver_executable_path: "chromedriver".to_string(),
        //remote_debugging_port: Some(0),
        capabilities: ChromeCapabilities::default()
            .add_args([
                "--enable-unsafe-extension-debugging",
                "--remote-debugging-pipe",
            ])
            .clone(),
        ..Default::default()
    }))
    .await;
    let ResultData::WebExtensionResult(WebExtensionResult::InstallResult(result)) = browser
        .send_bidi_command(CommandData::WebExtensionCommand(
            WebExtensionCommand::Install(Install {
                method: WebExtensionInstallMethod::WebExtensionInstall,
                params: InstallParameters {
                    extension_data: ExtensionData::ExtensionPath(ExtensionPath {
                        r#type: PathEnum::Path,
                        path: "../../tucan-plus-extension".to_string(),
                    }),
                },
            }),
        ))
        .await
        .unwrap()
    else {
        panic!()
    };
    println!("{:?}", result);
    sleep(Duration::from_secs(1)).await; // wait for extension installed
    browser
        .open_url(
            "https://www.tucan.tu-darmstadt.de/",
            Some(ReadinessState::Complete),
            None,
        )
        .await
        .unwrap();
    let element = browser
        .wait_for_node(css!("h1"), None, None, None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        element.get_text_content().await,
        "Willkommen bei TUCaN Plus!"
    );
    // Willkommen bei TUCaN Plus!
    browser.end_bidi_session().await.unwrap();
}
