//! Program to parse a list of eMeals URLs and generate recipes from them.

use chrono::Local;
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufReader};
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
    let file = File::open(filename).expect("Could not read input file.");
    let reader = BufReader::new(file);

    let urls: Vec<String> = reader
        .lines()
        .map(|line| line.expect("Error reading line"))
        .collect();

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
        .build()
        .unwrap();

    let resp = client.get(url).send()?;

    let document = Document::from_read(resp).unwrap();

    // Get all ingredients - main recipe and side dish
    let all_ingredients = document.find(Class("ingredients").descendant(Name("li")));
    for ingredient in all_ingredients {
        ingredients.lock().unwrap().push(ingredient.text());
    }

    // Debug doc dump...
    // println!("{:?}", document);

    recipes
        .lock()
        .unwrap()
        .push(latex_recipes::get_recipe(document)?);

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
                .unwrap_or_else(|_| panic!("Error processing URL: {}", url));
        }));
    }

    for task in tasks {
        task.join().unwrap();
    }

    // Ingredients and recipes should now be populated and unused by any subthreads,
    // so generate their respective files ingredients list.
    write_ingredients(ingredients.lock().unwrap().to_vec())?;
    latex_recipes::write_recipes(recipes.lock().unwrap().to_vec())?;

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
