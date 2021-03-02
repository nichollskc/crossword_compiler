# Crossworld builder

Tool to fit a list of words into a crossword grid. It uses a genetic algorithm to optimise the grid, trying to maximise the number of intersections and minimise the area of the grid.

# Todo

- Bug: Currently allows two across clues to be adjacent if only by a single square e.g. BEAR
                                                                                           BUTTON
- Scoring function flaky - I don't seem to have got it quite right yet. Perhaps I should give explicit examples e.g. this grid should score better than that grid.
- Random adding of words isn't random - it's just deterministic based on the Iterator
- Adjust scoring function during iterations? E.g. start with focus on cycles, square etc. but gradually increase focus on number of words placed
- Final output step to fill in as many words as possible in the grid
- Add code to remove words as well. Probably by splitting grid into connected components.
