use crossword;

#[test]
fn read_from_file() {
    let grid = crossword::grid::CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
    println!("{:#?}", grid);
    assert_eq!(grid.count_all_words(), 10);
    assert_eq!(grid.count_intersections(), 11);
    assert_eq!(grid.to_graph().count_cycles(), 2);
    assert_eq!(grid.get_grid_dimensions(), (9, 10));
    assert_eq!(grid.count_filled_cells(), 42);
    assert_eq!(grid.count_empty_cells(), 48);
    grid.check_valid();
}

#[test]
#[should_panic]
fn check_disconnected() {
    crossword::grid::CrosswordGridBuilder::new().from_file("tests/resources/disconnected.txt");
}

#[test]
fn single_word() {
    let grid = crossword::grid::CrosswordGrid::new_single_word("ALPHA");
    assert_eq!(grid.count_all_words(), 1);
    assert_eq!(grid.count_intersections(), 0);
    assert_eq!(grid.to_graph().count_cycles(), 0);
    assert_eq!(grid.get_grid_dimensions(), (1, 5));
    assert_eq!(grid.count_empty_cells(), 0);
    assert_eq!(grid.count_filled_cells(), 5);
    grid.check_valid();
}

#[test]
fn add_random_words() {
    let mut grid = crossword::grid::CrosswordGridBuilder::new().from_file("tests/resources/everyman_starter.txt");
    grid.add_unplaced_word("PROBONO");
    grid.add_unplaced_word("PASTURE");
    grid.add_unplaced_word("VETO");
    grid.add_unplaced_word("EROS");

    let mut success = true;
    while success {
        success = grid.place_random_word();
    }
    println!("{}", grid.to_string());
    assert_eq!(grid.count_placed_words(), 7);
}

#[test]
fn test_generator() {
    crossword::logging::init_logger(true);
    let words = vec!["BEARER", "ABOVE", "HERE", "INVALUABLE", "BANANA", "ROYAL", "AROUND", "ROE"];
    let mut generator = crossword::generator::CrosswordGenerator::new_from_singletons(words);
    generator.generate();
}
