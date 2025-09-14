use std::{
    cmp::Ordering,
    collections::HashMap,
    sync::{Arc, RwLock, mpsc::channel},
    thread::{self, sleep},
    time::{Duration, Instant},
};

use vampirc_uci::{UciSearchControl, UciTimeControl};

use crate::{
    board::{Board, Color, Piece, PieceKind},
    r#move::Move,
    uci::Status,
};

impl Board {}

#[derive(Debug)]
enum NodeKind {
    PvNode,
    CutNode,
    AllNode,
}

#[derive(Debug)]
struct TTNode {
    best_move: Move,
    depth: u8,
    score: Score,
    kind: NodeKind,
}

#[derive(Debug)]
pub struct Search<'a> {
    pub best_move: Move,
    depth: u8,
    board: &'a mut Board,

    tt: HashMap<u64, TTNode>,
    tt_hits: usize,

    nodes: usize,

    pub score: Score,

    start: Instant,

    stopper: Arc<RwLock<Status>>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Score {
    OwnMate(usize),
    OppMate(usize),
    Score(i64),
    Draw(usize),

    Stop,
}

impl Default for Score {
    fn default() -> Self {
        Self::Score(0)
    }
}

impl std::fmt::Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Score::OwnMate(ply) => write!(f, "M+{ply}"),
            Score::OppMate(ply) => write!(f, "M-{ply}"),
            Score::Score(score) => {
                if *score < 1000 {
                    write!(f, "{:.1}", *score as f64 / 100f64)
                } else {
                    write!(f, "{:.0}", *score as f64 / 100f64)
                }
            }
            Score::Draw(_) => write!(f, "0"),
            Score::Stop => write!(f, "?"),
        }
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Score::Stop, _) => Some(std::cmp::Ordering::Less),
            (_, Score::Stop) => Some(std::cmp::Ordering::Greater),

            (Score::OwnMate(ply1), Score::OwnMate(ply2)) => Some(ply2.cmp(ply1)),
            (Score::OwnMate(_), _) => Some(std::cmp::Ordering::Greater),

            (Score::OppMate(ply1), Score::OppMate(ply2)) => Some(ply1.cmp(ply2)),
            (Score::OppMate(_), _) => Some(std::cmp::Ordering::Less),

            (_, Score::OwnMate(_)) => Some(std::cmp::Ordering::Less),
            (_, Score::OppMate(_)) => Some(std::cmp::Ordering::Greater),

            (Score::Score(score1), Score::Score(score2)) => Some(score1.cmp(score2)),

            (Score::Score(score), Score::Draw(_)) => Some(score.cmp(&0)),
            (Score::Draw(_), Score::Score(score)) => Some(0.cmp(score)),
            (Score::Draw(_), Score::Draw(_)) => Some(std::cmp::Ordering::Equal),
        }
    }
}

impl std::ops::AddAssign for Score {
    fn add_assign(&mut self, rhs: Self) {
        match (self.clone(), rhs) {
            (Score::Stop, _) => (),
            (_, Score::Stop) => *self = Score::Stop,

            (Score::OwnMate(ply1), Score::OwnMate(ply2)) => *self = Score::OwnMate(ply1.min(ply2)),
            (Score::OwnMate(ply1), Score::OppMate(ply2)) => {
                *self = if ply1 < ply2 {
                    Score::OwnMate(ply1)
                } else {
                    Score::OppMate(ply2)
                }
            }
            (Score::OwnMate(_), _) => (),

            (Score::OppMate(ply1), Score::OppMate(ply2)) => *self = Score::OppMate(ply1.min(ply2)),
            (Score::OppMate(ply1), Score::OwnMate(ply2)) => {
                *self = if ply1 < ply2 {
                    Score::OppMate(ply1)
                } else {
                    Score::OwnMate(ply2)
                }
            }
            (Score::OppMate(ply1), Score::Draw(ply2)) => {
                *self = if ply1 < ply2 {
                    Score::OppMate(ply1)
                } else {
                    Score::Draw(ply2)
                }
            }
            (Score::OppMate(_), _) => (),

            (_, Score::OwnMate(ply)) => *self = Score::OwnMate(ply),
            (_, Score::OppMate(ply)) => *self = Score::OppMate(ply),

            (Score::Score(score1), Score::Score(score2)) => *self = Score::Score(score1 + score2),

            (Score::Score(_), Score::Draw(ply)) => *self = Score::Draw(ply),
            (Score::Draw(_), Score::Score(_)) => (),
            (Score::Draw(ply1), Score::Draw(ply2)) => *self = Score::Draw(ply1.min(ply2)),
        }
    }
}

