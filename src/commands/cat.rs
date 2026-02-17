use std::ffi::OsString;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::core::devicons;
use crate::core::style::Style;
use crate::core::theme;

#[derive(Clone, Copy)]
enum Mode {
    Plain,
    Pretty,
}

struct Opts {
    mode: Mode,
    number: Option<bool>,
    number_nonblank: bool,
    squeeze_blank: bool,
    show_ends: bool,
    show_tabs: bool,
    theme: Option<String>,
    files: Vec<String>,
}

impl Opts {
    fn default(mode: Mode) -> Self {
        Self {
            mode,
            number: None,
            number_nonblank: false,
            squeeze_blank: false,
            show_ends: false,
            show_tabs: false,
            theme: None,
            files: Vec::new(),
        }
    }
}

pub fn run_cat(args: &[OsString]) -> Result<(), String> {
    run_with_mode(args, Mode::Plain)
}

pub fn run_bat(args: &[OsString]) -> Result<(), String> {
    run_with_mode(args, Mode::Pretty)
}

fn run_with_mode(args: &[OsString], default_mode: Mode) -> Result<(), String> {
    if args
        .iter()
        .any(|a| matches!(a.to_string_lossy().as_ref(), "-h" | "--help"))
    {
        print_help();
        return Ok(());
    }

    let opts = parse(args, default_mode)?;
    let mut style = Style::for_stdout();
    let theme = theme::active(opts.theme.as_deref());

    if matches!(opts.mode, Mode::Plain) {
        style.color = false;
    }

    let number = opts.number.unwrap_or(matches!(opts.mode, Mode::Pretty));

    if opts.files.is_empty() {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        output_stream(None, &mut reader, &opts, number, &style, theme)?;
        return Ok(());
    }

    for file in &opts.files {
        if file == "-" {
            let stdin = io::stdin();
            let mut reader = BufReader::new(stdin.lock());
            output_stream(
                Some(Path::new("-")),
                &mut reader,
                &opts,
                number,
                &style,
                theme,
            )?;
            continue;
        }

        let path = PathBuf::from(file);
        let file = fs::File::open(&path).map_err(|err| format!("{}: {err}", path.display()))?;
        let mut reader = BufReader::new(file);
        output_stream(Some(&path), &mut reader, &opts, number, &style, theme)?;
    }

    Ok(())
}

fn parse(args: &[OsString], default_mode: Mode) -> Result<Opts, String> {
    let mut opts = Opts::default(default_mode);
    let mut it = args.iter().peekable();

    while let Some(arg) = it.next() {
        let s = arg.to_string_lossy();

        if s == "--" {
            for rest in it {
                opts.files.push(rest.to_string_lossy().to_string());
            }
            break;
        }

        if s == "--pretty" {
            opts.mode = Mode::Pretty;
            continue;
        }
        if s == "--plain" {
            opts.mode = Mode::Plain;
            continue;
        }
        if s == "--no-number" {
            opts.number = Some(false);
            continue;
        }
        if s == "--theme" {
            let Some(name) = it.next() else {
                return Err("--theme requires a theme name".to_string());
            };
            opts.theme = Some(name.to_string_lossy().to_string());
            continue;
        }

        if s.starts_with('-') && s.len() > 1 {
            for ch in s[1..].chars() {
                match ch {
                    'n' => opts.number = Some(true),
                    'b' => {
                        opts.number = Some(true);
                        opts.number_nonblank = true;
                    }
                    's' => opts.squeeze_blank = true,
                    'E' => opts.show_ends = true,
                    'T' => opts.show_tabs = true,
                    'p' => opts.mode = Mode::Plain,
                    _ => return Err(format!("unknown flag: -{ch}")),
                }
            }
        } else {
            opts.files.push(s.to_string());
        }
    }

    Ok(opts)
}

