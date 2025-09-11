remove_old_logs:
	rm -f info.log log.pgn

build:
	cargo build --release

ENGINE1=target/release/perftmaster
ENGINE1_NAME=perftmaster-v0.5.0

ENGINE2=binaries/v4
ENGINE2_NAME=perftmaster-v0.4.0

THREADS=16

TC=8+0.08

OPENINGS=openings/8moves_v3.pgn
OPENING_FORMAT=pgn

PGN_TIMELEFT=true

run: remove_old_logs build
	fastchess -engine cmd=$(ENGINE1) name=$(ENGINE1_NAME) -engine cmd=$(ENGINE2) name=$(ENGINE2_NAME) -concurrency $(THREADS) -each tc=$(TC) -rounds 100 -repeat -recover -openings file=$(OPENINGS) format=$(OPENING_FORMAT) -sprt elo0=-10 elo1=0 alpha=0.05 beta=0.05 -pgnout file=log.pgn timeleft=$(PGN_TIMELEFT) -log file=info.log engine=true