impl std::ops::Neg for Score {
    type Output = Score;

    fn neg(self) -> Self::Output {
        match self {
            Score::OwnMate(ply) => Score::OppMate(ply),
            Score::OppMate(ply) => Score::OwnMate(ply),
            Score::Score(score) => Score::Score(-score),
            Score::Draw(ply) => Score::Draw(ply),
            Score::Stop => Score::Stop,
        }
    }
}

impl Score {
    fn centipawns(&self) -> String {
        match self {
            Score::OwnMate(ply) => format!("M+{ply}"),
            Score::OppMate(ply) => format!("M-{ply}"),
            Score::Score(score) => format!("{score}"),
            Score::Draw(_) => format!("0"),
            Score::Stop => format!("?"),
        }
    }

    fn flip_score(self) -> Score {
        match self {
            Score::OwnMate(ply) => Score::OwnMate(ply),
            Score::OppMate(ply) => Score::OppMate(ply),
            Score::Score(score) => Score::Score(-score),
            Score::Draw(ply) => Score::Draw(ply),
            Score::Stop => Score::Stop,
        }
    }

    fn inc(self) -> Score {
        match self {
            Score::OwnMate(ply) => Score::OwnMate(ply + 1),
            Score::OppMate(ply) => Score::OppMate(ply + 1),
            Score::Score(score) => Score::Score(score),
            Score::Draw(ply) => Score::Draw(ply + 1),
            Score::Stop => Score::Stop,
        }
    }
}

impl<'a> Search<'a> {
    fn new(stopper: Arc<RwLock<Status>>, board: &'a mut Board) -> Self {
        Self {
            best_move: Move::default(),
            depth: 0,
            board,

            tt: HashMap::new(),
            tt_hits: 0,

            nodes: 0,

            score: Score::default(),

            start: Instant::now(),

            stopper,
        }
    }

    pub fn go(
        board: &'a mut Board,
        search_control: Option<UciSearchControl>,
        time_control: Option<UciTimeControl>,
        stopper: Arc<RwLock<Status>>,
    ) -> Search<'a> {
        let (sender, receiver) = channel();
        let (alpha, beta) = (Score::OppMate(0), Score::OwnMate(0));
        let mut search = Self::new(stopper.clone(), board);
        if let Some(time_control) = time_control {
            let move_time = match time_control {
                UciTimeControl::TimeLeft {
                    white_time: Some(white_time),
                    black_time: Some(black_time),
                    ..
                } => match search.board.turn {
                    Color::White => white_time.num_milliseconds() / 20,
                    Color::Black => black_time.num_milliseconds() / 20,
                    Color::None => unreachable!(),
                },
                _ => 0,
            };
            if move_time != 0 {
                let stopper = stopper.clone();
                thread::spawn(move || {
                    sleep(Duration::from_millis(move_time as u64));
                    if receiver.try_recv().is_err() {
                        *stopper.write().unwrap() = Status::Stopping;
                    }
                });
            }
        };

        let max_depth = match search_control {
            Some(UciSearchControl {
                depth: Some(depth), ..
            }) => depth,
            _ => u8::MAX,
        };

        let mut depth = 1;
        while *search.stopper.read().unwrap() != Status::Stopping && depth <= max_depth {
            search.depth = depth;
            search.score = search.negamax(search.depth, alpha, beta);
            println!(
                "info depth {} score cp {} nodes {} nps {} pv {}",
                search.depth,
                search.score.centipawns(),
                search.nodes,
                (search.nodes as f64 / search.start.elapsed().as_secs_f64()) as u64,
                search.best_move,
            );
            depth += 1;
        }

