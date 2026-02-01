use std::{collections::HashMap, path::Path};

use serde_json::json;
use webdriverbidi::{session::WebDriverBiDiSession, webdriver::capabilities::CapabilitiesRequest};

pub trait Browser {
    async fn start(unpacked_extension: &Path) -> WebDriverBiDiSession;
}

pub struct DesktopFirefox;

pub struct DesktopChromium;

pub struct AndroidFirefox;

pub struct AndroidEdgeCanary;

impl Browser for AndroidEdgeCanary {
    async fn start(unpacked_extension: &Path) -> WebDriverBiDiSession {
        // also start the webdriver here

        assert!(tokio::process::Command::new("adb")
        .arg("shell")
        .arg("echo \"chrome --allow-pre-commit-input --disable-background-networking --disable-background-timer-throttling --disable-backgrounding-occluded-windows --disable-features=IgnoreDuplicateNavs,Prewarm --disable-fre --disable-popup-blocking --enable-automation --enable-remote-debugging --enable-unsafe-extension-debugging --load-extension=/data/local/tmp/tucan-plus-extension --remote-debugging-pipe\" > /data/local/tmp/chrome-command-line")
        .spawn().unwrap().wait().await.unwrap().success());
        // /data/local/tmp/tucan-plus-extension

        let edge_options = json!({
            "args": ["--enable-unsafe-extension-debugging", "--remote-debugging-pipe", "--load-extension=/data/local/tmp/tucan-plus-extension-0.49.0"],
            "androidPackage": "com.microsoft.emmx.canary",
            "androidActivity": "com.microsoft.ruby.Main",
            "androidExecName": "chrome",
            "androidDeviceSocket": "chrome_devtools_remote",
            //"extensions": [extension_base64],
            "enableExtensionTargets": true
        });
        let mut capabilities = CapabilitiesRequest::default();

        capabilities.add_first_match(HashMap::from([
            ("browserName".to_owned(), json!("msedge")),
            ("ms:edgeOptions".to_owned(), edge_options),
        ]));

        let mut session = WebDriverBiDiSession::new("localhost".to_owned(), 4444, capabilities);
        session.start().await.unwrap();
        session
    }
}
