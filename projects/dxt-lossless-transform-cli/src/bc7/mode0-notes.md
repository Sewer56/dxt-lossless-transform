Mode0

## Vanilla

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  116.09 MiB      59.62%            13.3s       14.69 MB/s
zstd 22    126.69 MiB      65.06%            25.0s       7.78 MB/s

==================== XOR ENCODE ===================

Based on partition, trying to make all bits 0 by xor'ing with 1 when probability of 1 is >50%.

Compressor Results:
Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  118.85 MiB      61.04%            13.4s       14.56 MB/s
zstd 22    127.33 MiB      65.39%            25.2s       7.74 MB/s

70%

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  116.63 MiB      59.90%            13.3s       14.69 MB/s
zstd 22    126.86 MiB      65.15%            25.3s       7.70 MB/s

80%

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  116.14 MiB      59.64%            13.4s       14.59 MB/s
zstd 22    126.62 MiB      65.02%            25.4s       7.68 MB/s

50% every 65K chunk

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  131.83 MiB      67.70%            14.1s       13.80 MB/s
zstd 22    136.59 MiB      70.15%            23.3s       8.36 MB/s

50% every 4K chunk

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  136.25 MiB      69.97%            14.3s       13.61 MB/s
zstd 22    141.40 MiB      72.62%            22.7s       8.56 MB/s

50% every 1M

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  125.30 MiB      64.35%            13.9s       13.99 MB/s
zstd 22    131.18 MiB      67.37%            24.5s       7.95 MB/s

50% evert 4K, train on last data

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  137.06 MiB      70.39%            14.5s       13.45 MB/s
zstd 22    139.89 MiB      71.84%            21.9s       8.89 MB/s

50% every 4K, train on last data

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  152.09 MiB      78.11%            15.4s       12.68 MB/s
zstd 22    153.34 MiB      78.75%            21.3s       9.14 MB/s

50% every 16B, train on last data

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  136.12 MiB      69.91%            14.7s       13.23 MB/s
zstd 22    140.86 MiB      72.34%            21.7s       8.98 MB/s

XOR with last partition of same type

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  137.06 MiB      70.39%            13.9s       13.96 MB/s
zstd 22    141.29 MiB      72.56%            21.2s       9.20 MB/s

==================== DELTA ENCODE ===================

## [NOT AOS] Delta Encode Colours

Compressor Results:
Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  121.46 MiB      62.38%            13.6s       14.35 MB/s
zstd 22    128.88 MiB      66.18%            23.2s       8.38 MB/s

## Delta Encode Colour Bytes

Compressor Results:
Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  121.57 MiB      62.43%            13.9s       14.00 MB/s
zstd 22    129.11 MiB      66.31%            23.5s       8.27 MB/s

==================== SWAP BITS ===================

WHAT??

## GROUP ENDPOINTS

Compressor Results:
Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  122.49 MiB      62.91%            13.8s       14.15 MB/s
zstd 22    134.80 MiB      69.23%            24.3s       8.02 MB/s

## GROUP ENDPOINTS + ALIGN (CHEATED, dropped p bits)

Compressor Results:
Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  117.28 MiB      60.23%            13.2s       14.76 MB/s
zstd 22    131.02 MiB      67.29%            24.9s       7.83 MB/s

==================== SHIFT BITS ===================

## [NOT AOS] Insert 3 p bits before colours, byte aligning the colours.

Compressor Results:
Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  117.11 MiB      60.14%            13.5s       14.39 MB/s
zstd 22    130.19 MiB      66.86%            26.3s       7.41 MB/s

==================== SORT BITS ===================

In each section. Least to most likely to be 0.

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  131.11 MiB      67.33%            18.9s       10.32 MB/s
7z         132.49 MiB      68.04%            19.1s       10.17 MB/s

Opposite sort.

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  131.33 MiB      67.45%            19.0s       10.27 MB/s
7z         134.98 MiB      69.32%            20.4s       9.55 MB/s

=================== ARRAY OF STRUCTURE ===================  

## Separated, no byte align

Compressor Results:
Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  126.91 MiB      65.18%            14.1s       13.85 MB/s
zstd 22    132.62 MiB      68.11%            28.5s       6.84 MB/s

## Separated, byte align

Compressor Results:
Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  124.05 MiB      61.31%            13.9s       14.52 MB/s
zstd 22    130.23 MiB      64.37%            27.6s       7.32 MB/s

## Separated, byte align, 2 p bits in index padding

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  121.87 MiB      62.10%            13.5s       14.49 MB/s
zstd 22    128.94 MiB      65.70%            27.1s       7.24 MB/s

## Separated, byte align, 3 p bits in index padding

Compressor Results:
Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  123.12 MiB      61.78%            13.7s       14.57 MB/s
zstd 22    130.26 MiB      65.37%            27.5s       7.25 MB/s

## Separated, inject p bits before colours and before indices.

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  120.21 MiB      61.74%            13.5s       14.43 MB/s
zstd 22    127.68 MiB      65.57%            28.1s       6.93 MB/s

## mode0_structure_of_array_mode_partition_colour_bycolourchannel_deltaencoded

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  134.33 MiB      66.39%            14.4s       14.03 MB/s
zstd 22    136.02 MiB      67.23%            29.7s       6.82 MB/s

## mode0_structure_of_array_mode_partition_colour_bycolourchannel_deltaencoded with 8 bit colours

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  133.01 MiB      42.65%            17.9s       17.47 MB/s
zstd 22    139.14 MiB      44.62%            61.7s       5.05 MB/s

## mode0_structure_of_array_mode_partition_colour_extendedto8bitspercolorentry

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  130.70 MiB      41.91%            17.8s       17.49 MB/s
zstd 22    137.50 MiB      44.09%            63.4s       4.92 MB/s