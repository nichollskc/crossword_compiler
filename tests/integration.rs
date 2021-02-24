use crossword;

#[test]
fn read_from_file() {
    let grid = crossword::grid::CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
}
