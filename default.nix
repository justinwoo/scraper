{ pkgs ? import ./pinned.nix {} }:

let
  dynamic-linker = pkgs.stdenv.cc.bintools.dynamicLinker;

in
pkgs.stdenv.mkDerivation rec {
  name = "scraper";

  src = pkgs.fetchurl {
    url = "https://github.com/justinwoo/scraper/releases/download/2020-02-14/scraper";
    sha256 = "0rndf5yil02nw66v2mmqdjl4iki0i3wabhzc2z4n4f0rsnmcawkv";
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
