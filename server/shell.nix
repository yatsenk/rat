{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  name = "client-dev-shell";

  nativeBuildInputs = with pkgs; [
    pkg-config
    rustc
    cargo
    rustfmt
    clippy
  ];

  buildInputs = with pkgs; [
    libX11
    libXext
    libXfixes
    libXtst
    libXi
    libXrandr
    libxcb

    xdotool

    openssl
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.libX11
    pkgs.libXext
    pkgs.libXfixes
    pkgs.libXtst
    pkgs.libXi
    pkgs.libXrandr
    pkgs.libxcb
    pkgs.openssl
  ];

  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

  shellHook = ''
    echo "Rust dev shell ready. Run 'cargo build' to compile."
  '';
}
