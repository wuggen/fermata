fn main() {
    if let Err(e) = fermata::run() {
        eprintln!("{e}");
        std::process::exit(-1);
    }
}
