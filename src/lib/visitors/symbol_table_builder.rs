use crate::assembler::message::{AssemblerMessage, AssemblerMessageType};
use crate::nodes::_node_traits::NodeVisitor as nvst;
use crate::nodes::*;
use crate::symbol_table::SymbolTable;
use crate::{DataSectionStart, Flags, Span};
use easy_nodes::Node;

pub struct SymbolTableBuilder<'a> {
    current_pos: u16,
    symbol_table: SymbolTable,
    messages: Vec<AssemblerMessage>,
    flags: &'a Flags,
}

impl<'a> SymbolTableBuilder<'a> {
    pub fn new(flags: &'a Flags) -> Self {
        Self {
            current_pos: 0,
            symbol_table: SymbolTable::new(),
            messages: Vec::new(),
            flags,
        }
    }

    fn put_constant(&mut self, name: String, value: u16) -> Result<(), String> {
        self.symbol_table.put_constant(name, value)
    }

    fn put_current_address(&mut self, label: String) -> Result<(), String> {
        self.symbol_table.put_address(label, self.current_pos)
    }

    #[allow(unused_must_use)] // The result will be used when getting messages
    pub fn build(&mut self, node: &Node<Span, Program>) {
        node.accept(self);

        if !self.symbol_table.is_valid_layout() {
            self.messages.push(AssemblerMessage {
                msg_type: AssemblerMessageType::Error,
                description: String::from("sections .text and .code are overlaping"),
                span: None,
            })
        }
    }

    pub fn get_messages(&self) -> Vec<AssemblerMessage> {
        self.messages.clone()
    }

    pub fn get_symbol_table(self) -> SymbolTable {
        self.symbol_table
    }
}

impl<'a> NodeVisitor<()> for SymbolTableBuilder<'a> {
    fn visit_program(&mut self, _span: &Span, program: &Program) {
        self.current_pos = self.flags.text_section_start;
        let text_start = self.current_pos;
        if let Some(ts) = &program.text_section {
            ts.accept(self);
        }
        self.symbol_table
            .set_text_section(text_start, self.current_pos - text_start);

        if let DataSectionStart::Absolute(pos) = self.flags.data_section_start {
            self.current_pos = pos;
        }
        let data_start = self.current_pos;
        if let Some(ds) = &program.data_section {
            ds.accept(self);
        }
        self.symbol_table
            .set_data_section(data_start, self.current_pos - data_start);

        for constant in &program.constants {
            constant.accept(self);
        }
    }

    fn visit_data_section(&mut self, _span: &Span, ds: &DataSection) {
        if self.flags.auto_align_sections && self.current_pos % 2 != 0 {
            self.current_pos += 1;
        }
        for statement in &ds.statements {
            statement.accept(self);
        }
    }

    fn visit_text_section(&mut self, _span: &Span, ts: &TextSection) {
        if self.flags.auto_align_sections && self.current_pos % 2 != 0 {
            self.current_pos += 1;
        }
        for statement in &ts.statements {
            statement.accept(self);
        }
    }

    fn visit_statement(&mut self, _span: &Span, statement: &Statement) {
        match statement {
            Statement::Instruction(i) => i.accept(self),
            Statement::Label(l) => l.accept(self),
            Statement::RawData(r) => r.accept(self),
            Statement::Constant(c) => c.accept(self),
        }
    }

    fn visit_instruction(&mut self, _span: &Span, _node: &Instruction) {
        self.current_pos += 2;
    }

    fn visit_raw_data(&mut self, _span: &Span, raw_data: &RawData) {
        if let RawData::Words(_) = raw_data {
            if self.flags.auto_align_words && self.current_pos % 2 != 0 {
                self.current_pos += 1;
            }
        }

        self.current_pos += raw_data.get_size(self.current_pos);
    }

    fn visit_label(&mut self, span: &Span, label: &Label) {
        if let Err(e) = self.put_current_address(label.label.clone()) {
            self.messages.push(AssemblerMessage {
                msg_type: AssemblerMessageType::Error,
                description: e,
                span: Some(*span),
            });
        }
    }

    fn visit_constant(&mut self, span: &Span, constant: &Constant) {
        if let Err(e) = match constant.value.get_data() {
            Literal::Constant(c) => self.put_constant(constant.name.clone(), *c),
            _ => Err(String::from(
                ".set directive only accepts constant values! (Not functions nor labels)",
            )),
        } {
            self.messages.push(AssemblerMessage {
                msg_type: AssemblerMessageType::Error,
                description: e,
                span: Some(*span),
            })
        }
    }
}
