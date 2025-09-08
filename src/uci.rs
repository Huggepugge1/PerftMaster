use vampirc_uci::parse;
use vampirc_uci::{MessageList, Serializable, UciMessage};

use std::sync::{Arc, RwLock};

use crate::board::Board;

pub enum Status {
    Idle,
    Stopping,
}

pub fn run() {
    let stopper = Arc::new(RwLock::new(Status::Idle));
    let mut board = Board::new();
    loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed reading string");
        let messages: MessageList = parse(&input);
        for m in messages {
            match m {
                UciMessage::Uci => {
                    println!(
                        "{}",
                        UciMessage::Id {
                            name: None,
                            author: Some(String::from("Hugo Lindström")),
                        }
                        .serialize()
                    );
                    println!("{}", UciMessage::UciOk.serialize());
                }

                UciMessage::IsReady => println!("{}", UciMessage::ReadyOk.serialize()),

                UciMessage::UciNewGame => board.new_game(),
                UciMessage::Position {
                    startpos,
                    fen,
                    moves,
                } => {
                    board.load_position(startpos, fen, moves);
                    board.print();
                }

                UciMessage::Stop => {
                    *stopper.write().expect("Failed to stop the search") = Status::Stopping
                }
                other => eprintln!("Command not implemented: {other}"),
            };
        }
    }
}
