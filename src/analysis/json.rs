/// Escape a string's contents for a JSON string literal.
pub fn escape_json_string(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\u{0008}' => escaped.push_str("\\b"),
            '\u{000c}' => escaped.push_str("\\f"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if (c as u32) < 0x20 => escaped.push_str(&format!("\\u{:04x}", c as u32)),
            c => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::escape_json_string;

    #[test]
    fn escapes_every_json_control_character() {
        let input: String = (0..=0x1f).map(char::from_u32).map(Option::unwrap).collect();
        let escaped = escape_json_string(&input);
        for code in 0..=0x1f {
            let encoded = match code {
                8 => "\\b".to_string(),
                9 => "\\t".to_string(),
                10 => "\\n".to_string(),
                12 => "\\f".to_string(),
                13 => "\\r".to_string(),
                _ => format!("\\u{code:04x}"),
            };
            assert!(
                escaped.contains(&encoded),
                "missing escape for U+{code:04X}"
            );
        }
        assert_eq!(escape_json_string("quote\" slash\\"), "quote\\\" slash\\\\");
    }
}
