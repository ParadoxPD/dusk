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
fn rm_help_is_available() {
    dusk()
        .args(["rm", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("safe rm"))
        .stdout(predicate::str::contains("--trash-tui"));
}

#[test]
fn rm_default_moves_file_to_trash() {
    let td = tempdir().expect("tmpdir");
    let trash = td.path().join(".trash");
    let file = td.path().join("deleteme.txt");
    fs::write(&file, "x").expect("write");

    dusk()
        .env("DUSK_TRASH_DIR", trash.to_string_lossy().to_string())
        .args(["rm", file.to_string_lossy().as_ref()])
        .assert()
        .success();

    assert!(!file.exists());
    assert!(trash.join("files").exists());
    let moved_count = fs::read_dir(trash.join("files"))
        .expect("read files")
        .filter_map(Result::ok)
        .count();
    assert_eq!(moved_count, 1);
}

#[test]
fn rm_permanent_deletes_file() {
    let td = tempdir().expect("tmpdir");
    let trash = td.path().join(".trash");
    let file = td.path().join("hard-delete.txt");
    fs::write(&file, "x").expect("write");

    dusk()
        .env("DUSK_TRASH_DIR", trash.to_string_lossy().to_string())
        .args(["rm", "-P", file.to_string_lossy().as_ref()])
        .assert()
        .success();

    assert!(!file.exists());
    let files_dir = trash.join("files");
    if files_dir.exists() {
        let moved_count = fs::read_dir(files_dir)
            .expect("read files")
            .filter_map(Result::ok)
            .count();
        assert_eq!(moved_count, 0);
    }
}

#[test]
fn rm_dir_requires_recursive() {
    let td = tempdir().expect("tmpdir");
    let dir = td.path().join("subdir");
    fs::create_dir_all(&dir).expect("mkdir");

    dusk()
        .args(["rm", dir.to_string_lossy().as_ref()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Is a directory"));

    assert!(dir.exists());
}

#[test]
fn rm_recursive_moves_directory_to_trash() {
    let td = tempdir().expect("tmpdir");
    let trash = td.path().join(".trash");
    let dir = td.path().join("subdir");
    fs::create_dir_all(&dir).expect("mkdir");
    fs::write(dir.join("a.txt"), "hello").expect("write");

    dusk()
        .env("DUSK_TRASH_DIR", trash.to_string_lossy().to_string())
        .args(["rm", "-r", dir.to_string_lossy().as_ref()])
        .assert()
        .success();

    assert!(!dir.exists());
    assert!(trash.join("files").exists());
}

#[test]
fn rm_restore_restores_file_from_trash() {
    let td = tempdir().expect("tmpdir");
    let trash = td.path().join(".trash");
    let file = td.path().join("restore-me.txt");
    fs::write(&file, "x").expect("write");

    dusk()
        .env("DUSK_TRASH_DIR", trash.to_string_lossy().to_string())
        .args(["rm", file.to_string_lossy().as_ref()])
        .assert()
        .success();
    assert!(!file.exists());

    dusk()
        .env("DUSK_TRASH_DIR", trash.to_string_lossy().to_string())
        .args(["rm", "--restore", "restore-me"])
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("restored"));

    assert!(file.exists());
}

#[test]
fn rm_empty_trash_clears_all_items() {
    let td = tempdir().expect("tmpdir");
    let trash = td.path().join(".trash");
    let file1 = td.path().join("e1.txt");
    let file2 = td.path().join("e2.txt");
    fs::write(&file1, "x").expect("write");
    fs::write(&file2, "x").expect("write");

    dusk()
        .env("DUSK_TRASH_DIR", trash.to_string_lossy().to_string())
        .args([
            "rm",
            file1.to_string_lossy().as_ref(),
            file2.to_string_lossy().as_ref(),
        ])
        .assert()
        .success();

    dusk()
        .env("DUSK_TRASH_DIR", trash.to_string_lossy().to_string())
        .args(["rm", "--empty-trash", "-f"])
        .assert()
        .success()
        .stdout(predicate::str::contains("deleted"));

    let count = fs::read_dir(trash.join("files"))
        .expect("read files")
        .filter_map(Result::ok)
        .count();
    assert_eq!(count, 0);
}

#[test]
fn mv_cp_ln_help_available() {
    dusk().args(["mv", "--help"]).assert().success();
    dusk().args(["cp", "--help"]).assert().success();
    dusk().args(["ln", "--help"]).assert().success();
}

#[test]
fn mv_conflict_prompts_and_can_cancel() {
    let td = tempdir().expect("tmpdir");
    let src = td.path().join("src.txt");
    let dst = td.path().join("dst.txt");
    fs::write(&src, "src").expect("write src");
    fs::write(&dst, "dst").expect("write dst");

    dusk()
        .args([
            "mv",
            src.to_string_lossy().as_ref(),
            dst.to_string_lossy().as_ref(),
        ])
        .write_stdin("n\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("operation cancelled"));

    assert!(src.exists());
    assert_eq!(fs::read_to_string(&dst).expect("read dst"), "dst");
}

#[test]
fn cp_conflict_prompts_and_can_confirm() {
    let td = tempdir().expect("tmpdir");
    let src = td.path().join("copy-src.txt");
    let dst = td.path().join("copy-dst.txt");
    fs::write(&src, "src").expect("write src");
    fs::write(&dst, "dst").expect("write dst");

    dusk()
        .args([
            "cp",
            src.to_string_lossy().as_ref(),
            dst.to_string_lossy().as_ref(),
        ])
        .write_stdin("y\n")
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&dst).expect("read dst"), "src");
}

#[test]
fn ln_creates_symlink() {
    #[cfg(unix)]
    {
        let td = tempdir().expect("tmpdir");
        let src = td.path().join("target.txt");
        let dst = td.path().join("link.txt");
        fs::write(&src, "x").expect("write src");

        dusk()
            .args([
                "ln",
                "-s",
                src.to_string_lossy().as_ref(),
                dst.to_string_lossy().as_ref(),
            ])
            .assert()
            .success();

        assert!(dst.exists());
    }
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
fn git_diff_help_is_available() {
    dusk()
        .args(["git", "diff", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dusk git diff"))
        .stdout(predicate::str::contains("--tui"));
}

#[test]
fn git_redundant_aliases_show_migration_errors() {
    dusk()
        .args(["git", "graph"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Use `dusk git log`"));

    dusk()
        .args(["git", "viz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Use `dusk git status`"));
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
fn mv_cp_ln_require_system_binaries() {
    dusk()
        .env("PATH", "/definitely/missing/path")
        .args(["mv", "--help"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required system binary `mv`"));

    dusk()
        .env("PATH", "/definitely/missing/path")
        .args(["cp", "--help"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required system binary `cp`"));

    dusk()
        .env("PATH", "/definitely/missing/path")
        .args(["ln", "--help"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required system binary `ln`"));
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
