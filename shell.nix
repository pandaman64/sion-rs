with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "sion-rs";
  buildInputs = [
    bashInteractive
    rustup
    pkgconfig
    openssl
  ];
}
