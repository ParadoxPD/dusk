use std::cmp::Ordering;

use crate::core::style::Style;
use crate::core::theme;

use super::config::{Opts, SortMode};
use super::row::{EntryKind, Row};

pub(super) fn sort_rows(rows: &mut [Row], mode: SortMode, reverse: bool) {
    rows.sort_by(|a, b| sort_cmp(a, b, mode));
    if reverse {
        rows.reverse();
    }
}

fn sort_cmp(a: &Row, b: &Row, mode: SortMode) -> Ordering {
    match mode {
        SortMode::Name => a.sort_name.cmp(&b.sort_name),
        SortMode::Ext => a
            .sort_ext
            .cmp(&b.sort_ext)
            .then(a.sort_name.cmp(&b.sort_name)),
        SortMode::Type => a.kind.cmp(&b.kind).then(a.sort_name.cmp(&b.sort_name)),
        SortMode::Owner => a.owner.cmp(&b.owner).then(a.sort_name.cmp(&b.sort_name)),
        SortMode::Author => a.author.cmp(&b.author).then(a.sort_name.cmp(&b.sort_name)),
        SortMode::Size => a
            .size_bytes
            .cmp(&b.size_bytes)
            .then(a.sort_name.cmp(&b.sort_name)),
        SortMode::Time => a
            .mtime_epoch
            .cmp(&b.mtime_epoch)
            .then(a.sort_name.cmp(&b.sort_name)),
    }
}

pub(super) fn print_rows(rows: &[Row], opts: &Opts, style: &Style, theme: theme::Theme) {
    let (perm_w, owner_w, author_w, size_w, mod_w) =
        rows.iter()
            .fold((0usize, 0usize, 0usize, 0usize, 0usize), |acc, r| {
                (
                    acc.0.max(r.perms.len()),
                    acc.1.max(r.owner.len()),
                    acc.2.max(r.author.len()),
                    acc.3.max(r.size.len()),
                    acc.4.max(r.modified.len()),
                )
            });

    if opts.headers {
        if opts.long {
            let mut cols = vec![
                format!("{:perm_w$}", "PERMS", perm_w = perm_w),
                format!("{:owner_w$}", "OWNER", owner_w = owner_w),
            ];
            if opts.show_author {
                cols.push(format!("{:author_w$}", "AUTHOR", author_w = author_w));
            }
            cols.push(format!("{:>size_w$}", "SIZE", size_w = size_w));
            cols.push(format!("{:mod_w$}", "MODIFIED", mod_w = mod_w));
            cols.push("NAME".to_string());
            println!("{}", style.paint(theme.title, cols.join(" ")));
        } else {
            println!("{}", style.paint(theme.title, "NAME"));
        }
    }

    for row in rows {
        let body_color = match row.kind {
            EntryKind::Dir => theme.accent,
            EntryKind::Exec => theme.ok,
            EntryKind::Link => theme.warn,
            EntryKind::File => theme.info,
        };

        if !opts.long {
            println!("{}", style.paint(body_color, &row.display));
            continue;
        }

        let perms = format!("{:perm_w$}", row.perms, perm_w = perm_w);
        let owner = format!("{:owner_w$}", row.owner, owner_w = owner_w);
        let author = format!("{:author_w$}", row.author, author_w = author_w);
        let size = format!("{:>size_w$}", row.size, size_w = size_w);
        let modified = format!("{:mod_w$}", row.modified, mod_w = mod_w);

        let mut left_parts = vec![
            style.paint(theme.info, perms),
            style.paint(theme.subtle, owner),
        ];
        if opts.show_author {
            left_parts.push(style.paint(theme.subtle, author));
        }
        left_parts.push(style.paint(theme.accent, size));
        left_parts.push(style.paint(theme.number, modified));

        let left = left_parts.join(" ");
        println!("{left} {}", style.paint(body_color, &row.display));
    }
}
