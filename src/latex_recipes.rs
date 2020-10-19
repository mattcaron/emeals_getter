/// Module to handle generating latex files for recipes
use chrono::Local;
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::copy;
use std::io::prelude::*;
use std::io::Cursor;
use std::path::PathBuf;

const DOCUMENT_BEGIN: &str = r#"
\documentclass[12pt]{article}

\usepackage{fullpage}
\usepackage{fontspec}
\usepackage{multicol}
\usepackage{graphicx}

\setmainfont{Andika}

\begin{document}
"#;

const NEWPAGE: &str = r#"
\newpage
"#;

const DOCUMENT_END: &str = r#"
\end{document}
"#;

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
    let image_filename = split_url.last().unwrap();
    let image_path = PathBuf::from(format!("{}/{}", date, image_filename));
    let mut image_dest = File::create(image_path)?;
    let mut image_content = Cursor::new(reqwest::blocking::get(image_url)?.bytes()?);
    copy(&mut image_content, &mut image_dest)?;

    // Generate the LaTeX for the recipe
    let mut recipe_latex: String = String::new();

    // Start with the image at the top
    recipe_latex.push_str(
        format!(
            "\\includegraphics[width=\\linewidth]{{{}}}\n",
            image_filename
        )
        .as_str(),
    );

    // And then add the recipe name, with the optional side dish below it, slightly smaller.
    let title = recipe.find(Class("mainTitle")).next().unwrap().text();
    let subtitle_match = recipe.find(Class("sideTitle")).next();

    recipe_latex.push_str(format!("{{\\LARGE {}}}\n", title).as_str());
    match subtitle_match {
        Some(subtitle) => {
            recipe_latex.push_str(format!("{{\\Large {}}}\n", subtitle.text()).as_str());
        }
        None => {}
    }

    // get and emit times
    let prep_time = recipe
        .find(Class("times").descendant(Attr("itemprop", "prepTime")))
        .next()
        .unwrap()
        .text();
    let cook_time = recipe
        .find(Class("times").descendant(Attr("itemprop", "cookTime")))
        .next()
        .unwrap()
        .text();
    let total_time = recipe
        .find(Class("times").descendant(Attr("itemprop", "totalTime")))
        .next()
        .unwrap()
        .text();

    recipe_latex.push_str(format!("{} {} {}\n", prep_time, cook_time, total_time).as_str());

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
        file.write(format!("{}\n", recipe).as_bytes())?;
    }

    Ok(())
}
