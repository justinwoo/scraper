{ pkgs ? import <nixpkgs> {} }:

let
  dynamic-linker = pkgs.stdenv.cc.bintools.dynamicLinker;

in
pkgs.stdenv.mkDerivation rec {
  name = "scraper";

  src = pkgs.fetchurl {
    url = "https://github.com/justinwoo/scraper/releases/download/2020-01-31/scraper";
    sha256 = "1k6cpglajq9zid6a3kql6qrb05mai41caw6q253x66vqs5wvcm01";
  };

  buildInputs = [ pkgs.makeWrapper ];

  dontStrip = true;

  libPath = pkgs.lib.makeLibraryPath [
    pkgs.glibc
    pkgs.openssl_1_1.out
  ];

  unpackPhase = ''
    mkdir -p $out/bin
    TARGET=$out/bin/scraper

    cp $src $TARGET
    chmod +x $TARGET

    patchelf $TARGET \
      --interpreter ${dynamic-linker} \
      --set-rpath ${libPath}
  '';

  dontInstall = true;
}
