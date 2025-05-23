//! Program to parse a list of eMeals URLs and generate recipes from them.

use chrono::Local;
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{prelude::*, read_to_string};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use structopt::StructOpt;

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
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(error) => return Err(format!("Could not open input file: {error}").into()),
    };

    let contents = match read_to_string(file) {
        Ok(contents) => contents,
        Err(error) => return Err(format!("Could not read input file: {error}").into()),
    };

    let urls: Vec<String> = contents.lines().map(|line| line.to_string()).collect();

    Ok(urls)
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
fn process_url(
    url: &String,
    ingredients: Arc<Mutex<Vec<String>>>,
    recipes: Arc<Mutex<Vec<String>>>,
) -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::ClientBuilder::new()
        .user_agent("Mozilla/5.0")
        .build()?;

    let resp = client.get(url).send()?;

    let document = Document::from_read(resp)?;

    // Get all ingredients - main recipe and side dish
    let all_ingredients = document.find(Class("ingredients").descendant(Name("li")));

    // Note - we lock the output list here to avoid 2 things:
    // 1. lots of locking and unlocking
    // 2. interleaving the ingredients from different recipes

    match ingredients.lock() {
        Ok(mut ingredients) => {
            for ingredient in all_ingredients {
                ingredients.push(ingredient.text());
            }
        }
        Err(error) => {
            return Err(format!(
                "Failed to acquire ingredients mutex for adding ingredients list: {}",
                error
            )
            .into())
        }
    };

    // Debug doc dump...
    // println!("{:?}", document);

    match recipes.lock() {
        Ok(mut recipe) => recipe.push(latex_recipes::get_recipe(document)?),
        Err(error) => {
            return Err(format!(
                "Failed to acquire recipe mutex for writing an entry: {}",
                error
            )
            .into())
        }
    };

    Ok(())
}

/// Generate a text file for our ingredients for the week
///
/// # Arguments
/// * ingredients - Vector of ingredients to be put into our LaTeX document
///
/// # Returns
/// * On success, an empty Ok() is returned.
/// * On Failure, an Err() containing (potentially) useful information is returned.
///
pub fn write_ingredients(ingredients: Vec<String>) -> Result<(), Box<dyn Error>> {
    let date = Local::now().format("%Y%m%d");
    fs::create_dir_all(format!("{}", date))?;
    let file = PathBuf::from(format!("{}/groceries.txt", date));

    let mut file = File::create(file)?;

    for ingredient in ingredients {
        file.write_all(format!("{}\n", ingredient).as_bytes())?;
    }

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
fn get_urls(urls: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut tasks: Vec<thread::JoinHandle<_>> = Vec::new();
    let ingredients: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let recipes: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    for url in urls {
        // New variable to receive clones before being moved into the function
        let my_ingredient = ingredients.clone();
        let my_recipe = recipes.clone();
        tasks.push(thread::spawn(move || {
            process_url(&url, my_ingredient, my_recipe)
                .unwrap_or_else(|_| eprintln!("Error processing URL: {}", url));
        }));
    }

    for task in tasks {
        match task.join() {
            Ok(()) => (),
            Err(error) => {
                eprintln!("A task join() failed: {error:?}. The recipe list may be incomplete.")
            }
        };
    }

    // Ingredients and recipes should now be populated and unused by any subthreads,
    // so generate their respective files' ingredients list.
    match ingredients.lock() {
        Ok(ingredients) => write_ingredients(ingredients.to_vec())?,
        Err(error) => {
            return Err(format!(
                "Failed to acquire ingredients mutex for writing to file: {}",
                error
            )
            .into())
        }
    }

    match recipes.lock() {
        Ok(recipes) => latex_recipes::write_recipes(recipes.to_vec())?,
        Err(error) => {
            return Err(format!(
                "Failed to acquire recipe mutex for writing to file: {}",
                error
            )
            .into())
        }
    }

    Ok(())
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

    get_urls(urls)?;

    Ok(())
}
