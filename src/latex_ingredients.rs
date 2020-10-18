/// Module to handle generating latex files for ingredients
use chrono::Local;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

const DOCUMENT_BEGIN: &str = r#"
\documentclass[12pt]{article}

\usepackage{fullpage}
\usepackage{fontspec}
\usepackage{multicol}

\setmainfont{Andika}

\begin{document}
"#;

const HEADING_START_MATT: &str = r#"
\begin{center}
{\huge Grocery List - Matt}
\medskip

"#;

const HEADING_START_MAX: &str = r#"
\begin{center}
\huge{Grocery List - Max}
"#;

const HEADING_START_MILES: &str = r#"
\begin{center}
\huge{Grocery List - Miles}
"#;

const HEADING_ENDS: &str = r#"
\end{center}
"#;

const BEGIN_LIST: &str = r#"
\bigskip
\begin{multicols}{2}
\begin{itemize}
{\Large
"#;

const END_LIST: &str = r#"
}
\end{itemize}
\end{multicols}
"#;

const NEWPAGE: &str = r#"
\newpage
"#;

const DOCUMENT_END: &str = r#"
\end{document}
"#;

/// Ingredients structure
struct Ingredients {
    all: Vec<String>,   // All ingredients
    max: Vec<String>,   // Max's ingredients
    miles: Vec<String>, // Miles's ingredients
}

impl Ingredients {
    pub fn new() -> Ingredients {
        Ingredients {
            all: Vec::new(),
            max: Vec::new(),
            miles: Vec::new(),
        }
    }
}

/// Parse our list of ingredients, sorting them into buckets
///
/// # Arguments
/// * ingredients - Vector of ingredients to be put into our LaTeX document
///
/// # Returns
/// * On success, an Ok() containing an Ingredients structure is returned.
/// * On Failure, an Err() containing (potentially) useful information is returned.
///
fn parse_ingredients(ingredients: Vec<String>) -> Result<Ingredients, Box<dyn Error>> {
    let mut max: bool = true;
    let mut parsed_ingredients = Ingredients::new();

    for ingredient in ingredients {
        parsed_ingredients.all.push(ingredient.clone());
        if max {
            parsed_ingredients.max.push(ingredient);
        } else {
            parsed_ingredients.miles.push(ingredient);
        }
        max = !max;
    }

    Ok(parsed_ingredients)
}

/// Generate a LaTex file for our ingredients for the week
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
    let file = PathBuf::from(format!("{}/{}.tex", date, date));

    let parsed_ingredients = parse_ingredients(ingredients)?;

    let mut file = File::create(file)?;

    file.write(DOCUMENT_BEGIN.as_bytes())?;
    file.write(HEADING_START_MATT.as_bytes())?;
    file.write(format!("{}\n", date).as_bytes())?;
    file.write(HEADING_ENDS.as_bytes())?;
    file.write(BEGIN_LIST.as_bytes())?;

    for ingredient in parsed_ingredients.all {
        file.write(format!("\\item[] {}\n", ingredient).as_bytes())?;
    }

    file.write(END_LIST.as_bytes())?;
    file.write(NEWPAGE.as_bytes())?;
    file.write(HEADING_START_MAX.as_bytes())?;
    file.write(HEADING_ENDS.as_bytes())?;
    file.write(BEGIN_LIST.as_bytes())?;

    for ingredient in parsed_ingredients.max {
        file.write(format!("\\item[] {}\n", ingredient).as_bytes())?;
    }

    file.write(END_LIST.as_bytes())?;
    file.write(NEWPAGE.as_bytes())?;
    file.write(HEADING_START_MILES.as_bytes())?;
    file.write(HEADING_ENDS.as_bytes())?;
    file.write(BEGIN_LIST.as_bytes())?;

    for ingredient in parsed_ingredients.miles {
        file.write(format!("\\item[] {}\n", ingredient).as_bytes())?;
    }

    file.write(END_LIST.as_bytes())?;
    file.write(DOCUMENT_END.as_bytes())?;

    Ok(())
}
