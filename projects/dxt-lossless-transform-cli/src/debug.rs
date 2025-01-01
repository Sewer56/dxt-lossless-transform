use crate::error::TransformError;
use crate::*;
use argh::FromArgs;
use bc7::*;
use bitstream_io::{BigEndian, BitRead, BitReader, BitWrite, BitWriter, Endianness};
use core::ptr::copy_nonoverlapping;
use dxt_lossless_transform::util::msb_extract_bits::extract_msb_bits;
use dxt_lossless_transform::util::msb_insert_bits::insert_msb_bits;
use dxt_lossless_transform_api::*;
use rayon::*;
use std::io::{Cursor, SeekFrom};
use std::path::PathBuf;
use std::slice;

#[derive(FromArgs, Debug)]
/// Debug commands for analyzing DDS files
#[argh(subcommand, name = "debug")]
pub struct DebugCmd {
    #[argh(subcommand)]
    pub command: DebugCommands,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum DebugCommands {
    AnalyzeBC7(AnalyzeBC7Cmd),
    AnalyzeBC7Mode0BitDistributions(AnalyzeBC7Mode0BitDistributionsCmd),
    SplitByBlockTypeCmd(SplitByBlockTypeCmd),
    ByteAlignMode0Blocks(ByteAlignMode0Blocks),
    SetMode0TransformToMostCommon(SetMode0TransformToMostCommon),
    RemoveNonMode0Blocks(Bc7RemoveNonMode0Blocks),
    Mode0ToStructureOfArray(Bc7Mode0ToStructureOfArray),
}

#[derive(FromArgs, Debug)]
/// Analyze BC7 block types in DDS files
#[argh(subcommand, name = "analyze-bc7")]
pub struct AnalyzeBC7Cmd {
    /// input directory path
    #[argh(option)]
    pub input: PathBuf,
}

#[derive(FromArgs, Debug)]
/// Analyze BC7 block types in DDS files
#[argh(subcommand, name = "analyze-bc7-mode-0-bit-distributions")]
pub struct AnalyzeBC7Mode0BitDistributionsCmd {
    /// input directory path
    #[argh(option)]
    pub input: PathBuf,
}

#[derive(FromArgs, Debug)]
/// Transforms BC7 files by splitting the blocks into one section with first byte, and one section with the rest
#[argh(subcommand, name = "bc7-split-by-block-type")]
pub struct SplitByBlockTypeCmd {
    /// input directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub input: PathBuf,

    /// output directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub output: PathBuf,
}

#[derive(FromArgs, Debug)]
/// Transforms BC7 files by injecting the first 3 p-bits after the partition
/// such that the colour bytes are byte aligned.
#[argh(subcommand, name = "bc7-byte-align-mode-0-blocks")]
pub struct ByteAlignMode0Blocks {
    /// input directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub input: PathBuf,

    /// output directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub output: PathBuf,
}

#[derive(FromArgs, Debug)]
/// Changes the packed 'transform' value in mode 0 blocks to the most common value
#[argh(subcommand, name = "bc7-set-mode-0-transform-to-most-common")]
pub struct SetMode0TransformToMostCommon {
    /// input directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub input: PathBuf,

    /// output directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub output: PathBuf,
}

#[derive(FromArgs, Debug)]
/// Removes non-mode 0 blocks
#[argh(subcommand, name = "bc7-remove-non-mode-0-blocks")]
pub struct Bc7RemoveNonMode0Blocks {
    /// input directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub input: PathBuf,

    /// output directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub output: PathBuf,
}

#[derive(FromArgs, Debug)]
/// Transforms mode 0 blocks to structure of array.
#[argh(subcommand, name = "bc7-mode-0-to-structure-of-array")]
pub struct Bc7Mode0ToStructureOfArray {
    /// input directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub input: PathBuf,

