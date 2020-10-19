/// Module to handle generating latex files for recipes
use chrono::Local;
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use std::error::Error;
use std::io::Cursor;
use std::io::copy;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

/// Generate a LaTex file this recipe
///
/// # Arguments
/// * recipe - the parsed document representing the recipe
///
/// # Returns
/// * On success, an empty Ok() is returned.
/// * On Failure, an Err() containing (potentially) useful information is returned.
///
pub fn write_recipe(recipe: Document) -> Result<(), Box<dyn Error>> {
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
    

    Ok(())
}
