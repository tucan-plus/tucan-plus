#[cfg(not(target_arch = "wasm32"))]
use tucan_plus_api::router;

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    println!("{}", router().to_openapi().to_pretty_json().unwrap());
}

#[cfg(target_arch = "wasm32")]
pub fn main() {}