# aoc-helper

Basic CLI program to download input files from the Advent of Code server.

Requires setting the cookie via the -c flag.  You'll need to go into the developer console of your browser and grab the cookie labeled 'session'.

Automatically downloads the current day's input with no arguments and places it in the ./input directory with yyyy-dd.txt name.  You can specify other days and years as well.  'aoc-helper 2020 25' would download the last input for the 2020 advent.

Run aoc-helper -h for extra options
