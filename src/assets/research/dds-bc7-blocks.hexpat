#pragma description DirectDraw Surface

#pragma MIME image/vnd-ms.dds
#pragma MIME image/x-dds
#pragma endian little
#pragma pattern_limit 0
#pragma array_limit 0

// This is an ImHex pattern file!
// Author: Sewer56
import std.mem;
import std.core;

enum DXGI_FORMAT : u32 {
  DXGI_FORMAT_UNKNOWN,
  DXGI_FORMAT_R32G32B32A32_TYPELESS,
  DXGI_FORMAT_R32G32B32A32_FLOAT,
  DXGI_FORMAT_R32G32B32A32_UINT,
  DXGI_FORMAT_R32G32B32A32_SINT,
  DXGI_FORMAT_R32G32B32_TYPELESS,
  DXGI_FORMAT_R32G32B32_FLOAT,
  DXGI_FORMAT_R32G32B32_UINT,
  DXGI_FORMAT_R32G32B32_SINT,
  DXGI_FORMAT_R16G16B16A16_TYPELESS,
  DXGI_FORMAT_R16G16B16A16_FLOAT,
  DXGI_FORMAT_R16G16B16A16_UNORM,
  DXGI_FORMAT_R16G16B16A16_UINT,
  DXGI_FORMAT_R16G16B16A16_SNORM,
  DXGI_FORMAT_R16G16B16A16_SINT,
  DXGI_FORMAT_R32G32_TYPELESS,
  DXGI_FORMAT_R32G32_FLOAT,
  DXGI_FORMAT_R32G32_UINT,
  DXGI_FORMAT_R32G32_SINT,
  DXGI_FORMAT_R32G8X24_TYPELESS,
  DXGI_FORMAT_D32_FLOAT_S8X24_UINT,
  DXGI_FORMAT_R32_FLOAT_X8X24_TYPELESS,
  DXGI_FORMAT_X32_TYPELESS_G8X24_UINT,
  DXGI_FORMAT_R10G10B10A2_TYPELESS,
  DXGI_FORMAT_R10G10B10A2_UNORM,
  DXGI_FORMAT_R10G10B10A2_UINT,
  DXGI_FORMAT_R11G11B10_FLOAT,
  DXGI_FORMAT_R8G8B8A8_TYPELESS,
  DXGI_FORMAT_R8G8B8A8_UNORM,
  DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
  DXGI_FORMAT_R8G8B8A8_UINT,
  DXGI_FORMAT_R8G8B8A8_SNORM,
  DXGI_FORMAT_R8G8B8A8_SINT,
  DXGI_FORMAT_R16G16_TYPELESS,
  DXGI_FORMAT_R16G16_FLOAT,
  DXGI_FORMAT_R16G16_UNORM,
  DXGI_FORMAT_R16G16_UINT,
  DXGI_FORMAT_R16G16_SNORM,
  DXGI_FORMAT_R16G16_SINT,
  DXGI_FORMAT_R32_TYPELESS,
  DXGI_FORMAT_D32_FLOAT,
  DXGI_FORMAT_R32_FLOAT,
  DXGI_FORMAT_R32_UINT,
  DXGI_FORMAT_R32_SINT,
  DXGI_FORMAT_R24G8_TYPELESS,
  DXGI_FORMAT_D24_UNORM_S8_UINT,
  DXGI_FORMAT_R24_UNORM_X8_TYPELESS,
  DXGI_FORMAT_X24_TYPELESS_G8_UINT,
  DXGI_FORMAT_R8G8_TYPELESS,
  DXGI_FORMAT_R8G8_UNORM,
  DXGI_FORMAT_R8G8_UINT,
  DXGI_FORMAT_R8G8_SNORM,
  DXGI_FORMAT_R8G8_SINT,
  DXGI_FORMAT_R16_TYPELESS,
  DXGI_FORMAT_R16_FLOAT,
  DXGI_FORMAT_D16_UNORM,
  DXGI_FORMAT_R16_UNORM,
  DXGI_FORMAT_R16_UINT,
  DXGI_FORMAT_R16_SNORM,
  DXGI_FORMAT_R16_SINT,
  DXGI_FORMAT_R8_TYPELESS,
  DXGI_FORMAT_R8_UNORM,
  DXGI_FORMAT_R8_UINT,
  DXGI_FORMAT_R8_SNORM,
  DXGI_FORMAT_R8_SINT,
  DXGI_FORMAT_A8_UNORM,
  DXGI_FORMAT_R1_UNORM,
  DXGI_FORMAT_R9G9B9E5_SHAREDEXP,
  DXGI_FORMAT_R8G8_B8G8_UNORM,
  DXGI_FORMAT_G8R8_G8B8_UNORM,
  DXGI_FORMAT_BC1_TYPELESS,
  DXGI_FORMAT_BC1_UNORM,
  DXGI_FORMAT_BC1_UNORM_SRGB,
  DXGI_FORMAT_BC2_TYPELESS,
  DXGI_FORMAT_BC2_UNORM,
  DXGI_FORMAT_BC2_UNORM_SRGB,
  DXGI_FORMAT_BC3_TYPELESS,
  DXGI_FORMAT_BC3_UNORM,
  DXGI_FORMAT_BC3_UNORM_SRGB,
  DXGI_FORMAT_BC4_TYPELESS,
  DXGI_FORMAT_BC4_UNORM,
  DXGI_FORMAT_BC4_SNORM,
  DXGI_FORMAT_BC5_TYPELESS,
  DXGI_FORMAT_BC5_UNORM,
  DXGI_FORMAT_BC5_SNORM,
  DXGI_FORMAT_B5G6R5_UNORM,
  DXGI_FORMAT_B5G5R5A1_UNORM,
  DXGI_FORMAT_B8G8R8A8_UNORM,
  DXGI_FORMAT_B8G8R8X8_UNORM,
  DXGI_FORMAT_R10G10B10_XR_BIAS_A2_UNORM,
  DXGI_FORMAT_B8G8R8A8_TYPELESS,
  DXGI_FORMAT_B8G8R8A8_UNORM_SRGB,
  DXGI_FORMAT_B8G8R8X8_TYPELESS,
  DXGI_FORMAT_B8G8R8X8_UNORM_SRGB,
  DXGI_FORMAT_BC6H_TYPELESS,
  DXGI_FORMAT_BC6H_UF16,
  DXGI_FORMAT_BC6H_SF16,
  DXGI_FORMAT_BC7_TYPELESS,
  DXGI_FORMAT_BC7_UNORM,
  DXGI_FORMAT_BC7_UNORM_SRGB,
  DXGI_FORMAT_AYUV,
  DXGI_FORMAT_Y410,
  DXGI_FORMAT_Y416,
  DXGI_FORMAT_NV12,
  DXGI_FORMAT_P010,
  DXGI_FORMAT_P016,
  DXGI_FORMAT_420_OPAQUE,
  DXGI_FORMAT_YUY2,
  DXGI_FORMAT_Y210,
  DXGI_FORMAT_Y216,
  DXGI_FORMAT_NV11,
  DXGI_FORMAT_AI44,
  DXGI_FORMAT_IA44,
  DXGI_FORMAT_P8,
  DXGI_FORMAT_A8P8,
  DXGI_FORMAT_B4G4R4A4_UNORM,
  DXGI_FORMAT_P208,
  DXGI_FORMAT_V208,
  DXGI_FORMAT_V408,
  DXGI_FORMAT_FORCE_UINT,
};

