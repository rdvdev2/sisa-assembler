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
#[derive(PartialEq, Eq, Clone)]
pub enum AssemblerMessageType {
    Error,
    Warning,
    Help,
}

impl Display for AssemblerMessageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblerMessageType::Error => write!(f, "{}", "error".red()),
            AssemblerMessageType::Warning => write!(f, "{}", "warning".yellow()),
            AssemblerMessageType::Help => write!(f, "{}", "help".blue()),
        }
    }
}

impl AssemblerMessageType {
    pub(crate) fn get_color(&self) -> Color {
        match self {
            AssemblerMessageType::Error => Color::Red,
            AssemblerMessageType::Warning => Color::Yellow,
            AssemblerMessageType::Help => Color::Blue,
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
    let iter = code.chars().peekable();
    let mut line = 1;
    let mut context = String::new();

    for c in iter {
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