        let _ = sender.send(());
        return search;
    }

    const WHITE_PAWN_SQUARE_TABLE: [i64; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, -20, -20, 10, 10, 5, 5, -5, -10, 0, 0, -10, -5, 5, 0, 0,
        0, 20, 20, 0, 0, 0, 5, 5, 10, 25, 25, 10, 5, 5, 10, 10, 20, 30, 30, 20, 10, 10, 50, 50, 50,
        50, 50, 50, 50, 50, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    const BLACK_PAWN_SQUARE_TABLE: [i64; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5,
        5, 10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10,
        -20, -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    const KNIGHT_SQUARE_TABLE: [i64; 64] = [
        -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 5, 5, 0, -20, -40, -30, 5, 10, 15, 15,
        10, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 10, 15,
        15, 10, 0, -30, -40, -20, 0, 0, 0, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
    ];
    const WHITE_BISHOP_SQUARE_TABLE: [i64; 64] = [
        -20, -10, -10, -10, -10, -10, -10, -20, -10, 5, 0, 0, 0, 0, 5, -10, -10, 10, 10, 10, 10,
        10, 10, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 5, 10,
        10, 5, 0, -10, -10, 0, 0, 0, 0, 0, 0, -10, -20, -10, -10, -10, -10, -10, -10, -20,
    ];
    const BLACK_BISHOP_SQUARE_TABLE: [i64; 64] = [
        -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5,
        0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10,
        10, 10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
    ];
    const WHITE_ROOK_SQUARE_TABLE: [i64; 64] = [
        0, 0, 0, 5, 5, 0, 0, 0, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
        0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 5, 10, 10, 10, 10, 10, 10, 5,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];
    const BLACK_ROOK_SQUARE_TABLE: [i64; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0,
        0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0,
        -5, 0, 0, 0, 5, 5, 0, 0, 0,
    ];
    const WHITE_QUEEN_SQUARE_TABLE: [i64; 64] = [
        -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 5, 0, 0, 0, 0, -10, -10, 5, 5, 5, 5, 5, 0,
        -10, 0, 0, 5, 5, 5, 5, 0, -5, -5, 0, 5, 5, 5, 5, 0, -5, -10, 0, 5, 5, 5, 5, 0, -10, -10, 0,
        0, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
    ];
    const BLACK_QUEEN_SQUARE_TABLE: [i64; 64] = [
        -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0,
        -10, -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0,
        5, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
    ];
    const WHITE_KING_SQUARE_TABLE: [i64; 64] = [
        20, 30, 10, 0, 0, 10, 30, 20, 20, 20, 0, 0, 0, 0, 20, 20, -10, -20, -20, -20, -20, -20,
        -20, -10, -20, -30, -30, -40, -40, -30, -30, -20, -30, -40, -40, -50, -50, -40, -40, -30,
        -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40,
        -40, -50, -50, -40, -40, -30,
    ];
    const BLACK_KING_SQUARE_TABLE: [i64; 64] = [
        -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40,
        -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40,
        -40, -30, -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20,
        30, 10, 0, 0, 10, 30, 20,
    ];

    fn material_scores(&self) -> Score {
        let mut score = 0;
        for square in 0..64 {
            score += self.board.get_piece(square).score();
        }
        Score::Score(score)
    }

    fn square_table_scores(&self) -> Score {
        let mut score = 0;
        for square in 0..64 {
            let piece = self.board.get_piece(square);
            if piece != Piece::NONE {
                score += match piece {
                    Piece {
                        color: Color::White,
                        kind: PieceKind::Pawn,
                    } => Self::WHITE_PAWN_SQUARE_TABLE,
                    Piece {
                        color: Color::Black,
                        kind: PieceKind::Pawn,
                    } => Self::BLACK_PAWN_SQUARE_TABLE,
                    Piece {
                        color: Color::White,
                        kind: PieceKind::Rook,
                    } => Self::WHITE_ROOK_SQUARE_TABLE,
                    Piece {
                        color: Color::Black,
                        kind: PieceKind::Rook,
                    } => Self::BLACK_ROOK_SQUARE_TABLE,
                    Piece {
                        color: Color::White,
                        kind: PieceKind::Knight,
                    } => Self::KNIGHT_SQUARE_TABLE,
                    Piece {
                        color: Color::Black,
                        kind: PieceKind::Knight,
                    } => Self::KNIGHT_SQUARE_TABLE,
                    Piece {
                        color: Color::White,
                        kind: PieceKind::Bishop,
                    } => Self::WHITE_BISHOP_SQUARE_TABLE,
                    Piece {
                        color: Color::Black,
                        kind: PieceKind::Bishop,
                    } => Self::BLACK_BISHOP_SQUARE_TABLE,
                    Piece {
                        color: Color::White,
                        kind: PieceKind::Queen,
                    } => Self::WHITE_QUEEN_SQUARE_TABLE,
                    Piece {
                        color: Color::Black,
                        kind: PieceKind::Queen,
                    } => Self::BLACK_QUEEN_SQUARE_TABLE,
                    Piece {
                        color: Color::White,
                        kind: PieceKind::King,
                    } => Self::WHITE_KING_SQUARE_TABLE,
                    Piece {
                        color: Color::Black,
                        kind: PieceKind::King,
                    } => Self::BLACK_KING_SQUARE_TABLE,
                    _ => unreachable!(),
                }[square as usize]
                    * match piece.color {
                        Color::White => 1,
                        Color::Black => -1,
                        Color::None => unreachable!(),
                    };
            }
        }
        Score::Score(score)
    }

    fn checkmate_stalemate(&mut self) -> Score {
        let moves = self.board.generate_moves();
        let score = if moves.is_empty() {
            if moves.in_check {
                Score::OppMate(0)
            } else {
                Score::Draw(0)
            }
        } else {
            Score::Score(0)
        };
        match self.board.turn {
            Color::White => score,
            Color::Black => score.flip_score(),
            Color::None => unreachable!(),
        }
    }

    fn eval(&mut self) -> Score {
        let mut score = Score::Score(0);
        score += self.material_scores();
        score += self.square_table_scores();
        score += self.checkmate_stalemate();
        match self.board.turn {
            Color::White => score,
            Color::Black => score.flip_score(),
            Color::None => unreachable!(),
        }
    }

    fn mvv_lva(&mut self, a: Move, b: Move) -> Ordering {
        if a == self.best_move {
            Ordering::Less
        } else if b == self.best_move {
            Ordering::Greater
        } else if a.is_capture() && !b.is_capture() {
            Ordering::Less
        } else if !a.is_capture() && b.is_capture() {
            Ordering::Greater
        } else if a.is_capture() && b.is_capture() {
            if self.board.get_piece(a.to()).value() > self.board.get_piece(b.to()).value() {
                Ordering::Less
            } else if self.board.get_piece(a.to()).value() < self.board.get_piece(b.to()).value() {
                Ordering::Greater
            } else {
                self.board
                    .get_piece(a.from())
                    .value()
                    .cmp(&self.board.get_piece(b.from()).value())
            }
        } else {
            Ordering::Equal
        }
    }

    fn quiescence_search(&mut self, depth: u8, mut alpha: Score, beta: Score) -> Score {
        self.nodes += 1;
        if *self.stopper.read().unwrap() == Status::Stopping {
            return Score::Stop;
        }
        let mut best = self.eval();
        // Stand Pat
        if best >= beta {
            return best;
        }
        if best > alpha {
            alpha = best;
        }

        let mut moves = self.board.generate_moves().filter(|e| e.is_capture());
        moves.sort_by(|a, b| self.mvv_lva(*a, *b));

        for m in moves {
            self.board.make_move(m);

            let score = if depth == 0 {
                self.eval()
            } else {
                -self.quiescence_search(depth - 1, -beta, -alpha).inc()
            };

            self.board.unmake_move(m);
            if score > best {
                best = score;
                if score > alpha {
                    alpha = score;
                }
            }
            if score >= beta {
                break;
            }
        }

        best
    }

    fn negamax(&mut self, depth: u8, mut alpha: Score, beta: Score) -> Score {
        self.nodes += 1;
        if *self.stopper.read().unwrap() == Status::Stopping {
            return Score::Stop;
        }
        if let Some(tt_node) = self.tt.get(&self.board.zobrist_hash)
            && tt_node.depth >= depth
        {
            self.tt_hits += 1;
            match tt_node.kind {
                NodeKind::PvNode => return tt_node.score,
                NodeKind::CutNode => {
                    if tt_node.score >= beta {
                        return tt_node.score;
                    }
                }
                NodeKind::AllNode => {
                    if tt_node.score <= alpha {
                        return tt_node.score;
                    }
                }
            }
        }
        if depth == 0 {
            return self.quiescence_search(8, alpha, beta);
        }
        let (mut best_score, mut best_move) = (Score::OppMate(0), Move::NULL);
        let mut moves = self.board.generate_moves();
        let in_check = moves.in_check;
        moves.sort_by(|a, b| self.mvv_lva(*a, *b));
        for m in moves {
            self.board.make_move(m);
            let score = -self.negamax(depth - 1, -beta, -alpha).inc();
            self.board.unmake_move(m);
            if score >= beta {
                self.tt.insert(
                    self.board.zobrist_hash,
                    TTNode {
                        best_move,
                        depth,
                        score,
                        kind: NodeKind::CutNode,
                    },
                );
                return score;
            }
            if score > best_score {
                best_score = score;
                best_move = m;
                if depth == self.depth {
                    self.best_move = m;
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }

        if best_move == Move::NULL {
            if in_check {
                best_score = Score::OppMate(0);
            } else {
                best_score = Score::Draw(0);
            }
        }

        let node_kind = if best_score < alpha {
            NodeKind::AllNode
        } else {
            NodeKind::PvNode
        };
        self.tt.insert(
            self.board.zobrist_hash,
            TTNode {
                best_move,
                depth,
                score: best_score,
                kind: node_kind,
            },
        );
        best_score
    }
}
