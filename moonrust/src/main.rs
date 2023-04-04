use clap::Parser;
use moonrust::interpreter::environment;
use moonrust;
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

    // Read fj,e
    let src: String = match fs::read_to_string(file) {
        Ok(src) => src,
        Err(err) => {
            eprintln!("File read error [{file}; {err}]");
            process::exit(1);
        }
    };

    // TODO: delete (keeping it to check if reading file correctly)
    println!("Hello, {src}!");

    // Parse the source code: TODO - currently `from_str` is not implemented
    let ast = match src.parse::<moonrust::AST>() {
        Ok(ast) => ast,
        Err(ast_parse_error) => {
            eprintln!("Parse error [{ast_parse_error}]");
            process::exit(1);
        }
    };

    // Execute the program
    // Initialize environment
    // TODO: create new file for environment
    let mut env = environment::Env::new();
    // let mut env = moonrust::interpreter::Env::new();
    match ast.exec(&mut env) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Runtime error [{err}]");
            process::exit(1);
        }
    }

    // match "test".parse::<moonrust::AST>() {
    //     Err(_) => (),
    //     Ok(prog) => (),
    // }

    // TODO: read from the file (command line options: parsing, interpreting, repl - for future)
    // https://git.cs.rit.edu/psr2225/jo9347psr/-/blob/main/prog02/birch/src/main.rs
}
