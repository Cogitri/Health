{
  description = ''A health tracking app for the GNOME desktop
  
  Includes an overlay for usage inside nixpkgs and a development
  environment'';

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    mozilla.url = "github:mozilla/nixpkgs-mozilla";

    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { nixpkgs, mozilla, flake-utils, ... }:
    let
      inherit (flake-utils.lib) eachSystem system;
    in
    eachSystem [ system.x86_64-linux ]
      (system:
        let
          pkgs = import nixpkgs
            {
              inherit system;
              overlays = [
                mozilla.overlay
                (final: prev: {
                  trackerPatched = pkgs.tracker.overrideAttrs (old: rec {
                    patches = old.patches ++ [ ./build-aux/tracker-subsecond-accuracy.patch ];
                  });
                })
              ];
            };
          rustSrc = pkgs.latest.rustChannels.stable.rust.override { extensions = [ "rust-src" ]; };
          nativeBuildInputs = with pkgs; [
            cargo-audit
            cargo-bloat
            cargo-dephell
            cargo-expand
            cargo-outdated
            clang_13
            gdb
            libxml2
            lld_13
            meson
            ninja
            python3
            pkg-config
            rustfmt
            rustSrc
            valgrind
          ];
          buildInputs = with pkgs; [
            appstream-glib
            cairo
            desktop-file-utils
            gdk-pixbuf
            glib
            graphene
            gtk4.dev
            gtk4
            harfbuzz
            libadwaita
            libfaketime
            librsvg
            libsecret
            libsecret.dev
            pango
            trackerPatched
            trackerPatched.dev
            wayland
            wayland.dev
          ];
        in
        {
          devShell = pkgs.mkShell
            {
              name = "Health-shell";
              inherit nativeBuildInputs buildInputs;
              RUST_SRC_PATH = "${rustSrc}/lib/rustlib/src/rust/src";
              RUSTFLAGS = "-C linker=clang -C link-arg=--ld-path=${pkgs.lld_13}/bin/ld.lld";
              LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
              #LD_PRELOAD = "${pkgs.libfaketime}/lib/libfaketime.so.1";
              XDG_DATA_DIRS = with pkgs; "${gtk4.dev}/share:{libadwaita.dev}/share:${gdk-pixbuf.dev}/share:${gobject-introspection.dev}/share:${pango.dev}/share:${harfbuzz.dev}/share:${graphene}/share:${libadwaita.dev}/share";
              RUST_BACKTRACE = 1;
            };
        });
}
