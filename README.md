# emeals_getter

An application to generate a compiled document of recipes and an associated ingredient list from them for emeals recipes.

## Introduction

This is a custom application which performs a very specific task, for a very picky customer - my son Miles. When he helps me cook, he wants the recipes formatted a very specific way, and the print functionality on emeals' website doesn't fit that requirement. So, in order to format them that way, I found myself having to copy and paste each recipe into a word processor, massage it to be acceptable, and then print it. This utility automates the import and massaging, meaning that all that is necessary is a quick proof of the generated document (mainly to ensure the pagination is sane for duplex printing) before you hit print. This should save roughly half an hour per week.

There is also a generated ingredients file to be used as a basis for a grocery list, which is formatted by this application's companion application - [grocery_list_generator](https://github.com/mattcaron/grocery_list_generator). (The reason for the two-step process is the fact that I need to remove things that we already have in the pantry from the list, as well as add other things that we need.)

## Usage

1. Using the emeals app as normal, pick your recipes for the week.
1. Share those recipes to yourself via your favorite method - this gets the URL.
1. Make a text file containing the list of URLs, one per line.
1. Run `emeals_getter <listname>`.
1. The `.tex` file and supplemental files will be put in a subdirectory named for the date in YYYYMMDD format.
1. Generate the PDF from that `.tex` file. (See [my LaTeX scripts](https://github.com/mattcaron/latex_scripts) for help with this.)

## Caveats and Limitations

- This is likely fragile - any substantive changes in HTML will either result in erroneous output or an outright panic.
- The generated ingredients list is quite stupid - duplicate items are not consolidated. It may actually be better to use the phone app on a tablet, generate the grocery list that way, screenshot it, and just type it out.
    - If that API ever becomes available, I can hit it and do this, but my guess is that, by the time that happens, this program will be obsolete because they will have added the ability to do all this to the website.
    - Although, I could sniff the traffic on the phone and reverse engineer what they're doing... but that's like.. work.

## TODO

- Add test code.
- Automatically count pages on the generated doc adjust as necessary to ensure each recipe is, at most, the front and back of a single page.
- There is much copypasta tech debt that needs refactoring.
- Add a license file.