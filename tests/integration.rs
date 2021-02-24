use crossword;

#[test]
fn read_from_file() {
    crossword::grid::CrosswordGrid::from_file("tests/resources/simple_example.txt");
}
