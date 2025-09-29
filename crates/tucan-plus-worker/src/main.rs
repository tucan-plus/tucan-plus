use std::{cell::RefCell, time::Duration};

use diesel::{Connection as _, SqliteConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness as _, embed_migrations};
use log::info;
use tucan_plus_worker::{MIGRATIONS, MessageWithId, RequestResponseEnum};
use wasm_bindgen::prelude::*;
use web_sys::{BroadcastChannel, MessageEvent};

pub async fn sleep(duration: Duration) {
    let mut cb = |resolve: js_sys::Function, _reject: js_sys::Function| {
        let global = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();
        global
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                duration.as_millis().try_into().unwrap(),
            )
            .unwrap();
    };

    let p = js_sys::Promise::new(&mut cb);

    wasm_bindgen_futures::JsFuture::from(p).await.unwrap();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(msg: String);

    type Error;

    #[wasm_bindgen(constructor)]
    fn new() -> Error;

    #[wasm_bindgen(structural, method, getter)]
    fn stack(error: &Error) -> String;
}

#[wasm_bindgen(main)]
pub async fn main() {
    // From https://github.com/rustwasm/console_error_panic_hook, licensed under MIT and Apache 2.0
    std::panic::set_hook(Box::new(|info| {
        let mut msg = "Worker ".to_string();
        msg.push('\n');
        msg.push_str(&info.to_string());
        msg.push_str("\n\nStack:\n\n");
        let e = Error::new();
        let stack = e.stack();
        msg.push_str(&stack);
        msg.push_str("\n\n");
        error(msg.clone());
    }));
    console_log::init().unwrap();

    let global = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();

    let util = sqlite_wasm_rs::sahpool_vfs::install(
        &sqlite_wasm_rs::sahpool_vfs::OpfsSAHPoolCfg::default(),
        true,
    )
    .await
    .unwrap();

    let mut connection = SqliteConnection::establish("sqlite://tucan-plus.db?mode=rwc").unwrap();

    connection.run_pending_migrations(MIGRATIONS).unwrap();

    let connection = RefCell::new(connection);

    let broadcast_channel = BroadcastChannel::new("global").unwrap();

    let closure: Closure<dyn Fn(MessageEvent)> = Closure::new(move |event: MessageEvent| {
        //info!("Got message at worker {:?}", event.data());
        let global = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();

        let value: MessageWithId = serde_wasm_bindgen::from_value(event.data()).unwrap();
        let result = value.message.execute(&mut connection.borrow_mut());
        //info!("Got result at worker {:?}", result);

        let temporary_broadcast_channel = BroadcastChannel::new(&value.id).unwrap();

        temporary_broadcast_channel.post_message(&result).unwrap();
    });
    broadcast_channel
        .add_event_listener_with_callback("message", closure.as_ref().unchecked_ref())
        .unwrap();

    //util.export_db("tucan-plus.db").unwrap();
    closure.forget();

    global.post_message(&JsValue::from_str("ready")).unwrap();
}
