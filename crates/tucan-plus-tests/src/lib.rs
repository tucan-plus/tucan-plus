pub mod browsers;

use std::{
    collections::HashMap,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use base64::{Engine, prelude::BASE64_STANDARD};
use dotenvy::dotenv;
use serde_json::json;
use tokio::{process::Command, sync::OnceCell, time::sleep};
use webdriverbidi::{
    events::EventType,
    model::{
        browsing_context::{
            BrowsingContext, CloseParameters, CssLocator, GetTreeParameters, LocateNodesParameters,
            Locator, NavigateParameters, ReadinessState, SetViewportParameters, Viewport,
        },
        common::Extensible,
        input::{
            ElementOrigin, KeyDownAction, KeySourceAction, KeySourceActions, KeyUpAction, Origin,
            PerformActionsParameters, PointerCommonProperties, PointerDownAction,
            PointerMoveAction, PointerParameters, PointerSourceAction, PointerSourceActions,
            PointerType, PointerUpAction, SourceActions,
        },
        script::{
            CallFunctionParameters, ContextTarget, EvaluateParameters, GetRealmsParameters,
            IncludeShadowTree, LocalValue, NodeRemoteValue, RealmInfo, RemoteReference,
            ResultOwnership, SerializationOptions, SharedReference, Target,
        },
        session::SubscriptionRequest,
        web_extension::{ExtensionBase64Encoded, ExtensionData, ExtensionPath, InstallParameters},
    },
    session::WebDriverBiDiSession,
    webdriver::capabilities::CapabilitiesRequest,
};

use crate::browsers::{
    ANDROID_MUTEX, AndroidChromium, AndroidEdgeCanary, AndroidFirefox, Browser, BrowserBuilder,
    DesktopChromium, DesktopFirefox,
};

static ACTION_ID: AtomicUsize = AtomicUsize::new(1);

async fn setup_session<B: BrowserBuilder>() -> Box<dyn Browser> {
    let browser = B::start(Path::new(&std::env::var("EXTENSION_FILE").unwrap())).await;

    // chromedriver --port=4444 --enable-chrome-logs

    Box::new(browser)
}

async fn navigate(
    session: &mut WebDriverBiDiSession,
    ctx: BrowsingContext,
    url: String,
) -> anyhow::Result<()> {
    let navigate_params = NavigateParameters::new(ctx, url, Some(ReadinessState::Complete));
    session.browsing_context_navigate(navigate_params).await?;
    println!("navigated");
    Ok(())
}

fn generate_keypresses(input: &str) -> Vec<KeySourceAction> {
    input
        .chars()
        .flat_map(|c| {
            [
                KeySourceAction::KeyDownAction(KeyDownAction::new(c.to_string())),
                KeySourceAction::KeyUpAction(KeyUpAction::new(c.to_string())),
            ]
        })
        .collect()
}

async fn click_element(
    session: &mut WebDriverBiDiSession,
    browsing_context: String,
    node: &NodeRemoteValue,
) -> anyhow::Result<()> {
    let a: Box<[PointerSourceAction]> = Box::new([
        PointerSourceAction::PointerMoveAction(PointerMoveAction::new(
            5.0,
            5.0,
            None,
            Some(Origin::ElementOrigin(ElementOrigin::new(SharedReference {
                shared_id: node.shared_id.clone().unwrap(),
                handle: node.handle.clone(),
                extensible: Extensible::new(),
            }))),
            PointerCommonProperties::new(None, None, None, None, None, None, None),
        )),
        PointerSourceAction::PointerDownAction(PointerDownAction::new(
            0,
            PointerCommonProperties::new(None, None, None, None, None, None, None),
        )),
        PointerSourceAction::PointerUpAction(PointerUpAction::new(0)),
    ]);
    let a = a.into_vec();

    let id = ACTION_ID.fetch_add(1, Ordering::Relaxed);
    let b: Box<[SourceActions]> = Box::new([SourceActions::PointerSourceActions(
        PointerSourceActions::new(
            id.to_string(),
            Some(PointerParameters::new(Some(PointerType::Mouse))),
            a,
        ),
    )]);
    let b = b.into_vec();

    session
        .input_perform_actions(PerformActionsParameters::new(browsing_context.clone(), b))
        .await?;
    Ok(())
}

async fn write_text(
    session: &mut WebDriverBiDiSession,
    browsing_context: String,
    element: &str,
    input: &str,
) -> anyhow::Result<()> {
    println!("locating");

    let node = session
        .browsing_context_locate_nodes(LocateNodesParameters::new(
            browsing_context.clone(),
            Locator::CssLocator(CssLocator::new(element.to_owned())),
            None,
            None,
            None,
        ))
        .await?;
    let node = &node.nodes[0];

    println!("located");

    let result = session
        .script_call_function(CallFunctionParameters::new(
            r#"function abc(node) {
                        console.log("abc", node, node.getBoundingClientRect());
                        return JSON.parse(JSON.stringify(node.getBoundingClientRect()));
                    }
                    "#
            .to_owned(),
            false,
            Target::ContextTarget(ContextTarget::new(browsing_context.clone(), None)),
            Some(vec![LocalValue::RemoteReference(
                RemoteReference::SharedReference(SharedReference {
                    handle: node.handle.clone(),
                    shared_id: node.shared_id.clone().unwrap(),
                    extensible: HashMap::default(),
                }),
            )]),
            Some(ResultOwnership::Root),
            Some(SerializationOptions {
                max_dom_depth: Some(10),
                max_object_depth: Some(100),
                include_shadow_tree: Some(IncludeShadowTree::All),
            }),
            None,
            Some(true),
        ))
        .await?;

    // TODO FIXME webdriver bidi library fails to deserialize object
    println!("function evaluation {result:?}");

    click_element(session, browsing_context.clone(), node).await?;

    let id = ACTION_ID.fetch_add(1, Ordering::Relaxed);
    let e: Box<[SourceActions]> = Box::new([SourceActions::KeySourceActions(
        KeySourceActions::new(id.to_string(), generate_keypresses(input)),
    )]);
    let e = e.into_vec();

    session
        .input_perform_actions(PerformActionsParameters::new(browsing_context.clone(), e))
        .await?;

    Ok(())
}

