use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use criterion::{criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

fn bench_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/release/dusk")
}

fn sample_tree() -> tempfile::TempDir {
    let td = tempdir().expect("tmpdir");
    fs::create_dir_all(td.path().join("src")).expect("mkdir src");
    fs::create_dir_all(td.path().join("docs")).expect("mkdir docs");
    fs::write(td.path().join("src/main.rs"), "fn main() {}\n").expect("write");
    fs::write(td.path().join("src/lib.rs"), "pub fn x() -> i32 { 1 }\n").expect("write");
    fs::write(td.path().join("README.md"), "# Demo\n").expect("write");
    fs::write(td.path().join("docs/guide.md"), "hello\n").expect("write");
    td
}

fn run_bin(args: &[&str]) {
    let status = Command::new(bench_bin())
        .args(args)
        .env("DUSK_COLOR", "never")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("run dusk");
    assert!(status.success());
}

fn bench_help(c: &mut Criterion) {
    c.bench_function("cmd_help", |b| b.iter(|| run_bin(&[black_box("help")])));
}

fn bench_ls(c: &mut Criterion) {
    let td = sample_tree();
    let dir = td.path().to_string_lossy().to_string();

    c.bench_function("cmd_ls_default", |b| {
        b.iter(|| run_bin(&["ls", black_box(dir.as_str())]))
    });

    c.bench_function("cmd_ls_long", |b| {
        b.iter(|| run_bin(&["ls", "-l", black_box(dir.as_str())]))
    });
}

fn bench_cat_and_bat(c: &mut Criterion) {
    let td = sample_tree();
    let file = td.path().join("src/main.rs").to_string_lossy().to_string();

    c.bench_function("cmd_cat_plain", |b| {
        b.iter(|| run_bin(&["cat", black_box(file.as_str())]))
    });

    c.bench_function("cmd_bat_pretty", |b| {
        b.iter(|| run_bin(&["bat", black_box(file.as_str())]))
    });
}

fn bench_tree_and_xtree(c: &mut Criterion) {
    let td = sample_tree();
    let dir = td.path().to_string_lossy().to_string();

    c.bench_function("cmd_tree", |b| {
        b.iter(|| run_bin(&["tree", "-L", "2", black_box(dir.as_str())]))
    });

    c.bench_function("cmd_xtree_json", |b| {
        b.iter(|| run_bin(&["xtree", "--json", "-L", "2", black_box(dir.as_str())]))
    });
}

fn bench_git_and_diff_help(c: &mut Criterion) {
    c.bench_function("cmd_git_help", |b| {
        b.iter(|| run_bin(&[black_box("git"), black_box("--help")]))
    });

    c.bench_function("cmd_diff_help", |b| {
        b.iter(|| run_bin(&[black_box("diff"), black_box("--help")]))
    });
}

fn bench_themes(c: &mut Criterion) {
    c.bench_function("cmd_themes_list", |b| {
        b.iter(|| run_bin(&[black_box("themes"), black_box("list")]))
    });
}

criterion_group!(
    benches,
    bench_help,
    bench_ls,
    bench_cat_and_bat,
    bench_tree_and_xtree,
    bench_git_and_diff_help,
    bench_themes
);
criterion_main!(benches);
