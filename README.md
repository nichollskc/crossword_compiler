# Crossworld builder

Tool to fit a list of words into a crossword grid. It uses a genetic algorithm to optimise the grid, trying to maximise the number of intersections and minimise the area of the grid.

# Todo

- Add code to remove words as well. Probably by splitting grid into connected components.
- Don't stop as soon as score hasn't improved - wait e.g. 2 rounds of no increase

# Web app

- Rocket looks like a good tool for rust web dev
- Form with big input box for all clues/words etc.
- Form also includes generator settings (limited to avoid excessive CPU usage!) such as seed and max_rounds etc.
- On submission, parse contents, run generator and save lots of tex and pdf files.
- Save under e.g. /crosswords/2021/01/31/1
- Display previews of pdf files
- Robust parsing - make sure contents is clean e.g. no extra latex commands
- Host on pythonanywhere - I've set up an account pedigreecrosswords
-
guild run generate num-per-gen=[5,10,15,20,25,30] moves-between-scores=[1,2,3,4,5,6] num-children=[5,10,15,20] weight-non-square=[1,5,10,50,100,500,1000] weight-prop-filled=[1,5,10,50,100,500,1000] weight-prop-intersect=[1,5,10,50,100,500,1000] weight-num-cycles=[1,5,10,50,100,500,1000] weight-num-intersect=[1,5,10,50,100,500,1000] weight-words-placed=[1,5,10,50,100,500,1000] -o random --max-trials 100

96af330691054635b86a7127aba35e8d looks good
