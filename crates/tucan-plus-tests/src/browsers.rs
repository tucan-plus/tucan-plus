use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::Path,
    process::Stdio,
    sync::LazyLock,
    time::Duration,
};

use async_trait::async_trait;
use serde_json::json;
use tokio::{
    io::{AsyncBufReadExt as _, BufReader},
    sync::Mutex,
    time::sleep,
};
use webdriverbidi::{
    model::web_extension::{ExtensionData, ExtensionPath, InstallParameters},
    session::WebDriverBiDiSession,
    webdriver::capabilities::CapabilitiesRequest,
};

pub static ANDROID_MUTEX: tokio::sync::Mutex<()> = Mutex::const_new(());

pub trait BrowserBuilder: Browser + 'static {
    async fn start(unpacked_extension: &Path) -> Self;
}

#[async_trait]
pub trait Browser: Send + Sync + DerefMut<Target = WebDriverBiDiSession> {
    async fn load_extension(&mut self, unpacked_extension: &Path) {}
}

pub struct DesktopFirefox(WebDriverBiDiSession);

impl Deref for DesktopFirefox {
    type Target = WebDriverBiDiSession;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DesktopFirefox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BrowserBuilder for DesktopFirefox {
    async fn start(unpacked_extension: &Path) -> Self {
        // also start the webdriver here
        let mut cmd = tokio::process::Command::new("/home/moritz/Downloads/geckodriver");
        cmd.kill_on_drop(true);
        cmd.arg("--log=trace");
        cmd.arg("--port=0");
        cmd.arg("--websocket-port=1234");

        cmd.stdout(Stdio::piped());

        let mut child = cmd.spawn().expect("failed to spawn command");

        let stdout = child
            .stdout
            .take()
            .expect("child did not have a handle to stdout");

        let mut reader = BufReader::new(stdout).lines();

        // Ensure the child process is spawned in the runtime so it can
        // make progress on its own while we await for any output.
        tokio::spawn(async move {
            let status = child
                .wait()
                .await
                .expect("child process encountered an error");

            println!("child status was: {}", status);
        });

        let mut port: Option<u16> = None;
        while let Some(line) = reader.next_line().await.unwrap() {
            println!("Line: {}", line);
            const PATTERN: &str = "Listening on ";
            if let Some(index) = line.find(PATTERN) {
                port = Some(
                    line[index + PATTERN.len()..]
                        .split_once(":")
                        .unwrap()
                        .1
                        .parse()
                        .unwrap(),
                );
                break;
            }
        }
        cmd.stdout(Stdio::inherit());

        let port = port.unwrap();
        println!("port {:?}", port);

        let mut capabilities = CapabilitiesRequest::default();

        capabilities.add_first_match(HashMap::from([]));

        let mut session = WebDriverBiDiSession::new("localhost".to_owned(), port, capabilities);
        session.start().await.unwrap();
        Self(session)
    }
}

#[async_trait]
impl Browser for DesktopFirefox {
    async fn load_extension(&mut self, unpacked_extension: &Path) {
        self.web_extension_install(InstallParameters::new(ExtensionData::ExtensionPath(
            ExtensionPath::new(unpacked_extension.to_string_lossy().to_string()),
        )))
        .await
        .unwrap();
        sleep(Duration::from_secs(1)).await; // wait for extension to be installed
    }
}

pub struct DesktopChromium(WebDriverBiDiSession);

impl Deref for DesktopChromium {
    type Target = WebDriverBiDiSession;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DesktopChromium {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BrowserBuilder for DesktopChromium {
    async fn start(unpacked_extension: &Path) -> Self {
        // also start the webdriver here
        let mut cmd = tokio::process::Command::new("chromedriver");
        cmd.kill_on_drop(true);
        cmd.arg("--enable-chrome-logs");

        cmd.stdout(Stdio::piped());

        let mut child = cmd.spawn().expect("failed to spawn command");

        let stdout = child
            .stdout
            .take()
            .expect("child did not have a handle to stdout");

        let mut reader = BufReader::new(stdout).lines();

        // Ensure the child process is spawned in the runtime so it can
        // make progress on its own while we await for any output.
        tokio::spawn(async move {
            let status = child
                .wait()
                .await
                .expect("child process encountered an error");

            println!("child status was: {}", status);
        });

        let mut port: Option<u16> = None;
        while let Some(line) = reader.next_line().await.unwrap() {
            println!("Line: {}", line);
            const PATTERN: &str = " was started successfully on port ";
            if let Some(index) = line.find(PATTERN) {
                port = Some(
                    line[index + PATTERN.len()..line.len() - 1]
                        .to_owned()
                        .parse()
                        .unwrap(),
                );
                break;
            }
        }
        let port = port.unwrap();
        println!("port {:?}", port);

        cmd.stdout(Stdio::inherit());

        let chrome_options = json!({
            "args": ["--enable-unsafe-extension-debugging", "--remote-debugging-pipe", format!("--load-extension={}", unpacked_extension.display())],
            "enableExtensionTargets": true,
            "binary": "/home/moritz/Downloads/chrome-linux64/chrome"
        });
        let mut capabilities = CapabilitiesRequest::default();

        capabilities.add_first_match(HashMap::from([
            ("browserName".to_owned(), json!("chrome")),
            ("goog:chromeOptions".to_owned(), chrome_options),
        ]));

        let mut session = WebDriverBiDiSession::new("localhost".to_owned(), port, capabilities);
        session.start().await.unwrap();
        Self(session)
    }
}

impl Browser for DesktopChromium {}

pub struct AndroidFirefox(WebDriverBiDiSession);

impl Deref for AndroidFirefox {
    type Target = WebDriverBiDiSession;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AndroidFirefox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BrowserBuilder for AndroidFirefox {
    async fn start(unpacked_extension: &Path) -> Self {
        // also start the webdriver here
        let mut cmd = tokio::process::Command::new("/home/moritz/Downloads/geckodriver");
        cmd.kill_on_drop(true);
        cmd.arg("--port=0");

        cmd.stdout(Stdio::piped());

        let mut child = cmd.spawn().expect("failed to spawn command");

        let stdout = child
            .stdout
            .take()
            .expect("child did not have a handle to stdout");

        let mut reader = BufReader::new(stdout).lines();

        // Ensure the child process is spawned in the runtime so it can
        // make progress on its own while we await for any output.
        tokio::spawn(async move {
            let status = child
                .wait()
                .await
                .expect("child process encountered an error");

            println!("child status was: {}", status);
        });

        let mut port: Option<u16> = None;
        while let Some(line) = reader.next_line().await.unwrap() {
            println!("Line: {}", line);
            const PATTERN: &str = "Listening on ";
            if let Some(index) = line.find(PATTERN) {
                port = Some(
                    line[index + PATTERN.len()..]
                        .split_once(":")
                        .unwrap()
                        .1
                        .parse()
                        .unwrap(),
                );
                break;
            }
        }
        let port = port.unwrap();
        println!("port {:?}", port);

        cmd.stdout(Stdio::inherit());

        let mut capabilities = CapabilitiesRequest::default();

        capabilities.add_first_match(HashMap::from([
            ("browserName".to_owned(), json!("firefox")),
            (
                "moz:firefoxOptions".to_owned(),
                json!({
                    "androidPackage": "org.mozilla.firefox",
                    // DO NOT RUN ON PHYSICAL DEVICES AS IT WILL CLEAR YOUR DATA
                    "androidDeviceSerial": "emulator-5554",
                }),
            ),
        ]));

        let mut session = WebDriverBiDiSession::new("localhost".to_owned(), port, capabilities);
        session.start().await.unwrap();
        Self(session)
    }
}

#[async_trait]
impl Browser for AndroidFirefox {
    async fn load_extension(&mut self, unpacked_extension: &Path) {
        assert!(
            tokio::process::Command::new("adb")
                .arg("push")
                .arg(unpacked_extension)
                .arg("/data/local/tmp/tucan-plus-extension")
                .status()
                .await
                .unwrap()
                .success()
        );

        self.web_extension_install(InstallParameters::new(ExtensionData::ExtensionPath(
            ExtensionPath::new("/data/local/tmp/tucan-plus-extension".to_owned()),
        )))
        .await
        .unwrap();
        sleep(Duration::from_secs(1)).await; // wait for extension to be installed
    }
}

pub struct AndroidEdgeCanary(WebDriverBiDiSession);

impl Deref for AndroidEdgeCanary {
    type Target = WebDriverBiDiSession;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AndroidEdgeCanary {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BrowserBuilder for AndroidEdgeCanary {
    async fn start(unpacked_extension: &Path) -> Self {
        // bug/missing feature in chromedriver
        assert!(tokio::process::Command::new("adb")
        .arg("shell")
        .arg("echo \"chrome --allow-pre-commit-input --disable-background-networking --disable-background-timer-throttling --disable-backgrounding-occluded-windows --disable-features=IgnoreDuplicateNavs,Prewarm --disable-fre --disable-popup-blocking --enable-automation --enable-remote-debugging --enable-unsafe-extension-debugging --load-extension=/data/local/tmp/tucan-plus-extension --remote-debugging-pipe\" > /data/local/tmp/chrome-command-line")
        .status().await.unwrap().success());

        assert!(
            tokio::process::Command::new("adb")
                .arg("shell")
                .arg("rm -rf /data/local/tmp/tucan-plus-extension")
                .status()
                .await
                .unwrap()
                .success()
        );

        assert!(
            tokio::process::Command::new("adb")
                .arg("push")
                .arg(unpacked_extension)
                .arg("/data/local/tmp/tucan-plus-extension")
                .status()
                .await
                .unwrap()
                .success()
        );

        // also start the webdriver here
        let mut cmd =
            tokio::process::Command::new("/home/moritz/Downloads/edgedriver_linux64/msedgedriver");
        cmd.kill_on_drop(true);

        cmd.stdout(Stdio::piped());

        let mut child = cmd.spawn().expect("failed to spawn command");

        let stdout = child
            .stdout
            .take()
            .expect("child did not have a handle to stdout");

        let mut reader = BufReader::new(stdout).lines();

        // Ensure the child process is spawned in the runtime so it can
        // make progress on its own while we await for any output.
        tokio::spawn(async move {
            let status = child
                .wait()
                .await
                .expect("child process encountered an error");

            println!("child status was: {}", status);
        });

        let mut port: Option<u16> = None;
        while let Some(line) = reader.next_line().await.unwrap() {
            println!("Line: {}", line);
            const PATTERN: &str = " was started successfully on port ";
            if let Some(index) = line.find(PATTERN) {
                port = Some(
                    line[index + PATTERN.len()..line.len() - 1]
                        .to_owned()
                        .parse()
                        .unwrap(),
                );
                break;
            }
        }
        let port = port.unwrap();
        println!("port {:?}", port);

        cmd.stdout(Stdio::inherit());

        let edge_options = json!({
            "args": ["--enable-unsafe-extension-debugging", "--remote-debugging-pipe", "--load-extension=/data/local/tmp/tucan-plus-extension"],
            "androidPackage": "com.microsoft.emmx.canary",
            "androidActivity": "com.microsoft.ruby.Main",
            "androidExecName": "chrome",
            "androidDeviceSocket": "chrome_devtools_remote",
            "androidDeviceSerial": "emulator-5554",
            "enableExtensionTargets": true
        });
        let mut capabilities = CapabilitiesRequest::default();

        capabilities.add_first_match(HashMap::from([
            ("browserName".to_owned(), json!("msedge")),
            ("ms:edgeOptions".to_owned(), edge_options),
        ]));

        let mut session = WebDriverBiDiSession::new("localhost".to_owned(), port, capabilities);
        session.start().await.unwrap();
        Self(session)
    }
}

impl Browser for AndroidEdgeCanary {}

pub struct AndroidChromium(WebDriverBiDiSession);

impl Deref for AndroidChromium {
    type Target = WebDriverBiDiSession;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AndroidChromium {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BrowserBuilder for AndroidChromium {
    async fn start(unpacked_extension: &Path) -> Self {
        // bug/missing feature in chromedriver
        assert!(tokio::process::Command::new("adb")
        .arg("shell")
        .arg("echo \"chrome --allow-pre-commit-input --disable-background-networking --disable-background-timer-throttling --disable-backgrounding-occluded-windows --disable-features=IgnoreDuplicateNavs,Prewarm --disable-fre --disable-popup-blocking --enable-automation --enable-remote-debugging --enable-unsafe-extension-debugging --load-extension=/data/local/tmp/tucan-plus-extension --remote-debugging-pipe\" > /data/local/tmp/chrome-command-line")
        .status().await.unwrap().success());

        assert!(
            tokio::process::Command::new("adb")
                .arg("shell")
                .arg("rm -rf /data/local/tmp/tucan-plus-extension")
                .status()
                .await
                .unwrap()
                .success()
        );

        assert!(
            tokio::process::Command::new("adb")
                .arg("push")
                .arg(unpacked_extension)
                .arg("/data/local/tmp/tucan-plus-extension")
                .status()
                .await
                .unwrap()
                .success()
        );

        assert!(
            tokio::process::Command::new("adb")
                .args([
                    "shell",
                    "chmod",
                    "-R",
                    "0777",
                    "/data/local/tmp/tucan-plus-extension"
                ])
                .status()
                .await
                .unwrap()
                .success()
        );

        // also start the webdriver here
        let mut cmd = tokio::process::Command::new("chromedriver");
        cmd.kill_on_drop(true);

        cmd.stdout(Stdio::piped());

        let mut child = cmd.spawn().expect("failed to spawn command");

        let stdout = child
            .stdout
            .take()
            .expect("child did not have a handle to stdout");

        let mut reader = BufReader::new(stdout).lines();

        // Ensure the child process is spawned in the runtime so it can
        // make progress on its own while we await for any output.
        tokio::spawn(async move {
            let status = child
                .wait()
                .await
                .expect("child process encountered an error");

            println!("child status was: {}", status);
        });

        let mut port: Option<u16> = None;
        while let Some(line) = reader.next_line().await.unwrap() {
            println!("Line: {}", line);
            const PATTERN: &str = " was started successfully on port ";
            if let Some(index) = line.find(PATTERN) {
                port = Some(
                    line[index + PATTERN.len()..line.len() - 1]
                        .to_owned()
                        .parse()
                        .unwrap(),
                );
                break;
            }
        }
        let port = port.unwrap();
        println!("port {:?}", port);

        cmd.stdout(Stdio::inherit());

        let edge_options = json!({
            "args": ["--enable-unsafe-extension-debugging", "--remote-debugging-pipe", "--load-extension=/data/local/tmp/tucan-plus-extension"],
            "androidPackage": "org.chromium.chrome",
            "androidDeviceSerial": "emulator-5554",
            "enableExtensionTargets": true
        });
        let mut capabilities = CapabilitiesRequest::default();

        capabilities.add_first_match(HashMap::from([
            ("browserName".to_owned(), json!("chrome")),
            ("goog:chromeOptions".to_owned(), edge_options),
        ]));

        let mut session = WebDriverBiDiSession::new("localhost".to_owned(), port, capabilities);
        session.start().await.unwrap();
        Self(session)
    }
}

impl Browser for AndroidChromium {}
