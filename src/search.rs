use crate::app::App;

/// Parse a search query — if it starts with "x/" or "0x", treat as hex bytes.
/// Otherwise treat as ASCII.
pub fn parse_search_pattern(query: &str) -> Option<Vec<u8>> {
    let query = query.trim();
    if query.is_empty() {
        return None;
    }

    if let Some(hex_str) = query.strip_prefix("x/").or_else(|| query.strip_prefix("0x")) {
        // Hex pattern: pairs of hex digits, spaces allowed
        let hex_str: String = hex_str.chars().filter(|c| !c.is_whitespace()).collect();
        if !hex_str.len().is_multiple_of(2) {
            return None;
        }
        let mut bytes = Vec::new();
        for chunk in hex_str.as_bytes().chunks(2) {
            let s = std::str::from_utf8(chunk).ok()?;
            bytes.push(u8::from_str_radix(s, 16).ok()?);
        }
        Some(bytes)
    } else {
        // ASCII pattern
        Some(query.as_bytes().to_vec())
    }
}

pub fn execute_search(app: &mut App) {
    let query = app.search_input.clone();
    app.search_input.clear();

    if let Some(pattern) = parse_search_pattern(&query) {
        app.search_results.clear();
        // Find all occurrences
        let mut pos = 0;
        while let Some(found) = app.buffer.find(&pattern, pos) {
            app.search_results.push(found);
            pos = found + 1;
        }

        if app.search_results.is_empty() {
            app.status_message = Some(format!("Pattern not found: {query}"));
        } else {
            // Jump to first result after cursor
            app.search_index = app
                .search_results
                .iter()
                .position(|&r| r >= app.cursor)
                .unwrap_or(0);
            let target = app.search_results[app.search_index];
            app.move_cursor_to(target);
            app.status_message = Some(format!(
                "Match {}/{} at 0x{:08X}",
                app.search_index + 1,
                app.search_results.len(),
                target
            ));
        }
    } else {
        app.status_message = Some("Invalid search pattern".to_string());
    }
}

pub fn next_search_result(app: &mut App) {
    if app.search_results.is_empty() {
        return;
    }
    app.search_index = (app.search_index + 1) % app.search_results.len();
    let target = app.search_results[app.search_index];
    app.move_cursor_to(target);
    app.status_message = Some(format!(
        "Match {}/{} at 0x{:08X}",
        app.search_index + 1,
        app.search_results.len(),
        target
    ));
}

pub fn prev_search_result(app: &mut App) {
    if app.search_results.is_empty() {
        return;
    }
    app.search_index = if app.search_index == 0 {
        app.search_results.len() - 1
    } else {
        app.search_index - 1
    };
    let target = app.search_results[app.search_index];
    app.move_cursor_to(target);
    app.status_message = Some(format!(
        "Match {}/{} at 0x{:08X}",
        app.search_index + 1,
        app.search_results.len(),
        target
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ascii_pattern() {
        assert_eq!(parse_search_pattern("hello"), Some(b"hello".to_vec()));
    }

    #[test]
    fn parse_hex_pattern_0x() {
        assert_eq!(parse_search_pattern("0xDEAD"), Some(vec![0xDE, 0xAD]));
    }

    #[test]
    fn parse_hex_pattern_x_slash() {
        assert_eq!(parse_search_pattern("x/CAFE"), Some(vec![0xCA, 0xFE]));
    }

    #[test]
    fn parse_hex_with_spaces() {
        assert_eq!(parse_search_pattern("0xDE AD BE EF"), Some(vec![0xDE, 0xAD, 0xBE, 0xEF]));
    }

    #[test]
    fn parse_hex_odd_length_returns_none() {
        assert_eq!(parse_search_pattern("0xDEA"), None);
    }

    #[test]
    fn parse_hex_invalid_chars_returns_none() {
        assert_eq!(parse_search_pattern("0xGGHH"), None);
    }

    #[test]
    fn parse_empty_returns_none() {
        assert_eq!(parse_search_pattern(""), None);
        assert_eq!(parse_search_pattern("  "), None);
    }

    #[test]
    fn execute_search_finds_results() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"abcXYZabcXYZ").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        app.search_input = "XYZ".to_string();
        execute_search(&mut app);

        assert_eq!(app.search_results, vec![3, 9]);
        assert_eq!(app.cursor, 3);
    }

    #[test]
    fn execute_search_no_match() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"abcdef").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        app.search_input = "zzz".to_string();
        execute_search(&mut app);

        assert!(app.search_results.is_empty());
        assert!(app.status_message.as_ref().unwrap().contains("not found"));
    }

    #[test]
    fn next_prev_search_cycle() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"AABAA").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        app.search_input = "A".to_string();
        execute_search(&mut app);
        assert_eq!(app.search_results, vec![0, 1, 3, 4]);

        next_search_result(&mut app);
        assert_eq!(app.search_index, 1);
        assert_eq!(app.cursor, 1);

        prev_search_result(&mut app);
        assert_eq!(app.search_index, 0);
        assert_eq!(app.cursor, 0);

        // Wrap backwards
        prev_search_result(&mut app);
        assert_eq!(app.search_index, 3);
        assert_eq!(app.cursor, 4);
    }
}
