use anyhow::{Error, Result};
use std::cmp::Ordering;
use std::io::{BufRead, Write};

const DOTDOTDOT: &str = "...";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineStatus {
    Kept,
    Discardable,
    Discarded,
    DotDotDot,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub prio: Vec<u32>, // compared left to right, lowest prio = important line
    pub status: LineStatus,
    pub text: String,
}

impl Line {
    fn new(s: &str, len: usize) -> Line {
        Line {
            prio: Vec::new(),
            status: LineStatus::Kept,
            text: s.chars().take(len).collect(),
        }
    }
}

impl PartialEq for Line {
    fn eq(&self, other: &Self) -> bool {
        self.prio == other.prio
    }
}
impl PartialOrd for Line {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.prio.cmp(&other.prio))
    }
}

impl Eq for Line {}

impl Ord for Line {
    fn cmp(&self, other: &Self) -> Ordering {
        self.prio.cmp(&other.prio)
    }
}

#[derive(Clone, Debug)]
pub struct Lines {
    pub lines: Vec<Line>,
    pub target_lines: usize,
}

impl Lines {
    pub fn from_reader<R: BufRead>(
        reader: R,
        columns: usize,
        target_lines: usize,
    ) -> Result<Lines> {
        let lines: Vec<String> = reader.lines().collect::<Result<Vec<String>, _>>()?;
        let lines: Vec<Line> = lines.iter().map(|l| Line::new(&l, columns)).collect();
        Ok(Lines {
            lines,
            target_lines,
        })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> Result<()> {
        for line in &self.lines {
            match line.status {
                LineStatus::Kept | LineStatus::Discardable => writeln!(writer, "{}", line.text)?,
                LineStatus::DotDotDot => writeln!(writer, "{}", DOTDOTDOT)?,
                LineStatus::Discarded => (),
            }
        }
        Ok(())
    }

    pub fn kept_lines(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| {
                l.status == LineStatus::DotDotDot
                    || l.status == LineStatus::Kept
                    || l.status == LineStatus::Discardable
            })
            .count()
    }

