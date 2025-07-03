## FEATURE:

I want the /home/sewer/Project/dxt-lossless-transform/projects/core/dxt-lossless-transform-bc2 crate
to have its code structure mirror that of /home/sewer/Project/dxt-lossless-transform/projects/core/dxt-lossless-transform-bc1.

To be precise, I want you to reimplement as much of /home/sewer/Project/dxt-lossless-transform/projects/core/dxt-lossless-transform-bc1 code structure as possible, such that the functionality between the two crates is identical. For algorithms that are not yet implemented in BC2, such as `transform/with_split_colour`, 
you should generate all the methods and unit tests, however leave them, and their tests stubbed out. 
That means, providing all the method signatures, the documentation, and the unit tests. The unit tests (for now),
should default to passing by having an early return, but the remaining code in the test should be implemented.

## OTHER CONSIDERATIONS:

- Ignore unnecessary changes in the `experimental` module. We do not need BC2 equivalents of transform_bc1_with_normalize_blocks ; or similar. Leave the experimental module as is.
