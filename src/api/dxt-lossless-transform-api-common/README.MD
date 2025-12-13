# DXT Lossless Transform API Common

Common utilities and re-exports for the DXT lossless transform API crates.

## Features

### Memory Allocation

The `allocate` module provides cache line aligned memory allocation that automatically selects
appropriate alignment values based on the target architecture:

- **x86/x86_64**: 64-byte alignment  
- **aarch64**: 64-byte alignment
- **Other architectures**: 64-byte alignment

```rust
use dxt_lossless_transform_api_common::allocate::allocate_cache_line_aligned;

// Allocate 1024 bytes aligned to cache line boundary
let allocation = allocate_cache_line_aligned(1024).unwrap();
let ptr = allocation.as_ptr();
```