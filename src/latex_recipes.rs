/// Module to handle generating latex files for recipes
use chrono::Local;
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use std::error::Error;
use std::io::Cursor;
use std::io::copy;
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

/// Generate a LaTex fragement for this recipe, and get any images used in it
///
/// # Arguments
/// * recipe - the parsed document representing the recipe
///
/// # Returns
/// * On success, a String containing the LaTex Document fragment
///               describing the recipe is returned.
/// * On Failure, an Err() containing (potentially) useful information is returned.
///
pub fn get_recipe(recipe: Document) -> Result<String, Box<dyn Error>> {
    let date = Local::now().format("%Y%m%d");
    fs::create_dir_all(format!("{}", date))?;

    // Collect the image for the recipe
    let image_url = recipe
        .find(Class("recipe_image").descendant(Name("img")))
        .next()
        .unwrap()
        .attr("src")
        .unwrap();

    let split_url: Vec<&str> = image_url.split("/").collect();
    let image_path = PathBuf::from(format!("{}/{}", date, split_url.last().unwrap()));
    let mut image_dest = File::create(image_path)?;
    let mut image_content = Cursor::new(reqwest::blocking::get(image_url)?.bytes()?);
    copy(&mut image_content, &mut image_dest)?;

    // Generate the LaTeX for the recipe
    let mut recipe_latex: String = String::new();
    recipe_latex.push_str("Hello");

    Ok(recipe_latex)
}

/// Generate a LaTex document for our recipes
///
/// # Arguments
/// * recipes - a vector of LaTeX fragement recipe strings
///
/// # Returns
/// * On success, an empty Ok() is returned.
/// * On Failure, an Err() containing (potentially) useful information is returned.
///
pub fn write_recipes(recipes: Vec<String>) -> Result<(), Box<dyn Error>> {
    let date = Local::now().format("%Y%m%d");
    fs::create_dir_all(format!("{}", date))?;
    let file = PathBuf::from(format!("{}/recipes.tex", date));
    let mut file = File::create(file)?;
    
    for recipe in recipes {
        file.write(format!("\\item[] {}\n", recipe).as_bytes())?;
    }

    Ok(())
}