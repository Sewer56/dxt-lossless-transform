Mode0

## Vanilla

Compressor Results:
Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  115.32 MiB      70.32%            12.4s       13.22 MB/s
zstd 22    123.82 MiB      75.51%            19.8s       8.27 MB/s
7z         115.98 MiB      70.73%            10.7s       15.32 MB/s

==================== DELTA ENCODE ===================

## [NOT AOS] Delta Encode Colours

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  121.37 MiB      74.01%            13.2s       12.45 MB/s
zstd 22    126.65 MiB      77.23%            18.5s       8.88 MB/s

==================== SWAP BITS ===================

WHAT??

## GROUP ENDPOINTS

Compressor Results:
Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  119.46 MiB      72.85%            12.7s       12.89 MB/s
zstd 22    128.83 MiB      78.56%            18.8s       8.70 MB/s

## GROUP ENDPOINTS + ALIGN (CHEATED, dropped p bits)


==================== SHIFT BITS ===================

## [NOT AOS] Insert 3 p bits before colours, byte aligning the colours.


=================== ARRAY OF STRUCTURE ===================  

## Separated, no byte align


## Separated, byte align


## Separated, byte align, 2 p bits in index padding


## Separated, byte align, 3 p bits in index padding


## Separated, inject p bits before colours and before indices.

Tool       Size            Ratio           Time       Speed          
-----------------------------------------------------------------
bzip3 16m  120.08 MiB      73.23%            13.0s       12.66 MB/s
zstd 22    125.58 MiB      76.58%            18.6s       8.82 MB/s