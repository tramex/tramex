//! useful functions

/// Extract hexadecimal data from a vector of strings.
pub fn extract_hexe<T: AsRef<str>>(data: &[T]) -> Vec<u8> {
    let data: Vec<String> = data
        .iter()
        .filter(|one_string| {
            if let Some(first_char) = one_string.as_ref().trim().chars().next() {
                return first_char.is_numeric();
            }
            false
        })
        .map(|one_string| {
            if one_string.as_ref().len() > 57 {
                let str_piece = &one_string.as_ref().trim()[7..56];
                return str_piece.chars().filter(|c| !c.is_whitespace()).collect();
            }
            "".into()
        })
        .collect();
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
    hexe
}
