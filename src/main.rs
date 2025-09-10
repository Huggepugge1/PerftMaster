mod board;
mod r#move;
mod move_generator;
mod perft;
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
        depth: u16,

        #[arg(long)]
        fen: Option<String>,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Some(Command::Perft { depth, fen }) => perft::perft_test(depth, fen),
        None => uci::run(),
    }
}
