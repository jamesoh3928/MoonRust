use clap::Parser;
use moonrust::interpreter::environment;
use std::fs;
use std::process;

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// Path of the file to run
    #[clap(value_name = "FILE.lua")]
    file: String,
}

fn main() {
    let args = Args::parse();
    let file = &args.file;

    // Read file
    let src: String = match fs::read_to_string(file) {
        Ok(src) => src,
        Err(err) => {
            eprintln!("File read error [{file}; {err}]");
            process::exit(1);
        }
    };

    let ast = match src.parse::<moonrust::AST>() {
        Ok(ast) => ast,
        Err(ast_parse_error) => {
            eprintln!("Parse error [{ast_parse_error}]");
            process::exit(1);
        }
    };

    // TODO: add flag for printing AST
    println!("AST: {:#?}", ast);

    // Execute the program
    // Initialize environment
    let mut env = environment::Env::new();
    match ast.exec(&mut env) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Runtime error [{err}]");
            process::exit(1);
        }
    }
}
