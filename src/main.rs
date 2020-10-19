use select::document::Document;
use select::predicate::Attr;
use std::error::Error;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use tokio::runtime::Runtime;
use tokio::task;

mod latex_ingredients;
mod latex_recipes;

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

    let urls: Vec<String> = reader
        .lines()
        .map(|line| line.expect("Error reading line"))
        .collect();

    return Ok(urls);
}

/// Process a given URL
///
/// # Arguments
/// * url - URL for which we should get the HTML and generate appropriate output
/// * ingredients - reference counted mutexed vector of ingredient strings
/// * recipes - reference counted mutexed vector of recipes as LaTeX fragments
///
/// # Returns
/// * On success, an empty Ok() is returned.
/// * On Failure, an Err() containing (potentially) useful information is returned.
fn process_url(url: &String, ingredients: Arc<Mutex<Vec<String>>>, recipes: Arc<Mutex<Vec<String>>>) -> Result<(), Box<dyn Error>> {
    let resp = reqwest::blocking::get(url)?;

    let document = Document::from_read(resp).unwrap();

    // Get all ingredients - main recipe and side dish
    let all_ingredients = document.find(Attr("itemprop", "ingredients"));
    for ingredient in all_ingredients {
        ingredients.lock().unwrap().push(ingredient.text());
    }

    recipes.lock().unwrap().push(latex_recipes::get_recipe(document)?);

    Ok(())
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
async fn get_urls(urls: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut tasks: Vec<task::JoinHandle<_>> = Vec::new();
    let ingredients: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let recipes: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    for url in urls {
        // New variable to receive clones before being moved into the function
        let my_ingredient = ingredients.clone();
        let my_recipe = recipes.clone();
        tasks.push(tokio::spawn(async move {
            process_url(&url, my_ingredient, my_recipe)
                .expect(format!("Error processing URL: {}", url).as_str());
        }));
    }

    for task in tasks {
        task.await.unwrap();
    }

    // Ingredients and recipes should now be populated and unused by any subthreads,
    // so generate their respective files ingredients list.
    latex_ingredients::write_ingredients(ingredients.lock().unwrap().to_vec())?;
    latex_recipes::write_recipes(recipes.lock().unwrap().to_vec())?;
    
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
