use crate::assembler::{AssemblerMessage, AssemblerMessageType};
use crate::{DataSectionStart, Flags};
use crate::nodes::*;
use crate::symbol_table::SymbolTable;
use crate::visitors::Visitor;

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
            flags
        }
    }

    fn put_constant(&mut self, name: String, value: u16) -> Result<(), String> {
        self.symbol_table.put_constant(name, value)
    }

    fn put_current_address(&mut self, label: String) -> Result<(), String> {
        self.symbol_table.put_address(label, self.current_pos)
    }

    #[allow(unused_must_use)] // The result will be used when getting messages
    pub fn build(&mut self, node: &ProgramNode) {
        node.accept(self);

        if !self.symbol_table.is_valid_layout() {
            self.messages.push(AssemblerMessage {
                msg_type: AssemblerMessageType::ERROR,
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

impl<'a> Visitor<(), ()> for SymbolTableBuilder<'a> {
    fn visit_program(&mut self, node: &ProgramNode) -> Result<(), ()> {
        self.current_pos = self.flags.text_section_start;
        let text_start = self.current_pos;
        if let Some(ts) = &node.text_section {
            ts.accept(self)?;
        }
        self.symbol_table.set_text_section(text_start, self.current_pos - text_start);

        if let DataSectionStart::Absolute(pos) = self.flags.data_section_start {
            self.current_pos = pos;
        }
        let data_start = self.current_pos;
        if let Some(ds) = &node.data_section {
            ds.accept(self)?;
        }
        self.symbol_table.set_data_section(data_start, self.current_pos - data_start);

        for constant in &node.constants {
            constant.accept(self)?;
        }

        Ok(())
    }

    fn visit_data_section(&mut self, node: &DataSectionNode) -> Result<(), ()> {
        if self.flags.auto_align_sections && self.current_pos % 2 != 0 {
            self.current_pos += 1;
        }
        for statement in &node.statements {
            statement.accept(self)?;
        }
        Ok(())
    }

    fn visit_text_section(&mut self, node: &TextSectionNode) -> Result<(), ()> {
        if self.flags.auto_align_sections && self.current_pos % 2 != 0 {
            self.current_pos += 1;
        }
        for statement in &node.statements {
            statement.accept(self)?;
        }
        Ok(())
    }

    fn visit_statement(&mut self, node: &StatementNode) -> Result<(), ()> {
        match node {
            StatementNode::Instruction(i) => i.accept(self),
            StatementNode::Label(l) => l.accept(self),
            StatementNode::RawData(r) => r.accept(self),
            StatementNode::Constant(c) => c.accept(self),
        }
    }

    fn visit_raw_data(&mut self, node: &RawDataNode) -> Result<(), ()> {
        if let RawDataNode::Words(_) = node {
            if self.flags.auto_align_words && self.current_pos % 2 != 0 {
                self.current_pos += 1;
            }
        }

        self.current_pos += node.get_size(self.current_pos);
        Ok(())
    }

    fn visit_instruction(&mut self, _node: &InstructionNode) -> Result<(), ()> {
        self.current_pos += 2;
        Ok(())
    }

    fn visit_label(&mut self, node: &LabelNode) -> Result<(), ()> {
        if let Err(e) = self.put_current_address(node.label.clone()) {
            self.messages.push(AssemblerMessage {
                msg_type: AssemblerMessageType::ERROR,
                description: e,
                span: None,
            });
        }
        
        Ok(())
    }

    fn visit_constant(&mut self, node: &ConstantNode) -> Result<(), ()> {
        if let Err(e) = match node.value {
            LiteralNode::Constant(c) => self.put_constant(node.name.clone(), c),
            _ => Err(String::from(".set directive only accepts constant values! (Not functions nor labels)")),
        } {
            self.messages.push(AssemblerMessage {
                msg_type: AssemblerMessageType::ERROR,
                description: e,
                span: None,
            })
        }
        
        Ok(())
    }
}