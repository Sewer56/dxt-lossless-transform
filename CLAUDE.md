# Rules for DXT Lossless Transform

## Project Overview

This is a high-performance Rust library for lossless DXT texture compression with extensive SIMD optimizations and unsafe code for maximum performance. The project follows strict coding standards and performance requirements.

## Project Structure

The project is organized into several main directories under `/projects/`:

- **`/projects/core/`** - Houses the **unstable API** implementations
  - Contains the core transformation logic for each block format
  - Low-level crates without API stability guarantees
  - Optimized for maximum performance with frequent breaking changes

- **`/projects/api/`** - Houses the **stable API** 
  - Provides stable, backwards-compatible interfaces to the core functionality
  - Safe wrappers around the unstable core implementations
  - Recommended for external users and applications

- **`/projects/extensions/`** - Contains extensions (additions) that build on top of the stable API
  - Additional functionality like estimators and compressors
  - File format support and specialized utilities
  - All built using the stable API from `/projects/api/`

- **`/projects/tools/`** - Contains miscellaneous programs

## Code Style & Formatting

### Rust Formatting

- Use proper rustdoc format with elements in brackets like [`Color565`] instead of `Color565`
- Maintain original code order (including assembly sections) and all comments intact unless explicitly permitted
- Try to not exceed 500 lines of code per module, excluding tests. If that's not possible, split the module into multiple modules.

### Variable Names & Structure

- Preserve existing coding style: keep variable names and loop structures unchanged unless explicitly instructed otherwise
- Use descriptive names for performance-critical code
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)

### Import and Dependency Preferences

- Prefer `core` over `std` when possible for better no_std compatibility
- Prefer using short names and `use` statements at the top of the file.
- Only place `use` statements inside a function if that function has conditional compilation flags like `cfg`.

## Documentation Standards

### Function Documentation

- Use comprehensive rustdoc comments for all public functions
- Include detailed Safety sections for unsafe functions covering:
  - Pointer validity requirements
  - Memory alignment recommendations (16-byte minimum, 32-byte preferred)
  - Buffer overlap restrictions
  - Size requirements and divisibility constraints
- Include Parameters and Returns sections
- Add Remarks section for complex behaviors or performance notes

### Examples

- Include code examples in documentation when helpful
- Use `ignore` for examples that don't compile standalone
- Show both basic usage and safety requirements

## Performance Requirements

### Memory Management

- Prefer stack allocation when possible
- Use `allocate_align_64` for large aligned buffers

## Backwards Compatibility

- Our crates are not yet released; therefore, you may make backwards incompatible changes. Do not however rename methods unless it is necessary.

## Compilation

When building the `dxt-lossless-transform-cli` project, enable all features except for `nightly`, unless asked.

## Post-Change Verification

**CRITICAL: After making any code changes, ALWAYS perform these verification steps in order:**

0. **Verify Everything Compiles**: Run `cargo build --all-features --all-targets` to ensure there are no compilation errors
1. **Run Tests**: Execute `cargo test --all-features` to ensure all functionality works correctly
2. **Check Lints**: Run `cargo clippy --workspace --all-features -- -D warnings` to catch any warnings or issues
3. **Verify Documentation**: Run `cargo doc --workspace --all-features` to check for documentation errors
4. **Fix Documentation Links**: For any broken doc links, use the proper format: `` [`function_name`]: crate::function_name `` (e.g., `` [`dltbc1_free_ManualTransformBuilder`]: crate::dltbc1_free_ManualTransformBuilder ``)
5. **Big Endian Testing**: If `cross` is installed, run `cross test --package dxt-lossless-transform-dds --target powerpc64-unknown-linux-gnu` to test big endian compatibility (skip if `cross` is not available)
6. **Format Code**: Run `cargo fmt --all` as the final step to ensure consistent formatting

These steps are mandatory and must be completed successfully before considering any change complete.

## C API

If you are working with C APIs, read the rules in .cursor/rules/c_api.mdc