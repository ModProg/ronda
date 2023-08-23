use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};

use anyhow::{anyhow, Error, Result};
use clap::Parser;
use ron_edit::File;

#[derive(Parser)]
struct Opts {
    files: Option<Vec<PathBuf>>,
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    if let Some(files) = &opts.files {
        for file in files {
            let ron = fs::read_to_string(file)?;
            let ron = File::try_from(ron.as_str())
                .map_err(|err| anyhow!("Error while reading `{}`: {err}", file.display()))?;
            fs::write(file, ronda::format(&ron))?;
        }
    } else {
        let mut ron = String::new();
        io::stdin().read_to_string(&mut ron)?;
        let ron = File::try_from(ron.as_str()).map_err(Error::msg)?;
        print!("{}", ronda::format(&ron));
    }
    Ok(())
}
