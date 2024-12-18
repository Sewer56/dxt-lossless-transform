/*
 * DXT1 Block Rearrangement Optimization Explanation
 * =================================================
 *
 * Original sequential DXT1 data layout:
 * Two 16-bit colours (4 bytes total) followed by 4 bytes of indices:
 *
 * Address: 0       4       8   8      12      16
 *          +-------+-------+   +-------+-------+
 * Data:    | C0-C1 | I0-I3 |   | C2-C3 | I4-I8 |
 *          +-------+-------+   +-------+-------+
 *
 * Each 8-byte block contains:
 * - 4 bytes colours (2x RGB565 values)
 * - 4 bytes of packed indices (sixteen 2-bit indices)
 *
 * Optimized layout separates colours and indices into continuous streams:
 *
 * +-------+-------+-------+     +-------+  } colours section
 * |C0  C1 |C2  C3 |C4  C5 | ... |CN CN+1|  } (4 bytes per block: 2x RGB565)
 * +-------+-------+-------+     +-------+
 * +-------+-------+-------+     +-------+  } Indices section
 * | Idx0  | Idx1  | Idx2  | ... | IdxN  |  } (4 bytes per block: 16x 2-bit)
 * +-------+-------+-------+     +-------+
 *
 * This rearrangement improves compression because:
 * 1. Color endpoints tend to be spatially coherent
 * 2. Index patterns often repeat across blocks
 * 3. Separating them allows better compression of each stream
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

pub mod transform;
pub use transform::*;
