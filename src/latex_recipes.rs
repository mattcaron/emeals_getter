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
            "\\begin{{center}}\\includegraphics[height=3in]{{{}}}\\end{{center}}\n\n",
            image_filename
        )
        .as_str(),
    );

    // And then add the recipe name, with the optional side dish below it, slightly smaller.
    let title = recipe.find(Class("mainTitle")).next().unwrap().text();
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

    file.write(DOCUMENT_BEGIN.as_bytes())?;

    for recipe in recipes {
        file.write(format!("{}\n\\newpage\n", recipe).as_bytes())?;
    }

    file.write(DOCUMENT_END.as_bytes())?;

    Ok(())
}