    pub fn prune(&mut self) {
        while self.kept_lines() > self.target_lines {
            //dbg!(self.kept_lines());
            // kept to discardable (one line)
            if let Some(line_to_delete) = self
                .lines
                .iter_mut()
                .filter(|l| l.status == LineStatus::Kept)
                .max()
            {
                line_to_delete.status = LineStatus::Discardable;
            } else {
                panic!("no more lines prune!");
            }

            // discardable to discarded (zero or more lines)
            let mut status_last_line = LineStatus::Kept;
            for line in self.lines.iter_mut() {
                if line.status != LineStatus::Kept && status_last_line != LineStatus::Kept {
                    line.status = LineStatus::Discarded;
                }
                status_last_line = line.status.clone();
            }

            // discarded to dotdotdot
            let mut status_last_line = LineStatus::Kept;
            for line in self.lines.iter_mut().rev() {
                if line.status == LineStatus::Discardable
                    && status_last_line == LineStatus::Discarded
                {
                    line.status = LineStatus::DotDotDot;
                }
                status_last_line = line.status.clone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Error, Result, bail};
    use googletest::prelude::*;
    use std::io::{Cursor, stdout};

    const MULTILINE: &str = "first\nsecond\nthird\nfourth\nfifth\nsixt\n";

    fn make_lines_head() -> Lines {
        let r: Cursor<Vec<u8>> = Cursor::new(MULTILINE.into());
        let mut lines = Lines::from_reader(r, 10, 10).unwrap();
        lines.lines[0].prio.push(0);
        lines.lines[1].prio.push(1);
        lines.lines[2].prio.push(2);
        lines.lines[3].prio.push(3);
        lines.lines[4].prio.push(4);
        lines.lines[5].prio.push(5);
        lines
    }

    fn make_lines_tail() -> Lines {
        let r: Cursor<Vec<u8>> = Cursor::new(MULTILINE.into());
        let mut lines = Lines::from_reader(r, 10, 10).unwrap();
        lines.lines[0].prio.push(5);
        lines.lines[1].prio.push(4);
        lines.lines[2].prio.push(3);
        lines.lines[3].prio.push(2);
        lines.lines[4].prio.push(1);
        lines.lines[5].prio.push(0);
        lines
    }

    fn make_lines_snake() -> Lines {
        let r: Cursor<Vec<u8>> = Cursor::new(MULTILINE.into());
        let mut lines = Lines::from_reader(r, 10, 10).unwrap();
        lines.lines[0].prio.push(0);
        lines.lines[1].prio.push(1);
        lines.lines[2].prio.push(0);
        lines.lines[3].prio.push(1);
        lines.lines[4].prio.push(0);
        lines.lines[5].prio.push(1);
        lines
    }

    #[gtest]
    fn new_lines() {
        let t = "01234567890";
        let short_line = Line::new(t, 20);
        expect_that!(short_line.prio, is_empty());
        expect_that!(short_line.text, eq(t));

        let long_line = Line::new(t, 8);
        expect_that!(long_line.prio, is_empty());
        expect_that!(long_line.text, eq("01234567"));
    }

    #[gtest]
    fn cmp_lines() {
        let first_line = Line {
            prio: vec![10, 20, 30],
            text: String::from("x"),
            status: LineStatus::Kept,
        };
        let second_line = Line {
            prio: vec![10, 21, 30],
            text: String::from("x"),
            status: LineStatus::Kept,
        };
        let third_line = Line {
            prio: vec![11, 21, 30],
            text: String::from("x"),
            status: LineStatus::Kept,
        };
        let fourth_line = Line {
            prio: vec![12],
            text: String::from("x"),
            status: LineStatus::Kept,
        };
        let fifth_line = Line {
            prio: vec![12],
            text: String::from("y"),
            status: LineStatus::Kept,
        };
        expect_that!(first_line, lt(&second_line));
        expect_that!(second_line, lt(&third_line));
        expect_that!(third_line, lt(&fourth_line));
        expect_that!(fifth_line, eq(&fourth_line));
    }

    #[gtest]
    fn lines_read_write() -> Result<()> {
        let r: Cursor<Vec<u8>> = Cursor::new(MULTILINE.into());
        let mut w: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        let lines = Lines::from_reader(r, 10, 10)?;
        lines.write(&mut w)?;
        let s: String = String::from_utf8(w.into_inner())?;
        expect_that!(s, eq(MULTILINE));
        Ok(())
    }

    #[gtest]
    fn write_lines() -> Result<()> {
        let r: Cursor<Vec<u8>> = Cursor::new(MULTILINE.into());
        let mut w: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        let mut lines = Lines::from_reader(r, 10, 10)?;
        lines.lines[0].status = LineStatus::Discardable;
        lines.lines[1].status = LineStatus::Discarded;
        lines.lines[2].status = LineStatus::DotDotDot;
        lines.write(&mut w)?;
        let s: String = String::from_utf8(w.into_inner())?;
        let expected: &str = "first\n...\nfourth\nfifth\nsixt\n";
        expect_that!(s, eq(expected));
        Ok(())
    }

    #[gtest]
    fn lines_trim_columns() -> Result<()> {
        let r: Cursor<Vec<u8>> = Cursor::new(MULTILINE.into());
        let mut w: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        let lines = Lines::from_reader(r, 3, 10)?;
        lines.write(&mut w)?;
        let s: String = String::from_utf8(w.into_inner())?;
        let expected = "fir\nsec\nthi\nfou\nfif\nsix\n";
        expect_that!(s, eq(expected));
        Ok(())
    }

    #[gtest]
    fn kept_lines() -> Result<()> {
        let r: Cursor<Vec<u8>> = Cursor::new(MULTILINE.into());
        let mut lines = Lines::from_reader(r, 10, 10)?;
        expect_that!(lines.kept_lines(), eq(6));
        lines.lines[0].status = LineStatus::Discarded;
        expect_that!(lines.kept_lines(), eq(5));
        lines.lines[1].status = LineStatus::Discardable;
        expect_that!(lines.kept_lines(), eq(5));
        lines.lines[2].status = LineStatus::Discarded;
        expect_that!(lines.kept_lines(), eq(4));
        Ok(())
    }

    #[gtest]
    fn prune_head() -> Result<()> {
        let mut lines = make_lines_head();
        lines.target_lines = 6;
        lines.prune();
        expect_that!(
            lines.lines[0].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[1].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[2].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[3].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[4].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[5].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );

        lines.target_lines = 5;
        lines.prune();
        expect_that!(
            lines.lines[0].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[1].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[2].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[3].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(lines.lines[4].status, eq(&LineStatus::DotDotDot));
        expect_that!(lines.lines[5].status, eq(&LineStatus::Discarded));

        lines.target_lines = 1;
        lines.prune();
        expect_that!(lines.lines[0].status, eq(&LineStatus::DotDotDot));
        expect_that!(lines.lines[1].status, eq(&LineStatus::Discarded));
        expect_that!(lines.lines[2].status, eq(&LineStatus::Discarded));
        expect_that!(lines.lines[3].status, eq(&LineStatus::Discarded));
        expect_that!(lines.lines[4].status, eq(&LineStatus::Discarded));
        expect_that!(lines.lines[5].status, eq(&LineStatus::Discarded));
        Ok(())
    }

    #[gtest]
    fn prune_tail() -> Result<()> {
        let mut lines = make_lines_tail();
        lines.target_lines = 6;
        lines.prune();
        expect_that!(
            lines.lines[0].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[1].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[2].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[3].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[4].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[5].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );

        lines.target_lines = 4;
        lines.prune();
        expect_that!(lines.lines[0].status, eq(&LineStatus::DotDotDot));
        expect_that!(lines.lines[1].status, eq(&LineStatus::Discarded));
        expect_that!(lines.lines[2].status, eq(&LineStatus::Discarded));
        expect_that!(
            lines.lines[3].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[4].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );
        expect_that!(
            lines.lines[5].status,
            eq(&LineStatus::Kept).or(eq(&LineStatus::Discardable))
        );

        Ok(())
    }

    #[gtest]
    fn prune_snake() -> Result<()> {
        let mut lines = make_lines_snake();
        lines.target_lines = 4;
        lines.prune();
        expect_that!(lines.kept_lines(), le(4));
        expect_that!(lines.kept_lines(), ge(3));
        Ok(())
    }
}
