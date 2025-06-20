use crate::error::TransformError;
use argh::FromArgs;
use rayon::*;

#[derive(FromArgs, Debug)]
/// Debug commands for analyzing BC7 files
#[argh(subcommand, name = "debug-bc7")]
pub struct DebugCmd {
    #[argh(subcommand)]
    pub command: DebugCommands,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum DebugCommands {}

pub fn handle_debug_command(cmd: DebugCmd) -> Result<(), TransformError> {
    match cmd.command {}
}