fn print_help() {
    let style = Style::for_stdout();
    let theme = theme::active(None);
    let cmd = |s: &str| style.paint(theme.title, s);
    let opt = |s: &str| style.paint(theme.accent, s);
    let arg = |s: &str| style.paint(theme.ok, s);
    let desc = |s: &str| style.paint(theme.info, s);
    println!(
        "{}",
        cmd("dusk cat (cat-compatible, with optional bat-style pretty mode)")
    );
    println!();
    println!("{}", opt("USAGE"));
    println!(
        "  {} {} {}",
        opt("dusk"),
        cmd("cat"),
        arg("[OPTIONS] [FILE]...")
    );
    println!(
        "  {} {} {}",
        opt("dusk"),
        cmd("bat"),
        arg("[OPTIONS] [FILE]...")
    );
    println!();
    println!("{}", opt("CAT-COMPAT FLAGS"));
    println!("  {} {}", opt("-n"), desc("Number all output lines"));
    println!("  {} {}", opt("-b"), desc("Number nonblank output lines"));
    println!("  {} {}", opt("-s"), desc("Squeeze multiple blank lines"));
    println!("  {} {}", opt("-E"), desc("Display $ at end of each line"));
    println!("  {} {}", opt("-T"), desc("Display TAB as ^I"));
    println!();
    println!("{}", opt("PRETTY FLAGS"));
    println!(
        "  {} {}",
        opt("--pretty"),
        desc("Enable bat-like pretty output")
    );
    println!(
        "  {}, {} {}",
        opt("--plain"),
        opt("-p"),
        desc("Force plain cat-like output")
    );
    println!(
        "  {} {}",
        opt("--no-number"),
        desc("Disable line numbers in pretty mode")
    );
    println!(
        "  {} {} {}",
        opt("--theme"),
        arg("<name>"),
        desc("Theme for pretty mode")
    );
    println!(
        "  {}, {} {}",
        opt("-h"),
        opt("--help"),
        desc("Show this help")
    );
}

fn output_stream<R: BufRead>(
    path: Option<&Path>,
    reader: &mut R,
    opts: &Opts,
    number: bool,
    style: &Style,
    theme: theme::Theme,
) -> Result<(), String> {
    if matches!(opts.mode, Mode::Plain)
        && !number
        && !opts.number_nonblank
        && !opts.squeeze_blank
        && !opts.show_ends
        && !opts.show_tabs
    {
        let mut out = io::stdout().lock();
        let mut raw = Vec::new();
        reader
            .read_to_end(&mut raw)
            .map_err(|err| format!("read error: {err}"))?;
        out.write_all(&raw)
            .map_err(|err| format!("write error: {err}"))?;
        return Ok(());
    }

    if matches!(opts.mode, Mode::Pretty) {
        let label = path
            .map(|p| {
                if p == Path::new("-") {
                    "stdin".to_string()
                } else {
                    p.display().to_string()
                }
            })
            .unwrap_or_else(|| "stdin".to_string());
        let icon = path
            .filter(|p| p.to_str() != Some("-"))
            .map(devicons::file_icon)
            .unwrap_or("");
        let gap = if style.maybe_icon(icon).is_empty() {
            ""
        } else {
            " "
        };
        println!(
            "{}",
            style.paint(
                theme.title,
                format!("-- {}{}{} --", style.maybe_icon(icon), gap, label)
            )
        );
    }

    let mut idx = 0usize;
    let mut prev_blank = false;
    let mut line = String::new();
    loop {
        line.clear();
        let read = reader
            .read_line(&mut line)
            .map_err(|err| format!("read error: {err}"))?;
        if read == 0 {
            break;
        }

        let had_nl = line.ends_with('\n');
        if had_nl {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }

        let is_blank = line.is_empty();
        if opts.squeeze_blank && is_blank && prev_blank {
            continue;
        }
        prev_blank = is_blank;

        let mut rendered = if opts.show_tabs {
            line.replace('\t', "^I")
        } else {
            line.clone()
        };

        if opts.show_ends {
            rendered.push('$');
        }

        if matches!(opts.mode, Mode::Pretty) {
            rendered = stylize_line(&rendered, path, style, theme);
        }

        if number && (!opts.number_nonblank || !is_blank) {
            idx += 1;
            print!("{} ", style.paint(theme.number, format!("{:>5} |", idx)));
        }

        if had_nl {
            println!("{rendered}");
        } else {
            print!("{rendered}");
        }
    }

    Ok(())
}

fn stylize_line(line: &str, path: Option<&Path>, style: &Style, theme: theme::Theme) -> String {
    let ext = path
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let trimmed = line.trim_start();

    if (trimmed.starts_with("# ") || trimmed.starts_with("##")) && ext == "md" {
        return style.paint(theme.accent, line);
    }

    if trimmed.starts_with("//")
        || trimmed.starts_with('#')
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*')
    {
        return style.paint(theme.ok, line);
    }

    if line.contains("TODO") || line.contains("FIXME") {
        return style.paint(theme.warn, line);
    }

    if looks_like_decl(trimmed) {
        return style.paint(theme.info, line);
    }

    style.paint(theme.subtle, line)
}

fn looks_like_decl(line: &str) -> bool {
    let decls = [
        "fn ",
        "pub fn ",
        "struct ",
        "enum ",
        "impl ",
        "class ",
        "interface ",
        "type ",
    ];
    decls.iter().any(|d| line.starts_with(d))
}
