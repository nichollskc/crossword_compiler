use crossword;

fn main() {
    let mut grid = crossword::grid::builder::CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
    let second = grid.clone();
    let bee_word_id = grid.add_unplaced_word("BEE");
    grid.try_place_word_in_cell(crossword::grid::Location(0, 3), bee_word_id, 2, false);
    println!("{}", second.to_string());
    println!("{}", grid.to_string());
}
