/// Module to handle generating latex files for recipes
use chrono::Local;
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
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

\pagestyle{empty}

\begin{document}
"#;

const DOCUMENT_END: &str = r#"
\end{document}
"#;

/// Collect the image URL for a recipe, if any.
///
/// # Arguments
/// * recipe - the parsed document representing the recipe
///
/// * On success, a String containing the url of the image.
/// * On Failure, None.
fn get_image_url(recipe: &Document) -> Option<String> {
    match recipe
        .find(Class("recipe_image").descendant(Name("img")))
        .next()
    {
        Some(img) => Some(img.attr("src")?.to_string()),
        None => None,
    }
}

/// Generate a LaTex fragment for this recipe, and get any images used in it
///
/// # Arguments
/// * recipe - the parsed document representing the recipe
///
/// # Returns
/// * On success, a String containing the LaTex Document fragment
///   describing the recipe is returned.
/// * On Failure, an Err() containing (potentially) useful information is returned.
///
pub fn get_recipe(recipe: Document) -> Result<String, Box<dyn Error>> {
    let date = Local::now().format("%Y%m%d");
    fs::create_dir_all(format!("{}", date))?;

    // Get the recipe title first, because we use it for error messages later.
    // If the recipe has no title, this is unrecoverable.
    let title = match recipe.find(Class("mainTitle")).next() {
        Some(title) => title.text(),
        None => return Err("Unable to find title.".into()),
    };

    // Generate the LaTeX for the recipe
    let mut recipe_latex: String = String::new();

    // Get the the image URL for the recipe, if any
    let image_url = get_image_url(&recipe);

    // Download the image, if any
    match image_url {
        Some(url) => {
            let url_copy = url.clone();
            let split_url: Vec<&str> = url.split("/").collect();
            match split_url.last() {
                Some(image_filename) => {
                    let image_path = PathBuf::from(format!("{}/{}", date, image_filename));
                    let mut image_dest = File::create(image_path)?;
                    let mut image_content = Cursor::new(reqwest::blocking::get(url_copy)?.bytes()?);
                    copy(&mut image_content, &mut image_dest)?;

                    // Start with the image at the top
                    recipe_latex.push_str(
                        format!(
                    "\\begin{{center}}\\includegraphics[height=3in]{{{}}}\\end{{center}}\n\n",
                    image_filename
                )
                        .as_str(),
                    );
                }
                None => eprintln!(
                    "WARNING: Unable to figure out the filename in {url_copy}, not downloading."
                ),
            };
        }
        // This is recoverable - just warn that we don't have an image.
        None => eprintln!("WARNING: No image for recipe: \"{title}\""),
    }

    // And then add the recipe name, with the optional side dish below it, slightly smaller.
    let subtitle_match = recipe.find(Class("sideTitle")).next();
    let mut has_side = false;

    recipe_latex.push_str(format!("{{\\noindent\\Large {}}}\n\n", title).as_str());
    recipe_latex.push_str("\\medskip\n".to_string().as_str());
    if let Some(subtitle) = subtitle_match {
        recipe_latex.push_str(format!("{{\\noindent\\large {}}}\n\n", subtitle.text()).as_str());
        recipe_latex.push_str("\\medskip\n".to_string().as_str());
        has_side = true;
    }

    // Get and emit times

    let times = recipe.find(Class("times").descendant(Name("time")));

    for time in times {
        recipe_latex.push_str(format!("{} ", time.text()).as_str());
    }

    recipe_latex.push_str("\n\n\\bigskip\n".to_string().as_str());

    // Get and emit main recipe
    let main_recipe_ingredients = recipe.find(
        Class("mainInformation")
            .descendant(Class("ingredients"))
            .descendant(Name("li")),
    );
    let main_recipe_instructions = recipe.find(
        Class("mainInformation")
            .descendant(Class("instructions"))
            .descendant(Name("li")),
    );

    recipe_latex.push_str("{\\noindent\\large Ingredients}\n".to_string().as_str());
    recipe_latex.push_str("\\begin{itemize}\n".to_string().as_str());
    for ingredient in main_recipe_ingredients {
        recipe_latex.push_str(format!("    \\item[] {}\n", ingredient.text()).as_str());
    }
    recipe_latex.push_str("\\end{itemize}\n".to_string().as_str());
    recipe_latex.push_str("\\bigskip\n".to_string().as_str());
    recipe_latex.push_str("{\\noindent\\large Instructions}\n".to_string().as_str());
    recipe_latex.push_str("\\begin{enumerate}\n".to_string().as_str());
    for instruction in main_recipe_instructions {
        recipe_latex.push_str(format!("    \\item {}\n", instruction.text()).as_str());
    }
    recipe_latex.push_str("\\end{enumerate}\n".to_string().as_str());

    recipe_latex.push_str("\\bigskip\n".to_string().as_str());

    // Get and emit side recipe, if it exists
    let side_recipe_ingredients = recipe.find(
        Class("side_dish_section")
            .descendant(Class("ingredients"))
            .descendant(Name("li")),
    );
    let side_recipe_instructions = recipe.find(
        Class("side_dish_section")
            .descendant(Class("instructions"))
            .descendant(Name("li")),
    );

    if has_side {
        recipe_latex.push_str(
            "{\\noindent\\large Side Dish Ingredients}\n"
                .to_string()
                .as_str(),
        );
        recipe_latex.push_str("\\begin{itemize}\n".to_string().as_str());
        for ingredient in side_recipe_ingredients {
            recipe_latex.push_str(format!("    \\item[] {}\n", ingredient.text()).as_str());
        }
        recipe_latex.push_str("\\end{itemize}\n".to_string().as_str());
        recipe_latex.push_str("\\bigskip\n".to_string().as_str());
        recipe_latex.push_str(
            "{\\noindent\\large Side Dish Instructions}\n"
                .to_string()
                .as_str(),
        );
        recipe_latex.push_str("\\begin{enumerate}\n".to_string().as_str());
        for instruction in side_recipe_instructions {
            recipe_latex.push_str(format!("    \\item {}\n", instruction.text()).as_str());
        }
        recipe_latex.push_str("\\end{enumerate}\n".to_string().as_str());
    }

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

    file.write_all(DOCUMENT_BEGIN.as_bytes())?;

    for recipe in recipes {
        file.write_all(format!("{}\n\\newpage\n", recipe).as_bytes())?;
    }

    file.write_all(DOCUMENT_END.as_bytes())?;

    Ok(())
}