    /// output directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub output: PathBuf,
}

pub fn handle_debug_command(cmd: DebugCmd) -> Result<(), TransformError> {
    match cmd.command {
        DebugCommands::AnalyzeBC7(analyze_cmd) => {
            let analysis = analyze_bc7_directory(&analyze_cmd.input)?;
            analysis.print_results();
            Ok(())
        }
        DebugCommands::SplitByBlockTypeCmd(cmd) => {
            // Collect all files recursively first
            let mut entries = Vec::new();
            find_all_files(&cmd.input, &mut entries)?;

            // Process files in parallel
            entries.par_iter().for_each(|entry| {
                let process_entry_result = process_dir_entry(
                    entry,
                    &cmd.input,
                    &cmd.output,
                    crate::DdsFilter::All,
                    transform_bc7_split_block_type,
                    &(),
                );
                handle_process_entry_error(process_entry_result);
            });

            Ok(())
        }
        DebugCommands::ByteAlignMode0Blocks(cmd) => {
            // Collect all files recursively first
            let mut entries = Vec::new();
            find_all_files(&cmd.input, &mut entries)?;

            // Process files in parallel
            entries.par_iter().for_each(|entry| {
                let process_entry_result = process_dir_entry(
                    entry,
                    &cmd.input,
                    &cmd.output,
                    crate::DdsFilter::All,
                    transform_bc7_mode0_blocks,
                    &(),
                );
                handle_process_entry_error(process_entry_result);
            });

            Ok(())
        }
        DebugCommands::SetMode0TransformToMostCommon(cmd) => {
            let analysis = analyze_bc7_directory(&cmd.input)?;
            // Collect all files recursively first
            let mut entries = Vec::new();
            find_all_files(&cmd.input, &mut entries)?;

            // Process files in parallel
            entries.par_iter().for_each(|entry| {
                let process_entry_result = process_dir_entry(
                    entry,
                    &cmd.input,
                    &cmd.output,
                    crate::DdsFilter::All,
                    mode0_normalize_partition,
                    &analysis,
                );
                handle_process_entry_error(process_entry_result);
            });

            Ok(())
        }
        DebugCommands::RemoveNonMode0Blocks(cmd) => {
            // Collect all files recursively first
            let mut entries = Vec::new();
            find_all_files(&cmd.input, &mut entries)?;

            for entry in entries {
                _ = remove_non_mode0_blocks(&entry, &cmd.input, &cmd.output);
            }

            Ok(())
        }
        DebugCommands::Mode0ToStructureOfArray(cmd) => {
            // Collect all files recursively first
            let mut entries = Vec::new();
            find_all_files(&cmd.input, &mut entries)?;

            for entry in entries {
                _ = split_mode0_blocks(&entry, &cmd.input, &cmd.output);
            }

            Ok(())
        }
        DebugCommands::AnalyzeBC7Mode0BitDistributions(cmd) => {
            handle_analyze_bc7_mode0_bit_distributions(&cmd)
        }
    }
}

pub fn handle_analyze_bc7_mode0_bit_distributions(
    cmd: &AnalyzeBC7Mode0BitDistributionsCmd,
) -> Result<(), TransformError> {
    let mut entries = Vec::new();
    find_all_files(&cmd.input, &mut entries)?;

    let mut combined_distribution = BC7Mode0BitDistribution::new();

    for entry in entries {
        if let Ok(data) = fs::read(entry.path()) {
            // Skip the DDS header
            let dds_info = unsafe { parse_dds(data.as_ptr(), data.len()) }.unwrap();
            let data = &data[dds_info.data_offset as usize..];

            if let Ok(distribution) = analyze_bc7_mode0_bits(data) {
                // Add this file's distribution to combined total
                combined_distribution.total_blocks += distribution.total_blocks;

                // Combine partition bits
                for i in 0..4 {
                    for j in 0..2 {
                        combined_distribution.partition_bits[i][j] +=
                            distribution.partition_bits[i][j];
                    }
                }

                // Combine R endpoint bits
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.r0_bits,
                    &distribution.r0_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.r1_bits,
                    &distribution.r1_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.r2_bits,
                    &distribution.r2_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.r3_bits,
                    &distribution.r3_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.r4_bits,
                    &distribution.r4_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.r5_bits,
                    &distribution.r5_bits,
                );

                // Combine G endpoint bits
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.g0_bits,
                    &distribution.g0_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.g1_bits,
                    &distribution.g1_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.g2_bits,
                    &distribution.g2_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.g3_bits,
                    &distribution.g3_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.g4_bits,
                    &distribution.g4_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.g5_bits,
                    &distribution.g5_bits,
                );

                // Combine B endpoint bits
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.b0_bits,
                    &distribution.b0_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.b1_bits,
                    &distribution.b1_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.b2_bits,
                    &distribution.b2_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.b3_bits,
                    &distribution.b3_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.b4_bits,
                    &distribution.b4_bits,
                );
                BC7Mode0BitDistribution::combine_endpoint_bits(
                    &mut combined_distribution.b5_bits,
                    &distribution.b5_bits,
                );

                // Combine p bits
                for i in 0..6 {
                    for j in 0..2 {
                        combined_distribution.p_bits[i][j] += distribution.p_bits[i][j];
                    }
                }

                // Combine index bits
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index0_bits,
                    &distribution.index0_bits,
                );
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index1_bits,
                    &distribution.index1_bits,
                );
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index2_bits,
                    &distribution.index2_bits,
                );
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index3_bits,
                    &distribution.index3_bits,
                );
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index4_bits,
                    &distribution.index4_bits,
                );
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index5_bits,
                    &distribution.index5_bits,
                );
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index6_bits,
                    &distribution.index6_bits,
                );
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index7_bits,
                    &distribution.index7_bits,
                );
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index8_bits,
                    &distribution.index8_bits,
                );
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index9_bits,
                    &distribution.index9_bits,
                );
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index10_bits,
                    &distribution.index10_bits,
                );
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index11_bits,
                    &distribution.index11_bits,
                );
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index12_bits,
                    &distribution.index12_bits,
                );
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index13_bits,
                    &distribution.index13_bits,
                );
                BC7Mode0BitDistribution::combine_index_bits(
                    &mut combined_distribution.index14_bits,
                    &distribution.index14_bits,
                );
            }
        }
    }

    // Print the combined results
    combined_distribution.print_results();
    Ok(())
}

#[inline]
pub unsafe fn transform_bc7_split_block_type(
    _param: &(),
    mut input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    _format: DdsFormat,
) {
    let mut block_type_ptr = output_ptr;
    let block_type_end = block_type_ptr.add(len / 16); // each type is 1 byte
    let mut block_data_ptr = block_type_end;

    while block_type_ptr < block_type_end {
        let block_type = *input_ptr;

        *block_type_ptr = block_type;
        copy_nonoverlapping(input_ptr.add(1), block_data_ptr, 15); // remaining block bytes

        block_type_ptr = block_type_ptr.add(1);
        block_data_ptr = block_data_ptr.add(15);
        input_ptr = input_ptr.add(16);
    }
}

#[inline]
pub unsafe fn transform_bc7_mode0_blocks(
    _param: &(),
    mut input_ptr: *const u8,
    mut output_ptr: *mut u8,
    len: usize,
    _format: DdsFormat,
) {
    let input_end = input_ptr.add(len);

    while input_ptr < input_end {
        let block_type = *input_ptr;
        let mode = get_bc7_mode(block_type);

        if mode != 0 {
            // Skip non-mode 0 blocks
            copy_nonoverlapping(input_ptr, output_ptr, 16);
            input_ptr = input_ptr.add(16);
            output_ptr = output_ptr.add(16);
            continue;
        }

        mode0_rearrange_bits(input_ptr, output_ptr);

        // Advance pointers
        input_ptr = input_ptr.add(16);
        output_ptr = output_ptr.add(16);
    }
}

