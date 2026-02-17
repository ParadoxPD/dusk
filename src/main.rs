mod app;
mod commands;
mod core;
mod xtree;

fn main() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    if let Err(err) = app::run(std::env::args().collect()) {
        eprintln!("dusk: {err}");
        std::process::exit(1);
    }
}
