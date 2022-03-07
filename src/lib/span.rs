use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub struct Span {
    pub lo: FileLoc,
    pub hi: FileLoc,
}

#[derive(Debug, Copy, Clone)]
pub struct FileLoc {
    pub line: usize,
    pub col: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.lo.fmt(f)
    }
}

impl Display for FileLoc {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}
