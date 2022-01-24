let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  pkgs = import (fetchTarball("https://github.com/nixos/nixpkgs/archive/a6928c74cc64a141051160d1cdbbc3d2c1bbece2.tar.gz")) { overlays = [ moz_overlay ]; };

  rustSrc =
    pkgs.latest.rustChannels.stable.rust.override { extensions = [ "rust-src" ]; };
  buildInputs = with pkgs; [ 
    appstream-glib
    cairo
    cargo-audit
    cargo-bloat
    cargo-dephell
    cargo-expand
    cargo-outdated
    clang_13
    desktop-file-utils
    gdb
    gdk_pixbuf
    glib
    glib
    graphene
    gtk4.dev
    gtk4
    harfbuzz
    libadwaita
    librsvg
    libxml2
    lld_13
    meson
    ninja
    pango
    pkg-config
    python3
    rustfmt
    rustSrc
    tracker
    tracker.dev
    valgrind
    wayland
    wayland.dev
  ];
in pkgs.mkShell {
  buildInputs = buildInputs;


  RUST_SRC_PATH = "${rustSrc}/lib/rustlib/src/rust/src";
  RUSTFLAGS="-C linker=clang -C link-arg=--ld-path=${pkgs.lld_13}/bin/ld.lld";
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
  XDG_DATA_DIRS = with pkgs; "${gtk4.dev}/share:{libadwaita.dev}/share:${gdk_pixbuf.dev}/share:${gobject-introspection.dev}/share:${pango.dev}/share:${harfbuzz.dev}/share:${graphene}/share:${libadwaita.dev}/share";
  RUST_BACKTRACE = 1;
}
