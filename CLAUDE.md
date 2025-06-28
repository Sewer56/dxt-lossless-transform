# Rules for DXT Lossless Transform

## Project Overview

This is a high-performance Rust library for lossless DXT texture compression with extensive SIMD optimizations and unsafe code for maximum performance. The project follows strict coding standards and performance requirements.

## Code Style & Formatting

### Rust Formatting

- Use proper rustdoc format with elements in brackets like [`Color565`] instead of `Color565`
- Maintain original code order (including assembly sections) and all comments intact unless explicitly permitted

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

## Compilation

When building the `dxt-lossless-transform-cli` project, enable all features except for `nightly`, unless asked.

## Post-Change Verification

**CRITICAL: After making any code changes, ALWAYS perform these verification steps in order:**

1. **Run Tests**: Execute `cargo test --all-features` to ensure all functionality works correctly
2. **Check Lints**: Run `cargo clippy --workspace --all-features -- -D warnings` to catch any warnings or issues
3. **Verify Documentation**: Run `cargo doc --workspace --all-features` to check for documentation errors
4. **Fix Documentation Links**: For any broken doc links, use the proper format: `` [`function_name`]: crate::function_name `` (e.g., `` [`dltbc1_free_ManualTransformBuilder`]: crate::dltbc1_free_ManualTransformBuilder ``)
5. **Format Code**: Run `cargo fmt --all` as the final step to ensure consistent formatting

These steps are mandatory and must be completed successfully before considering any change complete.

## C API

If you are working with C APIs, read the rules in .cursor/rules/c_api.mdc