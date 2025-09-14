use crate::{
    board::Board,
    perft::Position,
    search::{Score, Search},
    uci::Status,
};

use vampirc_uci::{UciFen, UciSearchControl};

use std::{
    fs,
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::{Arc, RwLock},
};

pub fn search_test(depth: u8, fen: Option<String>) {
    let mut board = Board::new();

    let mut stockfish = setup_stockfish();

    if let Some(fen) = fen {
        board.load_position(Some(UciFen(fen.clone())), Vec::new());
        board.print();
        board.search_test(depth);
        stockfish_search_test(depth, &fen, &mut stockfish);
        quit_stockfish(&mut stockfish);
        return;
    }

    let data = fs::read_to_string("./chess-position-generator/perft_dataset.json").unwrap();
    let positions: Vec<Position> = serde_json::from_str(&data).unwrap();

    for p in &positions {
        board.load_position(Some(UciFen(p.fen.clone())), Vec::new());
        println!("fen: {}", &p.fen);
        match board.search_test(depth).score {
            Score::OwnMate(_) | Score::OppMate(_) => {
                stockfish_search_test(depth, &p.fen, &mut stockfish);
                return;
            }
            _ => (),
        }
    }
    quit_stockfish(&mut stockfish);
}

fn read_line(stockfish: &mut Child) -> String {
    let stdout = stockfish.stdout.as_mut().expect("Failed to get stdout");

    let mut reader = BufReader::new(stdout);

    let mut line = String::new();
    let _ = reader.read_line(&mut line).unwrap();
    line
}

fn read_until(stockfish: &mut Child, terminator: &str) -> String {
    let stdout = stockfish.stdout.as_mut().expect("Failed to get stdout");

    let mut reader = BufReader::new(stdout);

    let mut result = String::new();
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).unwrap();
        if n == 0 {
            break;
        }
        if line.trim().contains(terminator) {
            break;
        }
        result += &line;
    }
    result
}

fn setup_stockfish() -> Child {
    let mut stockfish = Command::new("stockfish")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start stockfish");

    read_line(&mut stockfish);

    let stdin = stockfish.stdin.as_mut().expect("Failed to get stdin");
    stdin.write_all(b"uci\n").expect("failed to write to stdin");
    stdin.flush().expect("Failed to flush");

    read_until(&mut stockfish, "uciok");

    stockfish
}

fn stockfish_search_test(depth: u8, fen: &str, stockfish: &mut Child) {
    let stdin = stockfish.stdin.as_mut().expect("Failed to get stdin");
    let position_command = format!("position fen {fen}\n",);
    stdin
        .write_all(position_command.as_bytes())
        .expect("failed to write to stdin");

    let search_command = format!("go depth {depth}\n");
    stdin
        .write_all(search_command.as_bytes())
        .expect("failed to write to stdin");
    stdin.flush().expect("Failed to flush");

    let string_infos = read_until(stockfish, "bestmove")
        .split("\n")
        .filter(|e| e != &"" && e.contains(&"cp"))
        .map(String::from)
        .collect::<Vec<_>>();

    println!("Stockfish\n{}", string_infos.join("\n"));
}

fn quit_stockfish(stockfish: &mut Child) {
    let mut stdin = stockfish.stdin.take().expect("Failed to get stdin");
    stdin
        .write_all(b"quit\n")
        .expect("failed to write to stdin");
    stdin.flush().expect("Failed to flush");

    let _ = stockfish.wait();
}

impl Board {
    fn search_test(&'_ mut self, depth: u8) -> Search<'_> {
        println!("Me");
        Search::go(
            self,
            Some(UciSearchControl {
                search_moves: Vec::new(),
                mate: None,
                depth: Some(depth),
                nodes: None,
            }),
            None,
            Arc::new(RwLock::new(Status::Go)),
        )
    }
}
