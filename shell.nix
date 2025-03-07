#{ pkgs ? import <nixpkgs> {overlays} }:
let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  pkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
in
pkgs.mkShell rec {
  buildInputs = with pkgs; [ latest.rustChannels.stable.rust cargo rustfmt rust-analyzer rustPackages.clippy wayland libxkbcommon pkg-config ];
  RUST_BACKTRACE = 1;
  LD_LIBRARY_PATH = builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" buildInputs;
}
/*
  with nixpkgs;
  stdenv.mkDerivation {
    name = "moz_overlay_shell";
    buildInputs = [
      # to use the latest nightly:
      nixpkgs.latest.rustChannels.nightly.rust
      # to use a specific nighly:
      (nixpkgs.rustChannelOf { date = "2018-04-11"; channel = "nightly"; }).rust
      # to use the project's rust-toolchain file:
      (nixpkgs.rustChannelOf { rustToolchain = ./rust-toolchain; }).rust
    ];
  }
*/
