//! DDS format constants and definitions
#![allow(dead_code)]

/// Magic header for DDS files
pub(crate) const DDS_MAGIC: u32 = 0x44445320_u32.to_be();

/// Offset of the FOURCC header used in DX9 and below.
pub(crate) const FOURCC_OFFSET: usize = 0x54;

pub(crate) const FOURCC_DXT1: u32 = 0x31545844_u32.to_le(); // 'DXT1'
pub(crate) const FOURCC_DXT2: u32 = 0x32545844_u32.to_le(); // 'DXT2'
pub(crate) const FOURCC_DXT3: u32 = 0x33545844_u32.to_le(); // 'DXT3'
pub(crate) const FOURCC_DXT4: u32 = 0x34545844_u32.to_le(); // 'DXT4'
pub(crate) const FOURCC_DXT5: u32 = 0x35545844_u32.to_le(); // 'DXT5'
pub(crate) const FOURCC_DX10: u32 = 0x30315844_u32.to_le(); // 'DX10'

/// Offset of the DXGI format header used in DX10 and above.
pub(crate) const DX10_FORMAT_OFFSET: usize = 0x80;

// DXGI format constants for DX10 header
pub(crate) const DXGI_FORMAT_BC1_TYPELESS: u32 = 70_u32.to_le();
pub(crate) const DXGI_FORMAT_BC1_UNORM: u32 = 71_u32.to_le();
pub(crate) const DXGI_FORMAT_BC1_UNORM_SRGB: u32 = 72_u32.to_le();

pub(crate) const DXGI_FORMAT_BC2_TYPELESS: u32 = 73_u32.to_le();
pub(crate) const DXGI_FORMAT_BC2_UNORM: u32 = 74_u32.to_le();
pub(crate) const DXGI_FORMAT_BC2_UNORM_SRGB: u32 = 75_u32.to_le();

pub(crate) const DXGI_FORMAT_BC3_TYPELESS: u32 = 76_u32.to_le();
pub(crate) const DXGI_FORMAT_BC3_UNORM: u32 = 77_u32.to_le();
pub(crate) const DXGI_FORMAT_BC3_UNORM_SRGB: u32 = 78_u32.to_le();

pub(crate) const DXGI_FORMAT_BC6H_TYPELESS: u32 = 94_u32.to_le();
pub(crate) const DXGI_FORMAT_BC6H_UF16: u32 = 95_u32.to_le();
pub(crate) const DXGI_FORMAT_BC6H_SF16: u32 = 96_u32.to_le();

pub(crate) const DXGI_FORMAT_BC7_TYPELESS: u32 = 97_u32.to_le();
pub(crate) const DXGI_FORMAT_BC7_UNORM: u32 = 98_u32.to_le();
pub(crate) const DXGI_FORMAT_BC7_UNORM_SRGB: u32 = 99_u32.to_le();

// Additional uncompressed DXGI formats
pub(crate) const DXGI_FORMAT_R8G8B8A8_TYPELESS: u32 = 27_u32.to_le();
pub(crate) const DXGI_FORMAT_R8G8B8A8_UNORM: u32 = 28_u32.to_le();
pub(crate) const DXGI_FORMAT_R8G8B8A8_UNORM_SRGB: u32 = 29_u32.to_le();
pub(crate) const DXGI_FORMAT_R8G8B8A8_UINT: u32 = 30_u32.to_le();
pub(crate) const DXGI_FORMAT_R8G8B8A8_SNORM: u32 = 31_u32.to_le();
pub(crate) const DXGI_FORMAT_R8G8B8A8_SINT: u32 = 32_u32.to_le();

pub(crate) const DXGI_FORMAT_B8G8R8A8_UNORM: u32 = 87_u32.to_le();
pub(crate) const DXGI_FORMAT_B8G8R8A8_TYPELESS: u32 = 90_u32.to_le();
pub(crate) const DXGI_FORMAT_B8G8R8A8_UNORM_SRGB: u32 = 91_u32.to_le();

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

// Common pixel format bit masks (verified with TexConv)
// R8G8B8A8_UNORM: R=byte0, G=byte1, B=byte2, A=byte3 (0xAABBGGRR)
pub(crate) const RGBA8888_RED_MASK: u32 = 0x000000FF;
pub(crate) const RGBA8888_GREEN_MASK: u32 = 0x0000FF00;
pub(crate) const RGBA8888_BLUE_MASK: u32 = 0x00FF0000;
pub(crate) const RGBA8888_ALPHA_MASK: u32 = 0xFF000000;

// B8G8R8A8_UNORM: R=byte2, G=byte1, B=byte0, A=byte3 (0xAAGGRRBB)
pub(crate) const ARGB8888_RED_MASK: u32 = 0x00FF0000;
pub(crate) const ARGB8888_GREEN_MASK: u32 = 0x0000FF00;
pub(crate) const ARGB8888_BLUE_MASK: u32 = 0x000000FF;
pub(crate) const ARGB8888_ALPHA_MASK: u32 = 0xFF000000;
