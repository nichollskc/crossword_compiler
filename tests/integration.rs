use crossword;

#[test]
fn read_from_file() {
    let grid = crossword::grid::CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
    assert_eq!(grid.count_words(), 10);
    assert_eq!(grid.count_intersections(), 11);
    assert_eq!(grid.to_graph().count_cycles(), 2);
    grid.check_valid();

    let grid = crossword::grid::CrosswordGridBuilder::new().from_file("tests/resources/disconnected.txt");
    assert_eq!(grid.count_words(), 5);
    assert_eq!(grid.count_intersections(), 3);
}

#[test]
#[should_panic]
fn check_disconnected() {
    let grid = crossword::grid::CrosswordGridBuilder::new().from_file("tests/resources/disconnected.txt");
    grid.check_valid();
}

#[test]
fn single_word() {
    let grid = crossword::grid::CrosswordGrid::new_single_word("ALPHA");
    assert_eq!(grid.count_words(), 1);
    assert_eq!(grid.count_intersections(), 0);
    assert_eq!(grid.to_graph().count_cycles(), 0);
    grid.check_valid();
}
