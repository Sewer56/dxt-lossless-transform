use crate::*;
use dxt_lossless_transform_api::*;
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Default)]
pub struct BC7BlockAnalysis {
    /// Total number of blocks analyzed
    pub total_blocks: u64,
    /// Distribution of modes (0-7)
    pub mode_counts: HashMap<u8, u64>,
    /// Distribution of first bytes in blocks
    pub first_byte_counts: HashMap<u8, u64>,
    /// Distribution of partition values in mode 0 blocks
    pub mode0_partition_counts: HashMap<u8, u64>,
    /// Distribution of r0 color values in mode 0 blocks
    pub mode0_first_color_bits: HashMap<u8, u64>,
}

impl BC7BlockAnalysis {
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyzes a single BC7 block and updates the counts
    pub fn analyze_block(&mut self, block: &[u8]) {
        let mode_byte = block[0];
        *self.first_byte_counts.entry(mode_byte).or_insert(0) += 1;

        let mode = get_bc7_mode(mode_byte);
        *self.mode_counts.entry(mode).or_insert(0) += 1;

        if mode == 0 {
            let partition = get_partition_from_mode_byte(mode_byte);
            *self.mode0_partition_counts.entry(partition).or_insert(0) += 1;

            let color = get_color_from_mode_byte(mode_byte);
            *self.mode0_first_color_bits.entry(color).or_insert(0) += 1;
        }

        self.total_blocks += 1;
    }

    pub fn most_common_partition(&self) -> Option<u8> {
        self.mode0_partition_counts
            .iter()
            .max_by_key(|&(_, count)| *count)
            .map(|(partition, _)| *partition)
    }

    pub fn most_common_partition_bits(&self) -> Option<u8> {
        self.most_common_partition().map(reverse_4bits)
    }

    pub fn most_common_color(&self) -> Option<u8> {
        self.mode0_first_color_bits
            .iter()
            .max_by_key(|&(_, count)| *count)
            .map(|(color, _)| *color)
    }

    pub fn most_common_color_bits(&self) -> Option<u8> {
        self.most_common_color().map(reverse_4bits)
    }

    /// Prints analysis results in a human-readable format
    pub fn print_results(&self) {
        if self.total_blocks == 0 {
            println!("No BC7 blocks found in the directory");
            return;
        }

        println!("\nBC7 Block Type Analysis:");
        println!("Total blocks analyzed: {}", self.total_blocks);

        // Print mode distribution
        println!("\nMode distribution:");
        let mut modes: Vec<_> = self.mode_counts.iter().collect();
        modes.sort_by_key(|&(mode, _)| mode);
        for (mode, count) in modes {
            let percentage = (*count as f64 / self.total_blocks as f64) * 100.0;
            println!("Mode {}: {} blocks ({:.2}%)", mode, count, percentage);
        }

        // Print first byte distribution
        println!("\nMost common first bytes:");
        let mut first_bytes: Vec<_> = self.first_byte_counts.iter().collect();
        first_bytes.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));
        for (byte, count) in first_bytes.iter().take(20) {
            let percentage = ((**count) as f64 / self.total_blocks as f64) * 100.0;
            println!("0x{:02X}: {} blocks ({:.2}%)", byte, count, percentage);
        }

        // Print mode 0 partition distribution
        if !self.mode0_partition_counts.is_empty() {
            println!("\nMode 0 partition distribution:");
            let mut partitions: Vec<_> = self.mode0_partition_counts.iter().collect();
            partitions.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));
            let mode0_total: u64 = self.mode0_partition_counts.values().sum();

            for (partition, count) in partitions {
                let partition_percentage = (*count as f64 / mode0_total as f64) * 100.0;
                let total_percentage = (*count as f64 / self.total_blocks as f64) * 100.0;
                println!(
                    "Partition {}: {} blocks ({:.2}% of mode 0 blocks, {:.2}% of total)",
                    partition, count, partition_percentage, total_percentage
                );
            }
        }

        // Print mode 0 color distribution
        if !self.mode0_first_color_bits.is_empty() {
            println!("\nMode 0 color distribution:");
            let mut colors: Vec<_> = self.mode0_first_color_bits.iter().collect();
            colors.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));
            let mode0_total: u64 = self.mode0_first_color_bits.values().sum();

            for (color, count) in colors {
                let color_percentage = (*count as f64 / mode0_total as f64) * 100.0;
                let total_percentage = (*count as f64 / self.total_blocks as f64) * 100.0;
                println!(
                    "Color 0x{:02X}: {} blocks ({:.2}% of mode 0 blocks, {:.2}% of total)",
                    color, count, color_percentage, total_percentage
                );
            }
        }
    }
}

