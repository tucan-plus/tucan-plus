# TUCaN Plus

![TUCaN Plus logo](./crates/tucan-plus-dioxus/assets/logo.svg)

## Features

### Semesterplanung

![Semesterplanung](./tucan-plus-extension/semesterplanung.png)

### Custom UI

![Leistungsspiegel](./tucan-plus-extension/leistungsspiegel.png)

### Shareable URLs

TUCaN URLs are automatically fixed so you can open URLs shared by others.

## Installation

Go to https://tucan-plus.github.io/tucan-plus/ and follow the instructions.

## Development

Requirements: [Rustup](https://www.rust-lang.org/tools/install)

Recommended: VSCodium with rust-analyzer extension

Just open the project in VSCodium.

### Running all tests

```
cargo test
```

### Running UI tests

**Important: Do NOT run in `nix develop` environment**

```
nix run .#test-dev
```

### Formatting

We use a fork of rustfmt to format our custom html extractor macro.
```
# install minimal profile
rustup toolchain install nightly-2025-09-08 --component rust-src --component rustc-dev --component llvm-tools-preview
rustup component remove --toolchain nightly-2025-09-08 rustfmt
cargo +nightly-2025-09-08 install --force --git https://github.com/tucan-plus/rustfmt --branch html-extractor-formatting rustfmt-nightly
cargo +nightly-2025-09-08 fmt

rustup toolchain install nightly-2025-09-08 --component rustfmt
```

### Running as local webserver

```bash
cargo install --git https://github.com/mohe2015/dioxus --branch my dioxus-cli

cd crates/tucan-plus-dioxus/
export WORKER_JS_PATH=/assets/wasm/tucan-plus-worker.js
export WORKER_WASM_PATH=/assets/wasm/tucan-plus-worker_bg.wasm
export SERVICE_WORKER_JS_PATH=/assets/wasm/tucan-plus-service-worker.js
export SERVICE_WORKER_WASM_PATH=/assets/wasm/tucan-plus-service-worker_bg.wasm
dx serve --platform web --features api --verbose

cargo run --manifest-path ~/Documents/dioxus/packages/cli/Cargo.toml serve --platform web --features api --verbose

cargo install wasm-bindgen-cli@0.2.101

cd crates/tucan-plus-worker/
dx serve --wasm --bundle web --base-path assets # --hot-patch this lets everything explode with "env" imports and sqlite import stuff broken
cp -r ./target/dx/tucan-plus-worker/debug/web/public/wasm/. ../tucan-plus-dioxus/assets/wasm/

# in second tab
cargo install --locked bacon
cd crates/tucan-plus-api/
bacon run

cargo install diesel_cli --no-default-features --features sqlite
DATABASE_URL=sqlite://$(mktemp) diesel database reset

# Service Workers in Firefox can't be ES Modules https://bugzilla.mozilla.org/show_bug.cgi?id=1360870
# Event handlers must be registered synchronously
cd crates/tucan-plus-service-worker/
cargo build --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/tucan-plus-service-worker.wasm --target no-modules --out-dir ./target/dx/tucan-plus-service-worker/debug/web/public/wasm/ --no-typescript
echo "wasm_bindgen.initSync({ module: Uint8Array.fromBase64(\"$(base64 -w0 target/dx/tucan-plus-service-worker/debug/web/public/wasm/tucan-plus-service-worker_bg.wasm)\")})" >> ./target/dx/tucan-plus-service-worker/debug/web/public/wasm/tucan-plus-service-worker.js
cp -r ./target/dx/tucan-plus-service-worker/debug/web/public/wasm/. ../tucan-plus-dioxus/assets/wasm/

# http://localhost:8080/#/
```

### Developing the extension

```
cd crates/tucan-plus-dioxus/
dx bundle --platform web --out-dir ../../tucan-plus-extension/ --base-path public --features direct --release
```

Go to Firefox Extensions, click settings, debug addons. Then click load temporary add-on and select ./tucan-plus-extension/manifest.json
See https://extensionworkshop.com/documentation/develop/debugging/.

### VSCode

https://github.com/rust-lang/cargo/issues/16038
https://github.com/rust-lang/cargo/issues/15177
https://github.com/rust-lang/cargo/issues/1197

Does not work but one by one works:
```
cargo check --target wasm32-unknown-unknown --target x86_64-unknown-linux-gnu
```

```
tucan-plus-multi-target
    tucan-plus
    wasm32-unknown-unknown
        .vscode/settings.json
        .cargo/config.toml
        Cargo.toml # workspace with subset of crates
        -> tucan-plus
    x86_64-unknown-linux-gnu
        .vscode/settings.json
        .cargo/config.toml
        Cargo.toml # workspace with subset of crates
        -> tucan-plus
```

```toml
[workspace]
members = [
    "./tucan-plus/crates/tucan-plus-dioxus/",
    "./tucan-plus/crates/tucan-connector/",
    "./tucan-plus/crates/tucan-plus-worker/",
    "./tucan-plus/crates/tucan-plus-service-worker/"
]
resolver = "3"
```

```toml
[target.wasm32-unknown-unknown]
rustflags = ["--cfg=web_sys_unstable_apis", "--cfg=getrandom_backend=\"wasm_js\""]
```

```json
{
    "rust-analyzer.cargo.target": "wasm32-unknown-unknown",
    "rust-analyzer.files.watcher": "server",
}
```

## Building extension (not for development)

```bash
podman build --output . .
```
This will produce a tucan-plus-extension.zip in the current working directory.

## Packaging

### Chromium

https://developer.chrome.com/docs/extensions/how-to/distribute/host-on-linux#packaging

Open Chromium -> chrome://extensions/ -> Pack extension -> Choose folder -> Pack. Store private key in a secure place

```bash
chromium --pack-extension=tucan-plus-extension --pack-extension-key=/path/to/tucan-plus-extension.pem
```

### Firefox

https://extensionworkshop.com/documentation/publish/distribute-sideloading/

ZIP the extension files.

Upload to AMO as unlisted extension and pray that it gets signed quickly.

## How does it work

This software consists of the tucan-connector component that extracts information from the html of [TUCaN](https://www.tucan.tu-darmstadt.de) and provides it as a nicer to use programming API. The tucan-injector component can then be used to show that data with a nicer UI that is written using the Rust frontend library [Dioxus](https://dioxus.dev/) and that is compiled to [WebAssembly](https://webassembly.org/). This WebAssembly can be injected into the actual TUCaN website using an extension. Then, some pages provide an overlay with the information in a nicer format and caching.

## API

```
bacon tucan-plus-api
```
http://localhost:3000/swagger-ui/

http://localhost:3000/api-docs/openapi.json

```
cargo run --bin schema > schema.json
```
https://editor-next.swagger.io/

## tucan-connector

.env
```
TUCAN_USERNAME=
TUCAN_PASSWORD=
SESSION_ID=
SESSION_KEY=
```

## Debugging

https://chromewebstore.google.com/detail/cc++-devtools-support-dwa/pdcpmagijalfljmkmjngeonclgbbannb

## Coverage

```
# https://doc.rust-lang.org/rustc/instrument-coverage.html#test-coverage
cd tucan-connector
RUSTFLAGS="-C instrument-coverage" cargo test
nix shell nixpkgs#llvmPackages_19.bintools-unwrapped
llvm-profdata merge *.profraw -o default.profdata

llvm-cov show -Xdemangler=/home/moritz/.cargo/bin/rustfilt /home/moritz/Documents/tucan-plus/target/debug/deps/tucan_connector-90eac6df256ec2c3 \
    -format=html \
    -output-dir=target/coverage \
    -instr-profile=default.profdata \
    -show-line-counts-or-regions \
    -show-instantiations

xdg-open target/coverage/index.html 
```

## Android

```
sudo systemctl stop firewalld.service
adb connect 172.18.61.176:43109
adb uninstall com.example.TucanPlusDioxus
dx serve --device --platform android --hotpatch --verbose

adb logcat -c
adb shell run-as com.example.TucanPlusDioxus logcat

dx bundle --platform android --device
/home/moritz/Documents/tucan-plus/target/dx/tucan-plus-dioxus/debug/android/app/app/build/outputs/apk/debug/app-debug.apk

dx serve --platform android --hotpatch --verbose
cargo run --manifest-path /home/moritz/Documents/dioxus/packages/cli/Cargo.toml serve --platform android --verbose
# grep for RustStdoutStderr

dx bundle --platform android --release
adb install target/dx/tucan-plus-dioxus/release/android/app/app/build/outputs/apk/release/app-release.apk


cargo run --manifest-path /home/moritz/Documents/dioxus/packages/cli/Cargo.toml build --platform android
```

## Linux

```
dx serve --platform linux --hotpatch --verbose

```

```

cat *_registration-N383703888296780\,-N0\,-N0\,-N0_B.Sc.\ Informatik\ \(2015\).json | jq 'sort_by(.path) | del(.[].studiumsauswahl) | del(.[].entries.[].module.registration_state) | del(.[].entries.[].courses.[].[1].registration_button_link) | del(.[].entries.[].courses.[].[1].location_or_additional_info) | del(.[].entries.[].courses.[].[1].limit_and_size)' > a
cat *_registration-N376333755785484\,-N0\,-N0\,-N0_B.Sc.\ Informatik\ \(2015\).json | jq 'sort_by(.path) | del(.[].studiumsauswahl) | del(.[].entries.[].module.registration_state) | del(.[].entries.[].courses.[].[1].registration_button_link) | del(.[].entries.[].courses.[].[1].location_or_additional_info) | del(.[].entries.[].courses.[].[1].limit_and_size)' | sed 's/N376333755785484/N383703888296780/g' > b
meld a b

oh no there are different numbers for the same PO (probably for each semester?) maybe because depending on your starting semester the modules change?

seems like if you already completed a module it will not show up in registration at the other possible paths any more
also it seems like it won't show new courses etc? probably as you are only allowed to do it once?

seems like your wahlbereiche will be reduced to 3 if you complete your bachelor


inkscape -w 512 ../crates/tucan-plus-dioxus/assets/logo.svg -o logo.png
```