bitfield HeaderFlags {
  caps        : 1;
  height      : 1;
  width       : 1;
  pitch       : 1;
  padding     : 8;
  pixelFormat : 1; // 0x1000
  padding     : 4;
  mipMapCount : 1; // 0x20000
  padding     : 1;
  linearSize  : 1; // 0x80000
  padding     : 3;
  depth       : 1; // 0x800000
  padding     : 7;
};

bitfield CapsFlags {
  padding    : 3;
  complex    : 1; // 0x8
  padding    : 8;
  texture    : 1; // 0x1000
  padding    : 9;
  mipmap     : 1; // 0x400000
  padding    : 9;
};

bitfield Caps2Flags {
  padding        : 9;
  cubemap        : 1;
  cubemap_plusX  : 1;
  cubemap_minusX : 1;
  cubemap_plusY  : 1;
  cubemap_minusY : 1;
  cubemap_plusZ  : 1;
  cubemap_minusZ : 1;
  padding        : 5;
  volume         : 1; // 0x200000
  padding        : 10;
};

bitfield PixelFormatFlags {
  alphaPixels       : 1;  // DDPF_ALPHAPIXELS
  alpha             : 1;  // DDPF_ALPHA
  fourCC            : 1;  // DDPF_FOURCC
  paletteIndexed4   : 1;  // DDPF_PALETTEINDEXED4
  paletteIndexedTo8 : 1;  // DDPF_PALETTEINDEXEDTO8
  paletteIndexed8   : 1;  // DDPF_PALETTEINDEXED8
  rgb               : 1;  // DDPF_RGB
  compressed        : 1;  // DDPF_COMPRESSED
  rgbToYuv          : 1;  // DDPF_RGBTOYUV
  yuv               : 1;  // DDPF_YUV
  zBuffer           : 1;  // DDPF_ZBUFFER
  paletteIndexed1   : 1;  // DDPF_PALETTEINDEXED1
  paletteIndexed2   : 1;  // DDPF_PALETTEINDEXED2
  zPixels           : 1;  // DDPF_ZPIXELS
  stencilBuffer     : 1;  // DDPF_STENCILBUFFER
  alphaResult       : 1;  // DDPF_ALPHARESULT
  luminance         : 1;  // DDPF_LUMINANCE
  bumpLuminance     : 1;  // DDPF_BUMPLUMINANCE
  bumpDudv          : 1;  // DDPF_BUMPDUDV
  padding           : 13; // Padding bits to complete 32-bit structure
};

