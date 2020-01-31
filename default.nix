{ pkgs ? import <nixpkgs> {} }:

let
  dynamic-linker = pkgs.stdenv.cc.bintools.dynamicLinker;

in
pkgs.stdenv.mkDerivation rec {
  name = "scraper";

  src = pkgs.fetchurl {
    url = "https://github.com/justinwoo/scraper/releases/download/2019-12-29/scraper";
    sha256 = "1x8s3qj611lnf4djzhird0mxmrrj5kq319czfwbplpnmhp3z25wg";
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
