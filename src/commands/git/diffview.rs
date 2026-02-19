use std::path::Path;

use crate::core::style::Style;
use crate::core::theme;

#[derive(Clone, Copy)]
enum SideKind {
    Context,
    Added,
    Removed,
}

pub(crate) fn render_side_by_side(
    output: &str,
    style: &Style,
    theme: theme::Theme,
    width: usize,
) -> Vec<String> {
    let mut rows = Vec::new();
    let width = width.max(80);
    let num_w = 5usize;
    let mid = " │ ";
    let cell_w = width.saturating_sub(num_w * 2 + mid.len() + 4) / 2;
    let cell_w = cell_w.max(16);

    let lines = output.lines().collect::<Vec<_>>();
    let mut i = 0usize;
    let mut old_ln = 1usize;
    let mut new_ln = 1usize;
    let mut ext: Option<String> = None;

    while i < lines.len() {
        let line = lines[i];

        if line.starts_with("diff --git ") {
            ext = parse_ext_from_diff_header(line);
            rows.push(style.paint(theme.accent, pad_plain(line, width)));
            i += 1;
            continue;
        }

        if line.starts_with("@@ ") {
            (old_ln, new_ln) = parse_hunk_header(line);
            rows.push(style.paint(theme.number, pad_plain(line, width)));
            i += 1;
            continue;
        }

        if is_meta(line) {
            rows.push(style.paint(theme.subtle, pad_plain(line, width)));
            i += 1;
            continue;
        }

        if line.starts_with('-') && !line.starts_with("---") && i + 1 < lines.len() {
            let next = lines[i + 1];
            if next.starts_with('+') && !next.starts_with("+++") {
                rows.push(format_diff_row(
                    style,
                    theme,
                    Some(old_ln),
                    Some(new_ln),
                    &line[1..],
                    &next[1..],
                    SideKind::Removed,
                    SideKind::Added,
                    cell_w,
                    ext.as_deref(),
                ));
                old_ln += 1;
                new_ln += 1;
                i += 2;
                continue;
            }
        }

        if line.starts_with('-') && !line.starts_with("---") {
            rows.push(format_diff_row(
                style,
                theme,
                Some(old_ln),
                None,
                &line[1..],
                "",
                SideKind::Removed,
                SideKind::Context,
                cell_w,
                ext.as_deref(),
            ));
            old_ln += 1;
            i += 1;
            continue;
        }

        if line.starts_with('+') && !line.starts_with("+++") {
            rows.push(format_diff_row(
                style,
                theme,
                None,
                Some(new_ln),
                "",
                &line[1..],
                SideKind::Context,
                SideKind::Added,
                cell_w,
                ext.as_deref(),
            ));
            new_ln += 1;
            i += 1;
            continue;
        }

        if let Some(ctx) = line.strip_prefix(' ') {
            rows.push(format_diff_row(
                style,
                theme,
                Some(old_ln),
                Some(new_ln),
                ctx,
                ctx,
                SideKind::Context,
                SideKind::Context,
                cell_w,
                ext.as_deref(),
            ));
            old_ln += 1;
            new_ln += 1;
            i += 1;
            continue;
        }

        rows.push(style.paint(theme.info, pad_plain(line, width)));
        i += 1;
    }

    rows
}

fn format_diff_row(
    style: &Style,
    theme: theme::Theme,
    old_ln: Option<usize>,
    new_ln: Option<usize>,
    left: &str,
    right: &str,
    left_kind: SideKind,
    right_kind: SideKind,
    width: usize,
    ext: Option<&str>,
) -> String {
    let old_no = old_ln
        .map(|n| format!("{n:>5}"))
        .unwrap_or_else(|| "     ".to_string());
    let new_no = new_ln
        .map(|n| format!("{n:>5}"))
        .unwrap_or_else(|| "     ".to_string());

    let left_plain = pad_plain(left, width);
    let right_plain = pad_plain(right, width);

    let left_col = kind_color(left_kind, theme);
    let right_col = kind_color(right_kind, theme);

    let left_cell = highlight_line(&left_plain, ext, left_col, style, theme);
    let right_cell = highlight_line(&right_plain, ext, right_col, style, theme);

    format!(
        "{} {} {} {} {}",
        style.paint(theme.number, old_no),
        left_cell,
        style.paint(theme.accent, "│"),
        style.paint(theme.number, new_no),
        right_cell,
    )
}

