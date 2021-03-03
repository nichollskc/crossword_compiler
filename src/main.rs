use crossword;

fn main() {
    crossword::logging::init_logger(true);
    let mut generator = crossword::generator::CrosswordGenerator::new_from_file("tests/resources/fifteensquared/quiptic-1109-by-pan.txt");
    let results = generator.generate();
    for grid in results.iter() {
        println!("{}", grid.to_string());
    }
}
