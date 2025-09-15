# Improvement log

## Version 0.1.0
Completely random play
perft-speed: ~5M nodes/s 
Elo: 0

## Version 0.2.0
### Whats changed
Implemented simple negamax algorithm with a material evaluation
Pawn: 100
Rook: 500
Knight: 300
Bishop: 300
Queen: 900
King: 100000

Running at depth 3

### Performance
perft-speed: ~5M nodes/s (unchanged code)
Elo: 442.8 +/- 28.3 (Against random)

## Version 0.3.0
### Whats changed
Implemented AB-pruning.

Running at depth 5

### Performance
perft-speed: ~5M nodes/s (unchanged code)
Elo: 478.0 +/- 108.4 (Against random)

## Version 0.4.0
### Whats changed
Implemented piece square tables, quiecence search, MVV-LVA and time management.

### Performance
#### OBS: Elo is now calculated on a new time control (8+0.08)
perft-speed: ~5M nodes/s (unchanged code)
Elo: 246.2 +/- 86.1 (Against random)

## Version 0.5.0
### Whats changed
Implemented transposition tables as well as some general improvements

### Performance
perft-speed: ~5M nodes/s
Elo: 524.10 +/- 93.16 (Against v4)

## Version 0.6.0
### Whats changed
Mostly better move generation

### Performance
#### At this point, random does not score a single draw
perft-speed: ~30M nodes/s
Elo: 61.4 +/- 27.4 (Against v5)

## Version 0.7.0
### Whats changed
Better scores and TT

### Performance
perft-speed: ~30M nodes/s (unchanged)
Elo: 229.8 +/- 63.5 (Against v6)
