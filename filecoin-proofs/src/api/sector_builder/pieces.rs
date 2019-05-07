use crate::api::sector_builder::metadata::PieceMetadata;
use sector_base::api::bytes_amount::UnpaddedBytesAmount;
use sector_base::api::bytes_amount::PaddedBytesAmount;
use std::cmp::max;

pub type PiecePadding = (UnpaddedBytesAmount, UnpaddedBytesAmount);

pub fn sum_piece_lengths<'a, T: Iterator<Item = &'a PieceMetadata>>(pieces: T) -> UnpaddedBytesAmount {
    pieces.fold(UnpaddedBytesAmount(0), |acc, p| {
        let (l, r) = get_piece_padding(acc, p.num_bytes);
        acc + l + p.num_bytes + r
    })
}

pub fn get_piece_by_key<'a>(
    pieces: &'a Vec<PieceMetadata>,
    piece_key: &str,
) -> Option<&'a PieceMetadata> {
    pieces.iter().find(|p| p.piece_key == piece_key)
}

pub fn get_piece_start(pieces: &Vec<PieceMetadata>, piece_key: &str) -> Option<UnpaddedBytesAmount> {
    if let Some(piece) = get_piece_by_key(pieces, piece_key) {
        let start_byte = sum_piece_lengths(pieces.iter().take_while(|p| p.piece_key != piece_key));
        let (left_padding, _) = get_piece_padding(start_byte, piece.num_bytes);

        Some(start_byte + left_padding)
    } else {
        None
    }
}

pub fn get_piece_padding(sector_length: UnpaddedBytesAmount, piece_length: UnpaddedBytesAmount) -> PiecePadding {
    let sector_length_on_disk = PaddedBytesAmount::from(sector_length);
    let piece_length_on_disk = PaddedBytesAmount::from(piece_length);

    let minimum_piece_length = 4 * 32;
    let adjusted_piece_length: u64 = max(minimum_piece_length, piece_length_on_disk.into());
    let piece_length_needed = (adjusted_piece_length - 1).next_power_of_two();
    let left_padding: u64 =
        (piece_length_needed - (u64::from(sector_length_on_disk) + piece_length_needed)) % piece_length_needed;
    let right_padding: u64 = piece_length_needed - u64::from(piece_length_on_disk);

    (UnpaddedBytesAmount::from(PaddedBytesAmount(left_padding)), UnpaddedBytesAmount::from(PaddedBytesAmount(right_padding)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaves(n: u64) -> u64 {
        n * 32
    }

    #[test]
    fn test_get_piece_info() {
        // minimally sized piece in clean sector
        assert_eq!(
            get_piece_padding(leaves(0), leaves(4)),
            (leaves(0), leaves(0))
        );

        // smaller than minimum piece in clean sector
        assert_eq!(
            get_piece_padding(leaves(0), leaves(3)),
            (leaves(0), leaves(1))
        );

        // slightly bigger piece in clean sector
        assert_eq!(
            get_piece_padding(leaves(0), leaves(5)),
            (leaves(0), leaves(3))
        );

        // minimal piece in populated sector
        assert_eq!(
            get_piece_padding(leaves(4), leaves(4)),
            (leaves(0), leaves(0))
        );

        // big piece in populated sector
        assert_eq!(
            get_piece_padding(leaves(4), leaves(5)),
            (leaves(4), leaves(3))
        );

        // bigger piece in populated sector
        assert_eq!(
            get_piece_padding(leaves(4), leaves(8)),
            (leaves(4), leaves(0))
        );

        // even bigger piece in populated sector
        assert_eq!(
            get_piece_padding(leaves(4), leaves(15)),
            (leaves(12), leaves(1))
        );

        // piece in misaligned sector
        assert_eq!(
            get_piece_padding(leaves(5), leaves(5)),
            (leaves(3), leaves(3))
        );
    }

    #[test]
    fn test_get_piece_start() {
        let mut pieces: Vec<PieceMetadata> = Default::default();

        pieces.push(PieceMetadata {
            piece_key: String::from("x"),
            num_bytes: UnpaddedBytesAmount(5),
            comm_p: None,
        });

        pieces.push(PieceMetadata {
            piece_key: String::from("y"),
            num_bytes: UnpaddedBytesAmount(300),
            comm_p: None,
        });

        pieces.push(PieceMetadata {
            piece_key: String::from("z"),
            num_bytes: UnpaddedBytesAmount(100),
            comm_p: None,
        });

        match get_piece_start(&pieces, "x") {
            Some(start) => assert_eq!(start, 0),
            None => panic!(),
        }

        match get_piece_start(&pieces, "y") {
            Some(start) => assert_eq!(start, 512),
            None => panic!(),
        }

        match get_piece_start(&pieces, "z") {
            Some(start) => assert_eq!(start, 1024),
            None => panic!(),
        }
    }
}
