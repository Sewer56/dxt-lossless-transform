use crate::*;
use bitstream_io::{BigEndian, BitRead, BitReader};
use dxt_lossless_transform_api::*;
use std::{
    collections::HashMap,
    fs,
    io::{Cursor, Read, Seek, SeekFrom},
    path::Path,
};

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

#[derive(Debug, Default)]
pub(crate) struct ColorEndpointBits {
    bits: [[u64; 2]; 4], // 4 bits per endpoint
}

#[derive(Debug, Default)]
pub(crate) struct IndexBits {
    bits: [[u64; 2]; 3], // 3 bits per index
}

#[derive(Debug, Default)]
pub struct BC7Mode0BitDistribution {
    // Track total blocks analyzed
    pub total_blocks: u64,

    // Bit frequency maps for each field
    // First field for 0, other for 1
    pub partition_bits: [[u64; 2]; 4], // 4 bits

    // Separate arrays for each endpoint
    pub r0_bits: ColorEndpointBits,
    pub r1_bits: ColorEndpointBits,
    pub r2_bits: ColorEndpointBits,
    pub r3_bits: ColorEndpointBits,
    pub r4_bits: ColorEndpointBits,
    pub r5_bits: ColorEndpointBits,

    pub g0_bits: ColorEndpointBits,
    pub g1_bits: ColorEndpointBits,
    pub g2_bits: ColorEndpointBits,
    pub g3_bits: ColorEndpointBits,
    pub g4_bits: ColorEndpointBits,
    pub g5_bits: ColorEndpointBits,

    pub b0_bits: ColorEndpointBits,
    pub b1_bits: ColorEndpointBits,
    pub b2_bits: ColorEndpointBits,
    pub b3_bits: ColorEndpointBits,
    pub b4_bits: ColorEndpointBits,
    pub b5_bits: ColorEndpointBits,

    pub p_bits: [[u64; 2]; 6], // 6 p-bits

    // Separate arrays for each index
    pub index0_bits: IndexBits,
    pub index1_bits: IndexBits,
    pub index2_bits: IndexBits,
    pub index3_bits: IndexBits,
    pub index4_bits: IndexBits,
    pub index5_bits: IndexBits,
    pub index6_bits: IndexBits,
    pub index7_bits: IndexBits,
    pub index8_bits: IndexBits,
    pub index9_bits: IndexBits,
    pub index10_bits: IndexBits,
    pub index11_bits: IndexBits,
    pub index12_bits: IndexBits,
    pub index13_bits: IndexBits,
    pub index14_bits: IndexBits,
}

impl BC7Mode0BitDistribution {
    pub fn new() -> Self {
        Self::default()
    }

    fn analyze_endpoint_bits<T: Read>(
        reader: &mut BitReader<T, BigEndian>,
        endpoints: &mut [&mut ColorEndpointBits; 6],
    ) -> std::io::Result<()> {
        for endpoint in endpoints.iter_mut() {
            for i in 0..4 {
                let bit = reader.read::<u8>(1)? as usize;
                endpoint.bits[i][bit] += 1;
            }
        }
        Ok(())
    }

