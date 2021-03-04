use std::collections::HashSet;
use std::fs;

use super::CrosswordGrid;
use super::Cell;
use super::Location;

static DOCUMENT_START: &str = "\\documentclass{article}\n\\usepackage[unboxed]{cwpuzzle}\n\n\\newcommand{\\CrosswordClue}[3]{\\textbf{#1} \\quad #3 \\\\}\n\\begin{document}\n";
static DOCUMENT_END: &str = "\n\n\\end{document}";

#[derive(Debug)]
pub struct CrosswordPrinter {
    grid: CrosswordGrid,
    last_clue_number: usize,
    visited_word_ids: HashSet<usize>,
    across_clues: String,
    down_clues: String,
    printed_grid: String,
}

impl CrosswordPrinter {
    pub fn new(grid: CrosswordGrid) -> Self {
        let (rows, cols) = grid.get_grid_dimensions();
        let puzzle_definition = format!("\\begin{{Puzzle}}{{{}}}{{{}}}\n", cols, rows);
        CrosswordPrinter {
            grid,
            last_clue_number: 0,
            visited_word_ids: HashSet::new(),
            across_clues: "\\section*{Across}\n".to_string(),
            down_clues: "\\section*{Down}\n".to_string(),
            printed_grid: puzzle_definition,
        }
    }

    fn add_clue(&mut self, clue_number: usize, word_id: usize, across: bool) {
        let word = self.grid.word_map.get(&word_id).unwrap();
        let clue = format!("\\CrosswordClue{{{}}}{{{}}}{{{}}}\n", clue_number, word.word_text, word.clue);
        if across {
            self.across_clues.push_str(&clue);
        } else {
            self.down_clues.push_str(&clue);
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

            if across_is_new || down_is_new {
                // First assign this cell a number to use for each clue
                self.last_clue_number += 1;
                cell_string = format!("|[{}]{}", self.last_clue_number, cell.to_char());
            } else {
                cell_string = format!("|{}", cell.to_char());
            }

            if across_is_new {
                self.add_clue(self.last_clue_number, across_id.unwrap(), true);
            }
            if down_is_new {
                self.add_clue(self.last_clue_number, down_id.unwrap(), false);
            }
        } else {
            // This cell is blank, so just put black cell code
            cell_string = "|*".to_string();
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

        format!("{}{}\\end{{Puzzle}}\n\n{}\n{}\n\n{}", DOCUMENT_START, self.printed_grid, self.across_clues, self.down_clues, DOCUMENT_END)
    }

    pub fn print_to_file(&mut self, filename: &str) {
        fs::write(filename, self.print().as_bytes()).expect("Unable to write to file!");
    }
}
