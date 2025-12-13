{
  pkgs,
  # lib,
  # config,
  # inputs,
  ...
}: {
  env = {
    # fix bindgen
    # Needed for zstandard (native C/C++ library)
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

    # fix general build of i686-unknown-linux-gnu targets
    # Use multilib clang as linker for i686 target.
    # Required because Rust's bundled LLD doesn't know about Nix's multilib sysroot paths.
    # clang_multi has the correct 32-bit library paths (glibc_multi) baked into its driver.
    CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.clang_multi}/bin/clang";

    # Fix `cargo bench` for i686-unknown-linux-gnu
    # Disable LTO for C/C++ code compiled by the `cc` crate for i686.
    # Some crates (like `alloca`) add -flto when CC=clang, but binutils ld can't
    # process LLVM IR bitcode. Using -fno-lto ensures native object files.
    CFLAGS_i686_unknown_linux_gnu = "-fno-lto";
    CXXFLAGS_i686_unknown_linux_gnu = "-fno-lto";

    # Add 32-bit library paths for linking (needed for libstdc++, libgcc_s).
    # Required for fuzz targets and other C++ code that links against libstdc++.
    CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-L ${pkgs.pkgsi686Linux.stdenv.cc.cc.lib}/lib";
  };

  # Essential build dependencies
  packages = with pkgs; [
    # clang with multilib support (both 32-bit and 64-bit)
    # This properly handles -m32 compilation with correct headers/libs
    clang_multi
  ];

  # Rust language configuration with nightly toolchain
  languages.rust = {
    enable = true;
    channel = "nightly";

    # Rust components
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
      "rust-src"
      "rust-docs"
    ];

    # Cross-compilation targets for testing
    targets = [
      "x86_64-unknown-linux-gnu"
      "i686-unknown-linux-gnu"
      "powerpc64-unknown-linux-gnu" # For big-endian testing
    ];
  };

  # Development scripts
  scripts = {
    dev-test.exec = ''
      echo "Running comprehensive test suite..."
      cargo test --all-features
    '';

    dev-check.exec = ''
      echo "Running code quality checks..."
      cargo clippy --workspace --all-features -- -D warnings
      cargo fmt --all -- --check
      cargo doc --workspace --all-features
    '';

    dev-bench.exec = ''
      echo "Running benchmarks..."
      cargo bench --all-features
    '';

    dev-cross-test.exec = ''
      if command -v cross >/dev/null 2>&1; then
        echo "Running cross-compilation tests..."
        cross test --package dxt-lossless-transform-dds --target powerpc64-unknown-linux-gnu
      else
        echo "Cross not available, install with: cargo install cross --git https://github.com/cross-rs/cross"
      fi
    '';

    dev-full-check.exec = ''
      echo "Running full verification pipeline..."
      dev-test && dev-check && echo "✅ All checks passed!"
    '';
  };

  env.CROSS_CUSTOM_TOOLCHAIN = "1";

  # Shell initialization
  enterShell = ''
    echo "🦀 DXT Lossless Transform Development Environment"
    echo "Rust version: $(rustc --version)"
    echo "Cargo version: $(cargo --version)"
    echo ""
    echo "Available scripts:"
    echo "  dev-test      - Run test suite"
    echo "  dev-check     - Run linting and formatting checks"
    echo "  dev-bench     - Run benchmarks"
    echo "  dev-cross-test - Run cross-compilation tests"
    echo "  dev-full-check - Run complete verification pipeline"
    echo ""
    echo "Cross-compilation targets configured:"
    echo "  x86_64-unknown-linux-gnu"
    echo "  i686-unknown-linux-gnu (32-bit)"
    echo "  powerpc64-unknown-linux-gnu (big-endian)"
  '';

  # Testing configuration
  enterTest = ''
    echo "Running devenv tests..."
    rustc --version | grep -q "nightly"
    cargo --version
    dev-full-check
  '';

  # Git hooks for code quality
  git-hooks.hooks = {
    # Rust formatting
    rustfmt.enable = true;

    # Clippy linting
    clippy = {
      enable = true;
      settings = {
        allFeatures = true;
        denyWarnings = true;
      };
    };
  };
}