    pub fn analyze_block<T: Read + Seek>(
        &mut self,
        reader: &mut BitReader<T, BigEndian>,
    ) -> std::io::Result<()> {
        // Skip mode bit (we know it's 1 for mode 0)
        reader.read::<u8>(1)?;

        // Read and analyze partition bits
        for i in 0..4 {
            let bit = reader.read::<u8>(1)? as usize;
            self.partition_bits[i][bit] += 1;
        }

        // Read and analyze R endpoints
        Self::analyze_endpoint_bits(
            reader,
            &mut [
                &mut self.r0_bits,
                &mut self.r1_bits,
                &mut self.r2_bits,
                &mut self.r3_bits,
                &mut self.r4_bits,
                &mut self.r5_bits,
            ],
        )?;

        // Read and analyze G endpoints
        Self::analyze_endpoint_bits(
            reader,
            &mut [
                &mut self.g0_bits,
                &mut self.g1_bits,
                &mut self.g2_bits,
                &mut self.g3_bits,
                &mut self.g4_bits,
                &mut self.g5_bits,
            ],
        )?;

        // Read and analyze B endpoints
        Self::analyze_endpoint_bits(
            reader,
            &mut [
                &mut self.b0_bits,
                &mut self.b1_bits,
                &mut self.b2_bits,
                &mut self.b3_bits,
                &mut self.b4_bits,
                &mut self.b5_bits,
            ],
        )?;

        // Read and analyze p-bits
        for i in 0..6 {
            let bit = reader.read::<u8>(1)? as usize;
            self.p_bits[i][bit] += 1;
        }

        // Read and analyze indices
        let indices = &mut [
            &mut self.index0_bits,
            &mut self.index1_bits,
            &mut self.index2_bits,
            &mut self.index3_bits,
            &mut self.index4_bits,
            &mut self.index5_bits,
            &mut self.index6_bits,
            &mut self.index7_bits,
            &mut self.index8_bits,
            &mut self.index9_bits,
            &mut self.index10_bits,
            &mut self.index11_bits,
            &mut self.index12_bits,
            &mut self.index13_bits,
            &mut self.index14_bits,
        ];

        for index in indices.iter_mut() {
            for i in 0..3 {
                let bit = reader.read::<u8>(1)? as usize;
                index.bits[i][bit] += 1;
            }
        }

        self.total_blocks += 1;
        Ok(())
    }

    fn print_endpoint_stats(endpoint_bits: &ColorEndpointBits, name: &str, total_blocks: u64) {
        println!("\n{} bit frequencies:", name);
        for (i, bits) in endpoint_bits.bits.iter().enumerate() {
            let zeros = (bits[0] as f64 / total_blocks as f64) * 100.0;
            let ones = (bits[1] as f64 / total_blocks as f64) * 100.0;
            println!("bit {}: 0={:.2}%, 1={:.2}%", i, zeros, ones);
        }
    }

    fn print_index_stats(index_bits: &IndexBits, index: usize, total_blocks: u64) {
        println!("\nIndex {} bit frequencies:", index);
        for (i, bits) in index_bits.bits.iter().enumerate() {
            let zeros = (bits[0] as f64 / total_blocks as f64) * 100.0;
            let ones = (bits[1] as f64 / total_blocks as f64) * 100.0;
            println!("bit {}: 0={:.2}%, 1={:.2}%", i, zeros, ones);
        }
    }

