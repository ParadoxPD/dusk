use std::ffi::OsString;
use std::fs;
use std::path::Path;

use crate::core::process;
use crate::core::style::Style;
use crate::core::theme;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Hex,
    Asm,
    Both,
}

pub fn run(args: &[OsString]) -> Result<(), String> {
    if args
        .iter()
        .any(|a| matches!(a.to_string_lossy().as_ref(), "--help" | "-?"))
    {
        print_help();
        return Ok(());
    }

    let mut want_hex = false;
    let mut want_asm = false;
    let mut mode_set = false;
    let mut theme_name: Option<String> = None;
    let mut files = Vec::new();

    let mut it = args.iter().peekable();
    while let Some(arg) = it.next() {
        let s = arg.to_string_lossy();
        match s.as_ref() {
            "--asm" => {
                want_asm = true;
                mode_set = true;
            }
            "--hex" => {
                want_hex = true;
                mode_set = true;
            }
            "--both" => {
                want_hex = true;
                want_asm = true;
                mode_set = true;
            }
            "--theme" => {
                let Some(name) = it.next() else {
                    return Err("--theme requires a theme name".to_string());
                };
                theme_name = Some(name.to_string_lossy().to_string());
            }
            _ if s.starts_with('-') => return Err(format!("unknown flag: {s}")),
            _ => files.push(s.to_string()),
        }
    }

    if files.is_empty() {
        return Err("dump requires at least one file".to_string());
    }

    if !mode_set {
        want_hex = true;
    }
    let mode = match (want_hex, want_asm) {
        (true, false) => Mode::Hex,
        (false, true) => Mode::Asm,
        (true, true) => Mode::Both,
        (false, false) => Mode::Hex,
    };

    let style = Style::for_stdout();
    let theme = theme::active(theme_name.as_deref());

    if matches!(mode, Mode::Asm | Mode::Both) {
        ensure_disassembler()?;
    }

    for (i, file) in files.iter().enumerate() {
        if i > 0 {
            println!();
        }

        let path = Path::new(file);
        if !path.exists() {
            return Err(format!("file not found: {}", path.display()));
        }

        println!(
            "{}",
            style.paint(theme.title, format!("== {} ==", path.display()))
        );

        if matches!(mode, Mode::Hex | Mode::Both) {
            render_hex(path, &style, theme)?;
        }
        if matches!(mode, Mode::Asm | Mode::Both) {
            render_asm(path, &style, theme)?;
        }
    }

    Ok(())
}

fn print_help() {
    let style = Style::for_stdout();
    let theme = theme::active(None);
    let cmd = |s: &str| style.paint(theme.title, s);
    let opt = |s: &str| style.paint(theme.accent, s);
    let arg = |s: &str| style.paint(theme.ok, s);
    let desc = |s: &str| style.paint(theme.info, s);

    println!("{}", cmd("dusk dump (hex + assembly dumper)"));
    println!();
    println!("{}", opt("USAGE"));
    println!(
        "  {} {} {}",
        opt("dusk"),
        cmd("dump"),
        arg("[OPTIONS] <file>...")
    );
    println!();
    println!("{}", opt("FLAGS"));
    println!("  {} {}", opt("--hex"), desc("Show hex dump"));
    println!(
        "  {} {}",
        opt("--asm"),
        desc("Show assembly dump (objdump/llvm-objdump)")
    );
    println!("  {} {}", opt("--both"), desc("Show both hex and assembly"));
    println!(
        "  {} {} {}",
        opt("--theme"),
        arg("<name>"),
        desc("Select color theme")
    );
    println!(
        "  {}, {} {}",
        opt("-?"),
        opt("--help"),
        desc("Show this help")
    );
}

fn render_hex(path: &Path, style: &Style, theme: theme::Theme) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| format!("failed reading {}: {e}", path.display()))?;
    println!("{}", style.paint(theme.accent, "-- HEX --"));

    for (row, chunk) in data.chunks(16).enumerate() {
        let offset = row * 16;
        let mut hex_cols = Vec::new();
        let mut ascii = String::new();

        for b in chunk {
            let h = format!("{:02x}", b);
            let hc = if *b == 0 {
                style.paint(theme.warn, h)
            } else if b.is_ascii_graphic() || *b == b' ' {
                style.paint(theme.ok, h)
            } else {
                style.paint(theme.info, h)
            };
            hex_cols.push(hc);

            ascii.push(if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            });
        }

        while hex_cols.len() < 16 {
            hex_cols.push("  ".to_string());
        }

        println!(
            "{}  {}  {}",
            style.paint(theme.number, format!("{:08x}", offset)),
            hex_cols.join(" "),
            style.paint(theme.subtle, ascii),
        );
    }

    Ok(())
}

