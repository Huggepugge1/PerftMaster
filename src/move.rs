use vampirc_uci::{UciMove, UciPiece, UciSquare};

use crate::board::{Bitmap, Board, CastleKind, PieceKind, Square};

#[derive(Clone, Copy, Debug)]
pub struct Move(Square);

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
    (square.file as Square - 'a' as Square) - ((square.rank - 1) * 8) as Square
}

impl Move {
    pub fn new(from: Square, to: Square, flags: Square) -> Self {
        Move(from | (to << 6) | (flags << 12))
    }

    pub fn from(&self) -> Square {
        (self.0 & 0b111111) as Square
    }

    pub fn to(&self) -> Square {
        ((self.0 >> 6) & 0b111111) as Square
    }

    pub fn flags(&self) -> Square {
        (self.0 >> 12) as Square
    }

    pub fn is_capture(&self) -> bool {
        self.flags() & 0b0100 > 0
    }

    pub fn is_promotion(&self) -> bool {
        self.flags() & 0b1000 > 0
    }

    pub fn promotion(&self) -> PieceKind {
        let flags = self.flags();
        match flags & 0x1011 {
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

        let result = Move(from | (to << 6));

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
}
