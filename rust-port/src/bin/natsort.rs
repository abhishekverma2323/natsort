use std::env;
use std::io;
use std::path::Path;

fn main() {
    let mut raw_arguments = env::args();
    let executable = raw_arguments
        .next()
        .unwrap_or_else(|| "natsort".to_string());
    let program = Path::new(&executable)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("natsort");
    let arguments: Vec<String> = raw_arguments.collect();

    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    let mut error = io::stderr().lock();

    let code =
        rust_port::run_cli_with_program(program, arguments, &mut input, &mut output, &mut error);

    if code != 0 {
        std::process::exit(code);
    }
}
