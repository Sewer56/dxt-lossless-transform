name: "Base PRP Template v2 - Context-Rich with Validation Loops"
description: |

## Purpose
Template optimized for AI agents to implement features with sufficient context and self-validation capabilities to achieve working code through iterative refinement.

## Core Principles
1. **Context is King**: Include ALL necessary documentation, examples, and caveats
2. **Validation Loops**: Provide executable tests/lints the AI can run and fix
3. **Information Dense**: Use keywords and patterns from the codebase
4. **Progressive Success**: Start simple, validate, then enhance
5. **Global rules**: Be sure to follow all rules in CLAUDE.md

---

## Goal
[What needs to be built - be specific about the end state and desires]

## Why
- [Business value and user impact]
- [Integration with existing features]
- [Problems this solves and for whom]

## What
[User-visible behavior and technical requirements]

### Success Criteria
- [ ] [Specific measurable outcomes]

## All Needed Context

### Documentation & References (list all context needed to implement the feature)
```yaml
# MUST READ - Include these in your context window
- url: [Official API docs URL]
  why: [Specific sections/methods you'll need]
  
- file: [path/to/example.rs]
  why: [Pattern to follow, gotchas to avoid]
  
- doc: [Library documentation URL] 
  section: [Specific section about common pitfalls]
  critical: [Key insight that prevents common errors]

- docfile: [PRPs/ai_docs/file.md]
  why: [docs that the user has pasted in to the project]

```

### Current Codebase tree (run `tree` in the root of the project) to get an overview of the codebase
```bash

```

### Desired Codebase tree with files to be added and responsibility of file
```bash

```

### Known Gotchas of our codebase & Library Quirks
```rust
// CRITICAL: [Crate name] requires [specific setup]
// Example: Comprehensive Safety docs required for unsafe functions
// Example: Prefer stack allocation, use allocate_align_64 for large buffers
```

## Implementation Blueprint

### Data models and structure

Create the core data models, we ensure type safety and consistency.
```rust
Examples: 
 - struct definitions with proper derives
 - enum variants for state representation
 - trait implementations for behavior
 - type aliases for clarity
 - #[repr(C)] for FFI-safe structs
 - const generics for compile-time parameters

```

### list of tasks to be completed to fullfill the PRP in the order they should be completed

```yaml
Task 1:
MODIFY src/existing_module.rs:
  - FIND pattern: "struct OldImplementation"
  - INJECT after line containing "impl OldImplementation"
  - PRESERVE existing method signatures and documentation

CREATE src/new_feature.rs:
  - MIRROR pattern from: src/similar_feature.rs
  - MODIFY struct name and core logic
  - KEEP error handling pattern identical
  - ADD module declaration to lib.rs or parent mod.rs

...(...)

Task N:
...

```


### Per task pseudocode as needed added to each task
```rust

// Task 1
// Pseudocode with CRITICAL details dont write entire code
fn new_feature(param: &str) -> Result<FeatureResult, FeatureError> {
    // PATTERN: Always validate input first (see src/validators.rs)
    let validated = validate_input(param)?; // returns ValidationError
    
    // GOTCHA: This crate requires specific buffer alignment
    let buffer = allocate_align_64(BUFFER_SIZE)?; // see src/allocate.rs
    
    // PATTERN: Use existing error handling patterns
    let result = match external_operation(validated) {
        Ok(data) => {
            // CRITICAL: Buffer must be exactly BLOCK_SIZE aligned
            process_blocks(&data, &mut buffer)?
        }
        Err(e) => return Err(FeatureError::Processing(e)),
    };
    
    // PATTERN: Standardized response format
    Ok(format_result(result)) // see src/utils/responses.rs
}
```

### Integration Points

```yaml
CARGO_TOML:
  - add feature: "new-feature = []"
  - add dependency: "new-dep = { version = \"1.0\", optional = true }"
  
MODULE_EXPORTS:
  - add to: src/lib.rs
  - pattern: "pub mod new_feature;"
  - pattern: "pub use new_feature::*;"
  
C_API:
  - add to: src/c_api/mod.rs
  - pattern: "pub extern \"C\" fn dltbc1_new_feature(...)"
  - add to: cbindgen_c.toml for header generation
  
FEATURES:
  - update: Cargo.toml features section
  - conditional compilation: "#[cfg(feature = \"new-feature\")]"
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run these FIRST - fix any errors before proceeding
cargo fmt                           # Auto-format code
cargo clippy -- -D warnings         # Linting with warnings as errors
cargo check                          # Basic compilation check

# Expected: No errors. If errors, READ the error and fix.
```

### Level 2: Unit Tests each new feature/file/function use existing test patterns
```rust
// ADD to existing file which contains new_feature
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path() {
        // Basic functionality works
        let result = new_feature("valid_input").unwrap();
        assert_eq!(result.status, Status::Success);
    }

    #[test]
    fn test_validation_error() {
        // Invalid input returns ValidationError
        let result = new_feature("");
        assert!(matches!(result, Err(FeatureError::Validation(_))));
    }
}
```

```bash
# Run and iterate until passing:
cargo test
cargo test test_new_feature -- --nocapture  # For specific test with output
# If failing: Read error, understand root cause, fix code, re-run
```

## Final validation Checklist
- [ ] All tests pass: `cargo test --all-features`
- [ ] No linting errors: `cargo clippy`
- [ ] No compilation errors: `cargo check`
- [ ] Documentation builds: `cargo doc`
- [ ] Code is formatted: `cargo fmt`
- [ ] Manual test successful: [specific cargo run command]
- [ ] Error cases handled gracefully with proper Result types
- [ ] Safety documentation complete for unsafe code
- [ ] C API headers generated if applicable: `cbindgen`

---

## Anti-Patterns to Avoid

- ❌ Don't create new patterns when existing ones work
- ❌ Don't skip validation because "it should work"  
- ❌ Don't ignore failing tests - fix them
- ❌ Don't use `unwrap()` or `expect()` in library code - use proper error handling
- ❌ Don't hardcode values that should be constants or configuration
- ❌ Don't use `unsafe` without comprehensive safety documentation
- ❌ Don't ignore clippy warnings - address them or use `#[allow()]` with justification
- ❌ Don't allocate when stack allocation is sufficient