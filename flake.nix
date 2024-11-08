{
  description = "Flake for Cosmoping";

  inputs = {
    devenv-root = {
      url = "file+file:///dev/null";
      flake = false;
    };
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    fenix.url = "github:nix-community/fenix";
    devenv.url = "github:cachix/devenv";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-utils.url = "github:numtide/flake-utils";
    nix2container.url = "github:nlewo/nix2container";
    nix2container.inputs.nixpkgs.follows = "nixpkgs";
    mk-shell-bin.url = "github:rrbutani/nix-mk-shell-bin";
  };

  nixConfig = {
    extra-substituters = [
      "https://tweag-jupyter.cachix.org"
      "https://devenv.cachix.org"
    ];
    extra-trusted-public-keys = [
      "tweag-jupyter.cachix.org-1:UtNH4Zs6hVUFpFBTLaA4ejYavPo5EFFqgd7G7FxGW9g="
      "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw="
    ];
  };

  outputs = inputs @ {
    flake-parts,
    flake-utils,
    nixpkgs,
    devenv-root,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.devenv.flakeModule
      ];

      systems = inputs.nixpkgs.lib.systems.flakeExposed;

      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        name = "cosmoping";
        ver = "0.1.0";
        homepage = "https://github.com/glottologist/cosmoping";
        description = "Cosmos SDK AddrBook Latency Report Generator";
        license = self'.pkgs.lib.licences.agpl3Plus;
        maintainers = with self'.pkgs.lib.maintainers; [
          {
            name = "Jason Ridgway-Taylor";
            email = "jason@glottologist.co.uk";
            github = "glottologist";
          }
        ];
      in rec {
        packages = rec {
          cosmoping = pkgs.rustPlatform.buildRustPackage {
            pname = name;
            version = ver;
            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            # do not let test results block the build process
            doCheck = false;

            # Temporary fake hash, replace with real one after first build
            cargoSha256 = pkgs.lib.fakeSha256;

            meta = with pkgs.stdenv.lib; {
              inherit maintainers homepage description licenses;
            };
          };

          default = self'.packages.cosmoping;
        };
        apps = {
          cosmopingApp = flake-utils.lib.mkApp {drv = self'.packages.${system}.cosmoping;};
        };

        devenv.shells.default = {
          devenv.root = let
            devenvRootFileContent = builtins.readFile devenv-root.outPath;
          in
            pkgs.lib.mkIf (devenvRootFileContent != "") devenvRootFileContent;
          name = "Cosmoping shell for Rust";
          env.GREET = "devenv for Cosmoping";
          env.PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          env.LOCATION_API_TOKEN = "<API_TOKEN>";
          packages = with pkgs; [
            git
            mdbook
            mdbook-i18n-helpers
            mdbook-mermaid
            openssh
            openssl
            pkg-config
          ];
          enterShell = ''
            cargo install cargo-watch
            cargo install cargo-nextest
            git --version
            nix --version
            rustc --version
            cargo --version
            mdbook --version
          '';
          languages = {
            rust.enable = true;
            rust.channel = "nightly";
            nix.enable = true;
          };
          scripts = {
            ntest.exec = ''
              cargo nextest run --all --nocapture
            '';
            watch.exec = ''
              cargo watch -c -q -w ./src -x build
            '';

            cosmoping.exec = ''
              cargo run -- latency --addrbook-path ./addrbook.json --output-path ./latencies.md --location-api-key $LOCATION_API_TOKEN

            '';
          };
          dotenv.enable = true;
          difftastic.enable = true;
          pre-commit = {
            hooks = {
              alejandra.enable = true;
              commitizen.enable = true;
              cargo-check.enable = true;
              clippy.enable = true;
              rustfmt.enable = true;
              nil.enable = true;
            };
            settings.rust.cargoManifestPath = "./Cargo.toml";
          };
        };
      };
    };
}
