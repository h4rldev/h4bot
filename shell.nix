with import <nixpkgs> {};

let
  pkgs = import <nixpkgs> {};
in
mkShell {
  allowUnfree = true;
  name = "h4bot";
  nativeBuildInputs = with pkgs; [
    rustup pkgconfig
  ];
  buildInputs = with pkgs; [
    openssl
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath [ openssl ];
  packages = with pkgs; [
    zsh
    trunk-io
  ];
  shellHook = ''
    echo "Welcome to h4bot's nix-shell :)"
  '';
  # Additional configuration (if needed)
  RUST_BACKTRACE = 1;
}