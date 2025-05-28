fn main() {
    if let Err(e) = rfermata::run() {
        eprintln!("{e}");
        std::process::exit(-1);
    }
}