unsafe fn mode0_rearrange_bits(input_ptr: *const u8, output_ptr: *mut u8) {
    /*
        The BC7 blocks use the following format:

        1 bit type
        4 bit partition
        72 bits of colour
        6 p bits
        45 bits of index


        We're rearranging such that the colour bits are byte aligned.
        This will improve compression ratio by making these bytes more compressible.

        To do this, we need to insert 3 p bits after the partition, and shift the 72 bits of colour forward.
    */

    // Grab pointer to the entire block (16 bytes)
    let first_u64 = (input_ptr as *const u64).read_unaligned();

    // Write out the first byte.
    let p_bits = extract_msb_bits(*input_ptr.add(9), 5, 7);
    let first_byte = insert_msb_bits(first_u64 as u8, p_bits, 5, 7);
    output_ptr.write_unaligned(first_byte); // ok

    // We shift left to get rid of mode + partition bits.
    // For this we need to byte swap on LittleEndian, because the mode + partition bits are
    // stored in upper bits and we need to get them on register edge.

    // Byte swap ensures register layout is this:
    // [Byte0][Byte1][Byte2][Byte3][Byte4][Byte5][Byte6][Byte7]
    let colours = first_u64.to_be();
    // mpppprrr
    let colours = colours << 5;
    // -> rrr ...
    let b2_b3: u16 = ((input_ptr.add(7) as *const u16).read_unaligned()).to_be() << 5;
    // 0x9B − 0x94 == 8
    let b2_b3 = (b2_b3 >> 8) as u8;
    // extract lower byte
    let colours = colours | b2_b3 as u64;
    let colours = to_orig_endian(colours);
    (output_ptr.add(1) as *mut u64).write_unaligned(colours);

    // Now I write remaining colour b4, b5
    let b4_b5: u16 = ((input_ptr.add(8) as *const u16).read_unaligned()).to_be() << 5;
    // 0x9C − 0x94 == 8
    let b4_b5 = (b4_b5 >> 8) as u8;
    // extract lower byte
    *(output_ptr.add(9)) = b4_b5;
    // ok

    // The 'shift' is complete. We now can copy the rest of the block.
    copy_nonoverlapping(input_ptr.add(10), output_ptr.add(10), 6);
    // ok
}

unsafe fn mode0_normalize_partition(
    analysis: &BC7BlockAnalysis,
    mut input_ptr: *const u8,
    mut output_ptr: *mut u8,
    len: usize,
    _format: DdsFormat,
) {
    /*
        The BC7 blocks use the following format:

        1 bit type
        4 bit partition
        72 bits of colour
        6 p bits
        45 bits of index


        We're rearranging such that the colour bits are byte aligned.
        This will improve compression ratio by making these bytes more compressible.

        To do this, we need to insert 3 p bits after the partition, and shift the 72 bits of colour forward.
    */

    let most_common_partition = analysis.most_common_partition_bits().unwrap();
    let most_common_colour = analysis.most_common_color().unwrap();

    let input_end = input_ptr.add(len);
    while input_ptr < input_end {
        let block_type = *input_ptr;
        let mode = get_bc7_mode(block_type);

        if mode != 0 {
            // Skip non-mode 0 blocks
            copy_nonoverlapping(input_ptr, output_ptr, 16);
            input_ptr = input_ptr.add(16);
            output_ptr = output_ptr.add(16);
            continue;
        }

        // Read first byte
        let mut first_byte = *input_ptr.add(0);

        // Patch partition
        let partition = extract_msb_bits(first_byte, 1, 4);
        let bits = (partition - most_common_partition) % 16;
        first_byte = insert_msb_bits(first_byte, bits, 1, 4);

        // Patch colour
        let colour = extract_msb_bits(first_byte, 5, 7);
        let bits = (colour - most_common_colour) % 8;
        first_byte = insert_msb_bits(first_byte, bits, 5, 7);

        copy_nonoverlapping(input_ptr.add(1), output_ptr.add(1), 15);
        *output_ptr = first_byte;

        // Advance pointers
        input_ptr = input_ptr.add(16);
        output_ptr = output_ptr.add(16);
    }
}

fn remove_non_mode0_blocks(
    dir_entry: &fs::DirEntry,
    input: &Path,
    output: &Path,
) -> Result<(), TransformError> {
    let path = dir_entry.path();
    let relative = path.strip_prefix(input).unwrap();
    let target_path = output.join(relative);

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let source_file = fs::read(path).unwrap();
    let mut target_file = Vec::<u8>::new();

    let dds_info = unsafe { parse_dds(source_file.as_ptr(), source_file.len()).unwrap() };

    // Copy DDS headers.
    target_file.extend_from_slice(&source_file[..dds_info.data_offset as usize]);

    // Now read all blocks
    unsafe {
        let mut input_ptr = source_file.as_ptr().add(dds_info.data_offset as usize);
        let data_len = source_file.len() - dds_info.data_offset as usize;
        while input_ptr < source_file.as_ptr().add(data_len) {
            let block_type = *input_ptr;
            let mode = get_bc7_mode(block_type);

            if mode != 0 {
                // Skip non-mode 0 blocks
                input_ptr = input_ptr.add(16);
                continue;
            }

            let block_slice = slice::from_raw_parts(input_ptr, 16);
            target_file.extend_from_slice(block_slice);
            input_ptr = input_ptr.add(16);
        }
    }

    fs::write(target_path, target_file).unwrap();
    Ok(())
}

/// Splits mode 0 blocks into structure of arrays.
fn split_mode0_blocks(
    dir_entry: &fs::DirEntry,
    input: &Path,
    output: &Path,
) -> Result<(), TransformError> {
    let path = dir_entry.path();
    let relative = path.strip_prefix(input).unwrap();
    let target_path = output.join(relative);

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let source_file = fs::read(path).unwrap();
    let mut target_file = Vec::<u8>::new();
    let dds_info = unsafe { parse_dds(source_file.as_ptr(), source_file.len()) };
    if dds_info.is_none() {
        return Err(TransformError::InvalidDdsFile);
    }
    let dds_info = dds_info.unwrap();

    // Copy DDS headers.
    target_file.extend_from_slice(&source_file[..dds_info.data_offset as usize]);
    mode0_structure_of_array_mode_partition_colour_bycolourchannel_deltaencoded(
        source_file,
        &mut target_file,
        dds_info,
    );

    fs::write(target_path, target_file).unwrap();
    Ok(())
}

