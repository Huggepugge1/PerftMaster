# Improvement log

## Version 1
Completely random play
perft-speed: ~5M nodes/s 
Elo: 0

## Version 2
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
Elo: 442.8 +/- 28.3,
