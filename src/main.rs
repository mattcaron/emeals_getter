use std::error::Error;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::task;
use tokio::runtime::Runtime;

/// Command line arguments
#[derive(StructOpt)]
struct Args {
    /// (Input) the file containing our list of URLs
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

/// Read our file in to a vector of URLs
///
/// # Arguments
/// * file - Name of file to open and parse
/// 
/// # Returns
/// * On success, an Ok() containing a Vector URLs as strings.
/// * On Failure, an Err() containing (potentially) useful information is returned.
/// 
fn read_file(filename: PathBuf) -> Result<Vec<String>, Box<dyn Error>> {

    let file = File::open(filename).expect("Could not read input file.");
    let reader = BufReader::new(file);

    let urls: Vec<String> = reader.lines().map(|line| line.expect("Error reading line")).collect();

    return Ok(urls);
}

/// Spin up parallel tokio tasks for URL processing, one for each URL in our vector
///
/// # Arguments
/// * urls - Vector of URLs for which we should get and process the HTML
/// 
/// # Returns
/// * On success, an empty Ok() is returned.
/// * On Failure, an Err() containing (potentially) useful information is returned.
///
async fn get_urls(urls: Vec<String>) ->  Result<(), Box<dyn Error>> {
    let mut tasks: Vec<task::JoinHandle<_>> = Vec::new();

    for url in urls {
        tasks.push(tokio::spawn(async move {
            println!("Url is: {}", url);
        }));
    }

    for task in tasks {
        task.await.unwrap();
    }

    return Ok(());

}

/// Main function
/// 
/// # Returns
/// * On success, an empty Ok() is returned.
/// * On Failure, an Err() containing (potentially) useful information is returned.
///
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::from_args();

    let urls = read_file(args.file)?;
    let rt = Runtime::new()?;

    rt.block_on(get_urls(urls))?;

    return Ok(());
}
