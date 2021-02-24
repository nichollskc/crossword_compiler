use crossword;

fn main() {
    let grid = crossword::grid::CrosswordGrid::from_file("tests/resources/simple_example.txt");
    println!("{:#?}", grid);
}
