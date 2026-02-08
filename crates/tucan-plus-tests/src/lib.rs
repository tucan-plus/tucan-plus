pub mod browsers;

use std::{
    collections::HashMap,
    path::Path,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use dotenvy::dotenv;
use secret_service::{EncryptionType, Item, SecretService};
use tokio::{
    sync::{Notify, OnceCell},
    time::sleep,
};
use webdriverbidi::{
    events::EventType,
    model::{
        browsing_context::{
            BrowsingContext, CloseParameters, CssLocator, GetTreeParameters, LocateNodesParameters,
            Locator, NavigateParameters, ReadinessState,
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
    },
    session::WebDriverBiDiSession,
};
use zbus::zvariant::OwnedObjectPath;

use crate::browsers::{
    ANDROID_MUTEX, AndroidChromium, AndroidFirefox, Browser, BrowserBuilder, DesktopChromium,
    DesktopFirefox,
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
    nodes: &[NodeRemoteValue],
) -> anyhow::Result<()> {
    let a: Vec<PointerSourceAction> = nodes
        .into_iter()
        .flat_map(|node| {
            [
                PointerSourceAction::PointerMoveAction(PointerMoveAction::new(
                    0.0,
                    0.0,
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
            ]
        })
        .collect();

    let id = ACTION_ID.fetch_add(1, Ordering::Relaxed);
    let b: Vec<SourceActions> = Vec::from_iter(
        [SourceActions::PointerSourceActions(
            PointerSourceActions::new(
                id.to_string(),
                Some(PointerParameters::new(Some(PointerType::Mouse))),
                a,
            ),
        )]
        .into_iter(),
    );

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

    let mut node = session
        .browsing_context_locate_nodes(LocateNodesParameters::new(
            browsing_context.clone(),
            Locator::CssLocator(CssLocator::new(element.to_owned())),
            None,
            None,
            None,
        ))
        .await?;
    let node = node.nodes.remove(0);

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

    click_element(session, browsing_context.clone(), &[node]).await?;

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

    let secret_service = get_secret_service().await;
    // TODO do this once

    let item = secret_service
        .get_item_by_path(
            OwnedObjectPath::try_from(
                "/org/freedesktop/secrets/collection/Passwords/620b653db40547b7902b498512bfea30",
            )
            .unwrap(),
        )
        .await
        .unwrap();
    let attributes = item.get_attributes().await.unwrap();
    let username = attributes["UserName"].clone();
    let totp = attributes["TOTP"].clone();
    let password = item.get_secret().await.unwrap();
    let password = str::from_utf8(&password).unwrap();

    let mut session = setup_session::<B>().await;

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

    // we should do this better?, wait for what we need. domcontentloaded or so?
    //sleep(Duration::from_secs(2)).await; // wait for frontend javascript to be executed

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

    // wait for animation would be nice
    sleep(Duration::from_secs(1)).await;

    println!("waited");

    let navigated = Arc::new(Notify::const_new());
    session
        .register_event_handler(EventType::BrowsingContextDomContentLoaded, {
            let navigated = navigated.clone();
            move |event| {
                let navigated = navigated.clone();
                async move {
                    println!("domcontentloaded {event}");
                    navigated.notify_one();
                }
            }
        })
        .await;

    session
        .session_subscribe(SubscriptionRequest::new(
            vec!["browsingContext.domContentLoaded".to_owned()],
            Some(vec![browsing_context.clone()]),
            None,
        ))
        .await
        .unwrap();

    let mut node = session
        .browsing_context_locate_nodes(LocateNodesParameters::new(
            browsing_context.clone(),
            Locator::CssLocator(CssLocator::new("#login-button".to_owned())),
            None,
            None,
            None,
        ))
        .await
        .unwrap();
    let node = node.nodes.remove(0);
    click_element(&mut session, browsing_context.clone(), &[node])
        .await
        .unwrap();

    // well SSO

    // #username
    // #password
    // button[type=submit]

    println!("waiting for page load");
    navigated.notified().await;
    session
        .unregister_event_handler(EventType::BrowsingContextDomContentLoaded)
        .await;

    write_text(
        &mut session,
        browsing_context.clone(),
        "#username",
        &username,
    )
    .await
    .unwrap();

    println!("input_login_username {:?}", start.elapsed());
    write_text(
        &mut session,
        browsing_context.clone(),
        "#password",
        &password,
    )
    .await
    .unwrap();

    let navigated = Arc::new(Notify::const_new());
    session
        .register_event_handler(EventType::BrowsingContextDomContentLoaded, {
            let navigated = navigated.clone();
            move |event| {
                let navigated = navigated.clone();
                async move {
                    println!("domcontentloaded {event}");
                    navigated.notify_one();
                }
            }
        })
        .await;
    session
        .session_subscribe(SubscriptionRequest::new(
            vec!["browsingContext.domContentLoaded".to_owned()],
            Some(vec![browsing_context.clone()]),
            None,
        ))
        .await
        .unwrap();

    let mut node = session
        .browsing_context_locate_nodes(LocateNodesParameters::new(
            browsing_context.clone(),
            Locator::CssLocator(CssLocator::new("button[name=_eventId_proceed]".to_owned())),
            None,
            None,
            None,
        ))
        .await
        .unwrap();
    let node = node.nodes.remove(0);
    click_element(&mut session, browsing_context.clone(), &[node])
        .await
        .unwrap();

    navigated.notified().await;
    session
        .unregister_event_handler(EventType::BrowsingContextDomContentLoaded)
        .await;

    // select[id=fudis_selected_token_ids_input]
    let mut node = session
        .browsing_context_locate_nodes(LocateNodesParameters::new(
            browsing_context.clone(),
            Locator::CssLocator(CssLocator::new(
                "#fudis_selected_token_ids_input".to_owned(),
            )),
            None,
            None,
            None,
        ))
        .await
        .unwrap();
    let node1 = node.nodes.remove(0);

    // https://github.com/puppeteer/puppeteer/blob/b163ce4593a8f014b86d67d53825fbeb679045ca/packages/puppeteer-core/src/api/ElementHandle.ts#L1008
    session
        .script_evaluate(EvaluateParameters::new(
            r##"
                    (() => {
                        const selectElement = document.querySelector('#fudis_selected_token_ids_input');
                        if (selectElement) {
                            selectElement.value = 'TOTP33027D68';
                            
                            // Manually trigger events so the site reacts to the change
                            selectElement.dispatchEvent(new Event('input', { bubbles: true }));
                            selectElement.dispatchEvent(new Event('change', { bubbles: true }));
                            return true;
                        }
                        return false;
                    })();
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

    // time not implemented on this platform

    sleep(Duration::from_secs(100)).await;

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

    let _realms = session
        .script_get_realms(GetRealmsParameters::new(
            Some(browsing_context.clone()),
            None,
        ))
        .await
        .unwrap();

    let _contexts = session
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

static ONCE_SECRET_SERVICE: OnceCell<SecretService> = OnceCell::const_new();

pub async fn get_secret_service() -> &'static SecretService<'static> {
    ONCE_SECRET_SERVICE
        .get_or_init(async || {
            let ss = SecretService::connect(EncryptionType::Dh).await.unwrap();
            let result = ss.search_items(HashMap::default()).await.unwrap();
            println!("entries");
            for item in result.unlocked {
                println!("{:?} {:?}", item.item_path, item.get_label().await);
            }
            for item in result.locked {
                println!("{:?} {:?}", item.item_path, item.get_label().await);
            }
            let item = ss
        .get_item_by_path(
            OwnedObjectPath::try_from(
                "/org/freedesktop/secrets/collection/Passwords/620b653db40547b7902b498512bfea30",
            )
            .unwrap(),
        )
        .await
        .unwrap();
            if item.is_locked().await.unwrap() {
                println!("unlock result {:?}", item.unlock().await);
            }
            ss
        })
        .await
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
    let guard = ANDROID_MUTEX.lock().await; // panicking poisons this
    it_works::<AndroidFirefox>().await
}
