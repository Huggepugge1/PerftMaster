use vampirc_uci::{UciFen, uci::UciMove};

use crate::r#move::Move;

pub type Bitmap = u64;
pub type Square = i16;

pub trait AsSquare {
    fn as_square(&self) -> String;
}

impl AsSquare for Square {
    fn as_square(&self) -> String {
        let mut string = String::from((b'a' + *self as u8 % 8) as char);
        string.push((b'1' + *self as u8 / 8) as char);
        string
    }
}

pub trait ToSquare {
    fn to_square(&self) -> Square;
}

impl ToSquare for String {
    fn to_square(&self) -> Square {
        (self.chars().nth(0).unwrap() as Square - 'a' as Square)
            + (self.chars().nth(1).unwrap() as Square - '1' as Square) * 8
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    #[default]
    None,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
    #[default]
    None,
}

impl std::ops::Add<Color> for Square {
    type Output = Self;

    fn add(self, rhs: Color) -> Self::Output {
        match rhs {
            Color::White => self + 8,
            Color::Black => self - 8,
            Color::None => unreachable!(),
        }
    }
}

impl std::ops::Sub<Color> for Square {
    type Output = Self;

    fn sub(self, rhs: Color) -> Self::Output {
        match rhs {
            Color::White => self - 8,
            Color::Black => self + 8,
            Color::None => unreachable!(),
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

impl Piece {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn value(self) -> i64 {
        match self.kind {
            PieceKind::Pawn => 100,
            PieceKind::Rook => 500,
            PieceKind::Knight => 320,
            PieceKind::Bishop => 330,
            PieceKind::Queen => 900,
            PieceKind::King => 100000,
            PieceKind::None => 0,
        }
    }

    pub fn score(self) -> i64 {
        self.value()
            * match self.color {
                Color::White => 1,
                Color::Black => -1,
                Color::None => 0,
            }
    }
}

pub enum CastleKind {
    KingSide,
    QueenSide,
    None,
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct IrreversibleAspects {
    capture: Piece,
    half_move_clock: u16,
    ep: Square,
    castling_rights: u8,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Board {
    pub white_pieces: Bitmap,
    pub black_pieces: Bitmap,

    pub pawns: Bitmap,
    pub rooks: Bitmap,
    pub knights: Bitmap,
    pub bishops: Bitmap,
    pub queens: Bitmap,
    pub kings: Bitmap,

    pub ep: Square,
    pub half_move_clock: u16,
    pub full_move_clock: u16,

    // White King | White Queen | Black King | Black Queen
    pub castling_rights: u8,
    pub turn: Color,

    pub game_stack: Vec<IrreversibleAspects>,

    pub zobrist_hash: u64,
    // Pieces (0..768) | Side to move (768..769) | Castling Rights (769..773) | En Passant (773..781)
    pub zobrist_values: [u64; 781],
}

impl Default for Board {
    fn default() -> Self {
        let mut zobrist_values = [0; 781];

        for i in zobrist_values.iter_mut() {
            *i = rand::random();
        }

        Self {
            white_pieces: Default::default(),
            black_pieces: Default::default(),
            pawns: Default::default(),
            rooks: Default::default(),
            knights: Default::default(),
            bishops: Default::default(),
            queens: Default::default(),
            kings: Default::default(),
            ep: Default::default(),
            half_move_clock: Default::default(),
            full_move_clock: Default::default(),
            castling_rights: Default::default(),
            turn: Default::default(),
            game_stack: Default::default(),
            zobrist_hash: Default::default(),
            zobrist_values,
        }
    }
}

impl Board {
    pub fn new() -> Board {
        Board::default()
    }

    pub fn new_game(&mut self) {}

    pub fn own_pieces(&self) -> Bitmap {
        match self.turn {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
            Color::None => unreachable!(),
        }
    }

    pub fn opponent_pieces(&self) -> Bitmap {
        match self.turn {
            Color::White => self.black_pieces,
            Color::Black => self.white_pieces,
            Color::None => unreachable!(),
        }
    }

    pub fn load_position(&mut self, fen: Option<UciFen>, moves: Vec<UciMove>) {
        self.clean_board();

        if fen.is_none() {
            self.load_startpos();
        } else {
            match fen {
                Some(UciFen(fen)) => self.load_fen(fen),
                None => unreachable!(),
            }
        }

        for ucimove in moves {
            let m = Move::from_ucimove(self, ucimove);
            self.make_move(m);
        }

        self.calculate_zobrist();
    }

    fn load_startpos(&mut self) {
        self.white_pieces = 0xffff;
        self.black_pieces = 0xffff << 48;

        self.pawns = 0xff00 | (0x00ff << 48);
        self.rooks = 0x0081 | (0x8100 << 48);
        self.knights = 0x0042 | (0x4200 << 48);
        self.bishops = 0x0024 | (0x2400 << 48);
        self.queens = 0x0008 | (0x0800 << 48);
        self.kings = 0x0010 | (0x1000 << 48);

        self.turn = Color::White;
        self.half_move_clock = 0;
        self.full_move_clock = 1;
    }

    fn load_fen(&mut self, fen: String) {
        let mut parts = fen.split(" ");
        let pieces = parts.next().unwrap();
        let turn = parts.next().unwrap();
        let castling = parts.next().unwrap();
        let en_passant = parts.next().unwrap();
        let halfmove_clock = parts.next().unwrap();
        let fullmove_clock = parts.next().unwrap();
        let mut pos: Square = 56;

        for piece in pieces.chars() {
            if piece == '/' {
                continue;
            } else if piece.is_ascii_digit() {
                pos += piece as Square - '0' as Square;
            } else {
                if piece.is_uppercase() {
                    self.white_pieces |= 1 << pos;
                    match piece {
                        'P' => self.pawns |= 1 << pos,
                        'R' => self.rooks |= 1 << pos,
                        'N' => self.knights |= 1 << pos,
                        'B' => self.bishops |= 1 << pos,
                        'Q' => self.queens |= 1 << pos,
                        'K' => self.kings |= 1 << pos,
                        _ => (),
                    }
                } else {
                    self.black_pieces |= 1 << pos;
                    match piece {
                        'p' => self.pawns |= 1 << pos,
                        'r' => self.rooks |= 1 << pos,
                        'n' => self.knights |= 1 << pos,
                        'b' => self.bishops |= 1 << pos,
                        'q' => self.queens |= 1 << pos,
                        'k' => self.kings |= 1 << pos,
                        _ => (),
                    }
                }
                pos += 1;
            }
            if pos > 8 && pos % 8 == 0 {
                pos -= 16;
            }
        }

        match turn {
            "w" => self.turn = Color::White,
            "b" => self.turn = Color::Black,
            _ => panic!("Fen needs a turn!"),
        }

        self.castling_rights = 0;
        for castling_right in castling.chars() {
            match castling_right {
                'K' => self.castling_rights |= 0b1000,
                'Q' => self.castling_rights |= 0b0100,
                'k' => self.castling_rights |= 0b0010,
                'q' => self.castling_rights |= 0b0001,
                _ => (),
            }
        }

        if en_passant != "-" {
            self.ep = en_passant.to_string().to_square();
        } else {
            self.ep = -1;
        }

        self.half_move_clock = halfmove_clock.parse().unwrap();
        self.full_move_clock = fullmove_clock.parse().unwrap();
    }

    fn calculate_zobrist(&mut self) {
        for square in 0..64 {
            let piece = self.get_piece(square);
            if piece.kind != PieceKind::None {
                self.zobrist_hash ^= self.zobrist_values
                    [square as usize + piece.kind as usize * 128 + piece.color as usize * 64];
            }
        }

        if self.turn == Color::Black {
            self.zobrist_hash ^= self.zobrist_values[768];
        }

        if self.castling_rights & 0b1000 > 0 {
            self.zobrist_hash ^= self.zobrist_values[769];
        }
        if self.castling_rights & 0b0100 > 0 {
            self.zobrist_hash ^= self.zobrist_values[770];
        }
        if self.castling_rights & 0b0010 > 0 {
            self.zobrist_hash ^= self.zobrist_values[771];
        }
        if self.castling_rights & 0b0001 > 0 {
            self.zobrist_hash ^= self.zobrist_values[772];
        }
        if self.ep != -1 {
            self.zobrist_hash ^= self.zobrist_values[773 + (self.ep % 8) as usize];
        }
    }

    pub fn annotate_move(&self, m: Move, promotion: PieceKind) -> Move {
        let mut flags = 0;
        let from = m.from();
        let to = m.to();
        let from_piece = self.get_piece(from);
        let to_piece = self.get_piece(to);

        match from_piece.kind {
            PieceKind::Pawn => {
                if Square::abs(to - from) == 16 {
                    flags = 0b0001;
                }
                if Square::abs(to - from) % 2 == 1 && to_piece.kind == PieceKind::None {
                    flags = 0b0101;
                }
            }
            PieceKind::King => {
                if to - from == 2 {
                    flags = 2;
                } else if to - from == -2 {
                    flags = 3;
                }
            }
            _ => (),
        }

        if to_piece.kind != PieceKind::None {
            flags |= 0b0100;
        }

        if promotion != PieceKind::None {
            match promotion {
                PieceKind::Rook => flags |= 0b1000,
                PieceKind::Knight => flags |= 0b1001,
                PieceKind::Bishop => flags |= 0b1010,
                PieceKind::Queen => flags |= 0b1011,
                _ => unreachable!(),
            }
        }
        m | (flags << 12)
    }

    pub fn make_move(&mut self, m: Move) {
        let mut irreversible_aspects = IrreversibleAspects {
            capture: Piece::new(),
            ep: self.ep,
            half_move_clock: self.half_move_clock,
            castling_rights: self.castling_rights,
        };

        let from = m.from();
        let to = m.to();

        let from_piece = self.get_piece(from);

        // Capture
        if m.is_capture() && !m.is_en_passant() {
            let to_piece = self.get_piece(to);
            irreversible_aspects.capture = to_piece;
            self.toggle_piece(to_piece, to);

            // If a rook is captured, remove castling_rights
            if to_piece.kind == PieceKind::Rook {
                self.modify_castling_rights_from_rook(to);
            }

        // En passant
        } else if m.is_en_passant() {
            let ep_piece = self.get_piece(self.ep - self.turn);
            irreversible_aspects.capture = ep_piece;
            self.toggle_piece(ep_piece, self.ep - self.turn);
        }
        if self.ep != -1 {
            self.zobrist_hash ^= self.zobrist_values[773 + (self.ep % 8) as usize];
            self.ep = -1;
        }
        if m.is_double_push() {
            self.ep = to - self.turn;
            self.zobrist_hash ^= self.zobrist_values[773 + (self.ep % 8) as usize];
        }

        // Promotion
        if m.is_promotion() {
            let promotion = m.promotion();
            self.toggle_promotion(promotion, to);
        }

        // Castling
        if m.is_castle() {
            self.toggle_castle(m, from);
            match self.turn {
                Color::White => {
                    if self.castling_rights & 0b1000 > 0 {
                        self.castling_rights &= 0b0111;
                        self.zobrist_hash ^= self.zobrist_values[769];
                    }
                    if self.castling_rights & 0b0100 > 0 {
                        self.castling_rights &= 0b1011;
                        self.zobrist_hash ^= self.zobrist_values[770];
                    }
                }
                Color::Black => {
                    if self.castling_rights & 0b0010 > 0 {
                        self.castling_rights &= 0b1101;
                        self.zobrist_hash ^= self.zobrist_values[771];
                    }
                    if self.castling_rights & 0b0001 > 0 {
                        self.castling_rights &= 0b1110;
                        self.zobrist_hash ^= self.zobrist_values[772];
                    }
                }
                Color::None => unreachable!(),
            }
        }

        // King moves
        if from_piece.kind == PieceKind::King {
            match self.turn {
                Color::White => {
                    if self.castling_rights & 0b1000 > 0 {
                        self.castling_rights &= 0b0111;
                        self.zobrist_hash ^= self.zobrist_values[769];
                    }
                    if self.castling_rights & 0b0100 > 0 {
                        self.castling_rights &= 0b1011;
                        self.zobrist_hash ^= self.zobrist_values[770];
                    }
                }
                Color::Black => {
                    if self.castling_rights & 0b0010 > 0 {
                        self.castling_rights &= 0b1101;
                        self.zobrist_hash ^= self.zobrist_values[771];
                    }
                    if self.castling_rights & 0b0001 > 0 {
                        self.castling_rights &= 0b1110;
                        self.zobrist_hash ^= self.zobrist_values[772];
                    }
                }
                Color::None => unreachable!(),
            }
        }

        // Rook moves
        if from_piece.kind == PieceKind::Rook {
            self.modify_castling_rights_from_rook(from);
        }

        if m.is_quiet() {
            self.half_move_clock += 1;
        } else {
            self.half_move_clock = 0;
        }

        self.move_piece(from_piece, m);
        self.change_turn();
        if self.turn == Color::White {
            self.full_move_clock += 1;
        }
        self.game_stack.push(irreversible_aspects);
    }

    pub fn unmake_move(&mut self, m: Move) {
        self.change_turn();
        let IrreversibleAspects {
            capture,
            ep,
            half_move_clock,
            castling_rights,
        } = self.game_stack.pop().unwrap();

        if ep != -1 {
            self.zobrist_hash ^= self.zobrist_values[773 + (ep % 8) as usize];
        }
        if self.ep != -1 {
            self.zobrist_hash ^= self.zobrist_values[773 + (self.ep % 8) as usize];
        }

        if self.castling_rights != castling_rights {
            let modified = self.castling_rights ^ castling_rights;
            if modified & 0b1000 > 0 {
                self.zobrist_hash ^= self.zobrist_values[769];
            }
            if modified & 0b0100 > 0 {
                self.zobrist_hash ^= self.zobrist_values[770];
            }
            if modified & 0b0010 > 0 {
                self.zobrist_hash ^= self.zobrist_values[771];
            }
            if modified & 0b0001 > 0 {
                self.zobrist_hash ^= self.zobrist_values[772];
            }
        }

        let m = m.reverse();

        let from = m.from();
        let to = m.to();

        // Promotion
        if m.is_promotion() {
            let promotion = m.promotion();
            self.toggle_promotion(promotion, from);
        }

        let from_piece = self.get_piece(from);

        // Capture
        if m.is_capture() && !m.is_en_passant() {
            self.toggle_piece(capture, from);

        // En passant
        } else if m.is_en_passant() {
            self.toggle_piece(capture, ep - self.turn);
        }

        // Castling
        if m.is_castle() {
            self.toggle_castle(m, to);
        }

        self.move_piece(from_piece, m);
        self.ep = ep;
        self.half_move_clock = half_move_clock;
        self.castling_rights = castling_rights;
        if self.turn == Color::Black {
            self.full_move_clock -= 1;
        }
    }

    fn move_piece(&mut self, piece: Piece, m: Move) {
        let bitmap = m.bitmap();
        match piece.color {
            Color::White => self.white_pieces ^= bitmap,
            Color::Black => self.black_pieces ^= bitmap,
            Color::None => unreachable!(),
        }

        match piece.kind {
            PieceKind::Pawn => self.pawns ^= bitmap,
            PieceKind::Rook => self.rooks ^= bitmap,
            PieceKind::Knight => self.knights ^= bitmap,
            PieceKind::Bishop => self.bishops ^= bitmap,
            PieceKind::Queen => self.queens ^= bitmap,
            PieceKind::King => self.kings ^= bitmap,
            PieceKind::None => unreachable!(),
        }

        self.zobrist_hash ^= self.zobrist_values
            [m.from() as usize + piece.kind as usize * 128 + piece.color as usize * 64];
        self.zobrist_hash ^= self.zobrist_values
            [m.to() as usize + piece.kind as usize * 128 + piece.color as usize * 64];
    }

    fn toggle_piece(&mut self, piece: Piece, square: Square) {
        let bitmap = 1 << square;
        match piece.color {
            Color::White => self.white_pieces ^= bitmap,
            Color::Black => self.black_pieces ^= bitmap,
            Color::None => unreachable!(),
        }

        match piece.kind {
            PieceKind::Pawn => self.pawns ^= bitmap,
            PieceKind::Rook => self.rooks ^= bitmap,
            PieceKind::Knight => self.knights ^= bitmap,
            PieceKind::Bishop => self.bishops ^= bitmap,
            PieceKind::Queen => self.queens ^= bitmap,
            PieceKind::King => self.kings ^= bitmap,
            PieceKind::None => unreachable!(),
        }

        self.zobrist_hash ^= self.zobrist_values
            [square as usize + piece.kind as usize * 128 + piece.color as usize * 64];
    }

    fn toggle_promotion(&mut self, piecekind: PieceKind, square: Square) {
        self.toggle_piece(
            Piece {
                color: self.turn,
                kind: PieceKind::Pawn,
            },
            square,
        );
        self.toggle_piece(
            Piece {
                color: self.turn,
                kind: piecekind,
            },
            square,
        );
    }

    fn toggle_castle(&mut self, m: Move, from: i16) {
        let castle = m.castle();
        match castle {
            CastleKind::KingSide => self.move_piece(
                Piece {
                    color: self.turn,
                    kind: PieceKind::Rook,
                },
                Move::new(from + 3, from + 1, 0),
            ),
            CastleKind::QueenSide => self.move_piece(
                Piece {
                    color: self.turn,
                    kind: PieceKind::Rook,
                },
                Move::new(from - 4, from - 1, 0),
            ),
            CastleKind::None => unreachable!(),
        }
    }

    fn modify_castling_rights_from_rook(&mut self, from: i16) {
        match from {
            0 => {
                if self.castling_rights & 0b0100 > 0 {
                    self.castling_rights &= 0b1011;
                    self.zobrist_hash ^= self.zobrist_values[770];
                }
            }
            7 => {
                if self.castling_rights & 0b1000 > 0 {
                    self.castling_rights &= 0b0111;
                    self.zobrist_hash ^= self.zobrist_values[769];
                }
            }
            56 => {
                if self.castling_rights & 0b0001 > 0 {
                    self.castling_rights &= 0b1110;
                    self.zobrist_hash ^= self.zobrist_values[772];
                }
            }
            63 => {
                if self.castling_rights & 0b0010 > 0 {
                    self.castling_rights &= 0b1101;
                    self.zobrist_hash ^= self.zobrist_values[771];
                }
            }
            _ => (),
        }
    }

    pub fn change_turn(&mut self) {
        if self.turn == Color::White {
            self.turn = Color::Black;
        } else {
            self.turn = Color::White;
        }
        self.zobrist_hash ^= self.zobrist_values[768];
    }

    pub fn get_piece(&self, square: Square) -> Piece {
        let mut piece = Piece::new();
        if (self.white_pieces & (1 << square)) > 0 {
            piece.color = Color::White;
        } else if (self.black_pieces & (1 << square)) > 0 {
            piece.color = Color::Black;
        }

        if (self.pawns & (1 << square)) > 0 {
            piece.kind = PieceKind::Pawn;
        } else if (self.rooks & (1 << square)) > 0 {
            piece.kind = PieceKind::Rook;
        } else if (self.knights & (1 << square)) > 0 {
            piece.kind = PieceKind::Knight;
        } else if (self.bishops & (1 << square)) > 0 {
            piece.kind = PieceKind::Bishop;
        } else if (self.queens & (1 << square)) > 0 {
            piece.kind = PieceKind::Queen;
        } else if (self.kings & (1 << square)) > 0 {
            piece.kind = PieceKind::King;
        }

        piece
    }

    pub fn print(&self) {
        println!(" --- --- --- --- --- --- --- ---");
        for i in 0..8 {
            print!("|");
            for j in 0..8 {
                print!(
                    " {} |",
                    piece_to_ascii(self.get_piece(63 - ((i * 8) + (7 - j))))
                );
            }
            println!();
            println!(" --- --- --- --- --- --- --- ---");
        }
    }

    fn clean_board(&mut self) {
        self.white_pieces = 0;
        self.black_pieces = 0;

        self.pawns = 0;
        self.rooks = 0;
        self.knights = 0;
        self.bishops = 0;
        self.queens = 0;
        self.kings = 0;

        self.ep = -1;
    }
}

pub fn piece_to_ascii(piece: Piece) -> char {
    let chr: char = match piece.kind {
        PieceKind::Pawn => 'p',
        PieceKind::Rook => 'r',
        PieceKind::Knight => 'n',
        PieceKind::Bishop => 'b',
        PieceKind::Queen => 'q',
        PieceKind::King => 'k',
        PieceKind::None => '_',
    };

    match piece.color {
        Color::White => chr.to_ascii_uppercase(),
        Color::Black => chr,
        Color::None => chr,
    }
}
