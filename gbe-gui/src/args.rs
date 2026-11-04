use std::{env::args, path::PathBuf};

pub struct CLIArgs {
    pub rom_path: PathBuf
}

impl CLIArgs {
    pub fn parse_args() -> Option<Self> {
        let args = args().skip(1).collect::<Vec<_>>();
        if args.len() < 1 {
            return None;
        }

        let slf = Self {
            rom_path: PathBuf::from(&args[0])
        };

        Some(slf)
    }
}