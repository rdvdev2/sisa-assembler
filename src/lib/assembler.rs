use std::fmt::{Display, Formatter};
use colorful::{Color, Colorful};
use crate::{Flags, Lexer, MachineCodeGenerator, Parser, SymbolTableBuilder};
use crate::span::Span;

pub struct Assembler {
    flags: Flags
}

pub struct AssemblerResult {
    pub machine_code: Option<Vec<u8>>,
    pub assembler_messages: Vec<AssemblerMessage>,
}

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

impl Assembler {
    pub fn new(flags: Flags) -> Self {
        Self {
            flags: flags
        }
    }

    pub fn assemble(&self, code: &str) -> AssemblerResult {
        let mut result = AssemblerResult {
            machine_code: None,
            assembler_messages: Vec::new(),
        };

        let lexer = Lexer::new(&code);
        let parser = Parser::new(lexer);
        let node = match parser.parse() {
            Ok(n) => n,
            Err(e) => {
                result.assembler_messages.push(e);
                return result;
            }
        };

        let mut symbol_table_builder = SymbolTableBuilder::new(&self.flags);
        symbol_table_builder.build(&node);
        result.assembler_messages.extend(symbol_table_builder.get_messages());
        let symbol_table = symbol_table_builder.get_symbol_table();

        let mut machine_code_generator = MachineCodeGenerator::new(&symbol_table, &self.flags);
        result.machine_code = machine_code_generator.generate(&node);
        result.assembler_messages.extend(machine_code_generator.get_messages());

        if result.assembler_messages.iter()
            .any(|msg| msg.msg_type == AssemblerMessageType::ERROR)
        {
            result.machine_code = None;
        }

        result
    }
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