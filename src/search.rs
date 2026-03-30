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

/// Check if the query ends with /i (case-insensitive flag).
/// Returns (pattern_str, case_insensitive).
fn parse_search_flags(query: &str) -> (&str, bool) {
    if let Some(stripped) = query.strip_suffix("/i") {
        (stripped, true)
    } else {
        (query, false)
    }
}

/// Find all occurrences of pattern in buffer, optionally case-insensitive.
fn find_all(app: &App, pattern: &[u8], case_insensitive: bool) -> Vec<usize> {
    let mut results = Vec::new();
    let len = app.buffer.len();
    if pattern.is_empty() || len == 0 {
        return results;
    }

    let mut pos = 0;
    while pos + pattern.len() <= len {
        let mut matched = true;
        for (j, &p) in pattern.iter().enumerate() {
            match app.buffer.get(pos + j) {
                Some(b) if case_insensitive => {
                    if !b.eq_ignore_ascii_case(&p) {
                        matched = false;
                        break;
                    }
                }
                Some(b) => {
                    if b != p {
                        matched = false;
                        break;
                    }
                }
                None => {
                    matched = false;
                    break;
                }
            }
        }
        if matched {
            results.push(pos);
            pos += 1;
        } else {
            pos += 1;
        }
    }
    results
}

