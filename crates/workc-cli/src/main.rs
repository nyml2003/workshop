mod cli;
mod presenters;

fn main() {
    match cli::task::run() {
        Ok(output) => {
            println!("{output}");
        }
        Err(error) => {
            eprintln!("{}", presenters::text::render_error(&error.to_string()));
            std::process::exit(1);
        }
    }
}
