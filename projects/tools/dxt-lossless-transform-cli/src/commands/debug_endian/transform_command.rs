use std::error::Error;
use std::path::{Path, PathBuf};

use dxt_lossless_transform_api_common::estimate::NoEstimation;
use dxt_lossless_transform_bc1_api::{Bc1ManualTransformBuilder, YCoCgVariant};
use dxt_lossless_transform_dds::DdsHandler;
use dxt_lossless_transform_file_formats_api::{file_io, TransformBundle};
use dxt_lossless_transform_file_formats_debug::{FileFormatBlockExtraction, TransformFormatFilter};

/// Handle transform of a single file with all manual combinations
pub fn handle_transform_single_file(
    input_file: PathBuf,
    output_dir: PathBuf,
) -> Result<(), Box<dyn Error>> {
    // Create output directory
    std::fs::create_dir_all(&output_dir)?;

    // Get the input filename to use for output
    let input_filename = input_file.file_name().ok_or("Invalid input file path")?;
    let output_file = output_dir.join(input_filename);

    // Detect the format of the input file
    let file_data = std::fs::read(&input_file)?;
    let handler = DdsHandler;

    let detected_format = match handler.extract_blocks(&file_data, TransformFormatFilter::All) {
        Ok(Some(extracted)) => extracted.format,
        Ok(None) => {
            return Err(
                format!("Could not detect format for file: {}", input_file.display()).into(),
            )
        }
        Err(e) => {
            return Err(format!(
                "Failed to extract blocks from file {}: {}",
                input_file.display(),
                e
            )
            .into())
        }
    };

    // Process all formats
    match detected_format {
        dxt_lossless_transform_file_formats_api::embed::TransformFormat::Bc1 => {
            // Test all manual combinations for BC1
            test_all_bc1_combinations(&input_file, &output_file, &handler)
        }
        dxt_lossless_transform_file_formats_api::embed::TransformFormat::Bc2 => {
            // TODO: BC2 manual transforms are not yet integrated into TransformBundle
            // Use default transform for now
            let bundle = TransformBundle::<NoEstimation>::new();
            file_io::transform_file_with_multiple_handlers(
                [DdsHandler],
                &input_file,
                &output_file,
                &bundle,
            )?;
            println!("✓ Success with default transform for BC2 (manual combinations not yet available in TransformBundle)");
            Ok(())
        }
        _ => {
            // Use default transform for other formats (BC3, BC7)
            let bundle = TransformBundle::<NoEstimation>::new();
            file_io::transform_file_with_multiple_handlers(
                [DdsHandler],
                &input_file,
                &output_file,
                &bundle,
            )?;
            println!("✓ Success with default transform for {detected_format:?}");
            Ok(())
        }
    }
}

/// Test all manual combinations for BC1 and use the first successful one
fn test_all_bc1_combinations(
    input_file: &Path,
    output_file: &Path,
    _handler: &DdsHandler,
) -> Result<(), Box<dyn Error>> {
    // Generate all manual combinations
    let decorrelation_variants = [
        YCoCgVariant::None,
        YCoCgVariant::Variant1,
        YCoCgVariant::Variant2,
        YCoCgVariant::Variant3,
    ];
    let split_options = [false, true];

    for variant in decorrelation_variants {
        for split in split_options {
            let builder = Bc1ManualTransformBuilder::new()
                .decorrelation_mode(variant)
                .split_colour_endpoints(split);

            let bundle = TransformBundle::<NoEstimation>::new().with_bc1_manual(builder);

            // Try to transform with this combination
            match file_io::transform_file_with_multiple_handlers(
                [DdsHandler],
                input_file,
                output_file,
                &bundle,
            ) {
                Ok(_) => {
                    println!("✓ Success with combination: decorrelation={variant:?}, split_endpoints={split}");
                    // Use the first successful combination for endian testing
                    return Ok(());
                }
                Err(e) => {
                    println!("✗ Failed with combination: decorrelation={variant:?}, split_endpoints={split}: {e}");
                }
            }
        }
    }

    Err("All manual transform combinations failed".into())
}

/// Handle untransform of all files in input directory
pub fn handle_untransform_single_file(
    input_dir: PathBuf,
    output_dir: PathBuf,
) -> Result<(), Box<dyn Error>> {
    // Create output directory
    std::fs::create_dir_all(&output_dir)?;

    // Process all files in the input directory
    for entry in std::fs::read_dir(&input_dir)? {
        let entry = entry?;
        let input_path = entry.path();

        if input_path.is_file() {
            let filename = input_path.file_name().ok_or("Invalid file name")?;
            let output_path = output_dir.join(filename);

            let handlers = [DdsHandler];
            file_io::untransform_file_with_multiple_handlers(handlers, &input_path, &output_path)?;
            println!(
                "Untransformed: {} -> {}",
                input_path.display(),
                output_path.display()
            );
        }
    }

    Ok(())
}
