{ pkgs ? import ./pinned.nix {} }:

let
  includePaths = pkgs.lib.makeLibraryPath [
    pkgs.glibc
    pkgs.openssl_1_1.out
  ];

in
pkgs.mkShell {
  shellHook = ''
    export LIBRARY_PATH=${includePaths}
    export C_INCLUDE_PATH="${pkgs.openssl_1_1.dev.outPath}/include"
  '';
}
