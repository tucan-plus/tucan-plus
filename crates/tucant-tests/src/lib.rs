use std::{error::Error, process::Stdio, time::Duration};

use thirtyfour::{extensions::addons::firefox::FirefoxTools, prelude::*};
use tokio::{
    io::{AsyncBufReadExt as _, BufReader},
    time::sleep,
};

pub enum Browser {
    Firefox,
    Chromium,
}

pub enum Mode {
    Extension,
    Api,
}

async fn run_with_chromium_api<F: Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send, A: FnOnce(Browser, Mode, WebDriver) -> F + Send + 'static>(fun: A) -> Result<(), Box<dyn Error + Send + Sync>> {
    // .arg("--enable-chrome-logs")
    let mut child = tokio::process::Command::new("chromedriver").kill_on_drop(true).stdout(Stdio::piped()).spawn()?;

    let stderr = child.stdout.take().unwrap();

    let task = tokio::spawn(async {
        let mut reader = BufReader::new(stderr).lines();

        let mut port: Option<u16> = None;
        while let Some(line) = reader.next_line().await? {
            println!("{line}");
            if line.starts_with("ChromeDriver was started successfully on port ") {
                port = Some(line.strip_prefix("ChromeDriver was started successfully on port ").unwrap().strip_suffix(".").unwrap().parse().unwrap());
                break;
            }
        }

        let mut caps = DesiredCapabilities::chrome();
        caps.set_no_sandbox()?;
        //caps.set_headless()?;
        let driver = WebDriver::new(format!("http://localhost:{}", port.unwrap()), caps).await?;
        driver.set_window_rect(0, 0, 1300, 768).await?;

        fun(Browser::Chromium, Mode::Api, driver).await?;

        Ok::<(), Box<dyn Error + Send + Sync>>(())
    })
    .await;
    child.kill().await?;
    child.wait().await?;
    task??;

    Ok(())
}

async fn run_with_chromium_extension<F: Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send, A: FnOnce(Browser, Mode, WebDriver) -> F + Send + 'static>(fun: A) -> Result<(), Box<dyn Error + Send + Sync>> {
    // .arg("--enable-chrome-logs")
    let mut child = tokio::process::Command::new("chromedriver").kill_on_drop(true).stdout(Stdio::piped()).spawn()?;

    let stderr = child.stdout.take().unwrap();

    let task = tokio::spawn(async {
        let mut reader = BufReader::new(stderr).lines();

        let mut port: Option<u16> = None;
        while let Some(line) = reader.next_line().await? {
            println!("{line}");
            if line.starts_with("ChromeDriver was started successfully on port ") {
                port = Some(line.strip_prefix("ChromeDriver was started successfully on port ").unwrap().strip_suffix(".").unwrap().parse().unwrap());
                break;
            }
        }

        let mut caps = DesiredCapabilities::chrome();
        caps.set_no_sandbox()?;
        caps.set_disable_gpu()?;
        caps.unset_headless()?;
        caps.add_arg(&format!("--load-extension={}", std::env::var("EXTENSION_DIR").unwrap()))?;
        let driver = WebDriver::new(format!("http://localhost:{}", port.unwrap()), caps).await?;
        driver.set_window_rect(0, 0, 1300, 768).await?;

        fun(Browser::Chromium, Mode::Extension, driver).await?;

        Ok::<(), Box<dyn Error + Send + Sync>>(())
    })
    .await;
    child.kill().await?;
    child.wait().await?;
    task??;

    Ok(())
}

async fn run_with_firefox_api<F: Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send, A: FnOnce(Browser, Mode, WebDriver) -> F + Send + 'static>(fun: A) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut child = tokio::process::Command::new("geckodriver").arg("--port=0").kill_on_drop(true).stdout(Stdio::piped()).spawn()?;

    let stderr = child.stdout.take().unwrap();

    let task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();

        let mut port: Option<u16> = None;
        while let Some(line) = reader.next_line().await? {
            println!("{line}");
            if line.contains("Listening on") {
                port = Some(line.rsplit_once(':').unwrap().1.parse().unwrap());
                break;
            }
        }

        let caps = DesiredCapabilities::firefox();
        let driver = WebDriver::new(format!("http://localhost:{}", port.unwrap()), caps).await?;
        driver.set_window_rect(0, 0, 1300, 768).await?;

        fun(Browser::Firefox, Mode::Api, driver).await?;

        Ok::<(), Box<dyn Error + Send + Sync>>(())
    })
    .await;

    child.kill().await?;
    child.wait().await?;
    task??;

    Ok(())
}

