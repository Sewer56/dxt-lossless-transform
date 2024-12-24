use argh::FromArgs;
use std::str::FromStr;

#[derive(Debug, Clone)]
enum DdsType {
    BC1,
    BC2,
    BC3,
}

// Implement FromStr to allow parsing from command line arguments
impl FromStr for DdsType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bc1" => Ok(DdsType::BC1),
            "bc2" => Ok(DdsType::BC2),
            "bc3" => Ok(DdsType::BC3),
            _ => Err(format!(
                "Invalid DDS type: {}. Valid types are: bc1, bc2, bc3",
                s
            )),
        }
    }
}

#[derive(FromArgs, Debug)]
/// File transformation tool for DDS files
struct TopLevel {
    #[argh(subcommand)]
    command: Commands,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum Commands {
    Transform(TransformCmd),
    Detransform(DetransformCmd),
}

#[derive(FromArgs, Debug)]
/// Transform DDS files from input directory to output directory
#[argh(subcommand, name = "transform")]
struct TransformCmd {
    /// input directory path
    #[argh(option)]
    input: String,

    /// output directory path
    #[argh(option)]
    output: String,

    /// filter by DDS type (bc1, bc2, bc3)
    #[argh(option)]
    filter: Option<DdsType>,
}

#[derive(FromArgs, Debug)]
/// Detransform DDS files from input directory to output directory
#[argh(subcommand, name = "detransform")]
struct DetransformCmd {
    /// input directory path
    #[argh(option)]
    input: String,

    /// output directory path
    #[argh(option)]
    output: String,

    /// filter by DDS type (bc1, bc2, bc3)
    #[argh(option)]
    filter: Option<DdsType>,
}

fn main() {
    let top_level: TopLevel = argh::from_env();

    match top_level.command {
        Commands::Transform(cmd) => {
            println!("Transforming DDS files:");
            println!("Input directory: {}", cmd.input);
            println!("Output directory: {}", cmd.output);
            if let Some(filter) = cmd.filter {
                println!("Filtering by type: {:?}", filter);
            }
            // Add transformation logic here
        }
        Commands::Detransform(cmd) => {
            println!("Detransforming DDS files:");
            println!("Input directory: {}", cmd.input);
            println!("Output directory: {}", cmd.output);
            if let Some(filter) = cmd.filter {
                println!("Filtering by type: {:?}", filter);
            }
            // Add detransformation logic here
        }
    }
}
