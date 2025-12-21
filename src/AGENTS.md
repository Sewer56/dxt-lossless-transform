# dxt-lossless-transform

High-performance Rust library for lossless DXT texture compression with extensive SIMD optimizations.

# Project Structure

- `api/` - Stable API crates (recommended for external users)
  - `dxt-lossless-transform-api-common/` - Shared functionality
  - `dxt-lossless-transform-bc1-api/` - BC1 stable API
  - `dxt-lossless-transform-bc2-api/` - BC2 stable API
  - `dxt-lossless-transform-bc3-api/` - BC3 stable API
  - `dxt-lossless-transform-bc7-api/` - BC7 stable API
  - `dxt-lossless-transform-file-formats-api/` - File format handling
- `core/` - Unstable API implementations (low-level, frequent breaking changes)
  - `dxt-lossless-transform-bc1/` - BC1 implementation
  - `dxt-lossless-transform-bc2/` - BC2 implementation
  - `dxt-lossless-transform-bc3/` - BC3 implementation
  - `dxt-lossless-transform-bc7/` - BC7 implementation
  - `dxt-lossless-transform-common/` - Shared code
- `extensions/` - Extensions built on stable API
  - `file-formats/dxt-lossless-transform-dds/` - DDS support
  - `compressors/dxt-lossless-transform-zstd/` - ZStandard estimation
  - `estimators/dxt-lossless-transform-ltu/` - LTU estimation
- `tools/dxt-lossless-transform-cli/` - CLI tool
- `fuzz/` - Fuzz testing targets

# Code Guidelines

- Optimize for performance; use zero-cost abstractions, avoid allocations.
- Keep modules under 500 lines (excluding tests); split if larger.
- Place `use` inside functions only for `#[cfg]` conditional compilation.
- Prefer `core` over `std` where possible (`core::mem` over `std::mem`).
- Use [`TypeName`] rustdoc links, not backticks.

# Documentation Standards

- Document public items with `///`
- Add examples in docs where helpful
- Use `//!` for module-level docs
- Focus comments on "why" not "what"
- Include Safety sections for unsafe functions

# Post-Change Verification

```bash
cargo build --workspace --all-features --all-targets --quiet
cargo test --workspace --all-features --quiet
cargo clippy --workspace --all-features --quiet -- -D warnings
cargo doc --workspace --all-features --quiet
cargo fmt --all --quiet
```

All must pass before submitting.
