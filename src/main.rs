fn main() {
    if let Err(e) = openspec::cli::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
