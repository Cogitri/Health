let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  pkgs = import (fetchTarball("channel:nixpkgs-unstable")) { overlays = [ moz_overlay ]; };

gtkPatched = pkgs.gtk4.overrideAttrs (old: rec {
  patches = [ ./build-aux/4136.patch ];
});

adwaitaNew = pkgs.libadwaita.overrideAttrs (oldAttrs: rec {
  version = "1.0.0.alpha.4";
  nativeBuildInputs = with pkgs; [
    docbook-xsl-nons
    gi-docgen
    gtk-doc
    libxml2 # for xmllint
    meson_0_60
    ninja
    pkg-config
    sassc
    vala
  ];
  src = pkgs.fetchFromGitLab {
    domain = "gitlab.gnome.org";
    owner = "GNOME";
    repo = "libadwaita";
    rev = version;
    sha256 = "0c4llrzrgnvn5qdg4ng5alxcs28zi767ds027nda95wjl82mx9fx";
  };
});
rustSrc =
    pkgs.latest.rustChannels.stable.rust.override { extensions = [ "rust-src" ]; };
buildInputs = with pkgs; [ 
  adwaitaNew
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
  gtkPatched
  harfbuzz
  libxml2
  meson_0_60
  mold
  ninja
  pango
  pkg-config
  rustfmt
  rustSrc
  tracker
  tracker.dev
  wayland
  wayland.dev
  valgrind
];

in pkgs.mkShell {
  buildInputs = buildInputs;


  RUST_SRC_PATH= "${rustSrc}/lib/rustlib/src/rust/src";
  RUSTFLAGS="-C linker=clang -C link-arg=--ld-path=${pkgs.mold}/bin/mold";
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
}