#[allow(dead_code)]
fn mode0_delta_encode_colours(source_file: Vec<u8>, target_file: &mut Vec<u8>, dds_info: DdsInfo) {
    // Now read all blocks
    let mut all_bytes = Vec::new();
    let mut writer_all = BitWriter::endian(Cursor::new(&mut all_bytes), BigEndian);

    let data_len_bits = (source_file.len() as u64 - dds_info.data_offset as u64) * 8;
    let mut input_reader = BitReader::endian(Cursor::new(&source_file), BigEndian);
    input_reader
        .seek_bits(SeekFrom::Start(dds_info.data_offset as u64 * 8))
        .unwrap();

    let end_pos = input_reader.position_in_bits().unwrap() + data_len_bits;
    while input_reader.position_in_bits().unwrap() < end_pos {
        // Write mode and partition
        writer_all
            .write(5, input_reader.read::<u64>(5).unwrap())
            .unwrap();

        // Write R channels
        let mut prev = 0;
        for _ in 0..3 {
            let value = input_reader.read::<u8>(8).unwrap();
            writer_all.write(8, value ^ prev).unwrap();
            prev = value;
        }

        // Write G channels
        let mut prev = 0;
        for _ in 0..3 {
            let value = input_reader.read::<u8>(8).unwrap();
            writer_all.write(8, value ^ prev).unwrap();
            prev = value;
        }

        // Write B channels
        let mut prev = 0;
        for _ in 0..3 {
            let value = input_reader.read::<u8>(8).unwrap();
            writer_all.write(8, value ^ prev).unwrap();
            prev = value;
        }

        // Write remaining bits
        writer_all
            .write(51, input_reader.read::<u64>(51).unwrap())
            .unwrap();
    }

    target_file.extend(&all_bytes);
}

#[allow(dead_code)]
fn mode0_group_endpoints(source_file: Vec<u8>, target_file: &mut Vec<u8>, dds_info: DdsInfo) {
    // Now read all blocks
    let mut all_bytes = Vec::new();
    let mut out = BitWriter::endian(Cursor::new(&mut all_bytes), BigEndian);

    let data_len_bits = (source_file.len() as u64 - dds_info.data_offset as u64) * 8;
    let mut inp = BitReader::endian(Cursor::new(&source_file), BigEndian);
    inp.seek_bits(SeekFrom::Start(dds_info.data_offset as u64 * 8))
        .unwrap();

    let end_pos = inp.position_in_bits().unwrap() + data_len_bits;
    while inp.position_in_bits().unwrap() < end_pos {
        // Write mode + partition
        let mode_partition = inp.read::<u32>(5).unwrap();
        out.write(5, mode_partition).unwrap();

        // Write RGB, paired
        let starting_pos = inp.position_in_bits().unwrap();
        let stride_g = 24;
        let stride_b = 24 * 2;

        // First pair
        bitstream_copy(&mut inp, &mut out, starting_pos, 0, 4);
        bitstream_copy(&mut inp, &mut out, starting_pos, stride_g, 4);
        bitstream_copy(&mut inp, &mut out, starting_pos, stride_b, 4);

        bitstream_copy(&mut inp, &mut out, starting_pos, 4, 4);
        bitstream_copy(&mut inp, &mut out, starting_pos, 4 + stride_g, 4);
        bitstream_copy(&mut inp, &mut out, starting_pos, 4 + stride_b, 4);
        // 3 bytes written

        // Second pair
        bitstream_copy(&mut inp, &mut out, starting_pos, 8, 4);
        bitstream_copy(&mut inp, &mut out, starting_pos, 8 + stride_g, 4);
        bitstream_copy(&mut inp, &mut out, starting_pos, 8 + stride_b, 4);

        bitstream_copy(&mut inp, &mut out, starting_pos, 12, 4);
        bitstream_copy(&mut inp, &mut out, starting_pos, 12 + stride_g, 4);
        bitstream_copy(&mut inp, &mut out, starting_pos, 12 + stride_b, 4);
        // 3 bytes written

        // Third pair
        bitstream_copy(&mut inp, &mut out, starting_pos, 16, 4);
        bitstream_copy(&mut inp, &mut out, starting_pos, 16 + stride_g, 4);
        bitstream_copy(&mut inp, &mut out, starting_pos, 16 + stride_b, 4);

        bitstream_copy(&mut inp, &mut out, starting_pos, 20, 4);
        bitstream_copy(&mut inp, &mut out, starting_pos, 20 + stride_g, 4);
        bitstream_copy(&mut inp, &mut out, starting_pos, 20 + stride_b, 4);
        // 3 bytes written

        // Write rest
        out.write(6, inp.read::<u32>(6).unwrap()).unwrap(); // p bits
        out.write(45, inp.read::<u64>(45).unwrap()).unwrap(); // index bits
    }

    target_file.extend(&all_bytes);
}

#[allow(dead_code)]
fn mode0_structure_of_array_noop(
    source_file: Vec<u8>,
    target_file: &mut Vec<u8>,
    dds_info: DdsInfo,
) {
    // Now read all blocks
    let mut all_bytes = Vec::new();
    let mut writer_all = BitWriter::endian(Cursor::new(&mut all_bytes), BigEndian);

    let data_len_bits = (source_file.len() as u64 - dds_info.data_offset as u64) * 8;
    let mut input_reader = BitReader::endian(Cursor::new(&source_file), BigEndian);
    input_reader
        .seek_bits(SeekFrom::Start(dds_info.data_offset as u64 * 8))
        .unwrap();

    let end_pos = input_reader.position_in_bits().unwrap() + data_len_bits;
    while input_reader.position_in_bits().unwrap() < end_pos {
        // Write all
        writer_all
            .write(64, input_reader.read::<u64>(64).unwrap())
            .unwrap();
        writer_all
            .write(64, input_reader.read::<u64>(64).unwrap())
            .unwrap();
    }

    target_file.extend(&all_bytes);
}

