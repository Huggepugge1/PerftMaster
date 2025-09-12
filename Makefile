remove_old_logs:
	rm -f info.log log.pgn

build:
	cargo build --release


OPENINGS=file=openings/8moves_v3.pgn
TC=inf/10+0.1

ENGINE1=target/release/perftmaster
ENGINE1_NAME=perftmaster-v0.6.0

ENGINE2=binaries/v5
ENGINE2_NAME=perftmaster-v0.5.0

SHARED_ENGINE_OPTIONS=proto=uci tc=$(TC)

THREADS=16

PGN_TIMELEFT=true

run: remove_old_logs build
	cutechess-cli -engine cmd=$(ENGINE1) stderr=info.log -engine cmd=$(ENGINE2) name=$(ENGINE2_NAME) -each $(SHARED_ENGINE_OPTIONS) -concurrency $(THREADS) -rounds 1000 -repeat 2 -games 2 -maxmoves 200 -openings $(OPENINGS) -sprt elo0=0 elo1=10 alpha=0.05 beta=0.05 -ratinginterval 10 -pgnout log.pgn
