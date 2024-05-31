//! useful functions

use crate::errors::{ErrorCode, TramexError};

/// Extract hexadecimal data from a vector of strings.
/// # Errors
/// Returns a TramexError if the hexe representation could not be extracted.
pub fn extract_hexe<T: AsRef<str>>(data: &[T]) -> Result<Vec<u8>, TramexError> {
    let iter = data.iter().filter(|one_string| {
        if let Some(first_char) = one_string.as_ref().trim().chars().next() {
            return first_char.is_numeric();
        }
        false
    });
    let mut data: Vec<String> = Vec::new();
    for one_string in iter {
        let trimed = one_string.as_ref().trim();
        if trimed.len() > 57 {
            let str_piece = &trimed[7..56];
            let chars_only: String = str_piece.chars().filter(|c| !c.is_whitespace()).collect();
            data.push(chars_only);
        } else {
            return Err(TramexError::new(
                format!("Error decoding hexe {:?} ({})", trimed, trimed.len()),
                ErrorCode::HexeDecodingError,
            ));
        }
    }
    let mut hexe: Vec<u8> = Vec::new();
    for one_string in data {
        let mut i = 0;
        while i < one_string.len() {
            let hex = &one_string[i..i + 2];
            if let Ok(hexa) = u8::from_str_radix(hex, 16) {
                hexe.push(hexa);
            }
            i += 2;
        }
    }
    Ok(hexe)
}