#[allow(dead_code)]
fn mode0_structure_of_array_mode_partition_colour_index_injectallpbits(
    source_file: Vec<u8>,
    target_file: &mut Vec<u8>,
    dds_info: DdsInfo,
) {
    // Now read all blocks
    let mut mode_part_pbytes = Vec::new();
    let mut writer_mode_part_p = BitWriter::endian(Cursor::new(&mut mode_part_pbytes), BigEndian);

    let mut rgb_bytes = Vec::new();
    let mut writer_rgb = BitWriter::endian(Cursor::new(&mut rgb_bytes), BigEndian);

    let mut index_bytes = Vec::new();
    let mut writer_index = BitWriter::endian(Cursor::new(&mut index_bytes), BigEndian);

    let data_len_bits = (source_file.len() as u64 - dds_info.data_offset as u64) * 8;
    let mut input_reader = BitReader::endian(Cursor::new(&source_file), BigEndian);
    input_reader
        .seek_bits(SeekFrom::Start(dds_info.data_offset as u64 * 8))
        .unwrap();

    let end_pos = input_reader.position_in_bits().unwrap() + data_len_bits;
    while input_reader.position_in_bits().unwrap() < end_pos {
        // Write mode
        writer_mode_part_p
            .write(1, input_reader.read::<u32>(1).unwrap())
            .unwrap();

        // Write partition
        writer_mode_part_p
            .write(4, input_reader.read::<u32>(4).unwrap())
            .unwrap();

        // Write rgb
        // Single Block
        writer_rgb
            .write(32, input_reader.read::<u32>(32).unwrap())
            .unwrap();
        writer_rgb
            .write(32, input_reader.read::<u32>(32).unwrap())
            .unwrap();
        writer_rgb
            .write(8, input_reader.read::<u32>(8).unwrap())
            .unwrap();

        // Write p bits
        let first_p_bits = input_reader.read::<u32>(3).unwrap();
        writer_mode_part_p.write(3, first_p_bits).unwrap();
        let second_p_bits = input_reader.read::<u32>(3).unwrap();

        // Write index
        writer_index
            .write(32, input_reader.read::<u32>(32).unwrap())
            .unwrap();
        writer_index
            .write(13, input_reader.read::<u32>(13).unwrap())
            .unwrap();
        writer_index.write(3, second_p_bits).unwrap(); // alignment check
    }

    target_file.extend(&mode_part_pbytes);
    target_file.extend(&rgb_bytes);
    target_file.extend(&index_bytes);
}

#[allow(dead_code)]
fn mode0_structure_of_array_mode_partition_colour_index_alignall(
    source_file: Vec<u8>,
    target_file: &mut Vec<u8>,
    dds_info: DdsInfo,
) {
    // Now read all blocks
    let mut mode_bytes = Vec::new();
    let mut writer_mode = BitWriter::endian(Cursor::new(&mut mode_bytes), BigEndian);

    let mut partition_bytes = Vec::new();
    let mut writer_partition = BitWriter::endian(Cursor::new(&mut partition_bytes), BigEndian);

    let mut rgb_bytes = Vec::new();
    let mut writer_rgb = BitWriter::endian(Cursor::new(&mut rgb_bytes), BigEndian);

    let mut p_bytes = Vec::new();
    let mut writer_p = BitWriter::endian(Cursor::new(&mut p_bytes), BigEndian);

    let mut index_bytes = Vec::new();
    let mut writer_index = BitWriter::endian(Cursor::new(&mut index_bytes), BigEndian);

    let data_len_bits = (source_file.len() as u64 - dds_info.data_offset as u64) * 8;
    let mut input_reader = BitReader::endian(Cursor::new(&source_file), BigEndian);
    input_reader
        .seek_bits(SeekFrom::Start(dds_info.data_offset as u64 * 8))
        .unwrap();

    let end_pos = input_reader.position_in_bits().unwrap() + data_len_bits;
    while input_reader.position_in_bits().unwrap() < end_pos {
        // Write mode
        writer_mode
            .write(1, input_reader.read::<u32>(1).unwrap())
            .unwrap();

        // Write partition
        writer_partition
            .write(4, input_reader.read::<u32>(4).unwrap())
            .unwrap();

        // Write rgb
        // Single Block
        writer_rgb
            .write(32, input_reader.read::<u32>(32).unwrap())
            .unwrap();
        writer_rgb
            .write(32, input_reader.read::<u32>(32).unwrap())
            .unwrap();
        writer_rgb
            .write(8, input_reader.read::<u32>(8).unwrap())
            .unwrap();

        // Write p bits
        writer_p
            .write(6, input_reader.read::<u32>(6).unwrap())
            .unwrap();
        writer_p.write(2, 0).unwrap(); // alignment check

        // Write index
        writer_index
            .write(32, input_reader.read::<u32>(32).unwrap())
            .unwrap();
        writer_index
            .write(13, input_reader.read::<u32>(13).unwrap())
            .unwrap();
        writer_index.write(3, 0).unwrap(); // alignment check
    }

    target_file.extend(&mode_bytes);
    target_file.extend(&partition_bytes);
    target_file.extend(&rgb_bytes);
    target_file.extend(&p_bytes);
    target_file.extend(&index_bytes);
}

