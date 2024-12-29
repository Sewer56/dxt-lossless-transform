/*
 * BC3/DXT5 Block Rearrangement Optimization Explanation
 * ==================================================
 *
 * Original sequential BC3 data layout:
 * 2 bytes of alpha endpoints followed by 6 bytes of alpha indices, then two 16-bit colours (4 bytes)
 * and 4 bytes of color indices:
 *
 * Address: 0       2       8       12      16  16      18      24      28      32
 *          +-------+-------+-------+-------+   +-------+-------+-------+-------+
 * Data:    |A0-A1  |AIdx0-47|C0-C1 |I0-I15 |  |A2-A3  |AIdx48-95|C2-C3 |I16-I31|
 *          +-------+-------+-------+-------+   +-------+-------+-------+-------+
 *
 * Each 16-byte block contains:
 * - 2 bytes of alpha endpoints (min/max alpha values for interpolation)
 * - 6 bytes of alpha indices (sixteen 3-bit indices for alpha interpolation)
 * - 4 bytes colours (2x RGB565 values)
 * - 4 bytes of packed color indices (sixteen 2-bit indices)
 *
 * Optimized layout separates alpha endpoints, alpha indices, colours and indices into continuous streams:
 *
 * +-------+-------+-------+     +-------+  } Alpha endpoints section
 * | A0-A1 | A2-A3 | A4-A5 | ... | AN    |  } (2 bytes per block: 2x 8-bit)
 * +-------+-------+-------+     +-------+
 * +-------+-------+-------+     +-------+  } Alpha indices section
 * |AI0-47 |AI48-95|  ...  | ... |AI N   |  } (6 bytes per block: 16x 3-bit)
 * +-------+-------+-------+     +-------+
 * +-------+-------+-------+     +-------+  } Colours section
 * |C0  C1 |C2  C3 |C4  C5 | ... |CN CN+1|  } (4 bytes per block: 2x RGB565)
 * +-------+-------+-------+     +-------+
 * +-------+-------+-------+     +-------+  } Indices section
 * | Idx0  | Idx1  | Idx2  | ... | IdxN  |  } (4 bytes per block: 16x 2-bit)
 * +-------+-------+-------+     +-------+
 *
 * This rearrangement improves compression because:
 * 1. Alpha endpoints often have high spatial coherency
 * 2. Alpha index patterns tend to repeat across similar regions
 * 3. Color endpoints tend to be spatially coherent
 * 4. Color index patterns often repeat across blocks
 * 5. Separating them allows better compression of each stream
 */

//pub mod detransform;
pub mod transform;
