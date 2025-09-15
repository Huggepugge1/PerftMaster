use vampirc_uci::parse_with_unknown;
use vampirc_uci::{MessageList, Serializable, UciMessage};

use std::sync::{Arc, RwLock};
use std::thread;

use crate::board::Board;
use crate::search::Search;

#[derive(Debug, PartialEq)]
pub enum Status {
    Idle,
    Go,
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
        let messages: MessageList = parse_with_unknown(&input);
        for m in messages {
            match m {
                UciMessage::Uci => {
                    println!(
                        "{}",
                        UciMessage::Id {
                            name: Some(String::from("Perftmaster v0.7.0")),
                            author: Some(String::from("Hugo Lindström")),
                        }
                        .serialize()
                    );
                    println!("{}", UciMessage::UciOk.serialize());
                }

                UciMessage::IsReady => println!("{}", UciMessage::ReadyOk.serialize()),

                UciMessage::Go {
                    time_control,
                    search_control,
                    ..
                } => {
                    *stopper.write().expect("Failed to start the search") = Status::Go;
                    let mut board = board.clone();
                    let stopper = stopper.clone();
                    thread::spawn(move || {
                        println!(
                            "{}",
                            UciMessage::BestMove {
                                best_move: Search::go(
                                    &mut board,
                                    search_control,
                                    time_control,
                                    stopper.clone()
                                )
                                .pv
                                .as_ucimove(),
                                ponder: None,
                            }
                        );
                        *stopper.write().expect("Failed to start the search") = Status::Idle;
                    });
                }

                UciMessage::UciNewGame => board.new_game(),
                UciMessage::Position { fen, moves, .. } => board.load_position(fen, moves),

                UciMessage::Stop => {
                    *stopper.write().expect("Failed to stop the search") = Status::Stopping
                }

                UciMessage::Quit => return,

                other => eprintln!("Command not implemented: {other}"),
            };
        }
    }
}