#[allow(dead_code)]
fn mode0_structure_of_array_mode_partition_colour_index_alignall_mixpandindex(
    source_file: Vec<u8>,
    target_file: &mut Vec<u8>,
    dds_info: DdsInfo,
) {
    // Now read all blocks
    let mut mode_bytes = Vec::new();
    let mut writer_mode = BitWriter::endian(Cursor::new(&mut mode_bytes), BigEndian);

    let mut partition_bytes = Vec::new();
    let mut writer_partition = BitWriter::endian(Cursor::new(&mut partition_bytes), BigEndian);

    let mut rgb_bytes = Vec::new();
    let mut writer_rgb = BitWriter::endian(Cursor::new(&mut rgb_bytes), BigEndian);

    let mut p_bytes = Vec::new();
    let mut writer_p = BitWriter::endian(Cursor::new(&mut p_bytes), BigEndian);

    let mut index_bytes = Vec::new();
    let mut writer_index = BitWriter::endian(Cursor::new(&mut index_bytes), BigEndian);

    let data_len_bits = (source_file.len() as u64 - dds_info.data_offset as u64) * 8;
    let mut input_reader = BitReader::endian(Cursor::new(&source_file), BigEndian);
    input_reader
        .seek_bits(SeekFrom::Start(dds_info.data_offset as u64 * 8))
        .unwrap();

    let end_pos = input_reader.position_in_bits().unwrap() + data_len_bits;
    while input_reader.position_in_bits().unwrap() < end_pos {
        // Write mode
        writer_mode
            .write(1, input_reader.read::<u32>(1).unwrap())
            .unwrap();

        // Write partition
        writer_partition
            .write(4, input_reader.read::<u32>(4).unwrap())
            .unwrap();

        // Write rgb
        // Single Block
        writer_rgb
            .write(32, input_reader.read::<u32>(32).unwrap())
            .unwrap();
        writer_rgb
            .write(32, input_reader.read::<u32>(32).unwrap())
            .unwrap();
        writer_rgb
            .write(8, input_reader.read::<u32>(8).unwrap())
            .unwrap();

        // Write p bits
        let two_p_bits = input_reader.read::<u32>(2).unwrap();
        writer_p
            .write(4, input_reader.read::<u32>(4).unwrap())
            .unwrap();

        // Write index
        writer_index
            .write(32, input_reader.read::<u32>(32).unwrap())
            .unwrap();
        writer_index
            .write(13, input_reader.read::<u32>(13).unwrap())
            .unwrap();
        writer_index.write(2, two_p_bits).unwrap(); // alignment check
        writer_index.write(1, 0).unwrap(); // alignment check
    }

    target_file.extend(&mode_bytes);
    target_file.extend(&partition_bytes);
    target_file.extend(&rgb_bytes);
    target_file.extend(&p_bytes);
    target_file.extend(&index_bytes);
}

#[allow(dead_code)]
fn mode0_structure_of_array_mode_partition_colour_byendpointchannelcolour(
    source_file: Vec<u8>,
    target_file: &mut Vec<u8>,
    dds_info: DdsInfo,
) {
    // Now read all blocks
    let mut mode_bytes = Vec::new();
    let mut writer_mode = BitWriter::endian(Cursor::new(&mut mode_bytes), BigEndian);

    let mut partition_bytes = Vec::new();
    let mut writer_partition = BitWriter::endian(Cursor::new(&mut partition_bytes), BigEndian);

    let mut r1_bytes = Vec::new();
    let mut writer_r1 = BitWriter::endian(Cursor::new(&mut r1_bytes), BigEndian);

    let mut g1_bytes = Vec::new();
    let mut writer_g1 = BitWriter::endian(Cursor::new(&mut g1_bytes), BigEndian);

    let mut b1_bytes = Vec::new();
    let mut writer_b1 = BitWriter::endian(Cursor::new(&mut b1_bytes), BigEndian);

    let mut r2_bytes = Vec::new();
    let mut writer_r2 = BitWriter::endian(Cursor::new(&mut r2_bytes), BigEndian);

    let mut g2_bytes = Vec::new();
    let mut writer_g2 = BitWriter::endian(Cursor::new(&mut g2_bytes), BigEndian);

    let mut b2_bytes = Vec::new();
    let mut writer_b2 = BitWriter::endian(Cursor::new(&mut b2_bytes), BigEndian);

    let mut r3_bytes = Vec::new();
    let mut writer_r3 = BitWriter::endian(Cursor::new(&mut r3_bytes), BigEndian);

    let mut g3_bytes = Vec::new();
    let mut writer_g3 = BitWriter::endian(Cursor::new(&mut g3_bytes), BigEndian);

    let mut b3_bytes = Vec::new();
    let mut writer_b3 = BitWriter::endian(Cursor::new(&mut b3_bytes), BigEndian);

    let mut p_bytes = Vec::new();
    let mut writer_p = BitWriter::endian(Cursor::new(&mut p_bytes), BigEndian);

    let mut index_bytes = Vec::new();
    let mut writer_index = BitWriter::endian(Cursor::new(&mut index_bytes), BigEndian);

    let data_len_bits = (source_file.len() as u64 - dds_info.data_offset as u64) * 8;
    let mut input_reader = BitReader::endian(Cursor::new(&source_file), BigEndian);
    input_reader
        .seek_bits(SeekFrom::Start(dds_info.data_offset as u64 * 8))
        .unwrap();

    let end_pos = input_reader.position_in_bits().unwrap() + data_len_bits;
    while input_reader.position_in_bits().unwrap() < end_pos {
        // Write mode
        writer_mode
            .write(1, input_reader.read::<u32>(1).unwrap())
            .unwrap();

        // Write partition
        writer_partition
            .write(4, input_reader.read::<u32>(4).unwrap())
            .unwrap();

        // By channel for each endpoint
        writer_r1
            .write(8, input_reader.read::<u32>(8).unwrap())
            .unwrap();

        writer_r2
            .write(8, input_reader.read::<u32>(8).unwrap())
            .unwrap();

        writer_r3
            .write(8, input_reader.read::<u32>(8).unwrap())
            .unwrap();

        writer_g1
            .write(8, input_reader.read::<u32>(8).unwrap())
            .unwrap();

        writer_g2
            .write(8, input_reader.read::<u32>(8).unwrap())
            .unwrap();

        writer_g3
            .write(8, input_reader.read::<u32>(8).unwrap())
            .unwrap();

        writer_b1
            .write(8, input_reader.read::<u32>(8).unwrap())
            .unwrap();

        writer_b2
            .write(8, input_reader.read::<u32>(8).unwrap())
            .unwrap();

        writer_b3
            .write(8, input_reader.read::<u32>(8).unwrap())
            .unwrap();

        // Write p bits
        writer_p
            .write(6, input_reader.read::<u32>(6).unwrap())
            .unwrap();
        writer_p.write(2, 0).unwrap();

        // Write index
        writer_index
            .write(32, input_reader.read::<u32>(32).unwrap())
            .unwrap();
        writer_index
            .write(13, input_reader.read::<u32>(13).unwrap())
            .unwrap();
        writer_index.write(3, 0).unwrap();
    }

    target_file.extend(&mode_bytes);
    target_file.extend(&partition_bytes);

    target_file.extend(&r1_bytes);
    target_file.extend(&g1_bytes);
    target_file.extend(&b1_bytes);

    target_file.extend(&r2_bytes);
    target_file.extend(&g2_bytes);
    target_file.extend(&b2_bytes);

    target_file.extend(&r3_bytes);
    target_file.extend(&g3_bytes);
    target_file.extend(&b3_bytes);

    target_file.extend(&p_bytes);
    target_file.extend(&index_bytes);
}

