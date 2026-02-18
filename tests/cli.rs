use std::fs;
use std::process::Command;

use predicates::prelude::*;
use tempfile::tempdir;

fn dusk() -> assert_cmd::Command {
    assert_cmd::Command::new(env!("CARGO_BIN_EXE_dusk"))
}

fn command_available(bin: &str) -> bool {
    Command::new(bin).arg("--version").output().is_ok()
}

fn has_disassembler() -> bool {
    command_available("objdump") || command_available("llvm-objdump")
}

#[test]
fn help_shows_native_commands() {
    dusk()
        .arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Native commands"));
}

#[test]
fn themes_lists_onedark_pro() {
    dusk()
        .args(["themes", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("onedark-pro"));
}

#[test]
fn ls_help_has_basic_flag() {
    dusk()
        .args(["ls", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--basic"));
}

#[test]
fn ls_h_is_human_not_help() {
    let td = tempdir().expect("tmpdir");
    fs::write(td.path().join("a.txt"), "x").expect("write");

    dusk()
        .args(["ls", "-lh", td.path().to_string_lossy().as_ref()])
        .assert()
        .success()
        .stdout(predicate::str::contains("--help").not());
}

#[test]
fn ls_all_shows_implied_dot_entries_and_almost_all_hides_them() {
    let td = tempdir().expect("tmpdir");
    fs::write(td.path().join(".hidden"), "x").expect("write hidden");
    fs::write(td.path().join("visible"), "x").expect("write visible");

    dusk()
        .args(["ls", "-a", "--basic", td.path().to_string_lossy().as_ref()])
        .assert()
        .success()
        .stdout(predicate::str::contains("./"))
        .stdout(predicate::str::contains("../"));

    dusk()
        .args(["ls", "-A", "--basic", td.path().to_string_lossy().as_ref()])
        .assert()
        .success()
        .stdout(predicate::str::contains("./").not())
        .stdout(predicate::str::contains("../").not());
}

#[test]
fn ls_headers_and_author_columns_are_printed() {
    let td = tempdir().expect("tmpdir");
    fs::write(td.path().join("file.txt"), "hello").expect("write file");

    dusk()
        .args([
            "ls",
            "-lH",
            "--author",
            td.path().to_string_lossy().as_ref(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("PERMS"))
        .stdout(predicate::str::contains("OWNER"))
        .stdout(predicate::str::contains("AUTHOR"))
        .stdout(predicate::str::contains("MODIFIED"));
}

#[test]
fn ls_file_type_does_not_append_exec_star() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let td = tempdir().expect("tmpdir");
        let p = td.path().join("runme");
        fs::write(&p, "echo hi\n").expect("write");
        let mut perms = fs::metadata(&p).expect("meta").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&p, perms).expect("chmod");

        dusk()
            .args([
                "ls",
                "--file-type",
                "--basic",
                td.path().to_string_lossy().as_ref(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("runme*").not());
    }
}

#[test]
fn ls_sort_by_ext_works() {
    let td = tempdir().expect("tmpdir");
    fs::write(td.path().join("a.rs"), "fn main(){}\n").expect("write");
    fs::write(td.path().join("b.md"), "# t\n").expect("write");

    dusk()
        .args([
            "ls",
            "--sort",
            "ext",
            "--basic",
            td.path().to_string_lossy().as_ref(),
        ])
        .assert()
        .success();
}

#[test]
fn cat_plain_reads_stdin() {
    dusk()
        .arg("cat")
        .write_stdin("line-a\nline-b\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("line-a\nline-b\n"));
}

#[test]
fn bat_pretty_shows_header() {
    let td = tempdir().expect("tmpdir");
    let path = td.path().join("sample.rs");
    fs::write(&path, "fn main() {}\n").expect("write sample");

    dusk()
        .arg("bat")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("--"));
}

#[test]
fn xtree_help_shows_tree_compat_flags() {
    dusk()
        .args(["xtree", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-I <pattern>"))
        .stdout(predicate::str::contains("--noreport"));
}

#[test]
fn tree_noreport_hides_summary_line() {
    let td = tempdir().expect("tmpdir");
    fs::create_dir_all(td.path().join("src")).expect("mkdir");
    fs::write(td.path().join("src/main.rs"), "fn main() {}\n").expect("write");

    dusk()
        .args([
            "tree",
            td.path().to_string_lossy().as_ref(),
            "-L",
            "2",
            "--noreport",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("directories").not());
}

#[test]
fn xtree_loc_prints_total_loc() {
    let td = tempdir().expect("tmpdir");
    fs::create_dir_all(td.path().join("src")).expect("mkdir");
    fs::write(td.path().join("src/main.rs"), "fn main() {}\nlet x = 1;\n").expect("write");

    dusk()
        .args(["xtree", "--loc", td.path().to_string_lossy().as_ref()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Total LOC:"));
}

#[test]
fn xtree_stats_includes_loc_column() {
    let td = tempdir().expect("tmpdir");
    fs::write(td.path().join("a.rs"), "fn a() {}\n").expect("write");
    fs::write(td.path().join("b.rs"), "fn b() {}\n").expect("write");

    dusk()
        .args(["xtree", "--stats", td.path().to_string_lossy().as_ref()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Total LOC:"))
        .stdout(predicate::str::contains("LOC"));
}

#[test]
fn xtree_loc_with_cat_exts_prints_filtered_loc() {
    let td = tempdir().expect("tmpdir");
    fs::write(td.path().join("a.rs"), "fn a() {}\n").expect("write");
    fs::write(td.path().join("b.md"), "# heading\nline\n").expect("write");

    dusk()
        .args([
            "xtree",
            "--loc",
            td.path().to_string_lossy().as_ref(),
            "-c",
            "rs",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Total LOC:"))
        .stdout(predicate::str::contains("LOC for -c [rs]: 1"));
}

#[test]
fn ls_basic_disables_ansi_even_when_forced() {
    dusk()
        .env("DUSK_COLOR", "always")
        .args(["ls", "--basic"])
        .assert()
        .success()
        .stdout(predicate::str::is_match("\\x1b\\[[0-9;]*m").unwrap().not());
}

#[test]
fn diff_help_and_git_help_are_available() {
    dusk()
        .args(["diff", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("side-by-side"));

    dusk()
        .args(["git", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("git log"));
}

#[test]
fn unknown_command_fails() {
    dusk()
        .arg("unknown-subcommand")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown command"));
}

#[test]
fn git_requires_git_binary() {
    dusk()
        .env("PATH", "/definitely/missing/path")
        .args(["git", "status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required system binary `git`"));
}

#[test]
fn find_requires_find_binary() {
    dusk()
        .env("PATH", "/definitely/missing/path")
        .args(["find", "."])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required system binary `find`"));
}

#[test]
fn rg_requires_rg_or_grep_binary() {
    dusk()
        .env("PATH", "/definitely/missing/path")
        .args(["rg", "main", "."])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "required system binary `rg` or `grep`",
        ));
}

#[test]
fn dump_hex_outputs_offsets() {
    let td = tempdir().expect("tmpdir");
    let p = td.path().join("bin.dat");
    fs::write(&p, [0x41_u8, 0x42, 0x00, 0xff]).expect("write");

    dusk()
        .args(["dump", "--hex", p.to_string_lossy().as_ref()])
        .assert()
        .success()
        .stdout(predicate::str::contains("00000000"));
}

#[test]
fn dump_default_mode_is_hex_only() {
    let td = tempdir().expect("tmpdir");
    let p = td.path().join("bin.dat");
    fs::write(&p, [0x41_u8, 0x42, 0x00, 0xff]).expect("write");

    dusk()
        .args(["dump", p.to_string_lossy().as_ref()])
        .assert()
        .success()
        .stdout(predicate::str::contains("-- HEX --"))
        .stdout(predicate::str::contains("-- ASM --").not());
}

#[test]
fn dump_asm_mode_is_asm_only_when_disassembler_available() {
    if !has_disassembler() {
        return;
    }

    dusk()
        .args(["dump", "--asm", env!("CARGO_BIN_EXE_dusk")])
        .assert()
        .success()
        .stdout(predicate::str::contains("-- ASM --"))
        .stdout(predicate::str::contains("-- HEX --").not());
}

#[test]
fn dump_hex_and_asm_together_show_both_when_disassembler_available() {
    if !has_disassembler() {
        return;
    }

    dusk()
        .args(["dump", "--hex", "--asm", env!("CARGO_BIN_EXE_dusk")])
        .assert()
        .success()
        .stdout(predicate::str::contains("-- HEX --"))
        .stdout(predicate::str::contains("-- ASM --"));
}

#[test]
fn dump_asm_requires_objdump_or_llvm_objdump() {
    let td = tempdir().expect("tmpdir");
    let p = td.path().join("bin.dat");
    fs::write(&p, [0x41_u8, 0x42]).expect("write");

    dusk()
        .env("PATH", "/definitely/missing/path")
        .args(["dump", "--asm", p.to_string_lossy().as_ref()])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "required system binary `objdump` or `llvm-objdump`",
        ));
}

#[test]
fn bat_lexical_highlighting_marks_keywords_when_forced_color() {
    let td = tempdir().expect("tmpdir");
    let p = td.path().join("sample.rs");
    fs::write(&p, "fn main() { let x = 42; }\n").expect("write");

    dusk()
        .env("DUSK_COLOR", "always")
        .args(["bat", "--no-number", p.to_string_lossy().as_ref()])
        .assert()
        .success()
        .stdout(predicate::str::is_match("\\x1b\\[[0-9;]*mfn\\x1b\\[0m").unwrap());
}
