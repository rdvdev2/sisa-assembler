use crate::Span;
use colorful::{Color, Colorful};
use std::fmt::{Display, Formatter};
use std::path;

#[derive(Clone)]
pub struct AssemblerMessage {
    pub msg_type: AssemblerMessageType,
    pub description: String,
    pub span: Option<Span>,
}

#[allow(dead_code)]
#[derive(PartialEq, Clone)]
pub enum AssemblerMessageType {
    ERROR,
    WARNING,
    HELP,
}

impl Display for AssemblerMessageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblerMessageType::ERROR => write!(f, "{}", "error".red()),
            AssemblerMessageType::WARNING => write!(f, "{}", "warning".yellow()),
            AssemblerMessageType::HELP => write!(f, "{}", "help".blue()),
        }
    }
}

impl AssemblerMessageType {
    pub(crate) fn get_color(&self) -> Color {
        match self {
            AssemblerMessageType::ERROR => Color::Red,
            AssemblerMessageType::WARNING => Color::Yellow,
            AssemblerMessageType::HELP => Color::Blue,
        }
    }
}

impl AssemblerMessage {
    pub fn write(self, source_path: &path::Path, code: &str) -> String {
        let title = format!(
            "{}{} {}",
            self.msg_type.to_string().bold(),
            ":".bold(),
            self.description.bold()
        );
        let file = format!("  {} {}", "-->".blue().bold(), source_path.display());
        let span = if let Some(span) = self.span {
            span.to_string()
        } else {
            String::from("1:1")
        };
        let context = if let Some(span) = self.span {
            write_context(code, span, self.msg_type)
        } else {
            String::new()
        };

        format!("{}\n{}:{}\n{}", title, file, span, context)
    }
}

fn write_context(code: &str, span: Span, msg_type: AssemblerMessageType) -> String {
    let mut iter = code.chars().peekable();
    let mut line = 1;
    let mut context = String::new();

    while let Some(c) = iter.next() {
        if c == '\n' {
            line += 1;
        } else if span.lo.line <= line && line <= span.hi.line {
            context.push(c);
        }
    }

    format!(
        "{}\n{}{}",
        context,
        " ".repeat(span.lo.col - 1),
        "^".repeat(span.lo.col.abs_diff(span.hi.col))
            .color(msg_type.get_color())
    )
}
