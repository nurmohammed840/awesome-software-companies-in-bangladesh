{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  packages = with pkgs; [
    # MUSL cross toolchain
    pkgsCross.musl64.stdenv.cc
  ];

  shellHook = ''
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-unknown-linux-musl-gcc
  '';
}
