/// Module to handle generating latex files for ingredients
use chrono::Local;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

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
        file.write(format!("{}\n", ingredient).as_bytes())?;
    }

    Ok(())
}