pub fn analyze_bc7_file(path: &Path) -> Result<BC7BlockAnalysis, TransformError> {
    let data = fs::read(path)?;
    let info =
        unsafe { parse_dds(data.as_ptr(), data.len()) }.ok_or(TransformError::InvalidDdsFile)?;

    // Filter out non-BC7 files
    if info.format != DdsFormat::BC7 {
        return Ok(BC7BlockAnalysis::new());
    }

    let mut analysis = BC7BlockAnalysis::new();

    // Skip the DDS header and any additional headers
    let data_offset = info.data_offset;
    let data = &data[data_offset as usize..];

    // BC7 blocks are 16 bytes each
    for block in data.chunks_exact(16) {
        analysis.analyze_block(block);
    }

    Ok(analysis)
}

pub fn analyze_bc7_directory(input: &Path) -> Result<BC7BlockAnalysis, TransformError> {
    let mut combined_analysis = BC7BlockAnalysis::new();
    let mut entries = Vec::new();
    find_all_files(input, &mut entries)?;

    for entry in entries {
        if let Ok(file_analysis) = analyze_bc7_file(&entry.path()) {
            // Merge counts from this file into the combined analysis
            for (mode, count) in file_analysis.mode_counts {
                *combined_analysis.mode_counts.entry(mode).or_insert(0) += count;
            }
            for (byte, count) in file_analysis.first_byte_counts {
                *combined_analysis.first_byte_counts.entry(byte).or_insert(0) += count;
            }
            for (partition, count) in file_analysis.mode0_partition_counts {
                *combined_analysis
                    .mode0_partition_counts
                    .entry(partition)
                    .or_insert(0) += count;
            }
            for (color, count) in file_analysis.mode0_first_color_bits {
                *combined_analysis
                    .mode0_first_color_bits
                    .entry(color)
                    .or_insert(0) += count;
            }
            combined_analysis.total_blocks += file_analysis.total_blocks;
        }
    }

    Ok(combined_analysis)
}

pub fn get_bc7_mode(mode_byte: u8) -> u8 {
    if mode_byte == 0 {
        8 // Invalid mode
    } else {
        mode_byte.leading_zeros() as u8
    }
}

pub fn reverse_4bits(bits: u8) -> u8 {
    ((bits & 0b0001) << 3) |  // d000
    ((bits & 0b0010) << 1) |  // 0b00
    ((bits & 0b0100) >> 1) |  // 00c0
    ((bits & 0b1000) >> 3) // 000a
}

pub fn reverse_3bits(bits: u8) -> u8 {
    ((bits & 0b0001) << 2) |  // d00
    (bits & 0b0010) |  // 0b0
    ((bits & 0b0100) >> 2) // 00c
}

pub fn get_partition_from_mode_byte(mode_byte: u8) -> u8 {
    let bits = (mode_byte >> 3) & 0b1111;
    reverse_4bits(bits)
}

pub fn get_color_from_mode_byte(mode_byte: u8) -> u8 {
    let bits = (mode_byte) & 0b111;
    reverse_3bits(bits)
}
