use crate::{board::Board, r#move::Move};

use serde::Deserialize;
use vampirc_uci::UciFen;

use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
};

#[derive(Deserialize, Debug)]
struct Position {
    fen: String,
    depths: HashMap<u16, HashMap<String, usize>>,
}

pub fn perft_test(max_depth: u16, fen: Option<String>) {
    let mut board = Board::new();

    if let Some(fen) = fen {
        board.load_position(Some(UciFen(fen.clone())), Vec::new());
        let perft = board.perft(max_depth, Move::NULL);
        let mut stockfish = setup_stockfish();
        let stockfish_perft = stockfish_perft(max_depth, &fen, Vec::new(), &mut stockfish);
        quit_stockfish(&mut stockfish);
        board.difference(perft, stockfish_perft, &fen, max_depth);
        println!("Test successful!");
        return;
    }

    let data = fs::read_to_string("./chess-position-generator/perft_dataset.json").unwrap();
    let positions: Vec<Position> = serde_json::from_str(&data).unwrap();

    let mut total = 0;

    for p in positions {
        board.load_position(Some(UciFen(p.fen.clone())), Vec::new());

        for (depth, stockfish_result) in p.depths.clone() {
            if depth > max_depth {
                continue;
            }
            let perft = board.perft(depth, Move::NULL);
            total += perft.nodes;
            if perft.nodes != stockfish_result.values().sum::<usize>() {
                let mut stockfish = setup_stockfish();
                let stockfish_perft = stockfish_perft(depth, &p.fen, Vec::new(), &mut stockfish);
                quit_stockfish(&mut stockfish);
                board.difference(perft, stockfish_perft, &p.fen, depth);
            }
        }
    }
    println!("Nodes searched: {total}");
    println!("Test successful!");
}

#[derive(Default, Clone, Debug)]
struct PerftResult {
    m: Move,
    nodes: usize,
    results: Vec<PerftResult>,
}

impl std::fmt::Display for PerftResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for result in &self.results {
            writeln!(f, "{}: {}", result.m, result.nodes)?;
        }
        writeln!(f, "Nodes searched: {}", self.nodes)?;
        Ok(())
    }
}

impl PerftResult {
    /// NOTE: Ignores flags
    fn contains_move(&self, m: Move) -> bool {
        for result in &self.results {
            if result.m.0 & 0b0000111111111111 == m.0 & 0b0000111111111111 {
                return true;
            }
        }
        false
    }

    fn new() -> Self {
        Self::default()
    }

    /// NOTE: Ignores flags
    fn get(&self, m: Move) -> Option<PerftResult> {
        for result in &self.results {
            if result.m.0 & 0b0000111111111111 == m.0 & 0b0000111111111111 {
                return Some(result.clone());
            }
        }
        None
    }
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

fn stockfish_perft(depth: u16, fen: &str, moves: Vec<Move>, stockfish: &mut Child) -> PerftResult {
    if depth == 0 {
        return PerftResult {
            m: *moves.last().unwrap(),
            nodes: 1,
            results: Vec::new(),
        };
    }
    let stdin = stockfish.stdin.as_mut().expect("Failed to get stdin");
    let position_command = format!(
        "position fen {fen} moves {}\n",
        moves
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    );
    stdin
        .write_all(position_command.as_bytes())
        .expect("failed to write to stdin");

    let perft_command = format!("go perft {depth}\n");
    stdin
        .write_all(perft_command.as_bytes())
        .expect("failed to write to stdin");
    stdin.flush().expect("Failed to flush");

    let string_perft = read_until(stockfish, "Nodes searched:")
        .split("\n")
        .filter(|e| e != &"")
        .map(String::from)
        .collect::<Vec<_>>();

    let split_perfts = string_perft
        .iter()
        .map(|e| e.split(":").collect::<Vec<_>>())
        .map(|e| {
            (
                Move::from_string_move(e[0]),
                e[1].trim().parse::<usize>().unwrap(),
            )
        })
        .collect::<Vec<_>>();

    let mut result = PerftResult::new();
    if let Some(m) = moves.last() {
        result.m = *m;
    }

    for perft in split_perfts {
        result.nodes += perft.1;
        let mut new_moves = moves.clone();
        new_moves.push(perft.0);
        result
            .results
            .push(stockfish_perft(depth - 1, fen, new_moves, stockfish));
    }

    result
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
    fn perft(&mut self, depth: u16, m: Move) -> PerftResult {
        if depth == 0 {
            return PerftResult {
                m,
                nodes: 1,
                results: Vec::new(),
            };
        }
        let mut result = PerftResult::new();
        result.m = m;
        for m in self.generate_moves() {
            self.make_move(m);
            let perft = self.perft(depth - 1, m);
            result.nodes += perft.nodes;
            result.results.push(perft);
            self.unmake_move(m);
        }
        result
    }

    fn difference(&mut self, perft: PerftResult, stockfish: PerftResult, fen: &str, depth: u16) {
        for perft_result in &perft.results {
            let PerftResult { m, nodes, .. } = perft_result;
            if !stockfish.contains_move(*m) {
                println!("Extra move!");
                self.print();
                println!("{m}");
                println!("Debug command:");
                println!("cargo run -- perft {depth} --fen \"{fen}\"");
                panic!();
            }
            if stockfish.get(*m).unwrap().nodes != *nodes {
                // Get the flags as well
                let m = perft.get(*m).unwrap().m;
                self.make_move(m);
                self.difference(perft_result.clone(), stockfish.get(m).unwrap(), fen, depth);
                self.unmake_move(m);
            }
        }
        for perft_result in &stockfish.results {
            let PerftResult { m, nodes, .. } = perft_result;
            if !perft.contains_move(*m) {
                println!("Move missing!");
                self.print();
                println!("{m}");
                println!("Debug command:");
                println!("cargo run -- perft {depth} --fen \"{fen}\"");
                panic!();
            }
            if perft.get(*m).unwrap().nodes != *nodes {
                // Get the flags as well
                let m = perft.get(*m).unwrap().m;
                self.make_move(m);
                self.difference(perft_result.clone(), stockfish.get(m).unwrap(), fen, depth);
                self.unmake_move(m);
            }
        }
    }
}
