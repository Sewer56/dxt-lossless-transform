use crate::*;
use bitstream_io::{BigEndian, BitRead, BitReader, BitWrite, BitWriter};
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
    pub bits: [[u64; 2]; 4], // 4 bits per endpoint
}

#[derive(Debug, Default)]
pub(crate) struct IndexBits {
    pub bits: [[u64; 2]; 3], // 3 bits per index
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

#[derive(Debug)]
pub struct PartitionFields {
    // RGB endpoints
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

    // P-bits
    pub p_bits: [[u64; 2]; 6],

    // Index bits
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

impl Default for PartitionFields {
    fn default() -> Self {
        Self {
            r0_bits: Default::default(),
            r1_bits: Default::default(),
            r2_bits: Default::default(),
            r3_bits: Default::default(),
            r4_bits: Default::default(),
            r5_bits: Default::default(),

            g0_bits: Default::default(),
            g1_bits: Default::default(),
            g2_bits: Default::default(),
            g3_bits: Default::default(),
            g4_bits: Default::default(),
            g5_bits: Default::default(),

            b0_bits: Default::default(),
            b1_bits: Default::default(),
            b2_bits: Default::default(),
            b3_bits: Default::default(),
            b4_bits: Default::default(),
            b5_bits: Default::default(),

            p_bits: [[0; 2]; 6],

            index0_bits: Default::default(),
            index1_bits: Default::default(),
            index2_bits: Default::default(),
            index3_bits: Default::default(),
            index4_bits: Default::default(),
            index5_bits: Default::default(),
            index6_bits: Default::default(),
            index7_bits: Default::default(),
            index8_bits: Default::default(),
            index9_bits: Default::default(),
            index10_bits: Default::default(),
            index11_bits: Default::default(),
            index12_bits: Default::default(),
            index13_bits: Default::default(),
            index14_bits: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct BC7PartitionBitDistribution {
    // Track blocks analyzed per partition
    pub blocks_per_partition: HashMap<u8, u64>,

    // Fields organized by partition
    pub fields_by_partition: HashMap<u8, PartitionFields>,
}

impl BC7PartitionBitDistribution {
    pub fn new() -> Self {
        Self::default()
    }

    fn analyze_endpoint_bits(
        reader: &mut BitReader<Cursor<&[u8]>, BigEndian>,
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

    fn analyze_index_bits(
        reader: &mut BitReader<Cursor<&[u8]>, BigEndian>,
        index: &mut IndexBits,
    ) -> std::io::Result<()> {
        for i in 0..3 {
            let bit = reader.read::<u8>(1)? as usize;
            index.bits[i][bit] += 1;
        }
        Ok(())
    }

    pub fn analyze_block(
        &mut self,
        reader: &mut BitReader<Cursor<&[u8]>, BigEndian>,
    ) -> std::io::Result<()> {
        // Skip mode bit (we know it's 1 for mode 0)
        reader.read::<u32>(1)?;

        // Read partition bits
        let partition = reader.read::<u8>(4)?;

        // Initialize or get partition data
        self.blocks_per_partition.entry(partition).or_insert(0);
        *self.blocks_per_partition.get_mut(&partition).unwrap() += 1;

        let fields = self.fields_by_partition.entry(partition).or_default();

        // Analyze R endpoints
        Self::analyze_endpoint_bits(
            reader,
            &mut [
                &mut fields.r0_bits,
                &mut fields.r1_bits,
                &mut fields.r2_bits,
                &mut fields.r3_bits,
                &mut fields.r4_bits,
                &mut fields.r5_bits,
            ],
        )?;

        // Analyze G endpoints
        Self::analyze_endpoint_bits(
            reader,
            &mut [
                &mut fields.g0_bits,
                &mut fields.g1_bits,
                &mut fields.g2_bits,
                &mut fields.g3_bits,
                &mut fields.g4_bits,
                &mut fields.g5_bits,
            ],
        )?;

        // Analyze B endpoints
        Self::analyze_endpoint_bits(
            reader,
            &mut [
                &mut fields.b0_bits,
                &mut fields.b1_bits,
                &mut fields.b2_bits,
                &mut fields.b3_bits,
                &mut fields.b4_bits,
                &mut fields.b5_bits,
            ],
        )?;

        // Analyze p-bits
        for i in 0..6 {
            let bit = reader.read::<u8>(1)? as usize;
            fields.p_bits[i][bit] += 1;
        }

        // Analyze indices
        let indices = &mut [
            &mut fields.index0_bits,
            &mut fields.index1_bits,
            &mut fields.index2_bits,
            &mut fields.index3_bits,
            &mut fields.index4_bits,
            &mut fields.index5_bits,
            &mut fields.index6_bits,
            &mut fields.index7_bits,
            &mut fields.index8_bits,
            &mut fields.index9_bits,
            &mut fields.index10_bits,
            &mut fields.index11_bits,
            &mut fields.index12_bits,
            &mut fields.index13_bits,
            &mut fields.index14_bits,
        ];

        for index in indices {
            Self::analyze_index_bits(reader, *index)?;
        }

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
        if self.blocks_per_partition.is_empty() {
            println!("No BC7 mode 0 blocks analyzed");
            return;
        }

        println!("\nBC7 Mode 0 Bit Distribution Analysis By Partition");

        // Sort partitions for consistent output
        let mut partitions: Vec<_> = self.blocks_per_partition.keys().collect();
        partitions.sort();

        for &partition in partitions {
            let total_blocks = self.blocks_per_partition[&partition];
            println!("\nPartition {}: {} blocks", partition, total_blocks);

            if let Some(fields) = self.fields_by_partition.get(&partition) {
                // Print R endpoint stats
                Self::print_endpoint_stats(&fields.r0_bits, "R0", total_blocks);
                Self::print_endpoint_stats(&fields.r1_bits, "R1", total_blocks);
                Self::print_endpoint_stats(&fields.r2_bits, "R2", total_blocks);
                Self::print_endpoint_stats(&fields.r3_bits, "R3", total_blocks);
                Self::print_endpoint_stats(&fields.r4_bits, "R4", total_blocks);
                Self::print_endpoint_stats(&fields.r5_bits, "R5", total_blocks);

                // Print G endpoint stats
                Self::print_endpoint_stats(&fields.g0_bits, "G0", total_blocks);
                Self::print_endpoint_stats(&fields.g1_bits, "G1", total_blocks);
                Self::print_endpoint_stats(&fields.g2_bits, "G2", total_blocks);
                Self::print_endpoint_stats(&fields.g3_bits, "G3", total_blocks);
                Self::print_endpoint_stats(&fields.g4_bits, "G4", total_blocks);
                Self::print_endpoint_stats(&fields.g5_bits, "G5", total_blocks);

                // Print B endpoint stats
                Self::print_endpoint_stats(&fields.b0_bits, "B0", total_blocks);
                Self::print_endpoint_stats(&fields.b1_bits, "B1", total_blocks);
                Self::print_endpoint_stats(&fields.b2_bits, "B2", total_blocks);
                Self::print_endpoint_stats(&fields.b3_bits, "B3", total_blocks);
                Self::print_endpoint_stats(&fields.b4_bits, "B4", total_blocks);
                Self::print_endpoint_stats(&fields.b5_bits, "B5", total_blocks);

                // Print p-bit frequencies
                println!("\nP-bit frequencies:");
                for (i, bits) in fields.p_bits.iter().enumerate() {
                    let zeros = (bits[0] as f64 / total_blocks as f64) * 100.0;
                    let ones = (bits[1] as f64 / total_blocks as f64) * 100.0;
                    println!("p{}: 0={:.2}%, 1={:.2}%", i, zeros, ones);
                }

                // Print index stats
                Self::print_index_stats(&fields.index0_bits, 0, total_blocks);
                Self::print_index_stats(&fields.index1_bits, 1, total_blocks);
                Self::print_index_stats(&fields.index2_bits, 2, total_blocks);
                Self::print_index_stats(&fields.index3_bits, 3, total_blocks);
                Self::print_index_stats(&fields.index4_bits, 4, total_blocks);
                Self::print_index_stats(&fields.index5_bits, 5, total_blocks);
                Self::print_index_stats(&fields.index6_bits, 6, total_blocks);
                Self::print_index_stats(&fields.index7_bits, 7, total_blocks);
                Self::print_index_stats(&fields.index8_bits, 8, total_blocks);
                Self::print_index_stats(&fields.index9_bits, 9, total_blocks);
                Self::print_index_stats(&fields.index10_bits, 10, total_blocks);
                Self::print_index_stats(&fields.index11_bits, 11, total_blocks);
                Self::print_index_stats(&fields.index12_bits, 12, total_blocks);
                Self::print_index_stats(&fields.index13_bits, 13, total_blocks);
                Self::print_index_stats(&fields.index14_bits, 14, total_blocks);
            }
        }
    }
}

pub fn analyze_bc7_mode0_partition_bits(
    data: &[u8],
) -> std::io::Result<BC7PartitionBitDistribution> {
    let mut distribution = BC7PartitionBitDistribution::new();
    let mut reader = BitReader::endian(Cursor::new(data), BigEndian);

    // Process each 128-bit block
    while let Ok(first_bit) = reader.read::<u32>(1) {
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

#[derive(Debug)]
struct XorTransformInfo {
    // For each partition, store which bits should be XORed (have >50% probability of being 1)
    bit_masks: HashMap<u8, BlockBitMasks>,
}

#[derive(Debug, Default)]
struct BlockBitMasks {
    // Array of 128 masks, one per bit in the block after mode+partition
    // true = this bit should be XORed (>50% probability of 1)
    masks: Vec<bool>,
}

impl XorTransformInfo {
    fn new(distribution: &BC7PartitionBitDistribution) -> Self {
        let mut bit_masks = HashMap::new();

        // For each partition
        for (&partition, fields) in &distribution.fields_by_partition {
            let total_blocks = distribution.blocks_per_partition[&partition] as f64;
            let mut masks = BlockBitMasks {
                masks: Vec::with_capacity(128),
            };

            // Calculate masks for R endpoints
            for endpoint in [
                &fields.r0_bits,
                &fields.r1_bits,
                &fields.r2_bits,
                &fields.r3_bits,
                &fields.r4_bits,
                &fields.r5_bits,
            ] {
                for i in 0..4 {
                    let ones = endpoint.bits[i][1] as f64;
                    masks.masks.push((ones / total_blocks) > 0.5);
                }
            }

            // Calculate masks for G endpoints
            for endpoint in [
                &fields.g0_bits,
                &fields.g1_bits,
                &fields.g2_bits,
                &fields.g3_bits,
                &fields.g4_bits,
                &fields.g5_bits,
            ] {
                for i in 0..4 {
                    let ones = endpoint.bits[i][1] as f64;
                    masks.masks.push((ones / total_blocks) > 0.5);
                }
            }

            // Calculate masks for B endpoints
            for endpoint in [
                &fields.b0_bits,
                &fields.b1_bits,
                &fields.b2_bits,
                &fields.b3_bits,
                &fields.b4_bits,
                &fields.b5_bits,
            ] {
                for i in 0..4 {
                    let ones = endpoint.bits[i][1] as f64;
                    masks.masks.push((ones / total_blocks) > 0.5);
                }
            }

            // Calculate masks for p-bits
            for i in 0..6 {
                let ones = fields.p_bits[i][1] as f64;
                masks.masks.push((ones / total_blocks) > 0.5);
            }

            // Calculate masks for indices
            for index in [
                &fields.index0_bits,
                &fields.index1_bits,
                &fields.index2_bits,
                &fields.index3_bits,
                &fields.index4_bits,
                &fields.index5_bits,
                &fields.index6_bits,
                &fields.index7_bits,
                &fields.index8_bits,
                &fields.index9_bits,
                &fields.index10_bits,
                &fields.index11_bits,
                &fields.index12_bits,
                &fields.index13_bits,
                &fields.index14_bits,
            ] {
                for i in 0..3 {
                    let ones = index.bits[i][1] as f64;
                    masks.masks.push((ones / total_blocks) > 0.8);
                }
            }

            bit_masks.insert(partition, masks);
        }

        XorTransformInfo { bit_masks }
    }
}

pub fn xor_transform_bc7_mode0(data: &[u8]) -> Vec<u8> {
    const ANALYSIS_CHUNK_SIZE: usize = 4294967295; // Must be multiple of block size (16 bytes)

    let mut output = Vec::new();
    let mut current_position = 0;

    // First chunk uses analysis of input data
    let first_chunk_end = ANALYSIS_CHUNK_SIZE.min(data.len());
    let first_chunk = &data[..first_chunk_end];
    let mut distribution = analyze_bc7_mode0_partition_bits(first_chunk).unwrap();
    let mut xor_info = XorTransformInfo::new(&distribution);

    while current_position < data.len() {
        let mut output_cursor = Cursor::new(&mut output);
        output_cursor.seek(SeekFrom::End(0)).unwrap();
        let mut writer = BitWriter::endian(&mut output_cursor, BigEndian);

        // Calculate the size of the current chunk
        let chunk_end = (current_position + ANALYSIS_CHUNK_SIZE).min(data.len());
        let chunk = &data[current_position..chunk_end];

        // Process this chunk
        let mut reader = BitReader::endian(Cursor::new(chunk), BigEndian);

        while let Ok(mode_bit) = reader.read::<u8>(1) {
            // Write mode bit unchanged
            writer.write(1, mode_bit).unwrap();

            if mode_bit == 1 {
                // Mode 0 block - apply transform
                // Read partition
                let partition = reader.read::<u8>(4).unwrap();
                // Write partition unchanged
                writer.write(4, partition).unwrap();

                if let Some(masks) = xor_info.bit_masks.get(&partition) {
                    // Transform rest of block bit by bit
                    for mask in &masks.masks {
                        let bit = reader.read::<u8>(1).unwrap();
                        let transformed = if *mask { bit ^ 1 } else { bit };
                        writer.write(1, transformed).unwrap();
                    }
                } else {
                    // No transform data for this partition - copy unchanged
                    for _ in 0..123 {
                        // Remaining bits in block
                        let bit = reader.read::<u8>(1).unwrap();
                        writer.write(1, bit).unwrap();
                    }
                }
            } else {
                // Non-mode 0 block - copy unchanged
                for _ in 0..127 {
                    // Remaining bits in block
                    let bit = reader.read::<u8>(1).unwrap();
                    writer.write(1, bit).unwrap();
                }
            }
        }

        // Flush the writer to ensure all bits are written to output
        writer.flush().unwrap();
        current_position = chunk_end;

        // If there's more data to process, analyze the previous output chunk
        // to determine the next XOR pattern
        if current_position < data.len() {
            let output_len = output.len();
            if output_len >= ANALYSIS_CHUNK_SIZE {
                let analysis_start = output_len - ANALYSIS_CHUNK_SIZE;
                distribution = analyze_bc7_mode0_partition_bits(&output[analysis_start..]).unwrap();
                xor_info = XorTransformInfo::new(&distribution);
            }
        }
    }

    output
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
