/*
 * BC2/DXT3 Block Rearrangement Optimization Explanation
 * ==================================================
 *
 * Original sequential BC2 data layout:
 * 8 bytes of alpha values followed by two 16-bit colours (4 bytes) and 4 bytes of color indices:
 *
 * Address: 0       8       12      16  16      24      28      32
 *          +-------+-------+-------+   +-------+-------+--------+
 * Data:    |A0-A15 | C0-C1 | I0-I15 |  |A16-A31| C2-C3 | I6-I31 |
 *          +-------+-------+-------+   +-------+-------+--------+
 *
 * Each 16-byte block contains:
 * - 8 bytes of explicit alpha (sixteen 4-bit alpha values)
 * - 4 bytes colours (2x RGB565 values)
 * - 4 bytes of packed color indices (sixteen 2-bit indices)
 *
 * Optimized layout separates alpha, colours and indices into continuous streams:
 *
 * +-------+-------+-------+     +-------+  } Alpha section
 * | A0    | A1    | A2    | ... | AN    |  } (8 bytes per block: 16x 4-bit)
 * +-------+-------+-------+     +-------+
 * +-------+-------+-------+     +-------+  } Colours section
 * |C0  C1 |C2  C3 |C4  C5 | ... |CN CN+1|  } (4 bytes per block: 2x RGB565)
 * +-------+-------+-------+     +-------+
 * +-------+-------+-------+     +-------+  } Indices section
 * | Idx0  | Idx1  | Idx2  | ... | IdxN  |  } (4 bytes per block: 16x 2-bit)
 * +-------+-------+-------+     +-------+
 *
 * This rearrangement improves compression because:
 * 1. Alpha values often have high spatial coherency
 * 2. Color endpoints tend to be spatially coherent
 * 3. Index patterns often repeat across blocks
 * 4. Separating them allows better compression of each stream
 *
 * Key differences from BC1/DXT1:
 * - Blocks are 16 bytes instead of 8 bytes
 * - Includes explicit 4-bit alpha values (no alpha interpolation)
 * - No special "transparent black" color combinations
 *
 * Requirements
 * ============
 *
 * A second, separate buffer to receive the results.
 *
 * While doing it in-place is technically possible, and would be beneficial in the sense that there
 * would be improved cache locality; unfortunately, that is not possible to do in a 'single pass'
 * while maintaining the spatial coherency/order.
 *
 * Introducing a second pass meanwhile would be a performance hit.
 *
 * This is possible to do with either allocating half of a buffer, and then copying the other half back,
 * or outputting it all to a single buffer. Outputting all to single buffer is faster.
 */
