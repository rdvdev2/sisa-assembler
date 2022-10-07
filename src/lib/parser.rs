use crate::assembler::message::{AssemblerMessage, AssemblerMessageType};
use crate::nodes::*;
use crate::span::Span;
use crate::tokens::Token::{self, *};
use easy_nodes::Node;
use plex::parser;

pub struct Parser<T: Iterator<Item = (Token, Span)>> {
    tokens: T,
}

#[allow(unused_braces)]                  // parser! {} generates a lot of those warnings
#[allow(clippy::redundant_closure_call)] // same deal
#[allow(clippy::ptr_arg)]                // some more
impl<T: Iterator<Item = (Token, Span)>> Parser<T> {
    parser! {
        fn _parse(Token, Span);

        (a, b) {
            Span {
                lo: a.lo,
                hi: b.hi
            }
        }

        program: Node<Span, Program> {
            data_section[ds] End => {
                let mut p = Program::empty();
                p.data_section = Some(ds);
                p.to_node(span!())
            }
            text_section[ts] End => {
                let mut p = Program::empty();
                p.text_section = Some(ts);
                p.to_node(span!())
            }
            data_section[ds] text_section[ts] End => {
                let mut p = Program::empty();
                p.data_section = Some(ds);
                p.text_section = Some(ts);
                p.to_node(span!())
            }
            text_section[ts] data_section[ds] End => {
                let mut p = Program::empty();
                p.data_section = Some(ds);
                p.text_section = Some(ts);
                p.to_node(span!())
            }
            constant[c] program[mut p] => {
                p.get_data_mut().constants.push(c);
                p
            }
        }

        data_section: Node<Span, DataSection> {
            BeginData => DataSection::empty().to_node(span!()),
            data_section[mut ds] statement[s] => {
                ds.get_data_mut().statements.push(s);
                *ds.get_common_mut() = span!();
                ds
            }
        }

        text_section: Node<Span, TextSection> {
            BeginText => TextSection::empty().to_node(span!()),
            text_section[mut ts] statement[s] => {
                ts.get_data_mut().statements.push(s);
                *ts.get_common_mut() = span!();
                ts
            }
        }

        statement: Node<Span, Statement> {
            instruction[i] => Statement::Instruction(i).to_node(span!()),
            label[l] => Statement::Label(l).to_node(span!()),
            raw_data[r] => Statement::RawData(r).to_node(span!()),
            constant[c] => Statement::Constant(c).to_node(span!()),
        }

        instruction: Node<Span, Instruction> {
            And reg[rd] Comma reg[ra] Comma reg[rb] => Instruction::And { rd, ra, rb }.to_node(span!()),
            Or  reg[rd] Comma reg[ra] Comma reg[rb] => Instruction::Or  { rd, ra, rb }.to_node(span!()),
            Xor reg[rd] Comma reg[ra] Comma reg[rb] => Instruction::Xor { rd, ra, rb }.to_node(span!()),
            Not reg[rd] Comma reg[ra]               => Instruction::Not { rd, ra }.to_node(span!()),
            Add reg[rd] Comma reg[ra] Comma reg[rb] => Instruction::Add { rd, ra, rb }.to_node(span!()),
            Sub reg[rd] Comma reg[ra] Comma reg[rb] => Instruction::Sub { rd, ra, rb }.to_node(span!()),
            Sha reg[rd] Comma reg[ra] Comma reg[rb] => Instruction::Sha { rd, ra, rb }.to_node(span!()),
            Shl reg[rd] Comma reg[ra] Comma reg[rb] => Instruction::Shl { rd, ra, rb }.to_node(span!()),

            Cmplt  reg[rd] Comma reg[ra] Comma reg[rb] => Instruction::Cmplt  { rd, ra, rb }.to_node(span!()),
            Cmple  reg[rd] Comma reg[ra] Comma reg[rb] => Instruction::Cmple  { rd, ra, rb }.to_node(span!()),
            Cmpeq  reg[rd] Comma reg[ra] Comma reg[rb] => Instruction::Cmpeq  { rd, ra, rb }.to_node(span!()),
            Cmpltu reg[rd] Comma reg[ra] Comma reg[rb] => Instruction::Cmpltu { rd, ra, rb }.to_node(span!()),
            Cmpleu reg[rd] Comma reg[ra] Comma reg[rb] => Instruction::Cmpleu { rd, ra, rb }.to_node(span!()),

            Addi reg[rd] Comma reg[ra] Comma lit[n6] => Instruction::Addi { rd, ra, n6 }.to_node(span!()),

            Ld  reg[rd] Comma lit[n6] Lpar reg[ra] Rpar => Instruction::Ld  { rd, n6, ra }.to_node(span!()),
            Ldb reg[rd] Comma lit[n6] Lpar reg[ra] Rpar => Instruction::Ldb { rd, n6, ra }.to_node(span!()),
            St  lit[n6] Lpar reg[ra] Rpar Comma reg[rb] => Instruction::St  { n6, ra, rb }.to_node(span!()),
            Stb lit[n6] Lpar reg[ra] Rpar Comma reg[rb] => Instruction::Stb { n6, ra, rb }.to_node(span!()),

            Jalr reg[rd] Comma reg[ra] => Instruction::Jalr { rd, ra }.to_node(span!()),
            Bz   reg[ra] Comma lit[n8] => Instruction::Bz   { ra, n8 }.to_node(span!()),
            Bnz  reg[ra] Comma lit[n8] => Instruction::Bnz  { ra, n8 }.to_node(span!()),

            Movi  reg[rd] Comma lit[n8] => Instruction::Movi  { rd, n8 }.to_node(span!()),
            Movhi reg[rd] Comma lit[n8] => Instruction::Movhi { rd, n8 }.to_node(span!()),

            In  reg[rd] Comma lit[n8] => Instruction::In  { rd, n8 }.to_node(span!()),
            Out lit[n8] Comma reg[ra] => Instruction::Out { n8, ra }.to_node(span!()),

            Nop => Instruction::Nop.to_node(span!()),
        }

        raw_data: Node<Span, RawData> {
            bytes[b] => RawData::Bytes(b).to_node(span!()),
            words[w] => RawData::Words(w).to_node(span!()),
            Space Lit(bytes) => RawData::Bytes(vec![Literal::Constant(0).to_node(span!());bytes as usize]).to_node(span!()),
            Space Lit(bytes) Comma lit[data] => RawData::Bytes(vec![data;bytes as usize]).to_node(span!()),
            Even => RawData::WordAlign.to_node(span!()),
        }

        bytes: Vec<Node<Span, Literal>> {
            Byte lit[b] => vec![b],
            bytes[mut bytes] Comma lit[b] => {
                bytes.push(b);
                bytes
            }
        }

        words: Vec<Node<Span, Literal>> {
            Word lit[w] => vec![w],
            words[mut words] Comma lit[w] => {
                words.push(w);
                words
            }
        }

        reg: Node<Span, Registry> {
            Reg(reg) => Registry { reg }.to_node(span!())
        }

        lit: Node<Span, Literal> {
            Lit(val) => Literal::Constant(val).to_node(span!()),
            symbol_ref[sr] => Literal::SymbolRef(sr).to_node(span!()),
            function[f] => Literal::Function(f).to_node(span!()),
        }

        label: Node<Span, Label> {
            Ident(label) Colon => Label { label }.to_node(span!())
        }

        symbol_ref: Node<Span, SymbolRef> {
            Ident(name) => SymbolRef { name }.to_node(span!())
        }

        function: Node<Span, Function> {
            Lo Lpar lit[l] Rpar => Function::Lo(l).to_node(span!()),
            Hi Lpar lit[l] Rpar => Function::Hi(l).to_node(span!()),
        }

        constant: Node<Span, Constant> {
            Set Ident(name) Comma lit[value] => Constant::new(name, value).to_node(span!()),
            Ident(name) Equals lit[value] => Constant::new(name, value).to_node(span!()),
        }
    }

    pub fn new(tokens: T) -> Self {
        Self { tokens }
    }

    pub fn parse(self) -> Result<Node<Span, Program>, AssemblerMessage> {
        Self::_parse(self.tokens).map_err(|(element, description)| {
            if let Some((_, span)) = element {
                AssemblerMessage {
                    msg_type: AssemblerMessageType::Error,
                    description: description.into(),
                    span: span.into(),
                }
            } else {
                AssemblerMessage {
                    msg_type: AssemblerMessageType::Error,
                    description: description.into(),
                    span: None,
                }
            }
        })
    }
}
