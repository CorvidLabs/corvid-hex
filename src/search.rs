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
        if hex_str.len() % 2 != 0 {
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