#[allow(dead_code)]
fn mode0_structure_of_array_mode_partition_colour_bycolourchannel(
    source_file: Vec<u8>,
    target_file: &mut Vec<u8>,
    dds_info: DdsInfo,
) {
    // Now read all blocks
    let mut mode_bytes = Vec::new();
    let mut writer_mode = BitWriter::endian(Cursor::new(&mut mode_bytes), BigEndian);

    let mut partition_bytes = Vec::new();
    let mut writer_partition = BitWriter::endian(Cursor::new(&mut partition_bytes), BigEndian);

    let mut r_bytes = Vec::new();
    let mut writer_r = BitWriter::endian(Cursor::new(&mut r_bytes), BigEndian);

    let mut g_bytes = Vec::new();
    let mut writer_g = BitWriter::endian(Cursor::new(&mut g_bytes), BigEndian);

    let mut b_bytes = Vec::new();
    let mut writer_b = BitWriter::endian(Cursor::new(&mut b_bytes), BigEndian);

    let mut p_bytes = Vec::new();
    let mut writer_p = BitWriter::endian(Cursor::new(&mut p_bytes), BigEndian);

    let mut index_bytes = Vec::new();
    let mut writer_index = BitWriter::endian(Cursor::new(&mut index_bytes), BigEndian);

    let data_len_bits = (source_file.len() as u64 - dds_info.data_offset as u64) * 8;
    let mut input_reader = BitReader::endian(Cursor::new(&source_file), BigEndian);
    input_reader
        .seek_bits(SeekFrom::Start(dds_info.data_offset as u64 * 8))
        .unwrap();

    let end_pos = input_reader.position_in_bits().unwrap() + data_len_bits;
    while input_reader.position_in_bits().unwrap() < end_pos {
        // Write mode
        writer_mode
            .write(1, input_reader.read::<u32>(1).unwrap())
            .unwrap();

        // Write partition
        writer_partition
            .write(4, input_reader.read::<u32>(4).unwrap())
            .unwrap();

        // Separate channels
        writer_r
            .write(24, input_reader.read::<u32>(24).unwrap())
            .unwrap();
        writer_g
            .write(24, input_reader.read::<u32>(24).unwrap())
            .unwrap();
        writer_b
            .write(24, input_reader.read::<u32>(24).unwrap())
            .unwrap();

        // Write p bits
        writer_p
            .write(6, input_reader.read::<u32>(6).unwrap())
            .unwrap();
        writer_p.write(2, 0).unwrap();

        // Write index
        writer_index
            .write(32, input_reader.read::<u32>(32).unwrap())
            .unwrap();
        writer_index
            .write(13, input_reader.read::<u32>(13).unwrap())
            .unwrap();
        writer_index.write(3, 0).unwrap();
    }

    target_file.extend(&mode_bytes);
    target_file.extend(&partition_bytes);
    target_file.extend(&r_bytes);
    target_file.extend(&g_bytes);
    target_file.extend(&b_bytes);

    target_file.extend(&p_bytes);
    target_file.extend(&index_bytes);
}

