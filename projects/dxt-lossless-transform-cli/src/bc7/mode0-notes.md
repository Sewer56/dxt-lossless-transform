Mode0

## Vanilla

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  116.09 MiB      59.62%            13.3s       14.69 MB/s
zstd 22    126.69 MiB      65.06%            25.0s       7.78 MB/s


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