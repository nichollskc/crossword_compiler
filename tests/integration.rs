use log::{info,debug};
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
    grid.add_unplaced_word("PROBONO", None);
    grid.add_unplaced_word("PASTURE", None);
    grid.add_unplaced_word("VETO", None);
    grid.add_unplaced_word("EROS", None);

    let mut success = true;
    while success {
        success = grid.place_random_word(13);
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

#[test]
fn test_generator_fifteen_squared() {
    crossword::logging::init_logger(true);
    let mut generator = crossword::generator::CrosswordGenerator::new_from_file("tests/resources/fifteensquared/quiptic-1109-by-pan.txt");
    generator.settings = crossword::generator::CrosswordGeneratorSettings::new(13, 5, 2, 5, 5);
    let results = generator.generate();
    for grid in results.iter() {
        debug!("{}", grid.to_string());
    }

    let mut generator2 = crossword::generator::CrosswordGenerator::new_from_file("tests/resources/fifteensquared/quiptic-1109-by-pan.txt");
    generator2.settings = crossword::generator::CrosswordGeneratorSettings::new(13, 5, 2, 5, 5);
    let results2 = generator2.generate();

    for grid in results2.iter() {
        debug!("{}", grid.to_string());
    }


    for i in 0..5 {
        assert_eq!(results[i].to_string(), results2[i].to_string(),
            "Expected grids from each identical generators to look identical. Failed for index {}", i);
    }
}

#[ignore] // Ignore by default as it is slow
#[test]
fn test_generator_fifteen_squared_branching() {
    crossword::logging::init_logger(true);
    info!("Starting branching generator");
    let mut generator = crossword::generator::CrosswordGenerator::new_from_file("tests/resources/fifteensquared/quiptic-1109-by-pan.txt");
    generator.settings = crossword::generator::CrosswordGeneratorSettings::new(13, 30, 2, 100, 1);
    let results = generator.generate();
    for grid in results.iter() {
        debug!("{}", grid.to_string());
    }
}

#[test]
fn test_printing() {
    let grid = crossword::grid::CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
    let mut printer = crossword::grid::CrosswordPrinter::new(grid);
    println!("{}", printer.print());
    debug!("{:#?}", printer);
}
