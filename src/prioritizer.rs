use crate::lines::Lines;
use anyhow::{Error, Result, bail};

#[derive(Clone, Debug)]
enum Confidence {
    Low,
    Medium,
    High,
    Certain,
}
impl From<Confidence> for u32 {
    fn from(value: Confidence) -> Self {
        match value {
            Confidence::Low => 10,
            Confidence::Medium => 20,
            Confidence::High => 30,
            Confidence::Certain => 100,
        }
    }
}

pub trait Prioritizer {
    fn confidence(&self) -> Confidence;
    fn prioritize(&self, lines: &mut Lines) -> Result<()>;
}

pub fn auto_prioritize(lines: &mut Lines) -> Result<()> {
    // TODO: just take some lines as samples
    let sample_lines = lines.clone();
    let head_and_tail_prioritizer = Box::new(HeadAndTail::new(&sample_lines));

    let prioritizers: Vec<Box<dyn Prioritizer>> = vec![
        Box::new(PathDepth::new(&sample_lines)),
        Box::new(FirstAlnum::new(&sample_lines)),
        head_and_tail_prioritizer,
    ];

    let prioritizer = prioritizers
        .iter()
        .max_by(|p, q| u32::from(p.confidence()).cmp(&(u32::from(q.confidence()))))
        .unwrap();
    dbg!(prioritizer.confidence());
    prioritizer.prioritize(lines)
}

pub struct Head {
    confidence: Confidence,
}

impl Head {
    fn new() -> Head {
        Head {
            confidence: Confidence::Low,
        }
    }
}
impl Prioritizer for Head {
    fn prioritize(&self, lines: &mut Lines) -> Result<()> {
        for (line_number, line) in &mut lines.lines.iter_mut().enumerate() {
            line.prio.push(line_number as u32);
        }
        Ok(())
    }

    fn confidence(&self) -> Confidence {
        self.confidence.clone()
    }
}

pub struct HeadAndTail {
    confidence: Confidence,
}
impl HeadAndTail {
    fn new(sample_lines: &Lines) -> HeadAndTail {
        HeadAndTail {
            confidence: Confidence::Medium,
        }
    }
}
impl Prioritizer for HeadAndTail {
    fn prioritize(&self, lines: &mut Lines) -> Result<()> {
        let len: usize = lines.lines.len();
        for (line_number, line) in &mut lines.lines.iter_mut().enumerate() {
            line.prio
                .push(line_number.min(len - line_number - 1) as u32);
        }
        Ok(())
    }

    fn confidence(&self) -> Confidence {
        self.confidence.clone()
    }
}

const SEPARATOR: &str = "/";
pub struct PathDepth {
    confidence: Confidence,
}
impl PathDepth {
    fn new(sample_lines: &Lines) -> PathDepth {
        let n_lines = sample_lines.lines.iter().count();
        let n_lines_with_separator = sample_lines
            .lines
            .iter()
            .filter(|l| l.text.contains(SEPARATOR))
            .count();
        if n_lines_with_separator >= n_lines - 2 && n_lines > 2 {
            PathDepth {
                confidence: Confidence::Certain
            }
        } else {
            PathDepth {
                confidence: Confidence::Low,
            }
        }
    }
}
impl Prioritizer for PathDepth {
    fn prioritize(&self, lines: &mut Lines) -> Result<()> {
        for line in &mut lines.lines.iter_mut() {
            line.prio.push(line.text.split(SEPARATOR).count() as u32);
        }
        Ok(())
    }

    fn confidence(&self) -> Confidence {
        self.confidence.clone()
    }
}

pub struct FirstAlnum {
    confidence: Confidence,
}
impl FirstAlnum {
    fn new(sample_lines: &Lines) -> FirstAlnum {
        let n_lines = sample_lines.lines.iter().count();
        let n_lines_with_separator = sample_lines
            .lines
            .iter()
            .filter(|l| l.text.contains("├") || l.text.contains("└") )
            .count();
        if n_lines_with_separator >= n_lines - 2 && n_lines > 2 {
            FirstAlnum {
                confidence: Confidence::Certain
            }
        } else {
            FirstAlnum {
                confidence: Confidence::Low,
            }
        }
    }
}
impl Prioritizer for FirstAlnum {
    fn prioritize(&self, lines: &mut Lines) -> Result<()> {
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

    fn confidence(&self) -> Confidence {
        self.confidence.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Error, Result, bail};
    use googletest::prelude::*;
    use std::io::Cursor;

    fn make_lines() -> Lines {
        let c = Cursor::new("first\nsecond\nthird\n");
        Lines::from_reader(c, 20, 20).unwrap()
    }

    #[gtest]
    fn head_prioritizer() -> Result<()> {
        let mut lines = make_lines();
        let p = Head::new();
        p.prioritize(&mut lines)?;
        expect_that!(&lines.lines[0].prio, eq(&vec![0]));
        expect_that!(&lines.lines[1].prio, eq(&vec![1]));
        expect_that!(&lines.lines[1].prio, len(eq(1)));
        Ok(())
    }

    #[gtest]
    fn head_and_tail_prioritizer() -> Result<()> {
        let mut lines = make_lines();
        let p = HeadAndTail::new(&lines);
        p.prioritize(&mut lines)?;
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
        let p = PathDepth::new(&lines);
        p.prioritize(&mut lines)?;
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
        let p = FirstAlnum::new(&lines);
        p.prioritize(&mut lines)?;
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
