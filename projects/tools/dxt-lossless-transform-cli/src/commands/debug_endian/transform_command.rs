use std::error::Error;
use std::path::{Path, PathBuf};

use dxt_lossless_transform_api_common::estimate::NoEstimation;
use dxt_lossless_transform_bc1_api::{Bc1ManualTransformBuilder, YCoCgVariant};
use dxt_lossless_transform_bc2_api::Bc2ManualTransformBuilder;
use dxt_lossless_transform_dds::{
    dds::parse_dds::{parse_dds, DdsFormat},
    DdsHandler,
};
use dxt_lossless_transform_file_formats_api::{file_io, TransformBundle};

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
    let detected_format = match parse_dds(&file_data) {
        Some(dds_info) => match dds_info.format {
            DdsFormat::BC1 => dxt_lossless_transform_file_formats_api::embed::TransformFormat::Bc1,
            DdsFormat::BC2 => dxt_lossless_transform_file_formats_api::embed::TransformFormat::Bc2,
            DdsFormat::BC3 => dxt_lossless_transform_file_formats_api::embed::TransformFormat::Bc3,
            DdsFormat::BC7 => dxt_lossless_transform_file_formats_api::embed::TransformFormat::Bc7,
            DdsFormat::BC6H => {
                dxt_lossless_transform_file_formats_api::embed::TransformFormat::Bc6H
            }
            _ => {
                return Err(format!(
                    "Unsupported DDS format in file: {} (format: {:?})",
                    input_file.display(),
                    dds_info.format
                )
                .into())
            }
        },
        None => return Err(format!("Failed to parse DDS file: {}", input_file.display()).into()),
    };

    // Process all formats
    match detected_format {
        dxt_lossless_transform_file_formats_api::embed::TransformFormat::Bc1 => {
            // Test all manual combinations for BC1
            test_all_bc1_combinations(&input_file, &output_file)
        }
        dxt_lossless_transform_file_formats_api::embed::TransformFormat::Bc2 => {
            // Test all manual combinations for BC2
            test_all_bc2_combinations(&input_file, &output_file)
        }
        _ => {
            // BC3, BC7 and other formats are not yet supported for manual transform testing
            Err(format!(
                "Manual transform testing not yet implemented for format: {detected_format:?}"
            )
            .into())
        }
    }
}

/// Test all manual combinations for a given format and use the first successful one
fn test_all_combinations<F>(
    input_file: &Path,
    output_file: &Path,
    format_name: &str,
    mut build_bundle: F,
) -> Result<(), Box<dyn Error>>
where
    F: FnMut(YCoCgVariant, bool) -> TransformBundle<NoEstimation>,
{
    let decorrelation_variants = [
        YCoCgVariant::None,
        YCoCgVariant::Variant1,
        YCoCgVariant::Variant2,
        YCoCgVariant::Variant3,
    ];
    let split_options = [false, true];

    for variant in decorrelation_variants {
        for split in split_options {
            let bundle = build_bundle(variant, split);

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

    Err(format!("All manual transform combinations failed for {format_name}").into())
}

/// Test all manual combinations for BC1 and use the first successful one
fn test_all_bc1_combinations(input_file: &Path, output_file: &Path) -> Result<(), Box<dyn Error>> {
    test_all_combinations(input_file, output_file, "BC1", |variant, split| {
        let builder = Bc1ManualTransformBuilder::new()
            .decorrelation_mode(variant)
            .split_colour_endpoints(split);
        TransformBundle::<NoEstimation>::new().with_bc1_manual(builder)
    })
}

/// Test all manual combinations for BC2 and use the first successful one
fn test_all_bc2_combinations(input_file: &Path, output_file: &Path) -> Result<(), Box<dyn Error>> {
    test_all_combinations(input_file, output_file, "BC2", |variant, split| {
        let builder = Bc2ManualTransformBuilder::new()
            .decorrelation_mode(variant)
            .split_colour_endpoints(split);
        TransformBundle::<NoEstimation>::new().with_bc2_manual(builder)
    })
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
