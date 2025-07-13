//! DDS format constants and definitions
#![allow(dead_code)]

/// Magic header for DDS files
pub(crate) const DDS_MAGIC: u32 = 0x20534444; // 'DDS ' in little-endian

/// Offset of the FOURCC header used in DX9 and below.
pub(crate) const FOURCC_OFFSET: usize = 0x54;

pub(crate) const FOURCC_DXT1: u32 = 0x31545844; // 'DXT1' in little-endian
pub(crate) const FOURCC_DXT2: u32 = 0x32545844; // 'DXT2' in little-endian
pub(crate) const FOURCC_DXT3: u32 = 0x33545844; // 'DXT3' in little-endian
pub(crate) const FOURCC_DXT4: u32 = 0x34545844; // 'DXT4' in little-endian
pub(crate) const FOURCC_DXT5: u32 = 0x35545844; // 'DXT5' in little-endian
pub(crate) const FOURCC_DX10: u32 = 0x30315844; // 'DX10' in little-endian

/// Offset of the DXGI format header used in DX10 and above.
pub(crate) const DX10_FORMAT_OFFSET: usize = 0x80;

// DXGI format constants for DX10 header
pub(crate) const DXGI_FORMAT_BC1_TYPELESS: u32 = 70;
pub(crate) const DXGI_FORMAT_BC1_UNORM: u32 = 71;
pub(crate) const DXGI_FORMAT_BC1_UNORM_SRGB: u32 = 72;

pub(crate) const DXGI_FORMAT_BC2_TYPELESS: u32 = 73;
pub(crate) const DXGI_FORMAT_BC2_UNORM: u32 = 74;
pub(crate) const DXGI_FORMAT_BC2_UNORM_SRGB: u32 = 75;

pub(crate) const DXGI_FORMAT_BC3_TYPELESS: u32 = 76;
pub(crate) const DXGI_FORMAT_BC3_UNORM: u32 = 77;
pub(crate) const DXGI_FORMAT_BC3_UNORM_SRGB: u32 = 78;

pub(crate) const DXGI_FORMAT_BC6H_TYPELESS: u32 = 94;
pub(crate) const DXGI_FORMAT_BC6H_UF16: u32 = 95;
pub(crate) const DXGI_FORMAT_BC6H_SF16: u32 = 96;

pub(crate) const DXGI_FORMAT_BC7_TYPELESS: u32 = 97;
pub(crate) const DXGI_FORMAT_BC7_UNORM: u32 = 98;
pub(crate) const DXGI_FORMAT_BC7_UNORM_SRGB: u32 = 99;

// Additional uncompressed DXGI formats
pub(crate) const DXGI_FORMAT_R8G8B8A8_TYPELESS: u32 = 27;
pub(crate) const DXGI_FORMAT_R8G8B8A8_UNORM: u32 = 28;
pub(crate) const DXGI_FORMAT_R8G8B8A8_UNORM_SRGB: u32 = 29;
pub(crate) const DXGI_FORMAT_R8G8B8A8_UINT: u32 = 30;
pub(crate) const DXGI_FORMAT_R8G8B8A8_SNORM: u32 = 31;
pub(crate) const DXGI_FORMAT_R8G8B8A8_SINT: u32 = 32;

pub(crate) const DXGI_FORMAT_B8G8R8A8_UNORM: u32 = 87;
pub(crate) const DXGI_FORMAT_B8G8R8A8_TYPELESS: u32 = 90;
pub(crate) const DXGI_FORMAT_B8G8R8A8_UNORM_SRGB: u32 = 91;

// Size of the regular DDS header
pub(crate) const DDS_HEADER_SIZE: usize = 0x80;
pub(crate) const DX10_HEADER_SIZE: usize = 20;

// DDS header field offsets for data length calculation
pub(crate) const DDS_FLAGS_OFFSET: usize = 0x08;
pub(crate) const DDS_HEIGHT_OFFSET: usize = 0x0C;
pub(crate) const DDS_WIDTH_OFFSET: usize = 0x10;
pub(crate) const DDS_MIPMAP_COUNT_OFFSET: usize = 0x1C;

// DDS pixel format offsets (within the 32-byte DDSPIXELFORMAT structure at offset 0x4C)
pub(crate) const DDS_PIXELFORMAT_OFFSET: usize = 0x4C;
pub(crate) const DDS_PIXELFORMAT_FLAGS_OFFSET: usize = 0x50;
pub(crate) const DDS_PIXELFORMAT_RGBBITCOUNT_OFFSET: usize = 0x58;

// DDS header flags
pub(crate) const DDSD_CAPS: u32 = 0x1;
pub(crate) const DDSD_HEIGHT: u32 = 0x2;
pub(crate) const DDSD_WIDTH: u32 = 0x4;
pub(crate) const DDSD_PIXELFORMAT: u32 = 0x1000;
pub(crate) const DDSD_LINEARSIZE: u32 = 0x80000;
pub(crate) const DDSD_MIPMAPCOUNT: u32 = 0x20000;

// DDS pixel format flags
pub(crate) const DDPF_ALPHAPIXELS: u32 = 0x1;
pub(crate) const DDPF_ALPHA: u32 = 0x2;
pub(crate) const DDPF_FOURCC: u32 = 0x4;
pub(crate) const DDPF_RGB: u32 = 0x40;
pub(crate) const DDPF_YUV: u32 = 0x200;
pub(crate) const DDPF_LUMINANCE: u32 = 0x20000;

// DDS pixel format mask offsets (within the 32-byte DDSPIXELFORMAT structure at offset 0x4C)
pub(crate) const DDS_PIXELFORMAT_RBITMASK_OFFSET: usize = 0x5C;
pub(crate) const DDS_PIXELFORMAT_GBITMASK_OFFSET: usize = 0x60;
pub(crate) const DDS_PIXELFORMAT_BBITMASK_OFFSET: usize = 0x64;
pub(crate) const DDS_PIXELFORMAT_ABITMASK_OFFSET: usize = 0x68;

// Because of Little Endian, the masks here are reversed.
// Common pixel format bit masks (verified with TexConv)
// R8G8B8A8_UNORM: R=byte0, G=byte1, B=byte2, A=byte3
pub(crate) const RGBA8888_RED_MASK: u32 = 0x000000FF;
pub(crate) const RGBA8888_GREEN_MASK: u32 = 0x0000FF00;
pub(crate) const RGBA8888_BLUE_MASK: u32 = 0x00FF0000;
pub(crate) const RGBA8888_ALPHA_MASK: u32 = 0xFF000000;

// B8G8R8A8_UNORM: R=byte2, G=byte1, B=byte0, A=byte3
pub(crate) const BGRA8888_RED_MASK: u32 = 0x00FF0000;
pub(crate) const BGRA8888_GREEN_MASK: u32 = 0x0000FF00;
pub(crate) const BGRA8888_BLUE_MASK: u32 = 0x000000FF;
pub(crate) const BGRA8888_ALPHA_MASK: u32 = 0xFF000000;

// B8G8R8 (BGR888): R=byte2, G=byte1, B=byte0
pub(crate) const BGR888_RED_MASK: u32 = 0x00FF0000;
pub(crate) const BGR888_GREEN_MASK: u32 = 0x0000FF00;
pub(crate) const BGR888_BLUE_MASK: u32 = 0x000000FF;
