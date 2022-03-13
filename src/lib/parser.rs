use crate::assembler::message::{AssemblerMessage, AssemblerMessageType};
use crate::nodes::*;
use crate::span::Span;
use crate::tokens::Token::{self, *};
use easy_nodes::Node;
use plex::parser;

pub struct Parser<T: Iterator<Item = (Token, Span)>> {
    tokens: T,
}

#[allow(unused_braces)] // parser! {} generates a lot of those warnings
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
            data_section[ds] END => {
                let mut p = Program::empty();
                p.data_section = Some(ds);
                p.to_node(span!())
            }
            text_section[ts] END => {
                let mut p = Program::empty();
                p.text_section = Some(ts);
                p.to_node(span!())
            }
            data_section[ds] text_section[ts] END => {
                let mut p = Program::empty();
                p.data_section = Some(ds);
                p.text_section = Some(ts);
                p.to_node(span!())
            }
            text_section[ts] data_section[ds] END => {
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
            BEGIN_DATA => DataSection::empty().to_node(span!()),
            data_section[mut ds] statement[s] => {
                ds.get_data_mut().statements.push(s);
                *ds.get_common_mut() = span!();
                ds
            }
        }

        text_section: Node<Span, TextSection> {
            BEGIN_TEXT => TextSection::empty().to_node(span!()),
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
            AND reg[rd] COMMA reg[ra] COMMA reg[rb] => Instruction::And { rd, ra, rb }.to_node(span!()),
            OR  reg[rd] COMMA reg[ra] COMMA reg[rb] => Instruction::Or  { rd, ra, rb }.to_node(span!()),
            XOR reg[rd] COMMA reg[ra] COMMA reg[rb] => Instruction::Xor { rd, ra, rb }.to_node(span!()),
            NOT reg[rd] COMMA reg[ra]               => Instruction::Not { rd, ra }.to_node(span!()),
            ADD reg[rd] COMMA reg[ra] COMMA reg[rb] => Instruction::Add { rd, ra, rb }.to_node(span!()),
            SUB reg[rd] COMMA reg[ra] COMMA reg[rb] => Instruction::Sub { rd, ra, rb }.to_node(span!()),
            SHA reg[rd] COMMA reg[ra] COMMA reg[rb] => Instruction::Sha { rd, ra, rb }.to_node(span!()),
            SHL reg[rd] COMMA reg[ra] COMMA reg[rb] => Instruction::Shl { rd, ra, rb }.to_node(span!()),

            CMPLT  reg[rd] COMMA reg[ra] COMMA reg[rb] => Instruction::Cmplt  { rd, ra, rb }.to_node(span!()),
            CMPLE  reg[rd] COMMA reg[ra] COMMA reg[rb] => Instruction::Cmple  { rd, ra, rb }.to_node(span!()),
            CMPEQ  reg[rd] COMMA reg[ra] COMMA reg[rb] => Instruction::Cmpeq  { rd, ra, rb }.to_node(span!()),
            CMPLTU reg[rd] COMMA reg[ra] COMMA reg[rb] => Instruction::Cmpltu { rd, ra, rb }.to_node(span!()),
            CMPLEU reg[rd] COMMA reg[ra] COMMA reg[rb] => Instruction::Cmpleu { rd, ra, rb }.to_node(span!()),

            ADDI reg[rd] COMMA reg[ra] COMMA lit[n6] => Instruction::Addi { rd, ra, n6 }.to_node(span!()),

            LD  reg[rd] COMMA lit[n6] LPAR reg[ra] RPAR => Instruction::Ld  { rd, n6, ra }.to_node(span!()),
            LDB reg[rd] COMMA lit[n6] LPAR reg[ra] RPAR => Instruction::Ldb { rd, n6, ra }.to_node(span!()),
            ST  lit[n6] LPAR reg[ra] RPAR COMMA reg[rb] => Instruction::St  { n6, ra, rb }.to_node(span!()),
            STB lit[n6] LPAR reg[ra] RPAR COMMA reg[rb] => Instruction::Stb { n6, ra, rb }.to_node(span!()),

            JALR reg[rd] COMMA reg[ra] => Instruction::Jalr { rd, ra }.to_node(span!()),
            BZ   reg[ra] COMMA lit[n8] => Instruction::Bz   { ra, n8 }.to_node(span!()),
            BNZ  reg[ra] COMMA lit[n8] => Instruction::Bnz  { ra, n8 }.to_node(span!()),

            MOVI  reg[rd] COMMA lit[n8] => Instruction::Movi  { rd, n8 }.to_node(span!()),
            MOVHI reg[rd] COMMA lit[n8] => Instruction::Movhi { rd, n8 }.to_node(span!()),

            IN  reg[rd] COMMA lit[n8] => Instruction::In  { rd, n8 }.to_node(span!()),
            OUT lit[n8] COMMA reg[ra] => Instruction::Out { n8, ra }.to_node(span!()),
        }

        raw_data: Node<Span, RawData> {
            bytes[b] => RawData::Bytes(b).to_node(span!()),
            words[w] => RawData::Words(w).to_node(span!()),
            SPACE LIT(bytes) => RawData::Bytes(vec![Literal::Constant(0).to_node(span!());bytes as usize]).to_node(span!()),
            SPACE LIT(bytes) COMMA lit[data] => RawData::Bytes(vec![data;bytes as usize]).to_node(span!()),
            EVEN => RawData::WordAlign.to_node(span!()),
        }

        bytes: Vec<Node<Span, Literal>> {
            BYTE lit[b] => vec![b],
            bytes[mut bytes] COMMA lit[b] => {
                bytes.push(b);
                bytes
            }
        }

        words: Vec<Node<Span, Literal>> {
            WORD lit[w] => vec![w],
            words[mut words] COMMA lit[w] => {
                words.push(w);
                words
            }
        }

        reg: Node<Span, Registry> {
            REG(reg) => Registry { reg }.to_node(span!())
        }

        lit: Node<Span, Literal> {
            LIT(val) => Literal::Constant(val).to_node(span!()),
            symbol_ref[sr] => Literal::SymbolRef(sr).to_node(span!()),
            function[f] => Literal::Function(f).to_node(span!()),
        }

        label: Node<Span, Label> {
            IDENT(label) COLON => Label { label }.to_node(span!())
        }

        symbol_ref: Node<Span, SymbolRef> {
            IDENT(name) => SymbolRef { name }.to_node(span!())
        }

        function: Node<Span, Function> {
            LO LPAR lit[l] RPAR => Function::Lo(l).to_node(span!()),
            HI LPAR lit[l] RPAR => Function::Hi(l).to_node(span!()),
        }

        constant: Node<Span, Constant> {
            SET IDENT(name) COMMA lit[value] => Constant::new(name, value).to_node(span!()),
            IDENT(name) EQUALS lit[value] => Constant::new(name, value).to_node(span!()),
        }
    }

    pub fn new(tokens: T) -> Self {
        Self { tokens }
    }

    pub fn parse(self) -> Result<Node<Span, Program>, AssemblerMessage> {
        Self::_parse(self.tokens).map_err(|(element, description)| {
            if let Some((_, span)) = element {
                AssemblerMessage {
                    msg_type: AssemblerMessageType::ERROR,
                    description: description.into(),
                    span: span.into(),
                }
            } else {
                AssemblerMessage {
                    msg_type: AssemblerMessageType::ERROR,
                    description: description.into(),
                    span: None,
                }
            }
        })
    }
}
