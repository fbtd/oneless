use crate::lines::Lines;
use anyhow::{Error, Result, bail};
use std::io::Cursor;

trait Prioritizer {
    fn prioritize(lines: &mut Lines) -> Result<()>;
}

struct Head {}
impl Prioritizer for Head {
    fn prioritize(lines: &mut Lines) -> Result<()> {
        for (line_number, line) in &mut lines.lines.iter_mut().enumerate() {
            line.prio.push(line_number as u32);
        }
        Ok(())
    }
}

struct HeadAndTail {}
impl Prioritizer for HeadAndTail {
    fn prioritize(lines: &mut Lines) -> Result<()> {
        let len: usize = lines.lines.len();
        for (line_number, line) in &mut lines.lines.iter_mut().enumerate() {
            line.prio
                .push(line_number.min(len - line_number - 1) as u32);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Error, Result, bail};
    use googletest::prelude::*;

    fn make_lines() -> Lines {
        let c = Cursor::new("first\nsecond\nthird\n");
        Lines::from_reader(c, 20, 20).unwrap()
    }

    #[gtest]
    fn head_prioritizer() -> Result<()> {
        let mut lines = make_lines();
        Head::prioritize(&mut lines)?;
        expect_that!(&lines.lines[0].prio, eq(&vec![0]));
        expect_that!(&lines.lines[1].prio, eq(&vec![1]));
        expect_that!(&lines.lines[1].prio, len(eq(1)));
        Ok(())
    }

    #[gtest]
    fn head_and_tail_prioritizer() -> Result<()> {
        let mut lines = make_lines();
        HeadAndTail::prioritize(&mut lines)?;
        expect_that!(&lines.lines[0].prio, eq(&vec![0]));
        expect_that!(&lines.lines[1].prio, eq(&vec![1]));
        expect_that!(&lines.lines[2].prio, eq(&vec![0]));
        expect_that!(&lines.lines[1].prio, len(eq(1)));
        Ok(())
    }
}