#[allow(dead_code)]
fn mode0_structure_of_array_mode_partition_colour_bycolourchannel_deltaencoded(
    source_file: Vec<u8>,
    target_file: &mut Vec<u8>,
    dds_info: DdsInfo,
) {
    // Now read all blocks
    let mut mode_bytes = Vec::new();
    let mut writer_mode = BitWriter::endian(Cursor::new(&mut mode_bytes), BigEndian);

    let mut partition_bytes = Vec::new();
    let mut writer_partition = BitWriter::endian(Cursor::new(&mut partition_bytes), BigEndian);

    let mut r_bytes = Vec::new();
    let mut writer_r = BitWriter::endian(Cursor::new(&mut r_bytes), BigEndian);

    let mut g_bytes = Vec::new();
    let mut writer_g = BitWriter::endian(Cursor::new(&mut g_bytes), BigEndian);

    let mut b_bytes = Vec::new();
    let mut writer_b = BitWriter::endian(Cursor::new(&mut b_bytes), BigEndian);

    let mut p_bytes = Vec::new();
    let mut writer_p = BitWriter::endian(Cursor::new(&mut p_bytes), BigEndian);

    let mut index_bytes = Vec::new();
    let mut writer_index = BitWriter::endian(Cursor::new(&mut index_bytes), BigEndian);

    let data_len_bits = (source_file.len() as u64 - dds_info.data_offset as u64) * 8;
    let mut input_reader = BitReader::endian(Cursor::new(&source_file), BigEndian);
    input_reader
        .seek_bits(SeekFrom::Start(dds_info.data_offset as u64 * 8))
        .unwrap();

    let end_pos = input_reader.position_in_bits().unwrap() + data_len_bits;
    while input_reader.position_in_bits().unwrap() < end_pos {
        // Write mode
        writer_mode
            .write(1, input_reader.read::<u32>(1).unwrap())
            .unwrap();

        // Write partition
        writer_partition
            .write(4, input_reader.read::<u32>(4).unwrap())
            .unwrap();

        // Separate channels
        let mut last = 0_u8;
        for _ in 0..3 {
            let value = input_reader.read::<u8>(8).unwrap();
            writer_r.write(8, value ^ last).unwrap();
            last = value;
        }

        let mut last = 0_u8;
        for _ in 0..3 {
            let value = input_reader.read::<u8>(8).unwrap();
            writer_g.write(8, value ^ last).unwrap();
            last = value;
        }

        let mut last = 0_u8;
        for _ in 0..3 {
            let value = input_reader.read::<u8>(8).unwrap();
            writer_b.write(8, value ^ last).unwrap();
            last = value;
        }
        // Write p bits
        writer_p
            .write(6, input_reader.read::<u32>(6).unwrap())
            .unwrap();
        writer_p.write(2, 0).unwrap();

        // Write index
        writer_index
            .write(32, input_reader.read::<u32>(32).unwrap())
            .unwrap();
        writer_index
            .write(13, input_reader.read::<u32>(13).unwrap())
            .unwrap();
        writer_index.write(3, 0).unwrap();
    }

    target_file.extend(&mode_bytes);
    target_file.extend(&partition_bytes);
    target_file.extend(&r_bytes);
    target_file.extend(&g_bytes);
    target_file.extend(&b_bytes);

    target_file.extend(&p_bytes);
    target_file.extend(&index_bytes);
}

#[allow(dead_code)]
fn mode0_structure_of_array_mode_partition_colour_bycolourchannel_deltaencodedatend(
    source_file: Vec<u8>,
    target_file: &mut Vec<u8>,
    dds_info: DdsInfo,
) {
    // Now read all blocks
    let mut mode_bytes = Vec::new();
    let mut writer_mode = BitWriter::endian(Cursor::new(&mut mode_bytes), BigEndian);

    let mut partition_bytes = Vec::new();
    let mut writer_partition = BitWriter::endian(Cursor::new(&mut partition_bytes), BigEndian);

    let mut r_bytes = Vec::new();
    let mut writer_r = BitWriter::endian(Cursor::new(&mut r_bytes), BigEndian);

    let mut g_bytes = Vec::new();
    let mut writer_g = BitWriter::endian(Cursor::new(&mut g_bytes), BigEndian);

    let mut b_bytes = Vec::new();
    let mut writer_b = BitWriter::endian(Cursor::new(&mut b_bytes), BigEndian);

    let mut p_bytes = Vec::new();
    let mut writer_p = BitWriter::endian(Cursor::new(&mut p_bytes), BigEndian);

    let mut index_bytes = Vec::new();
    let mut writer_index = BitWriter::endian(Cursor::new(&mut index_bytes), BigEndian);

    let data_len_bits = (source_file.len() as u64 - dds_info.data_offset as u64) * 8;
    let mut input_reader = BitReader::endian(Cursor::new(&source_file), BigEndian);
    input_reader
        .seek_bits(SeekFrom::Start(dds_info.data_offset as u64 * 8))
        .unwrap();

    let end_pos = input_reader.position_in_bits().unwrap() + data_len_bits;
    while input_reader.position_in_bits().unwrap() < end_pos {
        // Write mode
        writer_mode
            .write(1, input_reader.read::<u32>(1).unwrap())
            .unwrap();

        // Write partition
        writer_partition
            .write(4, input_reader.read::<u32>(4).unwrap())
            .unwrap();

        // Separate channels
        writer_r
            .write(24, input_reader.read::<u32>(24).unwrap())
            .unwrap();
        writer_g
            .write(24, input_reader.read::<u32>(24).unwrap())
            .unwrap();
        writer_b
            .write(24, input_reader.read::<u32>(24).unwrap())
            .unwrap();

        // Write p bits
        writer_p
            .write(6, input_reader.read::<u32>(6).unwrap())
            .unwrap();
        writer_p.write(2, 0).unwrap();

        // Write index
        writer_index
            .write(32, input_reader.read::<u32>(32).unwrap())
            .unwrap();
        writer_index
            .write(13, input_reader.read::<u32>(13).unwrap())
            .unwrap();
        writer_index.write(3, 0).unwrap();
    }

    let len = r_bytes.len();
    let mut last = 0;
    for x in 0..len {
        let value = r_bytes[x];
        r_bytes[x] = value ^ last;
        last = value;
    }

    let mut last = 0;
    for x in 0..len {
        let value = g_bytes[x];
        g_bytes[x] = value ^ last;
        last = value;
    }

    let mut last = 0;
    for x in 0..len {
        let value = b_bytes[x];
        b_bytes[x] = value ^ last;
        last = value;
    }

    target_file.extend(&mode_bytes);
    target_file.extend(&partition_bytes);
    target_file.extend(&r_bytes);
    target_file.extend(&g_bytes);
    target_file.extend(&b_bytes);

    target_file.extend(&p_bytes);
    target_file.extend(&index_bytes);
}

fn bitstream_copy<T: Endianness>(
    input: &mut BitReader<Cursor<&Vec<u8>>, T>,
    output: &mut BitWriter<Cursor<&mut Vec<u8>>, T>,
    reference_pos: u64,
    offset: u64,
    num_bits: u32,
) {
    input
        .seek_bits(SeekFrom::Start(reference_pos + offset))
        .unwrap();
    let value = input.read::<u64>(num_bits).unwrap();
    output.write(num_bits, value).unwrap();
}

fn to_orig_endian(value: u64) -> u64 {
    #[cfg(target_endian = "little")]
    {
        value.to_be()
    }
    #[cfg(target_endian = "big")]
    {
        value
    }
}
