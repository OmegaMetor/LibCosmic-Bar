{
  pkgs ? import <nixos> {
    overlays = [
      (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"))
    ];
  },
}:
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    wayland
    libxkbcommon
    pkg-config
    (rust-bin.stable.latest.default.override {
      extensions = [ "rust-src" ];
    })
  ];

  RUST_BACKTRACE = 1;
  LD_LIBRARY_PATH = builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" buildInputs;
}
