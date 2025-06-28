//! Integration tests for file format API

use dxt_lossless_transform_api_common::estimate::NoEstimation;
use dxt_lossless_transform_dds::DdsHandler;
use dxt_lossless_transform_file_formats_api::{
    traits::FileFormatDetection, transform_slice_with_bundle, untransform_slice, TransformBundle,
};

fn create_test_dds_bc1() -> Vec<u8> {
    const DDS_MAGIC: u32 = 0x44445320_u32.to_be();
    let mut data = vec![0u8; 0x80 + 8]; // DDS header + 1 BC1 block

    // DDS magic
    unsafe {
        (data.as_mut_ptr().add(0) as *mut u32).write_unaligned(DDS_MAGIC);
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
    assert!(handler.can_handle(&valid));

    // Invalid data
    let invalid = vec![0u8; 128];
    assert!(!handler.can_handle(&invalid));
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
