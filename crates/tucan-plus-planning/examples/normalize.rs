#[cfg(not(target_arch = "wasm32"))]
use futures_util::StreamExt as _;
#[cfg(not(target_arch = "wasm32"))]
use futures_util::stream::{FuturesOrdered, FuturesUnordered};
#[cfg(not(target_arch = "wasm32"))]
use tucan_connector::TucanConnector;
#[cfg(not(target_arch = "wasm32"))]
use tucan_plus_planning::{compress, recursive_anmeldung};
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::TucanError;
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::registration::{AnmeldungRequest, AnmeldungResponse};
#[cfg(not(target_arch = "wasm32"))]
use tucan_types::{DynTucan, LoginRequest, RevalidationStrategy, Tucan};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), TucanError> {
    dotenvy::dotenv().unwrap();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}

#[cfg(not(target_arch = "wasm32"))]
async fn async_main() -> Result<(), TucanError> {
    let args: Vec<String> = std::env::args().collect();
    let results: FuturesOrdered<_> = args
        .iter()
        .skip(1)
        .map(async |arg| {
            serde_json::from_str(&tokio::fs::read_to_string(arg).await.unwrap()).unwrap()
        })
        .collect();
    let results = results.collect::<Vec<Vec<AnmeldungResponse>>>().await;
    println!("{:?}", results);

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn main() {}