pub async fn it_works<B: BrowserBuilder>() {
    let _ = env_logger::try_init();
    dotenv().unwrap();

    let mut session = setup_session::<B>().await;

    let username = std::env::var("TUCAN_USERNAME").unwrap();
    let password = std::env::var("TUCAN_PASSWORD").unwrap();

    // for e.g. android this makes the most sense?
    /*session
    .web_extension_install(InstallParameters::new(ExtensionData::ExtensionBase64Encoded(ExtensionBase64Encoded::new(extension_base64))))
    .await.unwrap();*/
    // chrome only supports unpacked path
    session
        .load_extension(Path::new(&std::env::var("EXTENSION_FILE").unwrap()))
        .await;

    println!("get contexts");

    let contexts = session
        .browsing_context_get_tree(GetTreeParameters {
            max_depth: None,
            root: None,
        })
        .await
        .unwrap();

    println!("got contexts");

    let browsing_context = contexts.contexts[0].context.clone().clone();

    println!("1");
    /*
    session
        .register_event_handler(EventType::LogEntryAdded, async |event| {
            println!(
                "log entry {}",
                event
                    .as_object()
                    .unwrap()
                    .get_key_value("params")
                    .unwrap()
                    .1
                    .as_object()
                    .unwrap()
                    .get_key_value("args")
                    .unwrap()
                    .1
            );
        })
        .await;*/

    println!("2");

    session
        .register_event_handler(EventType::BrowsingContextUserPromptOpened, async |event| {
            println!("user prompt {event}");
        })
        .await;

    println!("3");

    session
        .session_subscribe(SubscriptionRequest::new(
            vec!["log.entryAdded".to_owned()],
            Some(vec![browsing_context.clone()]),
            None,
        ))
        .await
        .unwrap();

    println!("4");

    session
        .session_subscribe(SubscriptionRequest::new(
            vec!["browsingContext.userPromptOpened".to_owned()],
            Some(vec![browsing_context.clone()]),
            None,
        ))
        .await
        .unwrap();

    println!("5");

    // not supported on firefox android obviously, ahh this may have made edge weird
    /*session
    .browsing_context_set_viewport(SetViewportParameters {
        user_contexts: None,
        context: Some(browsing_context.clone()),
        viewport: Some(Viewport { width: 1300, height: 768 }),
        device_pixel_ratio: None,
    })
    .await.unwrap();*/

    println!("abc");

    let start = Instant::now();
    navigate(
        &mut session,
        browsing_context.clone(),
        "https://www.tucan.tu-darmstadt.de/".to_owned(),
    )
    .await
    .unwrap();

    // we should do this better?
    sleep(Duration::from_secs(5)).await; // wait for frontend javascript to be executed

    session
        .script_evaluate(EvaluateParameters::new(
            r##"
                    if (window.getComputedStyle(document.querySelector(".navbar-toggler")).display !== "none") {
                        document.querySelector(".navbar-toggler").click()
                    }
                    "##
            .to_owned(),
            Target::ContextTarget(ContextTarget::new(browsing_context.clone(), None)),
            true,
            None,
            None,
            Some(true),
        ))
        .await
        .unwrap();

    sleep(Duration::from_secs(1)).await;

    println!("waited");

    write_text(
        &mut session,
        browsing_context.clone(),
        "#login-username",
        &username,
    )
    .await
    .unwrap();

    // TODO get the area of the login field so we can visualize it

    println!("input_login_username {:?}", start.elapsed());
    write_text(
        &mut session,
        browsing_context.clone(),
        "#login-password",
        &password,
    )
    .await
    .unwrap();

    let node = session
        .browsing_context_locate_nodes(LocateNodesParameters::new(
            browsing_context.clone(),
            Locator::CssLocator(CssLocator::new("#login-button".to_owned())),
            None,
            None,
            None,
        ))
        .await
        .unwrap();
    let node = &node.nodes[0];
    click_element(&mut session, browsing_context.clone(), node)
        .await
        .unwrap();

    // time not implemented on this platform

    session
        .script_evaluate(EvaluateParameters::new(
            r##"
                    new Promise((resolve) => {
                        const observer = new MutationObserver((mutations, observer) => {
                            const element = document.querySelector("#logout-button");
                            if (element) {
                                observer.disconnect();
                                resolve(element);
                            }
                        });

                        observer.observe(document.body, {
                            childList: true,
                            subtree: true,
                        });
                    })
                    "##
            .to_owned(),
            Target::ContextTarget(ContextTarget::new(browsing_context.clone(), None)),
            true,
            None,
            None,
            Some(true),
        ))
        .await
        .unwrap();

    let realms = session
        .script_get_realms(GetRealmsParameters::new(
            Some(browsing_context.clone()),
            None,
        ))
        .await
        .unwrap();

    let RealmInfo::WindowRealmInfo(_window) = &realms.realms[0] else {
        panic!();
    };

    session
        .script_evaluate(EvaluateParameters::new(
            r#"chrome.runtime.sendMessage("open-in-tucan-page")"#.to_owned(),
            Target::ContextTarget(ContextTarget::new(browsing_context.clone(), None)),
            false,
            None,
            None,
            Some(true),
        ))
        .await
        .unwrap();

    sleep(Duration::from_secs(5)).await;

    let realms = session
        .script_get_realms(GetRealmsParameters::new(
            Some(browsing_context.clone()),
            None,
        ))
        .await
        .unwrap();

    let contexts = session
        .browsing_context_get_tree(GetTreeParameters {
            max_depth: None,
            root: Some(browsing_context.clone()),
        })
        .await
        .unwrap();

    session
                .script_evaluate(EvaluateParameters::new(
                    r#"window.dispatchEvent(new CustomEvent('tucan-plus', { detail: "open-in-tucan-page" }));"#.to_owned(),
                    Target::ContextTarget(ContextTarget::new(browsing_context.clone(), None)),
                    false,
                    None,
                    None,
                    Some(true),
                ))
                .await.unwrap();

    sleep(Duration::from_secs(5)).await;

    session
        .browsing_context_close(CloseParameters {
            context: browsing_context,
            prompt_unload: None,
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn desktop_firefox_main() {
    it_works::<DesktopFirefox>().await
}

#[tokio::test]
async fn desktop_chromium_main() {
    it_works::<DesktopChromium>().await
}
/*
#[tokio::test]
async fn android_edge_main() {
    let guard = ANDROID_MUTEX.lock().await;
    it_works::<AndroidEdgeCanary>().await
}
*/
#[tokio::test]
async fn android_chromium_main() {
    let guard = ANDROID_MUTEX.lock().await;
    it_works::<AndroidChromium>().await
}

#[tokio::test]
async fn android_firefox_main() {
    let guard = ANDROID_MUTEX.lock().await;
    it_works::<AndroidFirefox>().await
}
