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

#[derive(PartialEq, Debug, Clone, Copy)]
enum NodeKind {
    Pv,
    Cut,
    All,
    Stopped,
}

#[derive(Debug, Clone)]
struct TTNode {
    best_move: Move,
    depth: u8,
    score: Score,
    kind: NodeKind,
}

#[derive(Debug)]
pub struct Search {
    pub pv: Move,
    depth: u8,
    board: Board,

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
    #[allow(clippy::enum_variant_names)]
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

impl std::ops::Add for Score {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Score::Stop, _) => Score::Stop,
            (_, Score::Stop) => Score::Stop,

            (Score::OwnMate(ply1), Score::OwnMate(ply2)) => Score::OwnMate(ply1.min(ply2)),
            (Score::OwnMate(ply1), Score::OppMate(ply2)) => {
                if ply1 < ply2 {
                    self
                } else {
                    rhs
                }
            }
            (Score::OwnMate(_), _) => self,

            (Score::OppMate(ply1), Score::OppMate(ply2)) => Score::OppMate(ply1.min(ply2)),
            (Score::OppMate(ply1), Score::OwnMate(ply2)) => {
                if ply1 < ply2 {
                    self
                } else {
                    rhs
                }
            }
            (Score::OppMate(ply1), Score::Draw(ply2)) => {
                if ply1 < ply2 {
                    self
                } else {
                    rhs
                }
            }
            (Score::OppMate(_), _) => self,

            (_, Score::OwnMate(_)) => rhs,
            (_, Score::OppMate(_)) => rhs,

            (Score::Score(score1), Score::Score(score2)) => Score::Score(score1 + score2),

            (Score::Score(_), Score::Draw(_)) => rhs,
            (Score::Draw(_), Score::Score(_)) => self,
            (Score::Draw(ply1), Score::Draw(ply2)) => Score::Draw(ply1.min(ply2)),
        }
    }
}

impl std::ops::AddAssign for Score {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub for Score {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Score::Stop, _) => Score::Stop,
            (_, Score::Stop) => Score::Stop,

            (Score::OwnMate(ply1), Score::OwnMate(ply2)) => Score::OwnMate(ply1.min(ply2)),
            (Score::OwnMate(ply1), Score::OppMate(ply2)) => {
                if ply1 < ply2 {
                    self
                } else {
                    rhs
                }
            }
            (Score::OwnMate(_), _) => self,

            (Score::OppMate(ply1), Score::OppMate(ply2)) => Score::OppMate(ply1.min(ply2)),
            (Score::OppMate(ply1), Score::OwnMate(ply2)) => {
                if ply1 < ply2 {
                    self
                } else {
                    rhs
                }
            }
            (Score::OppMate(ply1), Score::Draw(ply2)) => {
                if ply1 < ply2 {
                    self
                } else {
                    rhs
                }
            }
            (Score::OppMate(_), _) => self,

            (_, Score::OwnMate(_)) => rhs,
            (_, Score::OppMate(_)) => rhs,

            (Score::Score(score1), Score::Score(score2)) => Score::Score(score1 - score2),

            (Score::Score(_), Score::Draw(_)) => rhs,
            (Score::Draw(_), Score::Score(_)) => self,
            (Score::Draw(ply1), Score::Draw(ply2)) => Score::Draw(ply1.min(ply2)),
        }
    }
}

impl std::ops::Mul<i64> for Score {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        match self {
            Score::Score(score) => {
                if score * rhs > 10000 {
                    Score::OwnMate(0)
                } else if score * rhs < -10000 {
                    Score::OppMate(0)
                } else {
                    Score::Score(score * rhs)
                }
            }
            _ => self,
        }
    }
}

