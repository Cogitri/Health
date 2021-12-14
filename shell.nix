let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  pkgs = import (fetchTarball("channel:nixpkgs-unstable")) { overlays = [ moz_overlay ]; };

mesonNew = pkgs.meson.overrideAttrs (old: rec {
  version = "0.60.2";

  src = pkgs.fetchFromGitHub{
    owner = "mesonbuild";
    repo = "meson";
    rev = "0.60.2";
    sha256 = "1z68zivpn1c6x34a037ibbp3jzxrhl5a8xz8ihwqc6k6i6nxpq3p";
  };

  patches = (pkgs.lib.take 1 old.patches) ++ [ ./build-aux/ldconfig.patch ./build-aux/more-env-vars.patch ./build-aux/gir-fallback-path.patch ]
    ++ (pkgs.lib.take 2 (pkgs.lib.drop 3 old.patches));
});

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
    mesonNew
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
buildInputs = [ 
  adwaitaNew
  gtkPatched
  mesonNew
  rustSrc
  pkgs.cairo
  pkgs.cargo-outdated 
  pkgs.clang_13
  pkgs.harfbuzz
  pkgs.gdk_pixbuf
  pkgs.glib
  pkgs.graphene
  pkgs.gtk4.dev
  pkgs.libxml2
  pkgs.mold
  pkgs.ninja
  pkgs.pango
  pkgs.pkg-config
  pkgs.rustc
  pkgs.rustfmt
  pkgs.tracker
  pkgs.tracker.dev
  pkgs.wayland
  pkgs.wayland.dev
];

in pkgs.mkShell {
  buildInputs = buildInputs;


  RUST_SRC_PATH= "${rustSrc}/lib/rustlib/src/rust/src";
  RUSTFLAGS="-C linker=clang -C link-arg=--ld-path=${pkgs.mold}/bin/mold";
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
}