pub fn execute_search(app: &mut App) {
    let query = app.search_input.clone();
    app.search_input.clear();

    let (pattern_str, case_insensitive) = parse_search_flags(&query);

    if let Some(pattern) = parse_search_pattern(pattern_str) {
        app.search_pattern_len = pattern.len();
        app.search_results = find_all(app, &pattern, case_insensitive);

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

/// Incremental search: update results without moving cursor or clearing input.
pub fn incremental_search(app: &mut App) {
    let query = &app.search_input;
    if query.is_empty() {
        app.search_results.clear();
        app.search_pattern_len = 0;
        return;
    }

    let (pattern_str, case_insensitive) = parse_search_flags(query);

    if let Some(pattern) = parse_search_pattern(pattern_str) {
        app.search_pattern_len = pattern.len();
        app.search_results = find_all(app, &pattern, case_insensitive);

        if app.search_results.is_empty() {
            app.search_index = 0;
        } else {
            // Point to nearest match at or after cursor
            app.search_index = app
                .search_results
                .iter()
                .position(|&r| r >= app.cursor)
                .unwrap_or(0);
        }
    } else {
        app.search_results.clear();
        app.search_pattern_len = 0;
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

/// Execute search-and-replace: replaces all occurrences of `find` with `replace`.
/// Both patterns are parsed via parse_search_pattern.
pub fn execute_replace(app: &mut App, find: &str, replace: &str) {
    let find_bytes = match parse_search_pattern(find) {
        Some(b) => b,
        None => {
            app.status_message = Some("Invalid find pattern".to_string());
            return;
        }
    };
    let replace_bytes = match parse_search_pattern(replace) {
        Some(b) => b,
        None => {
            app.status_message = Some("Invalid replace pattern".to_string());
            return;
        }
    };

    if find_bytes.len() != replace_bytes.len() {
        app.status_message = Some("Find and replace patterns must be same length (overwrite mode)".to_string());
        return;
    }

    let mut count = 0usize;
    let mut pos = 0;
    while let Some(found) = app.buffer.find(&find_bytes, pos) {
        for (i, &b) in replace_bytes.iter().enumerate() {
            app.buffer.set(found + i, b);
        }
        count += 1;
        pos = found + find_bytes.len();
    }

    if count == 0 {
        app.status_message = Some(format!("Pattern not found: {find}"));
    } else {
        app.search_results.clear();
        app.status_message = Some(format!("Replaced {count} occurrence{}", if count == 1 { "" } else { "s" }));
    }
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
        assert_eq!(app.search_pattern_len, 3);
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

    #[test]
    fn replace_ascii() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"AABBCC").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        execute_replace(&mut app, "BB", "XX");
        assert_eq!(app.buffer.get(2), Some(b'X'));
        assert_eq!(app.buffer.get(3), Some(b'X'));
        assert!(app.status_message.as_ref().unwrap().contains("Replaced 1"));
    }

    #[test]
    fn replace_multiple_occurrences() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"ABABAB").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        execute_replace(&mut app, "AB", "XY");
        assert_eq!(app.buffer.get(0), Some(b'X'));
        assert_eq!(app.buffer.get(1), Some(b'Y'));
        assert_eq!(app.buffer.get(4), Some(b'X'));
        assert!(app.status_message.as_ref().unwrap().contains("3"));
    }

    #[test]
    fn replace_different_lengths_rejected() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"AABB").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        execute_replace(&mut app, "AA", "X");
        assert!(app.status_message.as_ref().unwrap().contains("same length"));
    }

    #[test]
    fn replace_not_found() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"AABB").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        execute_replace(&mut app, "ZZ", "XX");
        assert!(app.status_message.as_ref().unwrap().contains("not found"));
    }

    #[test]
    fn replace_hex_patterns() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(&[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        execute_replace(&mut app, "0xDEAD", "0xCAFE");
        assert_eq!(app.buffer.get(0), Some(0xCA));
        assert_eq!(app.buffer.get(1), Some(0xFE));
    }

    #[test]
    fn case_insensitive_search() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"Hello HELLO hello").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        app.search_input = "hello/i".to_string();
        execute_search(&mut app);

        assert_eq!(app.search_results, vec![0, 6, 12]);
        assert_eq!(app.search_pattern_len, 5);
    }

    #[test]
    fn case_sensitive_search_default() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"Hello HELLO hello").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        app.search_input = "hello".to_string();
        execute_search(&mut app);

        assert_eq!(app.search_results, vec![12]);
    }

    #[test]
    fn incremental_search_updates_results() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"abcXYZabcXYZ").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        // Type "X" — should find 2 results
        app.search_input = "X".to_string();
        incremental_search(&mut app);
        assert_eq!(app.search_results, vec![3, 9]);
        assert_eq!(app.search_pattern_len, 1);

        // Type "XY" — should still find 2 results
        app.search_input = "XY".to_string();
        incremental_search(&mut app);
        assert_eq!(app.search_results, vec![3, 9]);
        assert_eq!(app.search_pattern_len, 2);

        // Type "XYZ" — still 2
        app.search_input = "XYZ".to_string();
        incremental_search(&mut app);
        assert_eq!(app.search_results, vec![3, 9]);
        assert_eq!(app.search_pattern_len, 3);

        // Clear — should clear results
        app.search_input.clear();
        incremental_search(&mut app);
        assert!(app.search_results.is_empty());
        assert_eq!(app.search_pattern_len, 0);
    }

    #[test]
    fn incremental_search_case_insensitive() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"Hello HELLO").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        app.search_input = "hello/i".to_string();
        incremental_search(&mut app);
        assert_eq!(app.search_results, vec![0, 6]);
    }

    #[test]
    fn parse_search_flags_strips_suffix() {
        let (pat, ci) = parse_search_flags("hello/i");
        assert_eq!(pat, "hello");
        assert!(ci);
    }

    #[test]
    fn parse_search_flags_no_flag() {
        let (pat, ci) = parse_search_flags("hello");
        assert_eq!(pat, "hello");
        assert!(!ci);
    }

    #[test]
    fn execute_search_hex_pattern() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(&[0xDE, 0xAD, 0x00, 0xDE, 0xAD]).unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        app.search_input = "0xDEAD".to_string();
        execute_search(&mut app);

        assert_eq!(app.search_results, vec![0, 3]);
        assert_eq!(app.search_pattern_len, 2);
    }

    #[test]
    fn execute_search_invalid_pattern() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"data").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        app.search_input = "0xGG".to_string();
        execute_search(&mut app);

        assert!(app.search_results.is_empty());
        assert!(app.status_message.as_ref().unwrap().contains("Invalid"));
    }

    #[test]
    fn execute_search_jumps_to_first_after_cursor() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"ABCABC").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();
        app.cursor = 2; // past first "A"

        app.search_input = "A".to_string();
        execute_search(&mut app);

        assert_eq!(app.search_results, vec![0, 3]);
        // Should jump to the match at 3, not 0
        assert_eq!(app.search_index, 1);
        assert_eq!(app.cursor, 3);
    }

    #[test]
    fn execute_search_wraps_to_first_when_cursor_past_all() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"AB..").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();
        app.cursor = 3; // past all matches

        app.search_input = "AB".to_string();
        execute_search(&mut app);

        assert_eq!(app.search_results, vec![0]);
        assert_eq!(app.search_index, 0);
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn next_prev_on_empty_results_is_noop() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"data").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        next_search_result(&mut app);
        assert_eq!(app.cursor, 0);

        prev_search_result(&mut app);
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn next_wraps_around() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"ABA").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        app.search_input = "A".to_string();
        execute_search(&mut app);
        assert_eq!(app.search_results, vec![0, 2]);

        next_search_result(&mut app);
        assert_eq!(app.cursor, 2);

        // Wrap around
        next_search_result(&mut app);
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn incremental_search_invalid_hex_clears() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"data").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        app.search_input = "0xGG".to_string();
        incremental_search(&mut app);
        assert!(app.search_results.is_empty());
        assert_eq!(app.search_pattern_len, 0);
    }

    #[test]
    fn incremental_search_nearest_match_after_cursor() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"AXBXCX").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();
        app.cursor = 3; // between second and third X

        app.search_input = "X".to_string();
        incremental_search(&mut app);
        assert_eq!(app.search_results, vec![1, 3, 5]);
        // Nearest at-or-after cursor=3 is index 1 (offset 3)
        assert_eq!(app.search_index, 1);
    }

    #[test]
    fn replace_invalid_find_pattern() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"data").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        execute_replace(&mut app, "0xGG", "0xFF");
        assert!(app.status_message.as_ref().unwrap().contains("Invalid find"));
    }

    #[test]
    fn replace_invalid_replace_pattern() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"data").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();

        execute_replace(&mut app, "0xFF", "0xGG");
        assert!(app.status_message.as_ref().unwrap().contains("Invalid replace"));
    }

    #[test]
    fn find_all_empty_buffer() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"").unwrap();
        let app = App::open(tmp.path().to_str().unwrap()).unwrap();

        let results = find_all(&app, b"x", false);
        assert!(results.is_empty());
    }

    #[test]
    fn find_all_empty_pattern() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"data").unwrap();
        let app = App::open(tmp.path().to_str().unwrap()).unwrap();

        let results = find_all(&app, b"", false);
        assert!(results.is_empty());
    }

    #[test]
    fn case_insensitive_find_all() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"aAbBaA").unwrap();
        let app = App::open(tmp.path().to_str().unwrap()).unwrap();

        let results = find_all(&app, b"aa", true);
        assert_eq!(results, vec![0, 4]);
    }
}
