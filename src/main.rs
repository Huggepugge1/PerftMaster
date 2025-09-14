mod board;
mod r#move;
mod move_generator;
mod perft;
mod search;
mod search_test;
mod uci;

use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Perft {
        depth: u8,

        #[arg(long)]
        fen: Option<String>,

        #[arg(long, short)]
        zobrist: bool,
    },
    Search {
        depth: u8,

        #[arg(long)]
        fen: Option<String>,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Some(Command::Perft {
            depth,
            fen,
            zobrist,
        }) => match zobrist {
            false => perft::perft_test(depth, fen),
            true => perft::zobrist_test(depth, fen),
        },
        Some(Command::Search { depth, fen }) => search_test::search_test(depth, fen),
        None => uci::run(),
    }
}