impl std::ops::MulAssign<i64> for Score {
    fn mul_assign(&mut self, rhs: i64) {
        *self = *self * rhs;
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
            Score::Draw(_) => "0".to_string(),
            Score::Stop => "?".to_string(),
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

impl Search {
    fn new(stopper: Arc<RwLock<Status>>, board: Board) -> Self {
        Self {
            pv: Move::default(),
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
        board: Board,
        search_control: Option<UciSearchControl>,
        time_control: Option<UciTimeControl>,
        stopper: Arc<RwLock<Status>>,
    ) -> Search {
        let (sender, receiver) = channel();
        let (mut alpha, mut beta) = (Score::OppMate(0), Score::OwnMate(0));
        let mut search_copy: Search = Self::new(stopper.clone(), board.clone());
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
            let mut window = (Score::Score(50), Score::Score(50));
            let mut score;
            let mut node_kind;
            loop {
                if depth != 1 {
                    alpha = search.score - window.0;
                    beta = search.score + window.1;
                }

                (score, node_kind) = search.negamax(search.depth, alpha, beta);
                eprintln!(
                    "{depth}: {}({}) <= {score} <= {}({}) {node_kind:?}",
                    alpha, window.0, beta, window.1
                );

                eprintln!(
                    "{depth}: {}({}) <= {} <= {}({}) {node_kind:?}",
                    alpha,
                    window.0,
                    search_copy
                        .negamax(search.depth, Score::OppMate(0), Score::OwnMate(0))
                        .0,
                    beta,
                    window.1
                );

                match node_kind {
                    NodeKind::Cut => window.1 *= 4,
                    NodeKind::All => window.0 *= 4,
                    NodeKind::Pv => break,
                    NodeKind::Stopped => break,
                }
            }
            search.score = score;
            println!(
                "info depth {} score cp {} nodes {} nps {} pv {}",
                search.depth,
                search.score.centipawns(),
                search.nodes,
                (search.nodes as f64 / search.start.elapsed().as_secs_f64()) as u64,
                search.pv,
            );
            depth += 1;
        }

        let _ = sender.send(());
        drop(search_copy);
        search
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
        if a.is_capture() && !b.is_capture() {
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

    fn quiescence_search(&mut self, mut alpha: Score, beta: Score) -> Score {
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

            let score = -self.quiescence_search(-beta, -alpha).inc();

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

    fn negamax(&mut self, depth: u8, mut alpha: Score, beta: Score) -> (Score, NodeKind) {
        self.nodes += 1;
        if *self.stopper.read().unwrap() == Status::Stopping {
            return (Score::Stop, NodeKind::Stopped);
        }
        let mut tt_best_move = None;
        if let Some(tt_node) = self.tt.get(&self.board.zobrist_hash) {
            if tt_node.depth >= depth {
                self.tt_hits += 1;
                match tt_node.kind {
                    NodeKind::Pv => return (tt_node.score, tt_node.kind),
                    NodeKind::Cut => {
                        if tt_node.score >= beta {
                            return (tt_node.score, tt_node.kind);
                        }
                    }
                    NodeKind::All => {
                        if tt_node.score <= alpha {
                            return (tt_node.score, tt_node.kind);
                        }
                    }
                    _ => unreachable!(),
                }
            } else if tt_node.kind == NodeKind::Pv || tt_node.kind == NodeKind::Cut {
                tt_best_move = Some(tt_node.best_move);
            }
        }
        if depth == 0 {
            return (self.quiescence_search(alpha, beta), NodeKind::Pv);
        }
        let (mut best_score, mut best_move) = (Score::OppMate(0), Move::NULL);
        let mut moves = self.board.generate_moves();
        let in_check = moves.in_check;
        moves.sort_by(|a, b| {
            if let Some(best_move) = tt_best_move {
                if *a == best_move {
                    return std::cmp::Ordering::Less;
                } else if *b == best_move {
                    return std::cmp::Ordering::Greater;
                }
            }
            self.mvv_lva(*a, *b)
        });
        for m in moves {
            self.board.make_move(m);
            let score = -self.negamax(depth - 1, -beta, -alpha).0.inc();
            self.board.unmake_move(m);
            if score >= beta {
                self.tt.insert(
                    self.board.zobrist_hash,
                    TTNode {
                        best_move,
                        depth,
                        score,
                        kind: NodeKind::Cut,
                    },
                );
                return (score, NodeKind::Cut);
            }
            if score > best_score {
                best_score = score;
                best_move = m;
                if score > alpha {
                    if depth == self.depth {
                        self.pv = m;
                    }
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
            NodeKind::All
        } else {
            NodeKind::Pv
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
        (best_score, node_kind)
    }
}
