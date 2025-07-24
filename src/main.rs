use std::error::Error;
use std::io;

mod lines;
mod cat;
mod prioritizer;

fn main() -> Result<(), Box<dyn Error>>{
    let stdin = io::stdin().lock();
    let stdout = io::stdout();

    let l = lines::Lines::from_reader(stdin, 10, 20)?;
    l.write(stdout)?;

    Ok(())
}
