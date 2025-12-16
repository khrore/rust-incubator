{
  pkgs ? import <nixpkgs> { },
}:
let
  overrides = builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
  libPath =
    with pkgs;
    lib.makeLibraryPath [
      openssl
    ];
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    # needed for linker
    llvmPackages.clangWithLibcAndBasicRtAndLibcxx
    llvmPackages.bintools
    # fastest linker
    mold

    # Database tool for working with Rust projects that use Diesel
    diesel-cli

    # command-line json parser
    jq

    rustup
    pkg-config
    openssl
  ];

  LD_LIBRARY_PATH = libPath;

  # https://github.com/rust-lang/rust-bindgen#environment-variables
  LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];

  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
    export EDITOR=nvim

    rustup default $RUSTC_VERSION
  '';

  RUSTC_VERSION = overrides.toolchain.channel;

  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "clang";
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-Clink-arg=-fuse-ld=mold";
}
