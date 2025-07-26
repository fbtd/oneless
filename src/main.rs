use std::io;

use anyhow::{Error, Result, bail};
use terminal_size::{Height, Width, terminal_size};

mod cat;
mod lines;
mod prioritizer;
use crate::prioritizer::Prioritizer;

const EXTRA_LINES_TO_DELETE: usize = 2; // allows to read last executed command and next one

fn main() -> Result<()> {
    let stdin = io::stdin().lock();
    let stdout = io::stdout();

    match terminal_size() {
        None => bail!("stdout not a TTY (unable to determine size)"),
        Some((Width(w), Height(h))) => {
            let mut l =
                lines::Lines::from_reader(stdin, w as usize, h as usize - EXTRA_LINES_TO_DELETE)?;
            prioritizer::auto_prioritize(&mut l)?;
            l.prune();
            l.write(stdout)?;
        }
    }
    Ok(())
}
