use moonrust;

fn main() {
    match "test".parse::<moonrust::AST>() {
        Err(_) => (),
        Ok(prog) => (),
    }

    // TODO: read from the file (command line options: parsing, interpreting, repl - for future)
    // https://git.cs.rit.edu/psr2225/jo9347psr/-/blob/main/prog02/birch/src/main.rs
}
