let
  pkgs = import (fetchTarball("channel:nixpkgs-unstable")) {};

mesonNew = pkgs.meson.overrideAttrs (oldAttrs: rec {
  pname = "meson";
  version = "0.60.2";
  src = pkgs.fetchFromGitHub{
    owner = "mesonbuild";
    repo = "meson";
    rev = "0.60.2";
    sha256 = "1z68zivpn1c6x34a037ibbp3jzxrhl5a8xz8ihwqc6k6i6nxpq3p";
  };
  patches = [ ];
});

in pkgs.mkShell {
  buildInputs = [ 
    mesonNew
    pkgs.cargo
    pkgs.cargo-outdated 
    pkgs.gtk4.dev
    pkgs.libadwaita.dev
    pkgs.ninja
    pkgs.pkg-config
    pkgs.rustc
    pkgs.rustfmt
    pkgs.tracker.dev
    pkgs.wayland.dev
  ];

  # Certain Rust tools won't work without this
  # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
  # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