async fn run_with_firefox_extension<F: Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send, A: FnOnce(Browser, Mode, WebDriver) -> F + Send + 'static>(fun: A) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut child = tokio::process::Command::new("geckodriver").kill_on_drop(true).stdout(Stdio::piped()).spawn()?;

    let stderr = child.stdout.take().unwrap();

    let task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();

        let mut port: Option<u16> = None;
        while let Some(line) = reader.next_line().await? {
            println!("{line}");
            if line.contains("Listening on") {
                port = Some(line.rsplit_once(':').unwrap().1.parse().unwrap());
                break;
            }
        }

        let caps = DesiredCapabilities::firefox();
        let driver = WebDriver::new(format!("http://localhost:{}", port.unwrap()), caps).await?;
        driver.set_window_rect(0, 0, 1300, 768).await?;
        let tools = FirefoxTools::new(driver.handle.clone());
        tools.install_addon(&std::env::var("EXTENSION_DIR").unwrap(), Some(true)).await?;

        fun(Browser::Firefox, Mode::Extension, driver).await?;

        // TODO stop driver?

        Ok::<(), Box<dyn Error + Send + Sync>>(())
    })
    .await;

    child.kill().await?;
    child.wait().await?;
    task??;

    Ok(())
}

pub async fn test(browser: Browser, mode: Mode, driver: WebDriver) -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenvy::dotenv().unwrap();

    sleep(Duration::from_secs(1)).await; // wait for extension to be installed

    driver
        .goto(match mode {
            Mode::Extension => "https://www.tucan.tu-darmstadt.de/",
            Mode::Api => "http://localhost:1420",
        })
        .await?;

    assert_eq!(driver.title().await?, "TUCaN't");

    assert_eq!(
        driver.current_url().await.unwrap().scheme(),
        match mode {
            Mode::Extension => match browser {
                Browser::Firefox => "moz-extension",
                Browser::Chromium => "chrome-extension",
            },
            Mode::Api => "http",
        }
    );

    let username_input = driver.query(By::Css("#login-username")).first().await?;
    let password_input = driver.find(By::Css("#login-password")).await?;
    let login_button = driver.find(By::Css("#login-button")).await?;

    let username = std::env::var("TUCAN_USERNAME").expect("env variable TUCAN_USERNAME missing");
    let password = std::env::var("TUCAN_PASSWORD").expect("env variable TUCAN_PASSWORD missing");

    username_input.send_keys(username).await?;
    password_input.send_keys(password).await?;
    // probably https://yew.rs/docs/concepts/html/events#event-delegation
    username_input.focus().await?;
    login_button.click().await?;

    sleep(Duration::from_secs(10)).await;

    driver.quit().await?;

    Ok(())
}

macro_rules! all_browsers {
    ($function_name: ident) => {
        ::paste::paste! {
            #[::tokio::test]
            pub async fn [<$function_name _firefox_extension>]() -> Result<(), Box<dyn Error + Send + Sync>> {
                run_with_firefox_extension($function_name).await
            }

            #[::tokio::test]
            pub async fn [<$function_name _firefox_api>]() -> Result<(), Box<dyn Error + Send + Sync>> {
                run_with_firefox_api($function_name).await
            }

            #[::tokio::test]
            pub async fn [<$function_name _chromium_extension>]() -> Result<(), Box<dyn Error + Send + Sync>> {
                run_with_chromium_extension($function_name).await
            }

            #[::tokio::test]
            pub async fn [<$function_name _chromium_api>]() -> Result<(), Box<dyn Error + Send + Sync>> {
                run_with_chromium_api($function_name).await
            }
        }
    };
}

all_browsers!(test);
