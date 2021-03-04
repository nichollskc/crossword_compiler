# Crossworld builder

Tool to fit a list of words into a crossword grid. It uses a genetic algorithm to optimise the grid, trying to maximise the number of intersections and minimise the area of the grid.

# Todo

- Scoring function flaky - I don't seem to have got it quite right yet. Perhaps I should give explicit examples e.g. this grid should score better than that grid.
- Adjust scoring function during iterations? E.g. start with focus on cycles, square etc. but gradually increase focus on number of words placed
- Allow adjustment of scoring function e.g. weights
- Final output step to fill in as many words as possible in the grid
- Add code to remove words as well. Probably by splitting grid into connected components.
- Write latex output to file and run 'pdflatex <file>.tex'
- Read in clues from file (example is `tests/resources/input_with_clues.txt`)
- Allow clues to be restricted to down or across (add required_direction: Option<Direction> to Word)

# Web app

- Rocket looks like a good tool for rust web dev
- Form with big input box for all clues/words etc.
- Form also includes generator settings (limited to avoid excessive CPU usage!) such as seed and max_rounds etc.
- On submission, parse contents, run generator and save lots of tex and pdf files.
- Save under e.g. /crosswords/2021/01/31/1
- Display previews of pdf files
- Robust parsing - make sure contents is clean e.g. no extra latex commands
- Host on pythonanywhere - I've set up an account pedigreecrosswords
