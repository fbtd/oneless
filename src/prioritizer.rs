use crate::lines::Lines;
use anyhow::{Error, Result, bail};
use std::io::Cursor;

pub trait Prioritizer {
    fn prioritize(lines: &mut Lines) -> Result<()>;
}

pub struct Autodetect {}
impl Prioritizer for Autodetect {
    fn prioritize(lines: &mut Lines) -> Result<()> {
        HeadAndTail::prioritize(lines)
    }
}


pub struct Head {}
impl Prioritizer for Head {
    fn prioritize(lines: &mut Lines) -> Result<()> {
        for (line_number, line) in &mut lines.lines.iter_mut().enumerate() {
            line.prio.push(line_number as u32);
        }
        Ok(())
    }
}

pub struct HeadAndTail {}
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

const SEPARATOR: &str = "/";
pub struct PathDepth {}
impl Prioritizer for PathDepth {
    fn prioritize(lines: &mut Lines) -> Result<()> {
        for line in &mut lines.lines.iter_mut() {
            line.prio.push(line.text.split(SEPARATOR).count() as u32);
        }
        Ok(())
    }
}

pub struct FirstAlnum {}
impl Prioritizer for FirstAlnum {
    fn prioritize(lines: &mut Lines) -> Result<()> {
        for line in &mut lines.lines.iter_mut() {
            line.prio.push(
                line.text
                    .chars()
                    .position(|c| c.is_ascii_alphanumeric())
                    .unwrap_or(0) as u32,
            );
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

    #[gtest]
    fn path_depth_prioritizer() -> Result<()> {
        //                   0  1    2      3      4      5  6
        let c = Cursor::new("x\nx/x\nx/x/x\nx/x/y\nx/x/z\ny\ny/x\n");
        let mut lines = Lines::from_reader(c, 20, 20).unwrap();
        PathDepth::prioritize(&mut lines)?;
        expect_that!(&lines.lines[0].prio, eq(&vec![1]));
        expect_that!(&lines.lines[1].prio, eq(&vec![2]));
        expect_that!(&lines.lines[2].prio, eq(&vec![3]));
        expect_that!(&lines.lines[3].prio, eq(&vec![3]));
        expect_that!(&lines.lines[4].prio, eq(&vec![3]));
        expect_that!(&lines.lines[5].prio, eq(&vec![1]));
        expect_that!(&lines.lines[6].prio, eq(&vec![2]));
        expect_that!(&lines.lines[1].prio, len(eq(1)));
        Ok(())
    }

    #[gtest]
    fn first_alnum_prioritizer() -> Result<()> {
        //                   0  1   2    3    4    5  6
        let c = Cursor::new("x\n x\n  x\n  y\n  z\ny\n x\n");
        let mut lines = Lines::from_reader(c, 20, 20).unwrap();
        FirstAlnum::prioritize(&mut lines)?;
        expect_that!(&lines.lines[0].prio, eq(&vec![0]));
        expect_that!(&lines.lines[1].prio, eq(&vec![1]));
        expect_that!(&lines.lines[2].prio, eq(&vec![2]));
        expect_that!(&lines.lines[3].prio, eq(&vec![2]));
        expect_that!(&lines.lines[4].prio, eq(&vec![2]));
        expect_that!(&lines.lines[5].prio, eq(&vec![0]));
        expect_that!(&lines.lines[6].prio, eq(&vec![1]));
        expect_that!(&lines.lines[1].prio, len(eq(1)));
        Ok(())
    }
}