enum DX10ResourceDimension : u32 {
  Unknown,
  Buffer,
  Texture1D,
  Texture2D,
  Texture3D,
};

enum DX10AlphaMode : u32 {
  Unknown,
  Straight,
  PreMultiplied,
  Opaque,
  Custom,
};

bitfield DX10MiscFlags {
  generateMips : 1;
  shared       : 1;
  textureCube  : 1;
  // following data are D3D10_RESOURCE_MISC_FLAG or D3D11_RESOURCE_MISC_FLAG
  // dependent and probably aren't used
  reserved     : 29;
};

struct DDS_HEADER_DX10 {
  DXGI_FORMAT           dxgiFormat;
  DX10ResourceDimension resourceDimension;
  DX10MiscFlags         miscFlags;
  u32                   arraySize;
  DX10AlphaMode         miscFlags2;
} [[static]];

struct DDS_PIXELFORMAT {
  u32                 size;
  PixelFormatFlags    flags;
  char                fourCC[4];
  u32                 RGBBitCount;
  u32                 rBitMask;
  u32                 gBitMask;
  u32                 bBitMask;
  u32                 aBitMask;
} [[static]];

struct DDS_HEADER {
  u32             size;
  HeaderFlags     flags;
  u32             height;
  u32             width;
  u32             pitchOrLinearSize;
  u32             depth;
  u32             mipMapCount;
  u32             reserved0[11];
  DDS_PIXELFORMAT ddspf;
  CapsFlags       caps;
  Caps2Flags      caps2;
  u32             caps3;
  u32             caps4;
  u32             reserved1;

  if (ddspf.fourCC == "DX10") {
    DDS_HEADER_DX10 dx10Header;
  }
} [[static]];

// BC7 Mode bitfield
bitfield BC7Mode {
    mode0       : 1;
    mode1       : 1;
    mode2       : 1;
    mode3       : 1;
    mode4       : 1;
    mode5       : 1;
    mode6       : 1;
    mode7       : 1;
};

fn reverse4bits(u8 b) {
    return ((b & 0b0001) << 3) |  // d000
    ((b & 0b0010) << 1) |  // 0b00
    ((b & 0b0100) >> 1) |  // 00c0
    ((b & 0b1000) >> 3); // 000a;
};


// BC7 Mode 0 struct - Total 128 bits
bitfield BC7BlockMode0 {
    unsigned mode      : 1;    // Mode bit (0 for mode 0)
    unsigned partition : 4 [[format("reverse4bits")]];

    unsigned color_r0: 4;
    unsigned color_r1: 4;
    unsigned color_r2: 4;
    unsigned color_r3: 4;
    unsigned color_r4: 4;
    unsigned color_r5: 4;
    
    unsigned color_g0: 4;
    unsigned color_g1: 4;
    unsigned color_g2: 4;
    unsigned color_g3: 4;
    unsigned color_g4: 4;
    unsigned color_g5: 4;
    
    unsigned color_b0: 4;
    unsigned color_b1: 4;
    unsigned color_b2: 4;
    unsigned color_b3: 4;
    unsigned color_b4: 4;
    unsigned color_b5: 4;

    unsigned p0 : 1;    // P-bit
    unsigned p1 : 1;    // P-bit
    unsigned p2 : 1;    // P-bit
    unsigned p3 : 1;    // P-bit
    unsigned p4 : 1;    // P-bit
    unsigned p5 : 1;    // P-bit

    unsigned index0  : 3;    // Index for pixel 0
    unsigned index1  : 3;    // Index for pixel 1
    unsigned index2  : 3;    // Index for pixel 2
    unsigned index3  : 3;    // Index for pixel 3
    unsigned index4  : 3;    // Index for pixel 4
    unsigned index5  : 3;    // Index for pixel 5
    unsigned index6  : 3;    // Index for pixel 6
    unsigned index7  : 3;    // Index for pixel 7
    unsigned index8  : 3;    // Index for pixel 8
    unsigned index9  : 3;    // Index for pixel 9
    unsigned index10 : 3;    // Index for pixel 10
    unsigned index11 : 3;    // Index for pixel 11
    unsigned index12 : 3;    // Index for pixel 12
    unsigned index13 : 3;    // Index for pixel 13
    unsigned index14 : 3;    // Index for pixel 14
};

