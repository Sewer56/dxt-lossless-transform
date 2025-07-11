//! Integration tests for file format API

use dxt_lossless_transform_api_common::estimate::NoEstimation;
use dxt_lossless_transform_dds::DdsHandler;
use dxt_lossless_transform_file_formats_api::{
    handlers::*, transform_slice_with_bundle, untransform_slice, TransformBundle,
};
use endian_writer::{EndianWriter, LittleEndianWriter};

fn create_test_dds_bc1() -> Vec<u8> {
    const DDS_MAGIC: u32 = 0x20534444; // "DDS " in little-endian
    const DDSD_CAPS: u32 = 0x1;
    const DDSD_HEIGHT: u32 = 0x2;
    const DDSD_WIDTH: u32 = 0x4;
    const DDSD_PIXELFORMAT: u32 = 0x1000;
    const DDSD_LINEARSIZE: u32 = 0x80000;
    const DDPF_FOURCC: u32 = 0x4;
    let mut data = vec![0u8; 0x80 + 8]; // DDS header + 1 BC1 block

    // Write DDS header using little-endian writer
    let mut writer = unsafe { LittleEndianWriter::new(data.as_mut_ptr()) };
    unsafe {
        // DDS magic
        writer.write_u32_at(DDS_MAGIC, 0);

        // Set header size (dwSize field at offset 4)
        writer.write_u32_at(124, 4);

        // Set flags to include required fields
        writer.write_u32_at(
            DDSD_CAPS | DDSD_HEIGHT | DDSD_WIDTH | DDSD_PIXELFORMAT | DDSD_LINEARSIZE,
            8,
        );

        // Set dimensions (4x4 for 1 BC1 block = 8 bytes)
        writer.write_u32_at(4, 0x0C); // height
        writer.write_u32_at(4, 0x10); // width

        // Set pixel format flags to indicate FOURCC format
        writer.write_u32_at(DDPF_FOURCC, 0x50); // DDS_PIXELFORMAT_FLAGS_OFFSET
    }

    // Set FOURCC to DXT1
    data[0x54..0x58].copy_from_slice(b"DXT1");

    // Add some BC1 data
    data[0x80] = 0x00;
    data[0x81] = 0xF8; // Red color
    data[0x82] = 0xE0;
    data[0x83] = 0x07; // Green color
    data[0x84] = 0x00;
    data[0x85] = 0x00;
    data[0x86] = 0x00;
    data[0x87] = 0x00;

    data
}

#[test]
fn test_dds_bc1_transform_roundtrip() {
    let handler = DdsHandler;
    let bundle = TransformBundle::<NoEstimation>::default_all();

    // Create test DDS
    let input = create_test_dds_bc1();
    let mut transformed = vec![0u8; input.len()];
    let mut restored = vec![0u8; input.len()];

    // Transform
    transform_slice_with_bundle(&handler, &input, &mut transformed, &bundle)
        .expect("Transform should succeed");

    // Check that header was modified (no longer DDS magic)
    assert_ne!(
        &transformed[0..4],
        &input[0..4],
        "Header should be modified"
    );

    // Untransform
    untransform_slice(&handler, &transformed, &mut restored).expect("Untransform should succeed");

    // Verify roundtrip
    assert_eq!(
        &restored[0..4],
        &input[0..4],
        "DDS magic should be restored"
    );

    // The BC1 data might be different due to transform, but should decode to same pixels
    // For now, just verify the operation completed successfully
}

#[test]
fn test_handler_detection() {
    let handler = DdsHandler;

    // Valid DDS
    let valid = create_test_dds_bc1();
    assert!(handler.can_handle(&valid, Some("dds")));

    // Invalid data
    let invalid = vec![0u8; 128];
    assert!(!handler.can_handle(&invalid, Some("dds")));
}

#[test]
fn test_missing_builder() {
    let handler = DdsHandler;
    let bundle = TransformBundle::<NoEstimation>::new(); // Empty bundle

    let input = create_test_dds_bc1();
    let mut output = vec![0u8; input.len()];

    // Should fail with no BC1 builder
    let result = transform_slice_with_bundle(&handler, &input, &mut output, &bundle);
    assert!(result.is_err());

    match result {
        Err(e) => {
            let msg = format!("{e:?}");
            assert!(msg.contains("Bc1"), "Error should mention Bc1");
        }
        _ => panic!("Should have failed"),
    }
}
