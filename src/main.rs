fn main() {
    if let Err(err) = apple_books_data_export::run() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}