    pub fn print_results(&self) {
        if self.total_blocks == 0 {
            println!("No BC7 mode 0 blocks analyzed");
            return;
        }

        println!("\nBC7 Mode 0 Bit Distribution Analysis");
        println!("Total blocks analyzed: {}", self.total_blocks);

        println!("\nPartition bit frequencies:");
        for (i, bits) in self.partition_bits.iter().enumerate() {
            let zeros = (bits[0] as f64 / self.total_blocks as f64) * 100.0;
            let ones = (bits[1] as f64 / self.total_blocks as f64) * 100.0;
            println!("Bit {}: 0={:.2}%, 1={:.2}%", i, zeros, ones);
        }

        // Print R endpoint stats
        Self::print_endpoint_stats(&self.r0_bits, "R0", self.total_blocks);
        Self::print_endpoint_stats(&self.r1_bits, "R1", self.total_blocks);
        Self::print_endpoint_stats(&self.r2_bits, "R2", self.total_blocks);
        Self::print_endpoint_stats(&self.r3_bits, "R3", self.total_blocks);
        Self::print_endpoint_stats(&self.r4_bits, "R4", self.total_blocks);
        Self::print_endpoint_stats(&self.r5_bits, "R5", self.total_blocks);

        // Print G endpoint stats
        Self::print_endpoint_stats(&self.g0_bits, "G0", self.total_blocks);
        Self::print_endpoint_stats(&self.g1_bits, "G1", self.total_blocks);
        Self::print_endpoint_stats(&self.g2_bits, "G2", self.total_blocks);
        Self::print_endpoint_stats(&self.g3_bits, "G3", self.total_blocks);
        Self::print_endpoint_stats(&self.g4_bits, "G4", self.total_blocks);
        Self::print_endpoint_stats(&self.g5_bits, "G5", self.total_blocks);

        // Print B endpoint stats
        Self::print_endpoint_stats(&self.b0_bits, "B0", self.total_blocks);
        Self::print_endpoint_stats(&self.b1_bits, "B1", self.total_blocks);
        Self::print_endpoint_stats(&self.b2_bits, "B2", self.total_blocks);
        Self::print_endpoint_stats(&self.b3_bits, "B3", self.total_blocks);
        Self::print_endpoint_stats(&self.b4_bits, "B4", self.total_blocks);
        Self::print_endpoint_stats(&self.b5_bits, "B5", self.total_blocks);

        println!("\nP-bit frequencies:");
        for (i, bits) in self.p_bits.iter().enumerate() {
            let zeros = (bits[0] as f64 / self.total_blocks as f64) * 100.0;
            let ones = (bits[1] as f64 / self.total_blocks as f64) * 100.0;
            println!("p{}: 0={:.2}%, 1={:.2}%", i, zeros, ones);
        }

        // Print index stats
        Self::print_index_stats(&self.index0_bits, 0, self.total_blocks);
        Self::print_index_stats(&self.index1_bits, 1, self.total_blocks);
        Self::print_index_stats(&self.index2_bits, 2, self.total_blocks);
        Self::print_index_stats(&self.index3_bits, 3, self.total_blocks);
        Self::print_index_stats(&self.index4_bits, 4, self.total_blocks);
        Self::print_index_stats(&self.index5_bits, 5, self.total_blocks);
        Self::print_index_stats(&self.index6_bits, 6, self.total_blocks);
        Self::print_index_stats(&self.index7_bits, 7, self.total_blocks);
        Self::print_index_stats(&self.index8_bits, 8, self.total_blocks);
        Self::print_index_stats(&self.index9_bits, 9, self.total_blocks);
        Self::print_index_stats(&self.index10_bits, 10, self.total_blocks);
        Self::print_index_stats(&self.index11_bits, 11, self.total_blocks);
        Self::print_index_stats(&self.index12_bits, 12, self.total_blocks);
        Self::print_index_stats(&self.index13_bits, 13, self.total_blocks);
        Self::print_index_stats(&self.index14_bits, 14, self.total_blocks);
    }

    #[inline]
    pub(crate) fn combine_endpoint_bits(dest: &mut ColorEndpointBits, src: &ColorEndpointBits) {
        for i in 0..4 {
            for j in 0..2 {
                dest.bits[i][j] += src.bits[i][j];
            }
        }
    }

    #[inline]
    pub(crate) fn combine_index_bits(dest: &mut IndexBits, src: &IndexBits) {
        for i in 0..3 {
            for j in 0..2 {
                dest.bits[i][j] += src.bits[i][j];
            }
        }
    }
}

pub fn analyze_bc7_mode0_bits(data: &[u8]) -> std::io::Result<BC7Mode0BitDistribution> {
    let mut distribution = BC7Mode0BitDistribution::new();
    let mut reader = BitReader::endian(Cursor::new(data), BigEndian);

    // Process each 128-bit block
    while let Ok(first_bit) = reader.read::<u8>(1) {
        // Rewind the 1 bit we just read to check mode
        reader.seek_bits(SeekFrom::Current(-1))?;

        // Only analyze mode 0 blocks
        if first_bit == 1 {
            distribution.analyze_block(&mut reader)?;
        } else {
            // Skip non-mode 0 blocks (128 bits)
            reader.seek_bits(SeekFrom::Current(128))?;
        }
    }

    Ok(distribution)
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
