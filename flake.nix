{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

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
          p.rust-bin.nightly.latest.minimal.override {
            targets = [ "wasm32-unknown-unknown" ];
            extensions = [ "rustfmt" ];
          };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainFor;

        dioxus-cli = craneLib.buildPackage {
          src = pkgs.fetchFromGitHub {
            owner = "mohe2015";
            repo = "dioxus";
            rev = "4e2b36831e2fef577fbc1ad10d82c79cf331afdb";
            hash = "sha256-fPKIOTkZee5bpTBdwSxWTNM4S3gL87rySQnD806aVNA=";
          };
          doCheck = false;
          strictDeps = true;
          pname = "dioxus-cli";
          cargoExtraArgs = "-p dioxus-cli --features no-downloads";
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };

        nativeArgs = {
          strictDeps = true;
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              (craneLib.fileset.commonCargoSources ./crates)
              (lib.fileset.fileFilter (
                file:
                lib.any file.hasExt [
                  "html"
                  "scss"
                ]
              ) ./.)
              ./tucan-plus-extension/bootstrap.bundle.min.js
              ./tucan-plus-extension/bootstrap.css
            ];
          };
          pname = "tucan-plus-workspace-native";
        };

        api = craneLib.buildPackage {
          cargoToml = ./crates/tucan-plus-api/Cargo.toml;
          cargoLock = ./crates/tucan-plus-api/Cargo.lock;
          preBuild = ''
            cd ./crates/tucan-plus-api
          '';
          strictDeps = true;
          pname = "tucan-plus-workspace-native-api";
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              (craneLib.fileset.commonCargoSources ./crates/tucan-types)
              (craneLib.fileset.commonCargoSources ./crates/key-value-database)
              (craneLib.fileset.commonCargoSources ./crates/html-extractor)
              (craneLib.fileset.commonCargoSources ./crates/tucan-connector)
              (craneLib.fileset.commonCargoSources ./crates/tucan-plus-api)
              (craneLib.fileset.commonCargoSources ./crates/html-handler)
            ];
          };
          cargoTestExtraArgs = "--no-run";
          cargoExtraArgs = "--package=tucan-plus-api";
        };

        schema =
          pkgs.runCommandNoCC "schema.json"
            {
            }
            ''
              ${api}/bin/schema > $out
            '';

        fileset-worker = lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ./crates/tucan-plus-worker)
          (craneLib.fileset.commonCargoSources ./crates/tucan-types)
          ./crates/tucan-plus-worker/migrations
        ];

        fileset-wasm = lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ./crates/key-value-database)
          (craneLib.fileset.commonCargoSources ./crates/html-extractor)
          (craneLib.fileset.commonCargoSources ./crates/tucan-connector)
          (craneLib.fileset.commonCargoSources ./crates/tucan-plus-dioxus)
          (craneLib.fileset.commonCargoSources ./crates/html-handler)
          (craneLib.fileset.commonCargoSources ./crates/tucan-plus-planning)
          ./crates/tucan-plus-dioxus/assets/logo.svg
          ./crates/tucan-plus-dioxus/assets/manifest.json
          ./crates/tucan-plus-dioxus/assets/bootstrap.css
          ./crates/tucan-plus-dioxus/assets/bootstrap.bundle.min.js
          ./crates/tucan-plus-dioxus/assets/bootstrap.patch.js
          ./crates/tucan-plus-dioxus/index.html
          fileset-worker
        ];

        wasm-bindgen = (pkgs.buildWasmBindgenCli rec {
          src = pkgs.fetchCrate {
            pname = "wasm-bindgen-cli";
            version = "0.2.101";
            hash = "sha256-txpbTzlrPSEktyT9kSpw4RXQoiSZHm9t3VxeRn//9JI=";
          };

          cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
            inherit src;
            inherit (src) pname version;
            hash = "sha256-J+F9SqTpH3T0MbvlNKVyKnMachgn8UXeoTF0Pk3Xtnc=";
          };
        });

        worker-args = {
          CARGO_TARGET_DIR = "./crates/tucan-plus-worker/target";
          cargoToml = ./crates/tucan-plus-worker/Cargo.toml;
          cargoLock = ./crates/tucan-plus-worker/Cargo.lock;
          preBuild = ''
            cd ./crates/tucan-plus-worker
          '';
          postBuild = ''
            cd ../..
          '';
          strictDeps = true;
          stdenv = p: p.emscriptenStdenv;
          doCheck = false;
          cargoExtraArgs = "--package=tucan-plus-worker";
          pname = "tucan-plus-workspace-tucan-plus-worker";
          buildPhaseCargoCommand = ''
            export CC=emcc
            export CXX=emcc
            CARGO_TARGET_DIR=target ${dioxus-cli}/bin/dx bundle --wasm --bundle web --release --out-dir $out --base-path public
          '';
          installPhaseCommand = '''';
          checkPhaseCargoCommand = '''';
          nativeBuildInputs = [
            pkgs.which
            pkgs.emscripten
            wasm-bindgen
            pkgs.binaryen
            (pkgs.writeShellScriptBin "git" ''
              echo ${self.rev or "dirty"}
            '')
          ];
          doNotPostBuildInstallCargoBinaries = true;
          src = lib.fileset.toSource {
            root = ./.;
            fileset = fileset-worker;
          };
        };

        worker = craneLib.buildPackage (worker-args // {
          cargoArtifacts = craneLib.buildDepsOnly (worker-args // {
            dummySrc = craneLib.mkDummySrc {
              src = worker-args.src;
              extraDummyScript = ''
                cp ${worker-args.cargoLock} $out/crates/tucan-plus-worker/Cargo.lock
                rm $out/crates/tucan-plus-worker/src/main.rs
                cp ${pkgs.writeText "main.rs" ''
                  use wasm_bindgen::prelude::*;

                  #[wasm_bindgen(main)]
                  pub async fn main() {

                  }
                ''} $out/crates/tucan-plus-worker/src/main.rs
              '';
            };
          });
        });

        client-args = {
          CARGO_TARGET_DIR = "./crates/tucan-plus-dioxus/target";
          cargoToml = ./crates/tucan-plus-dioxus/Cargo.toml;
          cargoLock = ./crates/tucan-plus-dioxus/Cargo.lock;
          preBuild = ''
            cd ./crates/tucan-plus-dioxus
          '';
          postBuild = ''
            cd ../..
          '';
          strictDeps = true;
          stdenv = p: p.emscriptenStdenv;
          doCheck = false;
          src = lib.fileset.toSource {
            root = ./.;
            fileset = fileset-wasm;
          };
          cargoExtraArgs = "--package=tucan-plus-dioxus";
          pname = "tucan-plus-workspace-tucan-plus-dioxus";
          buildPhaseCargoCommand = ''
            export CC=emcc
            export CXX=emcc
            mkdir -p assets/
            cp ${worker}/public/assets/tucan-plus-worker-*.js assets/
            cp ${worker}/public/assets/tucan-plus-worker_bg-*.wasm assets/
            export WORKER_JS_PATH_ARRAY=(assets/tucan-plus-worker-*.js)
            export WORKER_JS_PATH="/''${WORKER_JS_PATH_ARRAY[0]}"
            export WORKER_WASM_PATH_ARRAY=(assets/tucan-plus-worker_bg-*.wasm)
            export WORKER_WASM_PATH="/''${WORKER_WASM_PATH_ARRAY[0]}"
            CARGO_TARGET_DIR=target ${dioxus-cli}/bin/dx bundle --platform web --release --out-dir $out --base-path public --features direct
          '';
          installPhaseCommand = ''
          '';
          checkPhaseCargoCommand = ''
          '';
          nativeBuildInputs = [
            pkgs.which
            pkgs.emscripten
            wasm-bindgen
            pkgs.binaryen
            (pkgs.writeShellScriptBin "git" ''
              echo ${self.rev or "dirty"}
            '')
          ];
          doNotPostBuildInstallCargoBinaries = true;
        };

        client = craneLib.buildPackage (client-args // {
          cargoArtifacts = craneLib.buildDepsOnly (client-args // {
            buildPhaseCargoCommand = ''
              export CC=emcc
              export CXX=emcc
              CARGO_TARGET_DIR=target ${dioxus-cli}/bin/dx bundle --platform web --release --out-dir $out --base-path public --features direct
            '';
            dummySrc = craneLib.mkDummySrc {
              src = client-args.src;
              extraDummyScript = ''
                cp ${client-args.cargoLock} $out/crates/tucan-plus-dioxus/Cargo.lock
                rm $out/crates/tucan-plus-dioxus/src/main.rs
                cp ${pkgs.writeText "main.rs" ''
                  use wasm_bindgen::prelude::*;

                  #[wasm_bindgen(main)]
                  pub async fn main() {

                  }
                ''} $out/crates/tucan-plus-dioxus/src/main.rs
              '';
            };
          });
        });

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
          #my-app-clippy = craneLib.cargoClippy (
          #  nativeArgs
          #  // {
          #    cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          #  }
          #);

          #my-app-fmt = craneLib.cargoFmt (
          #  nativeArgs
          #  // {
          #    cargoToml = ./crates/tucan-plus-dioxus/Cargo.toml;
          #    cargoLock = ./crates/tucan-plus-dioxus/Cargo.lock;
          #    preBuild = ''
          #      cd ./crates/tucan-plus-dioxus
           #   '';
          #    cargoExtraArgs = "--all";
          #    src = source-with-build-instructions;
          #  }
          #);

          # https://nixos.org/manual/nixos/unstable/index.html#sec-nixos-tests
          # https://github.com/NixOS/nixpkgs/blob/a25a80403e18d80ffb9e5a2047c7936e57fbae68/nixos/tests/installed-tests/default.nix#L15
          # https://github.com/NixOS/nixpkgs/blob/a25a80403e18d80ffb9e5a2047c7936e57fbae68/nixos/tests/installed-tests/gnome-photos.nix#L10
          # nix run -L .#checks.x86_64-linux.extension-test.driverInteractive
          # test_script()
          # nix flake check -L
          extension-test = pkgs.testers.runNixOSTest {
            name = "extension-test";
            nodes = {
              machine = {pkgs, ...}: {
                virtualisation = {
                  sharedDirectories = {
                    projects = {source="/home/moritz/Documents/tucan-plus/demo-video/"; target="/home/test/tucan_plus";};
                  };
                };
                virtualisation.memorySize = 8192;

                services.gnome.at-spi2-core.enable = true;

                services.xserver.xkb.layout = "de";

                boot.kernelPackages = pkgs.linuxPackages_latest;

                services.displayManager.gdm.enable = true;
                services.desktopManager.gnome.enable = true;

                #services.gnome.core-apps.enable = false;
                #services.gnome.core-developer-tools.enable = false;
                #services.gnome.games.enable = false;
                environment.gnome.excludePackages = with pkgs; [ gnome-tour ]; # gnome-user-docs

                services.displayManager.autoLogin.enable = true;
                services.displayManager.autoLogin.user = "test";

                users.users.test = {
                  isNormalUser = true;
                  uid = 1000;
                };

                environment.systemPackages = [
                  pkgs.firefox
                  # TODO https://nixos.org/manual/nixpkgs/unstable/#ssec-gnome-common-issues-double-wrapped
                  (pkgs.python3.pkgs.buildPythonApplication {
                      pname = "run-test";
                      version = "3.32.2";
                      pyproject = true;
                      build-system = with pkgs.python3Packages; [ setuptools ];

                      dependencies = with pkgs.python3Packages; [
                        dogtail
                      ];

                      src = ./demo-video;

                      nativeBuildInputs = [
                        pkgs.wrapGAppsHook3
                        pkgs.gobject-introspection
                      ];

                      dontWrapGApps = true;

                      # Arguments to be passed to `makeWrapper`, only used by buildPython*
                      preFixup = ''
                        makeWrapperArgs+=("''${gappsWrapperArgs[@]}")
                      '';
                    }
                  )
                ];

                programs.dconf.enable = true;
                programs.dconf.profiles.test.databases = [
                  {
                    settings = {
                      "org/gnome/desktop/interface" = {
                        toolkit-accessibility = true;
                      };
                    };
                  }
                ];

                systemd.user.services = {
                  "org.gnome.Shell@wayland" = {
                    serviceConfig = {
                      ExecStart = [
                        # Clear the list before overriding it.
                        ""
                        # Eval API is now internal so Shell needs to run in unsafe mode.
                        # TODO: improve test driver so that it supports openqa-like manipulation
                        # that would allow us to drop this mess.
                        "${pkgs.gnome-shell}/bin/gnome-shell --unsafe-mode"
                      ];
                    };
                  };
                };

                system.stateVersion = "25.11";
              };
            };
            testScript = { nodes, ... }: lib.mkForce ''
              print("a")
              start_all()
              print("b")
              machine.wait_until_succeeds(
                  "systemd-run --pipe --machine=test@.host --user /usr/bin/env bash -c 'gdbus call --session -d org.gnome.Shell -o /org/gnome/Shell -m org.gnome.Shell.Eval Main.layoutManager._startingUp' | grep -q \"true,..false\"",
                  timeout=60
              )
              print("c")
              machine.succeed("systemd-run --machine=test@.host --user /usr/bin/env bash -c 'gsettings set org.gnome.desktop.interface toolkit-accessibility true'")
              print("d")
              machine.succeed("systemd-run --machine=test@.host --user /usr/bin/env bash -c firefox")
              print("e")
              machine.succeed("systemd-run --pipe --machine=test@.host --user /usr/bin/env bash -c tucan_plus")
              print("f")
            '';
            interactive = {
              sshBackdoor.enable = true; # ssh vsock/3 -o User=root
            };
            # https://wiki.nixos.org/wiki/Python
            # ssh vsock/3 -o User=root
            # machinectl shell test@
            # nix-shell -I nixpkgs=channel:nixos-unstable -p gobject-introspection gtk3 'python3.withPackages (ps: with ps; [ dogtail ])' --run python /home/test/tucan_plus/tucan_plus.py
          };
        };

        packages.schema = schema;
        packages.client = client;
        packages.server = api;
        packages.extension = extension;
        packages.extension-unpacked = extension-unpacked;
        packages.extension-source = source;
        packages.extension-source-unpacked = source-unpacked;
        packages.dioxus-cli = dioxus-cli;

        apps.server = flake-utils.lib.mkApp {
          name = "server";
          drv = api;
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
            export LD_LIBRARY_PATH="${pkgs.zlib}/lib''${LD_LIBRARY_PATH:+:}''${LD_LIBRARY_PATH}"
          '';

          packages = [
            pkgs.bashInteractive
            pkgs.diffoscope
            pkgs.bacon
            pkgs.geckodriver
            pkgs.chromedriver
            pkgs.chromium
            pkgs.firefox
            pkgs.nodejs_latest
          ];
        };
      }
    );
}