// Mode 1 structure (128 bits)
bitfield BC7BlockMode1 {
    unsigned mode      : 2;    // Mode bits (01)
    unsigned partition : 6;    // Partition number
    
    unsigned color_r0: 6;
    unsigned color_r1: 6;
    unsigned color_r2: 6;
    unsigned color_r3: 6;
    
    unsigned color_g0: 6;
    unsigned color_g1: 6;
    unsigned color_g2: 6;
    unsigned color_g3: 6;
    
    unsigned color_b0: 6;
    unsigned color_b1: 6;
    unsigned color_b2: 6;
    unsigned color_b3: 6;
    
    unsigned p0 : 1;    // P-bit
    unsigned p1 : 1;    // P-bit
    
    // 46 index bits split into 16 3-bit indices
    unsigned index0  : 3;
    unsigned index1  : 3;
    unsigned index2  : 3;
    unsigned index3  : 3;
    unsigned index4  : 3;
    unsigned index5  : 3;
    unsigned index6  : 3;
    unsigned index7  : 3;
    unsigned index8  : 3;
    unsigned index9  : 3;
    unsigned index10 : 3;
    unsigned index11 : 3;
    unsigned index12 : 3;
    unsigned index13 : 3;
    unsigned index14 : 2;    // Note: Last index is truncated
};

fn reverse3bits(u8 b) {
    b = (b & 0xF8) | ((b & 0x04) >> 2) | (b & 0x02) | ((b & 0x01) << 2);
    return b;
};

// Mode 2 structure (128 bits)
bitfield BC7BlockMode2 {
    unsigned mode      : 3 [[format("reverse3bits")]];    // Mode bits (001)
    unsigned partition : 6;    // Partition number
    
    unsigned color_r0: 5;
    unsigned color_r1: 5;
    unsigned color_r2: 5;
    unsigned color_r3: 5;
    unsigned color_r4: 5;
    unsigned color_r5: 5;
    
    unsigned color_g0: 5;
    unsigned color_g1: 5;
    unsigned color_g2: 5;
    unsigned color_g3: 5;
    unsigned color_g4: 5;
    unsigned color_g5: 5;
    
    unsigned color_b0: 5;
    unsigned color_b1: 5;
    unsigned color_b2: 5;
    unsigned color_b3: 5;
    unsigned color_b4: 5;
    unsigned color_b5: 5;
    
    // 29 index bits split into 15 2-bit indices
    unsigned index0  : 2;
    unsigned index1  : 2;
    unsigned index2  : 2;
    unsigned index3  : 2;
    unsigned index4  : 2;
    unsigned index5  : 2;
    unsigned index6  : 2;
    unsigned index7  : 2;
    unsigned index8  : 2;
    unsigned index9  : 2;
    unsigned index10 : 2;
    unsigned index11 : 2;
    unsigned index12 : 2;
    unsigned index13 : 2;
    unsigned index14 : 1;  // Last index is truncated
};

