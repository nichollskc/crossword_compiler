RUST_LOG=info,crossword::generator=debug,crossword::graph=warn cargo test test_generator_fif -- --nocapture > output2.txt 2>&1
RUST_LOG=info,crossword::generator=debug,crossword::graph=warn cargo test test_generator_fif -- --nocapture > output.txt 2>&1
vimdiff output*
