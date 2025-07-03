//! Integration test demonstrating LTU usage with BC1 API.

#[cfg(all(feature = "c-exports", feature = "std"))]
#[test]
fn test_ltu_with_bc1_api_integration() {
    use dxt_lossless_transform_ltu::c_api::*;

    // Create an LTU estimator
    let estimator = unsafe { dltltu_new_size_estimator() };
    assert!(!estimator.is_null());

    // Verify the estimator is properly configured
    let estimator_ref = unsafe { &*estimator };
    assert!(!estimator_ref.context.is_null());

    // Test data
    let test_data = [0x12u8, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];

    // Test the estimator functionality
    let mut estimated_size = 0;
    let result = unsafe {
        (estimator_ref.estimate_compressed_size)(
            estimator_ref.context,
            test_data.as_ptr(),
            test_data.len(),
            core::ptr::null_mut(),
            0,
            &mut estimated_size,
        )
    };

    assert_eq!(result, 0); // Success
    assert!(estimated_size <= test_data.len()); // Should be <= input size

    // Clean up
    unsafe {
        dltltu_free_size_estimator(estimator);
    }
}

#[cfg(all(feature = "c-exports", feature = "std"))]
#[test]
fn test_ltu_basic_functionality() {
    use dxt_lossless_transform_ltu::c_api::*;

    // Create estimator
    let estimator = unsafe { dltltu_new_size_estimator() };
    assert!(!estimator.is_null());

    // Test with different data sizes
    let test_sizes = [16, 64, 256, 1024];

    for size in test_sizes {
        let test_data = vec![0xFFu8; size];
        let mut estimated_size = 0;

        let estimator_ref = unsafe { &*estimator };
        let result = unsafe {
            (estimator_ref.estimate_compressed_size)(
                estimator_ref.context,
                test_data.as_ptr(),
                test_data.len(),
                core::ptr::null_mut(),
                0,
                &mut estimated_size,
            )
        };

        assert_eq!(result, 0); // Success
        assert!(estimated_size <= test_data.len()); // Should be <= input size
    }

    // Clean up
    unsafe {
        dltltu_free_size_estimator(estimator);
    }
}
