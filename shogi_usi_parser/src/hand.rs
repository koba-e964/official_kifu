use core::slice;
use shogi_core::{Color, Hand, Piece};

use crate::{Error, FromUsi, Result};

/// ```
/// # use shogi_core::{Hand, PieceKind};
/// use shogi_usi_parser::FromUsi;
/// // An example found in [the original spec](https://web.archive.org/web/20080131070731/http://www.glaurungchess.com/shogi/usi.html).
/// let hand = <[Hand; 2]>::from_usi_lite("RG4P2b2s3p").unwrap();
/// assert_eq!(hand[0].count(PieceKind::Rook), Some(1)); // black
/// assert_eq!(hand[0].count(PieceKind::Gold), Some(1)); // black
/// assert_eq!(hand[0].count(PieceKind::Silver), Some(0)); // black
/// assert_eq!(hand[0].count(PieceKind::Pawn), Some(4)); // black
/// assert_eq!(hand[1].count(PieceKind::Bishop), Some(2)); // white
/// assert_eq!(hand[1].count(PieceKind::Silver), Some(2)); // white
/// assert_eq!(hand[1].count(PieceKind::Pawn), Some(3)); // white
///
/// // Theoretically one can have up to 18 pawns.
/// let hand = <[Hand; 2]>::from_usi_lite("18p").unwrap();
/// assert_eq!(hand[1].count(PieceKind::Pawn), Some(18)); // white
///
/// // Not something strongly encouraged, but the order of pieces are irrelevant:
/// let hand = <[Hand; 2]>::from_usi_lite("PNSP").unwrap();
/// assert_eq!(hand[0].count(PieceKind::Silver), Some(1)); // black
/// assert_eq!(hand[0].count(PieceKind::Knight), Some(1)); // black
/// assert_eq!(hand[0].count(PieceKind::Pawn), Some(2)); // black
///
/// let hand = <[Hand; 2]>::from_usi_lite("-").unwrap();
/// assert_eq!(hand[0].count(PieceKind::Silver), Some(0)); // black
/// ```
impl FromUsi for [Hand; 2] {
    fn parse_usi_slice(s: &[u8]) -> Result<(&[u8], Self)> {
        if s.is_empty() {
            return Err(Error::InvalidInput {
                from: 0,
                to: 0,
                description: "A `[Hand; 2]` expected, but nothing found",
            });
        }
        if s[0] == b'-' {
            // empty
            return Ok((&s[1..], [Hand::default(); 2]));
        }
        // If there are some pieces in hand, each letter must represent a valid unpromoted piece or the number of same pieces.
        // Although [the standard](https://web.archive.org/web/20080131070731/http://www.glaurungchess.com/shogi/usi.html) defines the strict order of pieces,
        // this parser allows a slightly wider set of inputs: order doesn't matter, same pieces can appear multiple times.
        let mut index = 0;
        let mut hand = [Hand::default(); 2];
        while index < s.len() {
            let mut count = 1;
            let mut count_len = 0;
            if matches!(s[index], b'0'..=b'9') {
                // length of the number should be 1 or 2
                let mut this = s[index] - b'0';
                if index + 1 < s.len() && matches!(s[index + 1], b'0'..=b'9') {
                    this = 10 * this + (s[index + 1] - b'0');
                    count_len = 2;
                } else {
                    count_len = 1;
                }
                count = this;
            }
            let result = Piece::parse_usi_slice(&s[index + count_len..index + count_len + 1]);
            let piece = if let Ok((_, piece)) = result {
                piece
            } else {
                break;
            };
            let piece_kind = piece.piece_kind();
            match piece.color() {
                Color::Black => {
                    for _ in 0..count {
                        hand[0] = if let Some(newhand) = hand[0].added(piece_kind) {
                            newhand
                        } else {
                            break;
                        }
                    }
                }
                Color::White => {
                    for _ in 0..count {
                        hand[1] = if let Some(newhand) = hand[1].added(piece_kind) {
                            newhand
                        } else {
                            break;
                        }
                    }
                }
            }
            index += count_len + 1;
        }
        if index == 0 {
            // Nothing was read. Since empty hand is represented as "-", this is irrational.
            return Err(Error::InvalidInput {
                from: 0,
                to: 1,
                description: "A `[Hand; 2]` expected, but no pieces were found",
            });
        }
        Ok((&s[index..], hand))
    }
}

/// C interface of `<[Hand; 2]>::parse_usi_slice`.
/// If parse error occurs, it returns -1.
/// If parsing succeeds, it returns the number of read bytes.
///
/// # Safety
/// `hand` must be a valid pointer to Hand[2].
/// `s` must be a nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn Hand_parse_usi_slice(hand: &mut [Hand; 2], s: *const u8) -> isize {
    let mut length = 0;
    while *s.add(length) != 0 {
        length += 1;
    }
    let slice = slice::from_raw_parts(s, length);
    match <[Hand; 2]>::parse_usi_slice(slice) {
        Ok((slice, resulting_hand)) => {
            *hand = resulting_hand;
            slice.as_ptr().offset_from(s)
        }
        Err(_) => -1,
    }
}
