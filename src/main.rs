use crossword;

fn main() {
    crossword::logging::init_logger(true);
    let mut generator = crossword::generator::CrosswordGenerator::new_from_file("tests/resources/input_with_clues.txt");
    let results = generator.generate();
    for grid in results.iter() {
        println!("{}", grid.to_string());
        let mut printer = crossword::grid::CrosswordPrinter::new(grid.clone());
        printer.print_to_pdf("test");
    }
}
