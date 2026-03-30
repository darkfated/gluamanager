#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args_os().len() > 1 {
        if let Err(error) = gluamanager::cli::run() {
            eprintln!("{}", error.user_message());
            std::process::exit(1);
        }
        return;
    }

    gluamanager::run_gui();
}
