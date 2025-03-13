Switch Colourspace?
===================

I always wondered why video capture on desktop uses YCrCb, e.g. in OBS.
Apparently it's encoding a difference: https://www.computerlanguage.com/results.php?definition=YCrCb
from one component to another.

In theory, YCrCb.

Rearrange Items
===============

So far I've been trying to shift the items by moving bits from one field to another,
that actually risks hurting entropy. In particular, I was trying to move the p bits
after the partition bits, but these are high entropy. So I was hurting ratio.


Split Blocks
============

Just like the BC1-BC3 blocks; there are components wiith mixed entropy, such as colours.

Rotation Bits
============
https://eternaldevelopments.com/BCDecoder

This allows you to use higher precision for a colour channel.

Interleaving
============

Interleaving the colours, i.e. `R0 G0 B0 R1 G1 B1 ...` from `R0 R1 R2 G0 G1 G2 ...`
usually improves ratio if the colours make up at least 3 bytes (min lz match).


General Observations
============

- Always byte align fields if possible. This is best for LZ matches.
- Byte align lowest entropy/highest LZ fields first.