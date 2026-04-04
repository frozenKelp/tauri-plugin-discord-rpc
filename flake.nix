{
  description = "tauri-plugin-discord-rpc";

  inputs = {
    nixpkgs.url     = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url                   = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, rust-overlay, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" ];

      perSystem = { system, pkgs, lib, ... }:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };

          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          };

          runtimeLibs = with pkgs; [
            webkitgtk_4_1
            gtk3
            glib
            cairo
            pango
            atk
            gdk-pixbuf
            libsoup_3
            openssl
            dbus
            libappindicator-gtk3
            gsettings-desktop-schemas
          ];
        in
        {
          devShells.default = pkgs.mkShell {
            buildInputs = runtimeLibs;
            nativeBuildInputs = with pkgs; [
              rustToolchain
              pkg-config
              wrapGAppsHook3
              nodejs_22
              pnpm
            ];
            shellHook = ''
              export NO_STRIP=true
              export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig''${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
              export XDG_DATA_DIRS="${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}''${XDG_DATA_DIRS:+:$XDG_DATA_DIRS}"
            '';
          };

          formatter = pkgs.nixfmt-rfc-style;
        };
    };
}
