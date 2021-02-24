use crossword;

#[test]
fn read_from_file() {
    let grid = crossword::grid::CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
    assert_eq!(grid.count_words(), 5);
    grid.check_valid();
}
