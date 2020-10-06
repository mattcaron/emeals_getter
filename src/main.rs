use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;

/// Command line arguments
#[derive(StructOpt)]
struct Args {
    /// (Input) the file containing our list of URLs
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

/// Main function.
fn main() {
    let args = Args::from_args();

    let file = File::open(args.file).expect("Could not read input file.");
    let reader = BufReader::new(file);

    let lines: Vec<String> = reader.lines().map(|line| line.expect("Error reading line")).collect();

    println!("File contents:\n{:?}", lines)
}