// Mode 3 structure (128 bits)
bitfield BC7BlockMode3 {
    unsigned mode      : 4;    // Mode bits (0001)
    unsigned partition : 6;    // Partition number
    
    unsigned color_r0: 7;
    unsigned color_r1: 7;
    unsigned color_r2: 7;
    unsigned color_r3: 7;
    
    unsigned color_g0: 7;
    unsigned color_g1: 7;
    unsigned color_g2: 7;
    unsigned color_g3: 7;
    
    unsigned color_b0: 7;
    unsigned color_b1: 7;
    unsigned color_b2: 7;
    unsigned color_b3: 7;
    
    unsigned p0 : 1;    // P-bit
    unsigned p1 : 1;    // P-bit
    unsigned p2 : 1;    // P-bit
    unsigned p3 : 1;    // P-bit
    
    // 30 index bits split into 15 2-bit indices
    unsigned index0  : 2;
    unsigned index1  : 2;
    unsigned index2  : 2;
    unsigned index3  : 2;
    unsigned index4  : 2;
    unsigned index5  : 2;
    unsigned index6  : 2;
    unsigned index7  : 2;
    unsigned index8  : 2;
    unsigned index9  : 2;
    unsigned index10 : 2;
    unsigned index11 : 2;
    unsigned index12 : 2;
    unsigned index13 : 2;
    unsigned index14 : 2;
};


// Mode 4 structure (128 bits)
bitfield BC7BlockMode4 {
    unsigned mode     : 5;     // Mode bits (00001)
    unsigned rotation : 2;     // Color channel rotation
    unsigned idxMode  : 1;     // Index selection mode
    
    unsigned color_r0: 5;
    unsigned color_r1: 5;
    unsigned color_g0: 5;
    unsigned color_g1: 5;
    unsigned color_b0: 5;
    unsigned color_b1: 5;
    unsigned color_a0: 6;
    unsigned color_a1: 6;
    
    // 31 + 47 = 78 index bits
    // First set: 15 2-bit indices (30 bits)
    unsigned color_index0  : 2;
    unsigned color_index1  : 2;
    unsigned color_index2  : 2;
    unsigned color_index3  : 2;
    unsigned color_index4  : 2;
    unsigned color_index5  : 2;
    unsigned color_index6  : 2;
    unsigned color_index7  : 2;
    unsigned color_index8  : 2;
    unsigned color_index9  : 2;
    unsigned color_index10 : 2;
    unsigned color_index11 : 2;
    unsigned color_index12 : 2;
    unsigned color_index13 : 2;
    unsigned color_index14 : 2;
    unsigned pad           : 1;
    
    // Second set: 15 3-bit indices (45 bits)
    unsigned alpha_index0  : 3;
    unsigned alpha_index1  : 3;
    unsigned alpha_index2  : 3;
    unsigned alpha_index3  : 3;
    unsigned alpha_index4  : 3;
    unsigned alpha_index5  : 3;
    unsigned alpha_index6  : 3;
    unsigned alpha_index7  : 3;
    unsigned alpha_index8  : 3;
    unsigned alpha_index9  : 3;
    unsigned alpha_index10 : 3;
    unsigned alpha_index11 : 3;
    unsigned alpha_index12 : 3;
    unsigned alpha_index13 : 3;
    unsigned alpha_index14 : 3;
};

// Mode 5 structure (128 bits)
bitfield BC7BlockMode5 {
    unsigned mode     : 6;     // Mode bits (000001)
    unsigned rotation : 2;     // Color channel rotation
    
    unsigned color_r0: 7;
    unsigned color_r1: 7;
    unsigned color_g0: 7;
    unsigned color_g1: 7;
    unsigned color_b0: 7;
    unsigned color_b1: 7;
    
    unsigned color_a0: 8;
    unsigned color_a1: 8;
    
    // 31 bits color index
    unsigned color_index0  : 2;
    unsigned color_index1  : 2;
    unsigned color_index2  : 2;
    unsigned color_index3  : 2;
    unsigned color_index4  : 2;
    unsigned color_index5  : 2;
    unsigned color_index6  : 2;
    unsigned color_index7  : 2;
    unsigned color_index8  : 2;
    unsigned color_index9  : 2;
    unsigned color_index10 : 2;
    unsigned color_index11 : 2;
    unsigned color_index12 : 2;
    unsigned color_index13 : 2;
    unsigned color_index14 : 2;
    unsigned pad           : 1;
    
    // 31 bits alpha index
    unsigned alpha_index0  : 2;
    unsigned alpha_index1  : 2;
    unsigned alpha_index2  : 2;
    unsigned alpha_index3  : 2;
    unsigned alpha_index4  : 2;
    unsigned alpha_index5  : 2;
    unsigned alpha_index6  : 2;
    unsigned alpha_index7  : 2;
    unsigned alpha_index8  : 2;
    unsigned alpha_index9  : 2;
    unsigned alpha_index10 : 2;
    unsigned alpha_index11 : 2;
    unsigned alpha_index12 : 2;
    unsigned alpha_index13 : 2;
    unsigned alpha_index14 : 2;
    unsigned padding2      : 1;
};

