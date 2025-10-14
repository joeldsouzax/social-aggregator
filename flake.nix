{
  description = "social media aggregator";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nixpkgs-stable.url = "github:nixos/nixpkgs/nixos-25.05";
    utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs = { nixpkgs.follows = "nixpkgs"; };
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    sops-nix = {
      url = "github:Mic92/sops-nix";
      inputs.nixpkgs.follows = "nixpkgs-stable";
    };
  };
  outputs = { self, nixpkgs, nixpkgs-stable, utils, crane, fenix, advisory-db
    , sops-nix, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs) lib;
        craneLib = (crane.mkLib pkgs).overrideToolchain
          (fenix.packages.${system}.complete.toolchain);

        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
          buildInputs = with pkgs;
            [ openssl ] ++ lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.libiconv
            ];
          nativeBuildInputs = with pkgs;
            [ cmake pkg-config ] ++ lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.Libsystem
            ];
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        individualCrateArgs = commonArgs // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;

          doCheck = false;
        };

        workspaceSrc = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            (craneLib.fileset.commonCargoSources ./feeders)
            (craneLib.fileset.commonCargoSources ./aggregator)
            (craneLib.fileset.commonCargoSources ./commons/proto-definitions)
            (craneLib.fileset.commonCargoSources ./commons/workspace-hack)

          ];
        };

        mkPackage = name:
          let
            cargoTomlPath = ./${name}/Cargo.toml;
            _c = assert builtins.pathExists cargoTomlPath;
              throw "Cargo file does not exist: ${cargoTomlPath}";
            cargoToml = builtins.fromTOML (builtins.readFile cargoTomlPath);
            pname = cargoToml.package.name;
            version = cargoToml.package.version;
            bin = craneLib.buildPackage (individualCrateArgs // {
              inherit pname version;
              src = workspaceSrc;
              cargoExtraArgs = "-p ${pname}";
            });

            image = pkgs.dockerTools.streamLayeredImage {
              name = "joeldsouzax/${pname}";
              created = "now";
              tag = version;
              contents = [ bin ];
              config = {
                Env = [ ];
                Cmd = [ "${bin}/bin/${pname}" ];
                ExposedPorts = { "3000/tcp" = { }; };
                WorkingDir = "/";
              };
            };
          in image;

        ## crates
        ## personal scripts
        ##
        ##
        jikkou = pkgs.stdenv.mkDerivation rec {
          pname = "jikkou";
          version = "0.36.4";

          src = pkgs.fetchurl {
            url =
              "https://github.com/streamthoughts/jikkou/releases/download/v${version}/jikkou-${version}-linux-x86_64.zip";
            sha256 = "sha256-/UCDyiL5L5ygcuXZDMj95qqWZh49PmoWj0y6k8pHwCU=";
          };

          nativeBuildInputs = [ pkgs.unzip ];

          installPhase = ''
            install -Dm755 bin/jikkou $out/bin/jikkou
          '';

          meta = with pkgs.lib; {
            description =
              "A command-line tool to manage, automate, and provision your Apache Kafka resources";
            homepage = "https://github.com/streamthoughts/jikkou";
            license = licenses.asl20; # Apache License 2.0
            platforms = [ "x86_64-linux" ];
            mainProgram = "jikkou";
          };
        };

        pu = pkgs.writeShellScriptBin "start-infra" ''
          #!/usr/bin/env bash
          set -euo pipefail
          echo "--- Setting up data directories ---"
          mkdir -p .data
          sudo chown -P 1000:1000 .data

          # Apicurio
          mkdir -p .data/apicurio_db
          sudo chown -P 999:999 .data/apicurio_db

          # Kafka
          mkdir -p .data/controller-1 .data/controller-2 .data/controller-3
          sudo chown -R 1000:1000 .data/controller-1 .data/controller-2 .data/controller-3

          mkdir -p .data/broker-1 .data/broker-2 .data/broker-3 .data/broker-4
          sudo chown -R 1000:1000 .data/broker-1 .data/broker-2 .data/broker-3 .data/broker-4
          echo "Directory setup complete."
          echo ""

          SECRETS_FILE=".env"
          echo "Attempting to source secrets from $SECRETS_FILE..."
          if [ -f "$SECRETS_FILE" ]; then
            set -a
            source "$SECRETS_FILE"
            set +a
            echo "Secrets sourced successfully."
          else
            echo "Warning: Secrets file not found."
            echo "Attempting to decrypt now as a fallback..."
            # Also add the --output-type flag here
            sops --decrypt --output-type dotenv infra/secrets/dev.yaml > "$SECRETS_FILE"
            set -a
            source "$SECRETS_FILE"
            set +a
          fi

          sudo -E podman compose up -d
        '';

        pd = pkgs.writeShellScriptBin "stop-infra" ''
          exec podman-compose down
        '';

        i = pkgs.writeShellScriptBin "dive-image" ''
          gunzip --stdout result > /tmp/image.tar && dive docker-archive: ///tmp/image.tar
        '';

        ### deploying apps

        mkApp = name:
          let
            imageStream = mkPackage name;
            pushScriptDrv = pkgs.writeShellScriptBin "push-${name}-image" ''
              #!${pkgs.bash}/bin/bash
              set -euo pipefail

              PUSH_LATEST_TAG="''${PUSH_LATEST_TAG:-false}"
              SKOPEO_CMD="${pkgs.skopeo}/bin/skopeo"
              GZIP_CMD="${pkgs.gzip}/bin/gzip"
              IMAGE_STREAM_SCRIPT="${imageStream}"

              DESTINATION="docker://''${REGISTRY_URL,,}/''${IMAGE_NAME,,}:''${IMAGE_TAG}"
              DESTINATION_LATEST="docker://''${REGISTRY_URL,,}/''${IMAGE_NAME,,}:latest"
              CREDENTIALS="''${REGISTRY_USER}:''${REGISTRY_PASSWORD}"

              echo "--- Pushing ${name} Service Image ---"
              echo "Executing stream script: $IMAGE_STREAM_SCRIPT"
              echo "Piping stream via gzip to Skopeo..."
              echo "Source: docker-archive:/dev/stdin"
              echo "Destination: $DESTINATION"
              echo "User: $REGISTRY_USER"

              "$IMAGE_STREAM_SCRIPT" | "$GZIP_CMD" --fast | "$SKOPEO_CMD" copy \
                --dest-creds "$CREDENTIALS" \
                docker-archive:/dev/stdin \
                "$DESTINATION"

              if [[ "$PUSH_LATEST_TAG" == "true" ]]; then
                echo "Pushing latest tag to $DESTINATION_LATEST"
                "$IMAGE_STREAM_SCRIPT" | "$GZIP_CMD" --fast | "$SKOPEO_CMD" copy \
                  --dest-creds "$CREDENTIALS" \
                  docker-archive:/dev/stdin \
                  "$DESTINATION_LATEST"
              fi

              echo "--- Push complete for ${name} ---"
            '';
          in {
            type = "app";
            program = "${pushScriptDrv}/bin/push-${name}-image";
            meta = {
              description =
                "Pushes the ${name} OCI image stream to a configured registry using Skopeo";
            };
          };

        feeders = mkPackage "feeders";
        aggregator = mkPackage "aggregator";

      in with pkgs; {
        checks = {
          inherit feeders aggregator;
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          doc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });

          fmt = craneLib.cargoFmt { inherit src; };

          toml-fmt = craneLib.taploFmt {
            src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
            # taplo arguments can be further customized below as needed
            # taploExtraArgs = "format";
          };

          audit = craneLib.cargoAudit { inherit src advisory-db; };

          deny = craneLib.cargoDeny { inherit src; };
          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on other crate derivations
          # if you do not want the tests to run twice
          nextest = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });

          coverage =
            craneLib.cargoTarpaulin (commonArgs // { inherit cargoArtifacts; });

          # Ensure that cargo-hakari is up to date
          hakari = craneLib.mkCargoDerivation {
            inherit src;
            pname = "hakari";
            cargoArtifacts = null;
            doInstallCargoArtifacts = false;

            buildPhaseCargoCommand = ''
              cargo hakari generate --diff  # workspace-hack Cargo.toml is up-to-date
              cargo hakari manage-deps --dry-run  # all workspace crates depend on workspace-hack
              cargo hakari verify
            '';

            # Add cargo-hakari to the inputs from commonArgs
            nativeBuildInputs = commonArgs.nativeBuildInputs
              ++ [ pkgs.cargo-hakari ];
          };

        };

        packages = {
          inherit feeders aggregator pu pd i jikkou;
          ## integration test for auth
          integration = pkgs.testers.runNixOSTest ({
            name = "integration-test";
            nodes = { };
            testScript = { nodes, ... }: "\n";
          });
        };

        apps = { };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          shellHook = ''
            #!/usr/bin/env bash
            # Create a fancy welcome message
            #
            echo "Decrypting secrets to .sops-dev-secrets.env..."
            if sops --decrypt --output-type dotenv infra/secrets/dev.yaml > .env; then
              echo "Sourcing secrets for dev shell..."
              set -a
              source .env
              set +a
            else
              echo "Warning: Failed to decrypt secrets. Continuing without them."
            fi
            REPO_NAME=$(basename "$PWD")
            PROPER_REPO_NAME=$(echo "$REPO_NAME" | awk '{print toupper(substr($0,1,1)) tolower(substr($0,2))}')
            figlet -f doom "$PROPER_REPO_NAME" | lolcat -a -d 2
            cowsay -f dragon-and-cow "Welcome to the $PROPER_REPO_NAME development environment on ${system}!" | lolcat
          '';

          packages = [
            fenix.packages.${system}.rust-analyzer
            bacon
            figlet
            lolcat
            cowsay
            tmux
            dive
            cargo-hakari
            tree
            cloc
            skopeo
            gzip
            sops
            age
            cargo-edit
            cargo-expand
            websocat
            kafkactl
            nodePackages.nodejs
            nodePackages.pnpm
            nodePackages.typescript
            nodePackages.typescript-language-server
            jikkou
          ];
        };
      });
}