fn highlight_line(
    line: &str,
    ext: Option<&str>,
    base_color: &'static str,
    style: &Style,
    theme: theme::Theme,
) -> String {
    let Some(ext) = ext else {
        return style.paint(base_color, line);
    };

    let ext = ext.to_ascii_lowercase();
    let comment_marker = comment_marker(&ext);
    let mut out = String::new();

    let (code_part, comment_part) = if let Some(marker) = comment_marker {
        if let Some(pos) = line.find(marker) {
            (&line[..pos], Some(&line[pos..]))
        } else {
            (line, None)
        }
    } else {
        (line, None)
    };

    let chars = code_part.chars().collect::<Vec<_>>();
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];

        if ch == '"' || ch == '\'' {
            let quote = ch;
            let start = i;
            i += 1;
            while i < chars.len() {
                if chars[i] == quote && chars[i.saturating_sub(1)] != '\\' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            let token = chars[start..i].iter().collect::<String>();
            out.push_str(&style.paint(theme.accent, token));
            continue;
        }

        if ch.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_hexdigit()
                    || matches!(chars[i], '.' | '_' | 'x' | 'X' | 'o' | 'O' | 'b' | 'B'))
            {
                i += 1;
            }
            let token = chars[start..i].iter().collect::<String>();
            out.push_str(&style.paint(theme.number, token));
            continue;
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let token = chars[start..i].iter().collect::<String>();
            if is_keyword(&ext, &token) {
                out.push_str(&style.paint(theme.title, token));
            } else {
                out.push_str(&style.paint(base_color, token));
            }
            continue;
        }

        out.push_str(&style.paint(base_color, ch.to_string()));
        i += 1;
    }

    if let Some(comment) = comment_part {
        out.push_str(&style.paint(theme.subtle, comment));
    }

    out
}

fn comment_marker(ext: &str) -> Option<&'static str> {
    if matches!(
        ext,
        "rs" | "c"
            | "h"
            | "cpp"
            | "hpp"
            | "cc"
            | "java"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "go"
            | "kt"
            | "swift"
            | "php"
            | "scala"
            | "dart"
    ) {
        Some("//")
    } else if matches!(
        ext,
        "py" | "rb" | "sh" | "bash" | "zsh" | "yml" | "yaml" | "toml" | "ini" | "conf"
    ) {
        Some("#")
    } else if ext == "sql" {
        Some("--")
    } else {
        None
    }
}

fn is_keyword(ext: &str, token: &str) -> bool {
    const COMMON: &[&str] = &[
        "if", "else", "for", "while", "match", "return", "break", "continue", "true", "false",
        "null", "nil", "None", "let", "const", "var", "fn", "function", "class", "struct", "enum",
        "impl", "trait", "pub", "use", "mod", "import", "from", "export", "async", "await", "try",
        "catch", "throw", "switch", "case", "default", "new", "this", "super", "self", "mut",
    ];

    if COMMON.contains(&token) {
        return true;
    }

    match ext {
        "rs" => matches!(token, "Result" | "Option" | "Some" | "None" | "Ok" | "Err"),
        "py" => matches!(token, "def" | "lambda" | "pass" | "yield" | "with" | "as"),
        "go" => matches!(
            token,
            "package" | "func" | "defer" | "go" | "chan" | "select"
        ),
        "js" | "jsx" | "ts" | "tsx" => {
            matches!(token, "typeof" | "instanceof" | "interface" | "type")
        }
        _ => false,
    }
}

fn kind_color(kind: SideKind, theme: theme::Theme) -> &'static str {
    match kind {
        SideKind::Context => theme.info,
        SideKind::Added => theme.ok,
        SideKind::Removed => theme.warn,
    }
}

fn is_meta(line: &str) -> bool {
    line.starts_with("index ")
        || line.starts_with("--- ")
        || line.starts_with("+++ ")
        || line.starts_with("new file mode")
        || line.starts_with("deleted file mode")
        || line.starts_with("similarity index")
        || line.starts_with("rename from ")
        || line.starts_with("rename to ")
        || line.starts_with("commit ")
        || line.starts_with("Author:")
        || line.starts_with("Date:")
        || line.starts_with("\\ No newline at end of file")
}

fn parse_hunk_header(line: &str) -> (usize, usize) {
    let mut old_ln = 1usize;
    let mut new_ln = 1usize;
    let parts = line.split_whitespace().collect::<Vec<_>>();

    if parts.len() >= 3 {
        if let Some(old) = parts[1].strip_prefix('-') {
            old_ln = old
                .split(',')
                .next()
                .and_then(|n| n.parse::<usize>().ok())
                .unwrap_or(1);
        }
        if let Some(new) = parts[2].strip_prefix('+') {
            new_ln = new
                .split(',')
                .next()
                .and_then(|n| n.parse::<usize>().ok())
                .unwrap_or(1);
        }
    }

    (old_ln, new_ln)
}

fn parse_ext_from_diff_header(line: &str) -> Option<String> {
    let Some(idx) = line.rfind(" b/") else {
        return None;
    };
    let path = &line[idx + 3..];
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
}

fn pad_plain(s: &str, width: usize) -> String {
    let t = truncate_plain(s, width);
    if t.chars().count() >= width {
        t
    } else {
        format!("{t:<width$}")
    }
}

fn truncate_plain(s: &str, width: usize) -> String {
    if width <= 1 {
        return String::new();
    }
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i + 1 >= width {
            out.push('…');
            return out;
        }
        out.push(ch);
    }
    out
}
