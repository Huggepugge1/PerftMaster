use crate::board::Board;

use serde::Deserialize;
use vampirc_uci::UciFen;

use std::{collections::HashMap, fs};

#[derive(Deserialize, Debug)]
struct Position {
    fen: String,
    depths: HashMap<u16, HashMap<String, usize>>,
}

pub fn perft_test(max_depth: u16, fen: Option<String>, m: Option<String>) {
    let mut board = Board::new();

    if let Some(fen) = fen {
        board.load_position(Some(UciFen(fen)), Vec::new());
        let perft = board.perft(max_depth);
        board.print();
        if let Some(m) = m {
            let mut found = false;
            for (perft_move, nodes) in perft {
                if perft_move == m {
                    println!("{perft_move}: {nodes}");
                    found = true;
                    break;
                }
            }
            if !found {
                println!("Missing move: {m}");
            }
        } else {
            for (m, nodes) in perft {
                println!("{m}: {nodes}");
            }
        }
        return;
    }

    let data = fs::read_to_string("./chess-position-generator/perft_dataset.json").unwrap();
    let positions: Vec<Position> = serde_json::from_str(&data).unwrap();

    for p in positions {
        board.load_position(Some(UciFen(p.fen.clone())), Vec::new());

        for (depth, stockfish_result) in p.depths {
            if depth > max_depth {
                break;
            }
            let perft = board.perft(depth);
            board.difference(perft, stockfish_result, &p.fen, depth);
        }
    }
}

impl Board {
    fn perft(&mut self, depth: u16) -> HashMap<String, usize> {
        if depth == 0 {
            return HashMap::from([(String::new(), 1)]);
        }
        let mut result = HashMap::new();
        for m in self.generate_moves() {
            self.make_move(m);
            result.insert(
                m.to_string(),
                self.perft(depth - 1)
                    .iter()
                    .fold(0, |a, (_m, nodes)| a + nodes),
            );
            self.unmake_move(m);
        }
        result
    }

    fn difference(
        &self,
        perft: HashMap<String, usize>,
        stockfish: HashMap<String, usize>,
        fen: &str,
        depth: u16,
    ) {
        for (m, nodes) in &perft {
            if !stockfish.contains_key(m) {
                println!("Extra move!");
                self.print();
                println!("{m}");
                println!("Debug command:");
                println!("cargo run -- perft {depth} --fen \"{fen}\" --move {m}");
                panic!();
            }
            if stockfish.get(m).unwrap() != nodes {
                println!("Not the same number of nodes!");
                self.print();
                println!("Stockfish: {}: {}", m, stockfish.get(m).unwrap());
                println!("Me: {m}: {nodes}");
                println!("Debug command:");
                println!("cargo run -- perft {depth} --fen \"{fen}\" --move {m}");
                panic!();
            }
        }
        for (m, nodes) in &stockfish {
            if !perft.contains_key(m) {
                println!("Move missing!");
                self.print();
                println!("{m}");
                println!("Debug command:");
                println!("cargo run -- perft {depth} --fen \"{fen}\" --move {m}");
                panic!();
            }
            if perft.get(m).unwrap() != nodes {
                println!("Not the same number of nodes!");
                self.print();
                println!("Stockfish: {}: {}", m, perft.get(m).unwrap());
                println!("Me: {m}");
                println!("Debug command:");
                println!("cargo run -- perft {depth} --fen \"{fen}\" --move {m}");
                panic!();
            }
        }
    }
}
