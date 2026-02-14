{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
          config.android_sdk.accept_license = true;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        rustToolchainFor =
          p:
          p.rust-bin.stable.latest.minimal.override {
            targets = [
              "wasm32-unknown-unknown"
            ];
            extensions = [ "rustfmt" ];
          };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainFor;

        cargoDioxus =
          craneLib:
          {
            profile ? "--release",
            dioxusCommand ? "bundle",
            dioxusExtraArgs ? "",
            dioxusMainArgs ? "",
            cargoExtraArgs ? "",
            notBuildDepsOnly ? { },
            buildDepsOnly ? { },
            dioxusBuildDepsOnlyCommand ? "build",
            ...
          }@origArgs:
          let
            args = {
              pnameSuffix = "-dioxus";
            }
            // (builtins.removeAttrs origArgs [
              "dioxusCommand"
              "dioxusExtraArgs"
              "dioxusMainArgs"
              "cargoExtraArgs"
              "notBuildDepsOnly"
              "buildDepsOnly"
              "dioxusBuildDepsOnlyCommand"
            ]);
          in
          craneLib.mkCargoDerivation (
            {
              buildPhaseCargoCommand = ''
                DX_HOME=$(mktemp -d) DIOXUS_LOG=trace,walrus=debug ${pkgs.dioxus-cli}/bin/dx ${dioxusCommand} --trace ${profile} --base-path public ${dioxusExtraArgs} ${dioxusMainArgs} ${cargoExtraArgs}
              '';
              cargoArtifacts = craneLib.buildDepsOnly (
                {
                  # build, don't bundle
                  # TODO make dx home persistent as it's useful
                  buildPhaseCargoCommand = ''
                    DX_HOME=$(mktemp -d) DIOXUS_LOG=trace,walrus=debug ${pkgs.dioxus-cli}/bin/dx ${dioxusBuildDepsOnlyCommand} --trace ${profile} --base-path public ${dioxusExtraArgs} ${cargoExtraArgs}
                  '';
                  doCheck = false;
                  dummySrc = craneLib.mkDummySrc {
                    src = args.src;
                    extraDummyScript = ''
                      cp ${./crates/tucan-plus-dioxus/Dioxus.toml} $out/crates/tucan-plus-dioxus/Dioxus.toml
                    '';
                  };
                }
                // args
                // buildDepsOnly
              );
            }
            // args
            // notBuildDepsOnly
          );

        fileset-worker = lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ./crates/tucan-plus-worker)
          (craneLib.fileset.commonCargoSources ./crates/tucan-types)
          ./crates/tucan-plus-worker/migrations
          ./.cargo/config.toml
          ./Cargo.toml
          ./Cargo.lock
        ];

        fileset-dioxus = lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ./crates/tucan-plus-dioxus)
          ./crates/tucan-plus-dioxus/assets/logo.svg
          ./crates/tucan-plus-dioxus/assets/logo.png
          ./crates/tucan-plus-dioxus/assets/manifest.json
          ./crates/tucan-plus-dioxus/assets/bootstrap.css
          ./crates/tucan-plus-dioxus/assets/bootstrap.bundle.min.js
          ./crates/tucan-plus-dioxus/assets/bootstrap.patch.js
          ./crates/tucan-plus-dioxus/Dioxus.toml
          ./crates/tucan-plus-dioxus/.cargo/config.toml
        ];

        fileset-wasm = lib.fileset.unions [
          ./Cargo.toml
          ./Cargo.lock
          (craneLib.fileset.commonCargoSources ./crates/html-extractor)
          (craneLib.fileset.commonCargoSources ./crates/tucan-connector)
          (craneLib.fileset.commonCargoSources ./crates/html-handler)
          fileset-dioxus
          fileset-worker
        ];

        fileset-extension = lib.fileset.unions [
          ./tucan-plus-extension/background.js
          ./tucan-plus-extension/fix-session-id-in-url.js
          ./tucan-plus-extension/context-menu.js
          ./tucan-plus-extension/content-script.js
          ./tucan-plus-extension/content-script-redirect.js
          ./tucan-plus-extension/open-in-tucan.js
          ./tucan-plus-extension/bootstrap.bundle.min.js
          ./tucan-plus-extension/bootstrap.css
          ./tucan-plus-extension/manifest.json
          ./tucan-plus-extension/mobile.css
          ./tucan-plus-extension/mobile.js
          ./tucan-plus-extension/options.html
          ./tucan-plus-extension/options.js
          ./tucan-plus-extension/popup.html
          ./tucan-plus-extension/popup.js
          ./tucan-plus-extension/custom-ui.js
          ./tucan-plus-extension/recover-tabs.js
          ./tucan-plus-extension/url-mappings.js
          ./tucan-plus-extension/utils.js
          ./tucan-plus-extension/rules.json
          ./tucan-plus-extension/logo.png
        ];

        fileset-api = lib.fileset.unions [
          ./Cargo.toml
          ./Cargo.lock
          (craneLib.fileset.commonCargoSources ./crates/html-handler)
          (craneLib.fileset.commonCargoSources ./crates/html-extractor)
          (craneLib.fileset.commonCargoSources ./crates/tucan-connector)
          (craneLib.fileset.commonCargoSources ./crates/tucan-plus-api)
          fileset-worker
        ];

        fileset-tests = lib.fileset.unions [
          ./Cargo.toml
          ./Cargo.lock
          (craneLib.fileset.commonCargoSources ./crates/tucan-plus-tests)
        ];

        api-server = craneLib.buildPackage {
          strictDeps = true;
          buildInputs = [
            pkgs.sqlite
          ];
          pname = "tucan-plus-workspace-native-api";
          src = lib.fileset.toSource {
            root = ./.;
            fileset = fileset-api;
          };
          cargoTestExtraArgs = "--no-run";
          cargoExtraArgs = "--package=tucan-plus-api";
        };

        schema =
          pkgs.runCommandNoCC "schema.json"
            {
            }
            ''
              ${api-server}/bin/schema > $out
            '';

        client-args = rec {
          dioxusExtraArgs = "--features direct --web";
          CARGO_PROFILE_WASM_RELEASE_DEBUG = "false"; # for non-wasm-split
          dioxusMainArgs = "--out-dir $out"; # --wasm-split --features wasm-split
          buildDepsOnly = {
            preBuild = ''
            '';
            dummySrc = craneLib.mkDummySrc {
              src = client-args.src;
              extraDummyScript = ''
                rm $out/crates/tucan-plus-dioxus/src/main.rs
                cp ${pkgs.writeText "main.rs" ''
                  use wasm_bindgen::prelude::*;

                  #[wasm_bindgen(main)]
                  pub async fn main() {

                  }
                ''} $out/crates/tucan-plus-dioxus/src/main.rs
              '';
            };
          };
          notBuildDepsOnly = {
            preBuild = ''
              rm -R ./target/dx/tucan-plus-dioxus/release/web/public/ || true
            '';
            # temporary https://github.com/DioxusLabs/dioxus/issues/4758
            postBuild = ''
              rm $out/public/wasm/chunk_*.wasm || true
              rm $out/public/wasm/module_*.wasm || true
              substituteInPlace $out/public/assets/tucan-plus-dioxus-*.js --replace-fail "importMeta.url" "import.meta.url" || true
            '';
            nativeBuildInputs = nativeBuildInputs ++ [
              # don't rebuild deps if version changes, maybe later patch this in post-build?
              (pkgs.writeShellScriptBin "git" ''
                echo ${self.rev or "dirty"}
              '')
            ];
          };
          CC_wasm32_unknown_unknown = "${pkgs.llvmPackages_21.clang-unwrapped}/bin/clang";
          CXX_wasm32_unknown_unknown = "${pkgs.llvmPackages_21.clang-unwrapped}/bin/clang++";
          AR_wasm32_unknown_unknown = "${pkgs.llvmPackages_21.bintools-unwrapped}/bin/llvm-ar";
          AS_wasm32_unknown_unknown = "${pkgs.llvmPackages_21.bintools-unwrapped}/bin/llvm-as";
          STRIP_wasm32_unknown_unknown = "${pkgs.llvmPackages_21.bintools-unwrapped}/bin/llvm-strip";
          CPATH = "${lib.getLib pkgs.llvmPackages_21.clang-unwrapped}/lib/clang/${lib.versions.major (lib.getVersion pkgs.llvmPackages_21.clang-unwrapped)}/include";
          strictDeps = true;
          doCheck = false;
          src = lib.fileset.toSource {
            root = ./.;
            fileset = fileset-wasm;
          };
          cargoExtraArgs = "--package=tucan-plus-dioxus";
          pname = "tucan-plus-workspace-tucan-plus-dioxus";
          installPhaseCommand = '''';
          checkPhaseCargoCommand = '''';
          nativeBuildInputs = [
            pkgs.which
            #wasm-bindgen
            pkgs.binaryen
          ];
          doNotPostBuildInstallCargoBinaries = true;
        };

        client = cargoDioxus craneLib (client-args);

        build-tests = craneLib.buildPackage {
          doCheck = false;
          src = lib.fileset.toSource {
            root = ./.;
            fileset = fileset-tests;
          };
          installPhaseCommand = ''
            echo START CUSTOM CODE
            mkdir $out
            echo $cargoBuildLog
            cat "$cargoBuildLog" | ${pkgs.jq}/bin/jq -r 'select(.reason == "compiler-artifact" and .profile.test == true) | .executable' | xargs -I '{}' install -C '{}' $out/tucan_plus_tests
            echo END CUSTOM CODE
          '';
          cargoExtraArgs = "--package=tucan-plus-tests --tests";
          pname = "tucan-plus-tests";
        };

        extension-unpacked = pkgs.stdenv.mkDerivation {
          pname = "tucan-plus-extension";
          version = (lib.importJSON ./tucan-plus-extension/manifest.json).version;

          src = lib.fileset.toSource {
            root = ./tucan-plus-extension;
            fileset = fileset-extension;
          };

          installPhase = ''
            mkdir $out
            cp -r $src/. $out/
            cp -r ${client}/public/. $out/public/
          '';
        };

        extension = pkgs.runCommand "tucan-plus-extension.zip" { } ''
          cd ${extension-unpacked}
          ${pkgs.zip}/bin/zip -r $out *
          ${pkgs.strip-nondeterminism}/bin/strip-nondeterminism --type zip $out
        '';

        source-with-build-instructions = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            fileset-wasm
            fileset-worker
            fileset-extension
            ./flake.nix
            ./flake.lock
            ./Dockerfile
            ./README.md
            ./rustfmt.toml
          ];
        };

        source = pkgs.runCommand "tucan-plus-extension-source.zip" { } ''
          cd ${source-with-build-instructions}
          ${pkgs.zip}/bin/zip -r $out *
          ${pkgs.strip-nondeterminism}/bin/strip-nondeterminism --type zip $out
        '';

        source-unpacked = pkgs.runCommand "tucan-plus-extension-source.zip" { } ''
          cp -r ${source-with-build-instructions} $out
        '';
      in
      rec {
        formatter = pkgs.nixfmt-tree;
        checks = {
          #inherit api schema client;

          # todo also clippy the frontend
          my-app-clippy = craneLib.cargoClippy ({
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            src = source-with-build-instructions;
          });

          my-app-fmt = craneLib.cargoFmt ({
            cargoExtraArgs = "--all";
            src = source-with-build-instructions;
          });
        };

        packages.client = client;
        packages.api-server = api-server;

        packages.extension = extension;
        packages.extension-unpacked = extension-unpacked;
        packages.extension-source = source;
        packages.extension-source-unpacked = source-unpacked;

        apps.api-server = flake-utils.lib.mkApp {
          name = "api-server";
          drv = api-server;
        };

        packages.publish =
          let
            version = (lib.importJSON ./tucan-plus-extension/manifest.json).version;
          in
          pkgs.writeShellScriptBin "publish" ''
            set -ex
            mkdir -p out
            cd out
            # seems like chromium writes into the parent folder of the pack-extension argument
            chmod -R ug+rw tucan-plus-extension-${version} || true
            rm -Rf tucan-plus-extension-${version}
            cp -r ${extension-unpacked} tucan-plus-extension-${version}
            ${pkgs.chromium}/bin/chromium --no-sandbox --pack-extension=tucan-plus-extension-${version} --pack-extension-key=$CHROMIUM_EXTENSION_SIGNING_KEY
            chmod 644 tucan-plus-extension-${version}.crx

            chmod -R ug+rw tucan-plus-extension-${version}
            rm -Rf tucan-plus-extension-${version}
            cp -r ${extension-unpacked} tucan-plus-extension-${version}
            chmod -R ug+rw tucan-plus-extension-${version}

            ${pkgs.web-ext}/bin/web-ext sign --channel unlisted --source-dir tucan-plus-extension-${version} --upload-source-code ${source}
            chmod 644 web-ext-artifacts/tucan_plus-${version}.xpi
            cp web-ext-artifacts/tucan_plus-${version}.xpi tucan-plus-extension-${version}.xpi
          '';

        packages.build-tests = build-tests;
        packages.test = pkgs.writeShellApplication {
          name = "test";

          runtimeInputs = [
            (pkgs.runCommand "chromedriver" {} ''
              mkdir -p $out/bin
              cp ${pkgs.fetchzip {
                url = "https://storage.googleapis.com/chrome-for-testing-public/146.0.7668.0/linux64/chromedriver-linux64.zip";
                hash = "sha256-AM4cabCzIPtKnEk7P54jkzj9KSafaG3NNaUGwk+eMGA=";
              }}/chromedriver $out/bin
            '')
            pkgs.geckodriver
            pkgs.chromium
            pkgs.firefox
            (
              pkgs.androidenv.emulateApp {
                name = "emulate-MyAndroidApp";
                platformVersion = "36";
                abiVersion = "x86_64"; # armeabi-v7a, mips, x86_64
                systemImageType = "google_apis_playstore";
              }
            )
          ];


          # 02-04 01:24:09.180  8676  8676 W chromium: [WARNING:extensions/browser/load_error_reporter.cc:73] Extension error: Failed to load extension from: /storage/emulated/0/Android/data/org.chromium.chrome/files/tucan-plus-extension. Manifest file is missing or unreadable
          # adb shell run-as org.chromium.chrome touch /data/local/tmp/tucan-plus-extension/a
          # I think the actual problem is that chromium does rule compilation sometimes and probably is not allowed to write there
          # 02-04 00:42:30.242  4120  4120 W chromium: [WARNING:extensions/browser/load_error_reporter.cc:73] Extension error: Failed to load extension from: /data/local/tmp/tucan-plus-extension. rules.json: Internal error while parsing rules.          # https://commondatastorage.googleapis.com/chromium-browser-snapshots/index.html?prefix=AndroidDesktop_x64/
          # 02-04 01:06:35.246  4003  4023 E chromium: [ERROR:sandbox/policy/linux/landlock_gpu_policy_android.cc:93] Ruleset creation failed: Function not implemented (38)


          # https://commondatastorage.googleapis.com/chromium-browser-snapshots/index.html?prefix=AndroidDesktop_arm64/
          # https://www.googleapis.com/download/storage/v1/b/chromium-browser-snapshots/o/AndroidDesktop_arm64%2F1578993%2Fchrome-android-desktop.zip?generation=1770154620269384&alt=media
          # https://archive.mozilla.org/pub/fenix/releases/147.0.2/android/fenix-147.0.2-android/fenix-147.0.2.multi.android-universal.apk
          text = ''
            set -ex
            NIX_ANDROID_EMULATOR_FLAGS="-gpu swiftshader_indirect" run-test-emulator
            adb install ${pkgs.fetchzip {
              url = "https://www.googleapis.com/download/storage/v1/b/chromium-browser-snapshots/o/AndroidDesktop_x64%2F1579023%2Fchrome-android-desktop.zip?generation=1770157374116558&alt=media&.zip";
              hash = "sha256-j+X0zE6cSfs0OHwqA/Z/LXYdB5zUxFree/XfXx+6eHA=";
            }}/apks/ChromePublic.apk
            adb install ${pkgs.fetchurl {
              url = "https://archive.mozilla.org/pub/fenix/releases/147.0.2/android/fenix-147.0.2-android/fenix-147.0.2.multi.android-universal.apk";
              hash = "sha256-jcHD6We2Wwsx55ZXmmrZ4hbR7AH/ICU5RvWeOXE6eYk=";
            }}
            EXTENSION_FILE=$(mktemp -d)
            export EXTENSION_FILE
            cp -r ${extension-unpacked}/. "$EXTENSION_FILE"/
            chmod -R ug+rw "$EXTENSION_FILE"
            ${build-tests}/tucan_plus_tests android_chromium_main
          '';
        };

        devShells.default = pkgs.mkShell {
          shellHook = ''
            export PATH=~/.cargo/bin/:$PATH
          '';
          buildInputs = [
            pkgs.openssl
            pkgs.sqlite
            pkgs.at-spi2-atk
            pkgs.atkmm
            pkgs.cairo
            pkgs.gdk-pixbuf
            pkgs.glib
            pkgs.gtk3
            pkgs.harfbuzz
            pkgs.librsvg
            pkgs.libsoup_3
            pkgs.pango
            pkgs.webkitgtk_4_1
            pkgs.openssl
            pkgs.xdotool
            pkgs.zlib
          ];
          packages = [
            pkgs.bashInteractive
            pkgs.nixfmt-tree
            pkgs.nixfmt-rfc-style
            pkgs.wabt
            pkgs.wasm-tools
            pkgs.nodejs
            pkgs.bun
            pkgs.pkg-config
            pkgs.gobject-introspection
            pkgs.jdk
            pkgs.android-tools
            pkgs.binaryen
            pkgs.llvmPackages_21.bintools
            pkgs.dioxus-cli
          ];
        };
      }
    );
}
