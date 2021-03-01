use crossword;

fn main() {
    let mut grid = crossword::grid::CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
    println!("{}", grid.to_string());
}