fn render_asm(path: &Path, style: &Style, theme: theme::Theme) -> Result<(), String> {
    println!("{}", style.paint(theme.accent, "-- ASM --"));
    let dis = select_disassembler().ok_or_else(|| {
        "required system binary `objdump` or `llvm-objdump` is not available in PATH (needed for dusk dump --asm)".to_string()
    })?;

    let output = process::run_capture(dis, &["-d", "-M", "intel", path.to_string_lossy().as_ref()])
        .map_err(|e| format!("failed to run {dis}: {e}"))?;

    for line in output.lines() {
        println!("{}", color_asm_line(line, style, theme));
    }

    Ok(())
}

fn color_asm_line(line: &str, style: &Style, theme: theme::Theme) -> String {
    let t = line.trim();
    if t.is_empty() {
        return String::new();
    }
    if t.ends_with(':') && !t.contains('\t') {
        return style.paint(theme.title, line);
    }

    if let Some((addr, rest)) = line.split_once(':') {
        let mut parts = rest.trim_start().split_whitespace().collect::<Vec<_>>();
        if parts.is_empty() {
            return style.paint(theme.number, line);
        }

        let mut opcode_bytes: Vec<&str> = Vec::new();
        while !parts.is_empty() && is_opcode_byte(parts[0]) {
            opcode_bytes.push(parts.remove(0));
        }

        if parts.is_empty() {
            let addr_field = format!("{:>10}", addr.trim());
            let opcodes_field = format!("{:<24}", opcode_bytes.join(" "));
            return format!(
                "{}  {}",
                style.paint(theme.number, addr_field),
                style.paint(theme.info, opcodes_field)
            );
        }

        let mnemonic = parts[0];
        let operands = parts[1..].join(" ");
        let addr_field = format!("{:>10}", addr.trim());
        let opcodes_field = format!("{:<24}", opcode_bytes.join(" "));

        let m_color = if mnemonic.starts_with('j') || mnemonic == "call" {
            theme.warn
        } else {
            theme.ok
        };

        let mut op_buf = String::new();
        for tok in operands.split(',') {
            let tok = tok.trim();
            let colored = if tok.starts_with('r')
                || tok == "eax"
                || tok == "ebx"
                || tok == "ecx"
                || tok == "edx"
            {
                style.paint(theme.accent, tok)
            } else if tok.starts_with("0x") || tok.chars().all(|c| c.is_ascii_digit()) {
                style.paint(theme.number, tok)
            } else {
                style.paint(theme.info, tok)
            };
            if !op_buf.is_empty() {
                op_buf.push_str(&style.paint(theme.subtle, ", "));
            }
            op_buf.push_str(&colored);
        }

        return format!(
            "{}  {}  {}  {}",
            style.paint(theme.number, addr_field),
            style.paint(theme.subtle, opcodes_field),
            style.paint(m_color, format!("{:<8}", mnemonic)),
            op_buf
        );
    }

    style.paint(theme.info, line)
}

fn is_opcode_byte(token: &str) -> bool {
    token.len() == 2 && token.chars().all(|c| c.is_ascii_hexdigit())
}

fn ensure_disassembler() -> Result<(), String> {
    if select_disassembler().is_some() {
        Ok(())
    } else {
        Err(
            "required system binary `objdump` or `llvm-objdump` is not available in PATH (needed for dusk dump --asm)"
                .to_string(),
        )
    }
}

fn select_disassembler() -> Option<&'static str> {
    if process::command_exists("objdump") {
        Some("objdump")
    } else if process::command_exists("llvm-objdump") {
        Some("llvm-objdump")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::is_opcode_byte;

    #[test]
    fn opcode_byte_detection_only_matches_two_hex_digits() {
        assert!(is_opcode_byte("48"));
        assert!(is_opcode_byte("ff"));
        assert!(!is_opcode_byte("add"));
        assert!(!is_opcode_byte("0x48"));
        assert!(!is_opcode_byte("123"));
    }
}
