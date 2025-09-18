remove_old_logs:
	rm -f info.log log.pgn

build:
	cargo build --release


OPENINGS=file=openings/8moves_v3.pgn
TC=inf/10+0.1

ENGINE1=target/release/perftmaster

ENGINE2=binaries/v7

SHARED_ENGINE_OPTIONS=proto=uci tc=$(TC)

THREADS=1

PGN_TIMELEFT=true

run: remove_old_logs build
	cutechess-cli -engine cmd=$(ENGINE1) stderr=info.log -engine cmd=$(ENGINE2) -each $(SHARED_ENGINE_OPTIONS) -concurrency $(THREADS) -rounds 1000 -repeat 2 -games 2 -maxmoves 200 -openings $(OPENINGS) -sprt elo0=0 elo1=10 alpha=0.05 beta=0.05 -ratinginterval 10 -pgnout log.pgn

profile_go: build
	rm -f callgrind.out.*
	valgrind --tool=callgrind ./target/release/perftmaster
	kcachegrind ./callgrind.out.*

profile_perft: build
	rm -f callgrind.out.*
	valgrind --tool=callgrind ./target/release/perftmaster perft 2
	kcachegrind ./callgrind.out.*
