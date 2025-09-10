use vampirc_uci::{UciMove, UciPiece, UciSquare};

use crate::board::{AsSquare, Bitmap, Board, CastleKind, PieceKind, Square, ToSquare};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Move(pub Square);

impl Default for Move {
    fn default() -> Self {
        Move::NULL
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from = self.from();
        let to = self.to();

        let from_string = from.as_square();
        let to_string = to.as_square();

        let promote = match self.promotion() {
            PieceKind::Rook => "r",
            PieceKind::Knight => "n",
            PieceKind::Bishop => "b",
            PieceKind::Queen => "q",
            PieceKind::None => "",
            _ => unreachable!(),
        };

        write!(f, "{}{}{}", from_string, to_string, promote)
    }
}

impl std::ops::BitOr<Square> for Move {
    type Output = Self;

    fn bitor(self, rhs: Square) -> Self::Output {
        Move(self.0 | rhs)
    }
}

impl From<Move> for Square {
    fn from(value: Move) -> Self {
        value.0
    }
}

impl std::ops::Shl<i64> for Move {
    type Output = Self;

    fn shl(self, rhs: i64) -> Self::Output {
        Move(self.0 << rhs)
    }
}

impl std::ops::Shr<i64> for Move {
    type Output = Self;

    fn shr(self, rhs: i64) -> Self::Output {
        Move(self.0 >> rhs)
    }
}

fn ucisquare_to_square(square: UciSquare) -> Square {
    (square.file as Square - 'a' as Square) + ((square.rank - 1) * 8) as Square
}

impl Move {
    pub const NULL: Self = Move(-1);

    pub fn new(from: Square, to: Square, flags: Square) -> Self {
        Move(from | (to << 6) | (flags << 12))
    }

    pub fn add_promotion_if_possible(from: Square, to: Square, flags: Square) -> Vec<Self> {
        if to / 8 == 0 || to / 8 == 7 {
            vec![
                Move::new(from, to, flags | 0b1000),
                Move::new(from, to, flags | 0b1001),
                Move::new(from, to, flags | 0b1010),
                Move::new(from, to, flags | 0b1011),
            ]
        } else {
            vec![Move::new(from, to, flags)]
        }
    }

    pub fn from(&self) -> Square {
        (self.0 & 0b111111) as Square
    }

    pub fn to(&self) -> Square {
        ((self.0 >> 6) & 0b111111) as Square
    }

    // Flags:
    // 0b0000: Quiet Move
    // 0b0001: Double Pawn Push
    // 0b0010: King Castle
    // 0b0011: Queen Castle
    // 0b0100: Capture
    // 0b0101: En passant
    // 0b1000: Rook Promotion
    // 0b1001: Knight Promotion
    // 0b1010: Bishop Promotion
    // 0b1011: Queen Promotion
    // 0b1100: Rook Promotion and Capture
    // 0b1101: Knight Promotion and Capture
    // 0b1110: Bishop Promotion and Capture
    // 0b1111: Queen Promotion and Capture
    //
    // 0b0100: Capture
    // 0b1000: Promotion
    pub fn flags(&self) -> Square {
        (self.0 >> 12) as Square
    }

    pub fn is_capture(&self) -> bool {
        self.flags() & 0b0100 > 0
    }

    pub fn is_en_passant(&self) -> bool {
        self.flags() == 0b0101
    }

    pub fn is_double_push(&self) -> bool {
        self.flags() == 0b0001
    }

    pub fn is_promotion(&self) -> bool {
        self.flags() & 0b1000 > 0
    }

    pub fn is_quiet(&self) -> bool {
        self.flags() == 0b0000
    }

    pub fn promotion(&self) -> PieceKind {
        let flags = self.flags();
        match flags & 0b1011 {
            0b1000 => PieceKind::Rook,
            0b1001 => PieceKind::Knight,
            0b1010 => PieceKind::Bishop,
            0b1011 => PieceKind::Queen,
            _ => PieceKind::None,
        }
    }

    pub fn is_castle(&self) -> bool {
        let flags = self.flags();
        flags == 0b0010 || flags == 0b0011
    }

    pub fn castle(&self) -> CastleKind {
        let flags = self.flags();
        match flags {
            0b0010 => CastleKind::KingSide,
            0b0011 => CastleKind::QueenSide,
            _ => CastleKind::None,
        }
    }

    pub fn reverse(&self) -> Move {
        Move(self.to() | (self.from() << 6) | (self.flags() << 12))
    }

    pub fn bitmap(&self) -> Bitmap {
        (1 << self.from()) | (1 << self.to())
    }

    pub fn from_ucimove(board: &Board, m: UciMove) -> Self {
        let from = ucisquare_to_square(m.from);
        let to = ucisquare_to_square(m.to);

        let result = Move::new(from, to, 0);

        board.annotate_move(
            result,
            match m.promotion {
                Some(piece) => match piece {
                    UciPiece::Rook => PieceKind::Rook,
                    UciPiece::Knight => PieceKind::Knight,
                    UciPiece::Bishop => PieceKind::Bishop,
                    UciPiece::Queen => PieceKind::Queen,
                    _ => unreachable!(),
                },
                None => PieceKind::None,
            },
        )
    }

    pub fn as_ucimove(&self) -> UciMove {
        UciMove {
            from: UciSquare {
                file: ('a' as u8 + (self.from() % 8) as u8) as char,
                rank: (self.from() / 8) as u8 + 1,
            },
            to: UciSquare {
                file: ('a' as u8 + (self.to() % 8) as u8) as char,
                rank: (self.to() / 8) as u8 + 1,
            },
            promotion: match self.promotion() {
                PieceKind::Rook => Some(UciPiece::Rook),
                PieceKind::Knight => Some(UciPiece::Knight),
                PieceKind::Bishop => Some(UciPiece::Bishop),
                PieceKind::Queen => Some(UciPiece::Queen),
                PieceKind::None => None,
                _ => unreachable!(),
            },
        }
    }

    pub fn from_string_move(m: &str) -> Self {
        let from = m[0..2].to_string().to_square();
        let to = m[2..4].to_string().to_square();

        let mut flags = 0;

        if m.len() == 5 {
            match m.chars().nth(4).unwrap() {
                'r' => flags = 0b1000,
                'n' => flags = 0b1001,
                'b' => flags = 0b1010,
                'q' => flags = 0b1011,
                _ => unreachable!(),
            }
        }

        Move::new(from, to, flags)
    }
}
