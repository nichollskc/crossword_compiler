use std::collections::HashSet;
use std::fs;
use std::process::Command;

use handlebars::{Handlebars, RenderContext, Helper, Context, JsonRender, HelperResult, Output};
use serde_json::{Value,json};

use super::CrosswordGrid;
use super::Cell;
use super::Location;

fn wrap_in_braces(h: &Helper,
                  _: &Handlebars,
                  _: &Context,
                  _: &mut RenderContext,
                  out: &mut dyn Output) -> HelperResult {
    let param = h.param(0).unwrap();

    out.write("{")?;
    out.write(param.value().render().as_ref())?;
    out.write("}")?;
    Ok(())
}

#[derive(Debug)]
pub struct CrosswordPrinter {
    // Grid to be printed
    grid: CrosswordGrid,
    // Keeps track of what number clue we have printed so far
    last_clue_number: usize,
    // Keeps track of which words we have already numbered
    visited_word_ids: HashSet<usize>,
    // Vector of information about each across clue (in order)
    across_clues: Vec<Value>,
    // Vector of information about each down clue (in order)
    down_clues: Vec<Value>,
    // Grid so far, with each cell formatted
    printed_grid: String,
    // Format to use for an empty cell e.g. {} for white or * for black
    empty_cell_format: String,
    // Format to use for a filled cell e.g. Sf to print the solution (and the number!)
    filled_cell_format: String,
    // If true, don't put any answer information into the latex document
    obscure_answers: bool,
}

impl CrosswordPrinter {
    pub fn new(grid: CrosswordGrid, blank_cells_black: bool, show_solution: bool) -> Self {
        let empty_cell_format = if blank_cells_black {
            "    *"
        } else {
            "   {}"
        };
        let filled_cell_format = if show_solution {
            "[Sf]"
        } else {
            ""
        };
        CrosswordPrinter::new_with_settings(grid, empty_cell_format, filled_cell_format, !show_solution)
    }

    fn new_with_settings(grid: CrosswordGrid,
                         empty_cell_format: &str,
                         filled_cell_format: &str,
                         obscure_answers: bool) -> Self {
        CrosswordPrinter {
            grid,
            last_clue_number: 0,
            visited_word_ids: HashSet::new(),
            across_clues: vec![],
            down_clues: vec![],
            printed_grid: String::new(),
            empty_cell_format: empty_cell_format.to_string(),
            filled_cell_format: filled_cell_format.to_string(),
            obscure_answers,
        }
    }

    pub fn new_default(grid: CrosswordGrid) -> Self {
        CrosswordPrinter::new_with_settings(grid, "   {}", "", true)
    }

    fn add_clue(&mut self, clue_number: usize, word_id: usize, across: bool) {
        let word = self.grid.word_map.get(&word_id).unwrap();
        let answer = if self.obscure_answers {
            ""
        } else {
            &word.word_text
        };
        let clue_info = json!({
            "number": clue_number,
            "answer": answer,
            "clue": word.clue,
        });
        if across {
            self.across_clues.push(clue_info);
        } else {
            self.down_clues.push(clue_info);
        }
    }

    fn process_cell(&mut self, cell: &Cell) {
        let cell_string: String;

        let across_id = cell.get_across_word_id();
        let down_id = cell.get_down_word_id();

        if across_id.is_some() || down_id.is_some() {
            // We need to write in this cell's letter and potentially write out the clues
            let mut across_is_new = false;
            let mut down_is_new = false;

            if let Some(id) = across_id {
                across_is_new = self.visited_word_ids.insert(id);
            }
            if let Some(id) = down_id {
                down_is_new = self.visited_word_ids.insert(id);
            }

            let letter = if self.obscure_answers {
                'X'
            } else {
                cell.to_char()
            };

            if across_is_new || down_is_new {
                // First assign this cell a number to use for each clue
                self.last_clue_number += 1;
                cell_string = format!("|[{}]{}{}",
                                      self.last_clue_number,
                                      self.filled_cell_format,
                                      letter);
            } else {
                cell_string = format!("|[]{}  {}", self.filled_cell_format, letter);
            }

            if across_is_new {
                self.add_clue(self.last_clue_number, across_id.unwrap(), true);
            }
            if down_is_new {
                self.add_clue(self.last_clue_number, down_id.unwrap(), false);
            }
        } else {
            // This cell is blank, so just put black cell code
            cell_string = format!("|{}", self.empty_cell_format);
        }

        self.printed_grid.push_str(&cell_string);
    }

    fn end_cell_row(&mut self) {
        self.printed_grid.push_str("|.\n");
    }

    pub fn print(&mut self) -> String {
        let mut row = self.grid.top_left_cell_index.0 + 1;
        let mut col = self.grid.top_left_cell_index.1 + 1;
        while row < self.grid.bottom_right_cell_index.0 {
            while col < self.grid.bottom_right_cell_index.1 {
                let c: Cell = self.grid.cell_map.get(&Location(row, col)).unwrap().clone();
                self.process_cell(&c);
                col += 1;
            }
            col = self.grid.top_left_cell_index.1 + 1;
            row += 1;
            self.end_cell_row();
        }

        let (rows, cols) = self.grid.get_grid_dimensions();
        let data = &json!({
            "num_cols": cols,
            "num_rows": rows,
            "puzzle_content": self.printed_grid,
            "across_clues": self.across_clues,
            "down_clues": self.down_clues
        });
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(handlebars::no_escape);
        handlebars.register_helper("braced", Box::new(wrap_in_braces));
        handlebars.register_template_file("template", "./templates/latex_template.hbs").unwrap();
        handlebars.render("template", &data).unwrap()
    }

    pub fn print_to_file(&mut self, filename: &str) {
        fs::write(filename, self.print().as_bytes()).expect("Unable to write to file!");
    }

    pub fn print_to_pdf(&mut self, filename_root: &str) {
        let tex_file = format!("{}.tex", filename_root);
        let pdf_file = format!("{}.pdf", filename_root);
        self.print_to_file(&tex_file);
        Command::new("pdflatex")
            .arg("-output-directory")
            .arg("latex_output")
            .arg(tex_file)
            .output()
            .expect("Command failed");
        println!("{}", pdf_file);
    }
}