// Mode 6 structure (128 bits)
bitfield BC7BlockMode6 {
    unsigned mode     : 7;     // Mode bits (0000001)
    
    unsigned color_r0: 7;
    unsigned color_r1: 7;
    unsigned color_g0: 7;
    unsigned color_g1: 7;
    unsigned color_b0: 7;
    unsigned color_b1: 7;
    unsigned color_a0: 7;
    unsigned color_a1: 7;
    
    unsigned p0: 1;
    unsigned p1: 1;
    
    // 63 bits index data split into 15 4-bit indices
    unsigned index0  : 4;
    unsigned index1  : 4;
    unsigned index2  : 4;
    unsigned index3  : 4;
    unsigned index4  : 4;
    unsigned index5  : 4;
    unsigned index6  : 4;
    unsigned index7  : 4;
    unsigned index8  : 4;
    unsigned index9  : 4;
    unsigned index10 : 4;
    unsigned index11 : 4;
    unsigned index12 : 4;
    unsigned index13 : 4;
    unsigned index14 : 4;
    unsigned pad     : 3;
};

// Mode 7 structure (128 bits)
bitfield BC7BlockMode7 {
    unsigned mode      : 8;    // Mode bits (00000001)
    unsigned partition : 6;    // Partition number
    
    unsigned color_r0: 5;
    unsigned color_r1: 5;
    unsigned color_r2: 5;
    unsigned color_r3: 5;
    
    unsigned color_g0: 5;
    unsigned color_g1: 5;
    unsigned color_g2: 5;
    unsigned color_g3: 5;
    
    unsigned color_b0: 5;
    unsigned color_b1: 5;
    unsigned color_b2: 5;
    unsigned color_b3: 5;
    
    unsigned color_a0: 5;
    unsigned color_a1: 5;
    unsigned color_a2: 5;
    unsigned color_a3: 5;
    
    unsigned p0 : 1;    // P-bit
    unsigned p1 : 1;    // P-bit
    unsigned p2 : 1;    // P-bit
    unsigned p3 : 1;    // P-bit
    
    // 30 index bits split into 15 2-bit indices
    unsigned index0  : 2;
    unsigned index1  : 2;
    unsigned index2  : 2;
    unsigned index3  : 2;
    unsigned index4  : 2;
    unsigned index5  : 2;
    unsigned index6  : 2;
    unsigned index7  : 2;
    unsigned index8  : 2;
    unsigned index9  : 2;
    unsigned index10 : 2;
    unsigned index11 : 2;
    unsigned index12 : 2;
    unsigned index13 : 2;
    unsigned index14 : 2;
};

fn get_bc7_mode(u8 first_byte) {
    // Count leading zeros to determine mode
    u8 mode = 0;
    u8 mask = 0x80; // Start with highest bit
    while ((first_byte & mask) == 0 && mode < 8) {
        mode += 1;
        mask >>= 1;
    }
    return mode;
};


struct BC7Block {
    u8 bc7_mode = get_bc7_mode(std::mem::read_unsigned($, 1));
    if (bc7_mode == 0) {
        be BC7BlockMode0 block;
    } else if (bc7_mode == 1) {
        be BC7BlockMode1 block;
    } else if (bc7_mode == 2) {
        be BC7BlockMode2 block;
    } else if (bc7_mode == 3) {
        be BC7BlockMode3 block;
    } else if (bc7_mode == 4) {
        be BC7BlockMode4 block;
    } else if (bc7_mode == 5) {
        be BC7BlockMode5 block;
    } else if (bc7_mode == 6) {
        be BC7BlockMode6 block;
    } else if (bc7_mode == 7) {
        be BC7BlockMode7 block;
    }
} [[inline]];

// BC7 block array that follows the header
struct BC7Blocks {
    if (header.ddspf.fourCC == "DX10" && header.dx10Header.dxgiFormat == DXGI_FORMAT::DXGI_FORMAT_BC7_UNORM) {
        BC7Block blocks[while (!std::mem::eof())];
    }
} [[inline]];

char id[4] @ 0x0;
DDS_HEADER header @ 0x4;
BC7Blocks blocks @ addressof(header) + header.size + (header.ddspf.fourCC == "DX10" ? sizeof(DDS_HEADER_DX10) : 0);