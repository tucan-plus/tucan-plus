{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:mohe2015/nixpkgs/update-dogtail";

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
          ./crates/tucan-plus-dioxus/index.html
          ./crates/tucan-plus-dioxus/Dioxus.toml
          ./crates/tucan-plus-dioxus/.cargo/config.toml
        ];

        fileset-wasm = lib.fileset.unions [
          ./Cargo.toml
          ./Cargo.lock
          (craneLib.fileset.commonCargoSources ./crates/html-extractor)
          (craneLib.fileset.commonCargoSources ./crates/tucan-connector)
          (craneLib.fileset.commonCargoSources ./crates/html-handler)
          (craneLib.fileset.commonCargoSources ./crates/tucan-plus-planning)
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

        tests = craneLib.cargoTest {
          cargoArtifacts = craneLib.buildDepsOnly {
            cargoExtraArgs = "--package=tucan-plus-tests";
            pname = "tucan-plus-tests";
            src = lib.fileset.toSource {
              root = ./.;
              fileset = fileset-tests;
            };
          };
          src = lib.fileset.toSource {
            root = ./.;
            fileset = fileset-tests;
          };
          cargoExtraArgs = "--package=tucan-plus-tests";
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

        packages.tests = tests;

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

        packages.test = pkgs.writeShellApplication {
          name = "test";

          runtimeInputs = [
            pkgs.chromedriver
            pkgs.geckodriver
            pkgs.chromium
            pkgs.firefox
          ];

          text = ''
            set -ex
            EXTENSION_DIR=$(mktemp -d)
            export EXTENSION_DIR
            cp -r ${extension-unpacked}/. "$EXTENSION_DIR"/
            chmod -R ug+rw "$EXTENSION_DIR"
            cargo test --package tucan-plus-tests -- --nocapture
          '';
        };

        packages.test-dev = pkgs.writeShellApplication {
          name = "test-dev";

          text = ''
            set -ex
            EXTENSION_DIR=$(mktemp -d)
            export EXTENSION_DIR
            cp -r ${extension-unpacked}/. "$EXTENSION_DIR"/
            chmod -R ug+rw "$EXTENSION_DIR"
            cargo test --package tucan-plus-tests -- --nocapture
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
