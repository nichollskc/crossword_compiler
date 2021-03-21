# Crossworld builder

Tool to fit a list of words into a crossword grid. It uses a genetic algorithm to optimise the grid, trying to maximise the number of intersections and minimise the area of the grid.

# Todo

- Improve error handling - remove asserts and panics and unwraps as much as possible, use anyhow to convert errors to a generic error type with basic context (and catch and deal with these in the web version)
- Could use the matrix method to add a word in the best possible place, rather than just randomly (perhaps as another MoveType)
- Tom's tournament bracket idea for generating grids - start with singleton words, pair up in way that maximises score and diversity of children and then fix those pairs. Then in each round we find the best way to pair up the elements of the previous round and again fix those pairs and combine them to make the next generation. This makes it easier to combine grids as we can do anything as long as they belong to a different bracket.
- It might work well to do this until we have e.g. 4 words in each grid and then just try and aggregate them all together. This second idea can be reduced to just "split the words into groups of 4 that seem likely to work well/do work well" (roughly 32^4 combinations) and then "find different ways to combine these" (roughly 8! combinations - which order do we try and combine)

# Tuning hyperparameters

I have set up GuildAI to keep track of scores achieved.

source venv/bin/guild-env
guild view

Run new results:
guild run generate num-per-gen=[5,10,15,20,25,30] moves-between-scores=[1,2,3,4,5,6] num-children=[5,10,15,20] weight-non-square=[1,5,10,50,100,500,1000] weight-prop-filled=[1,5,10,50,100,500,1000] weight-prop-intersect=[1,5,10,50,100,500,1000] weight-num-cycles=[1,5,10,50,100,500,1000] weight-num-intersect=[1,5,10,50,100,500,1000] weight-words-placed=[1,5,10,50,100,500,1000] -o random --max-trials 100

96af330691054635b86a7127aba35e8d looks good

# Profiling

Flamegraph
cargo install flamegraph
sudo cargo flamegraph --bin crossword -- --input-file tests/resources/fifteensquared/quiptic-1109-by-pan.txt

Then interact using a browser:
file:///Users/kath/docs/Programming/rust/crossword/flamegraph.svg

# Code coverage

Travis build set up to use grcov and submit to codecov

# Error handling/logging

warn! should be used whenever we have reached a potentially bad state, but we can recover. The state should be one that should be considered bad by all callers (e.g. a graph has an edge where the nodes are not present).
If the state is not universally considered bad, an error should be returned by the function and the caller can decide whether it's fatal or not.
