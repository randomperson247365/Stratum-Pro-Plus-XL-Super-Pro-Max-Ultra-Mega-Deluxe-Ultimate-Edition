{
  description = "Stratum Desktop Environment — Rust DE built on River";

  inputs = {
    nixpkgs.url     = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname   = "stratum-de";
          version = "0.1.0";
          src     = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs       = with pkgs; [ libxkbcommon wayland ];

          postInstall = ''
            install -Dm644 data/stratum.desktop \
              $out/share/wayland-sessions/stratum.desktop
            install -Dm644 data/stratum-settings.desktop \
              $out/share/applications/stratum-settings.desktop
            install -Dm755 contrib/river-init \
              $out/bin/stratum-river-init
            install -Dm644 data/default-config.toml \
              $out/etc/stratum/config.toml
          '';
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc cargo clippy rust-analyzer
            libxkbcommon wayland pkg-config
            river
          ];
        };
      });
}
