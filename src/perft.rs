use crate::board::Board;

use ordermap::OrderMap;
use serde::Deserialize;
use vampirc_uci::UciFen;

use std::fs;

#[derive(Deserialize, Debug)]
struct Position {
    fen: String,
    depths: OrderMap<u16, OrderMap<String, usize>>,
}

pub fn perft_test(max_depth: u16, fen: Option<String>) {
    let mut board = Board::new();

    if let Some(fen) = fen {
        board.load_position(Some(UciFen(fen)), Vec::new());
        board.perft(max_depth);
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
    fn perft(&mut self, depth: u16) -> OrderMap<String, usize> {
        if depth == 0 {
            return OrderMap::from([(String::new(), 1)]);
        }
        let mut result = OrderMap::new();
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
        perft: OrderMap<String, usize>,
        stockfish: OrderMap<String, usize>,
        fen: &str,
        depth: u16,
    ) {
        for (m, nodes) in &perft {
            if !stockfish.contains_key(m) {
                println!("Extra move!");
                self.print();
                println!("{m}");
                println!("uci");
                println!("cargo run -- perft {depth} --fen {fen}");
                panic!();
            }
            if stockfish.get(m).unwrap() != nodes {
                println!("Not the same number of nodes!");
                self.print();
                println!("Stockfish: {}: {}", m, stockfish.get(m).unwrap());
                println!("Me: {m}: {nodes}");
                println!("cargo run -- perft {depth} --fen {fen}");
                panic!();
            }
        }
        for (m, nodes) in &stockfish {
            if !perft.contains_key(m) {
                println!("Move missing!");
                self.print();
                println!("{m}");
                println!("cargo run -- perft {depth} --fen \"{fen}\"");
                panic!();
            }
            if perft.get(m).unwrap() != nodes {
                println!("Not the same number of nodes!");
                self.print();
                println!("Stockfish: {}: {}", m, perft.get(m).unwrap());
                println!("Me: {m}");
                println!("cargo run -- perft {depth} --fen {fen}");
                panic!();
            }
        }
    }
}
