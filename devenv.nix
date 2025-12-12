{
  pkgs,
  # lib,
  # config,
  # inputs,
  ...
}: {
  env = {
    # Needed for zstandard (native C/C++ library)
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
  };

  # Essential build dependencies
  packages = with pkgs; [
    clang # zstd
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
