use std::fs;
use std::path::PathBuf;

use clap::Parser;
use ron_edit::File;

#[derive(Parser)]
struct Opts {
    file: PathBuf,
}

fn main() {
    let opts = Opts::parse();
    let ron = fs::read_to_string(&opts.file).unwrap();
    let ron = File::try_from(ron.as_str()).unwrap();
    fs::write(opts.file, ronda::format(&ron)).unwrap();
}
