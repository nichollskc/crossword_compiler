use crossword;

fn main() {
    let mut grid = crossword::grid::CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
    println!("{:#?}", grid);
    println!("{}", grid.to_string());
    grid.fit_to_size();
    println!("{:#?}", grid);
    println!("{}", grid.to_string());
}
