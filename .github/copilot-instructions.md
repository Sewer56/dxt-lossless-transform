# Cursor Rules for DXT Lossless Transform

## Project Overview

This is a high-performance Rust library for lossless DXT texture compression with extensive SIMD optimizations and unsafe code for maximum performance. The project follows strict coding standards and performance requirements.

## Code Style & Formatting

### Rust Formatting

- Always use `cargo fmt` for code formatting
- Run `cargo clippy` and fix all warnings before submitting code
- Use proper rustdoc format with elements in brackets like [`Color565`] instead of `Color565`
- Maintain original code order (including assembly sections) and all comments intact unless explicitly permitted

### Variable Names & Structure

- Preserve existing coding style: keep variable names and loop structures unchanged unless explicitly instructed otherwise
- Use descriptive names for performance-critical code
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)

### Import and Dependency Preferences

- Prefer `core` over `std` when possible for better no_std compatibility
- Prefer adding explicit `use` statements instead of fully qualified paths, unless the code is inside a feature block that blocks/enables compilation

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

## C API Naming Conventions

### Function Naming Pattern

C API functions follow a consistent naming pattern with PascalCase for types and methods:

- **Creation/Destruction Functions**: `dltbc1_{action}_{TypeName}`
  - `dltbc1_new_EstimateOptionsBuilder`
  - `dltbc1_free_EstimateOptionsBuilder`
  - `dltbc1_new_TransformContext`
  - `dltbc1_free_TransformContext`
  - `dltbc1_clone_TransformContext`

- **Type-Associated Methods**: `dltbc1_{TypeName}_{Method}`
  - `dltbc1_EstimateOptionsBuilder_SetUseAllDecorrelationModes`
  - `dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal`
  - `dltbc1_TransformContext_SetDecorrelationMode`
  - `dltbc1_TransformContext_SetSplitColourEndpoints`
  - `dltbc1_TransformContext_GetSplitColourEndpoints`
  - `dltbc1_TransformContext_ResetToDefaults`

- **Action Functions**: `dltbc1_{action}`
  - `dltbc1_transform`
  - `dltbc1_untransform`
  - `dltbc1_error_message`

Action functions should be reserved to functions which are ABI unstable, i.e. those whose parameters
may change between versions.

### Type Name Preservation

- Keep type names with their original capitalization (e.g., `EstimateOptionsBuilder`, `TransformContext`)
- Use PascalCase for method names (e.g., `SetDecorrelationMode`, `BuildAndDetermineOptimal`)
- Group methods with their associated types for easy identification

## Performance Requirements

### Memory Management

- Prefer stack allocation when possible
- Use `allocate_align_64` for large aligned buffers

## Compilation

When building the `dxt-lossless-transform-cli` project, enable all features except for `nightly`, unless asked.

## Testing Requirements

### Post-Change Verification

**CRITICAL: After making any code changes, ALWAYS perform these verification steps in order:**

1. **Run Tests**: Execute `cargo test --all-features` to ensure all functionality works correctly
2. **Check Lints**: Run `cargo clippy --workspace --all-features -- -D warnings` to catch any warnings or issues
3. **Verify Documentation**: Run `cargo doc --workspace --all-features` to check for documentation errors
4. **Fix Documentation Links**: For any broken doc links, use the proper format: `` [`function_name`]: crate::function_name `` (e.g., `` [`dltbc1_free_ManualTransformBuilder`]: crate::dltbc1_free_ManualTransformBuilder ``)
5. **Big Endian Testing**: If `cross` is installed, run `cross test --package dxt-lossless-transform-dds --target powerpc64-unknown-linux-gnu` to test big endian compatibility (skip if `cross` is not available)
6. **Format Code**: Run `cargo fmt --all` as the final step to ensure consistent formatting

These steps are mandatory and must be completed successfully before considering any change complete.