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

pub(crate) const DXGI_FORMAT_BC7_TYPELESS: u32 = 97_u32.to_le();
pub(crate) const DXGI_FORMAT_BC7_UNORM: u32 = 98_u32.to_le();
pub(crate) const DXGI_FORMAT_BC7_UNORM_SRGB: u32 = 99_u32.to_le();

// Size of the regular DDS header
pub(crate) const DDS_HEADER_SIZE: usize = 0x80;
pub(crate) const DX10_HEADER_SIZE: usize = 20;
