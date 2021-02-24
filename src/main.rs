use crossword;

fn main() {
    let grid = crossword::grid::CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
    println!("{:#?}", grid);
}
