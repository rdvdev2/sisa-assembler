use crate::span::{FileLoc, Span};
use crate::tokens::Token::*;
use crate::tokens::*;
use plex::lexer;

pub struct Lexer<'a> {
    input: &'a str,
    remaining: &'a str,
    cur_line: usize,
    cur_col: usize,
}

impl<'a> Lexer<'a> {
    lexer! {
        fn next_token(tok) -> Token;

        r";[^\n]*\n" => Ignore,

        r"\.text" => BeginText,
        r"\.data" => BeginData,
        r"\.end" => End,

        r"\.byte" => Byte,
        r"\.word" => Word,
        r"\.space" => Space,
        r"\.even" => Even,

        r"\.set" => Set,

        r"," => Comma,
        r"\(" => Lpar,
        r"\)" => Rpar,
        r":" => Colon,
        r"=" => Equals,

        r"AND" => And,
        r"OR" => Or,
        r"XOR" => Xor,
        r"NOT" => Not,
        r"ADD" => Add,
        r"SUB" => Sub,
        r"SHA" => Sha,
        r"SHL" => Shl,
        r"CMPLT" => Cmplt,
        r"CMPLE" => Cmple,
        r"CMPEQ" => Cmpeq,
        r"CMPLTU" => Cmpltu,
        r"CMPLEU" => Cmpleu,
        r"ADDI" => Addi,
        r"LD" => Ld,
        r"ST" => St,
        r"LDB" => Ldb,
        r"STB" => Stb,
        r"JALR" => Jalr,
        r"BZ" => Bz,
        r"BNZ" => Bnz,
        r"MOVI" => Movi,
        r"MOVHI" => Movhi,
        r"IN" => In,
        r"OUT" => Out,
        r"NOP" => Nop,

        r"R[0-7]" => parse_reg(tok),
        r"[0-9]+" => parse_int_lit(tok),
        r"[-+][0-9]+" => parse_int_lit(tok),
        r"0(x|X)[0-9a-fA-f]+" => parse_hex_lit(tok),

        r"lo" => Lo,
        r"hi" => Hi,

        r"[a-zA-Z][a-zA-Z0-9\_\-]*" => Ident(tok.into()),

        r"[\n\t\s\r ]+" => Ignore,
        r"." => Invalid(tok.into())
    }

    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            remaining: input,
            cur_line: 1,
            cur_col: 1,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (Token, Span);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (token, remaining) = match Lexer::next_token(self.remaining) {
                Some((t, r)) => (t, r),
                _ => return None,
            };

            let start = self.input.len() - self.remaining.len();
            let end = self.input.len() - remaining.len();
            let lo = FileLoc {
                line: self.cur_line,
                col: self.cur_col,
            };

            for c in self.input.get(start..end).unwrap().chars() {
                match c {
                    '\n' => {
                        self.cur_col = 1;
                        self.cur_line += 1;
                    }
                    _ => self.cur_col += 1,
                }
            }

            if let Token::Ignore = token {
                self.remaining = remaining;
            } else {
                self.remaining = remaining;

                if let Token::End = token {
                    self.remaining = "";
                }

                return Some((
                    token,
                    Span {
                        lo,
                        hi: FileLoc {
                            line: self.cur_line,
                            col: self.cur_col,
                        },
                    },
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches::assert_matches;

    #[test]
    fn lex_instruction() {
        let mut lexer = Lexer::new("LDB R1, 32(R3)");

        assert_matches!(lexer.next(), Some((Token::Ldb, _)));
        assert_matches!(lexer.next(), Some((Token::Reg(1), _)));
        assert_matches!(lexer.next(), Some((Token::Comma, _)));
        assert_matches!(lexer.next(), Some((Token::Lit(32), _)));
        assert_matches!(lexer.next(), Some((Token::Lpar, _)));
        assert_matches!(lexer.next(), Some((Token::Reg(3), _)));
        assert_matches!(lexer.next(), Some((Token::Rpar, _)));
        assert_matches!(lexer.next(), None);
    }

    #[test]
    fn lex_ident() {
        let mut lexer = Lexer::new("lab: (lab)");

        match lexer.next() {
            Some((Token::Ident(name), _)) => assert_eq!(name, "lab"),
            _ => panic!(),
        }
        assert_matches!(lexer.next(), Some((Token::Colon, _)));
        assert_matches!(lexer.next(), Some((Token::Lpar, _)));
        match lexer.next() {
            Some((Token::Ident(name), _)) => assert_eq!(name, "lab"),
            _ => panic!(),
        }
        assert_matches!(lexer.next(), Some((Token::Rpar, _)));
        assert_matches!(lexer.next(), None);
    }

    #[test]
    fn lex_invalid() {
        let mut lexer = Lexer::new(".");

        assert_matches!(lexer.next(), Some((Token::Invalid(_), _)));
        assert_matches!(lexer.next(), None);
    }

    #[test]
    fn lex_whitespace() {
        let mut lexer = Lexer::new("    \n    \t   ");

        assert_matches!(lexer.next(), None);
    }

    #[test]
    fn lex_functions() {
        let mut lexer = Lexer::new("lo(0) hi(0)");

        assert_matches!(lexer.next(), Some((Token::Lo, _)));
        assert_matches!(lexer.next(), Some((Token::Lpar, _)));
        assert_matches!(lexer.next(), Some((Token::Lit(0), _)));
        assert_matches!(lexer.next(), Some((Token::Rpar, _)));
        assert_matches!(lexer.next(), Some((Token::Hi, _)));
        assert_matches!(lexer.next(), Some((Token::Lpar, _)));
        assert_matches!(lexer.next(), Some((Token::Lit(0), _)));
        assert_matches!(lexer.next(), Some((Token::Rpar, _)));
        assert_matches!(lexer.next(), None);
    }

    #[test]
    fn lex_directives() {
        let mut lexer = Lexer::new(".text .data .byte .word .space .even .end .text");

        assert_matches!(lexer.next(), Some((Token::BeginText, _)));
        assert_matches!(lexer.next(), Some((Token::BeginData, _)));
        assert_matches!(lexer.next(), Some((Token::Byte, _)));
        assert_matches!(lexer.next(), Some((Token::Word, _)));
        assert_matches!(lexer.next(), Some((Token::Space, _)));
        assert_matches!(lexer.next(), Some((Token::Even, _)));
        assert_matches!(lexer.next(), Some((Token::End, _)));
        assert_matches!(lexer.next(), None);
    }

    #[test]
    fn span() {
        let mut lexer = Lexer::new("    \n MOVI");

        assert_matches!(
            lexer.next(),
            Some((
                _,
                Span {
                    lo: FileLoc { col: 2, line: 2 },
                    hi: FileLoc { col: 6, line: 2 }
                }
            ))
        );
        assert_matches!(lexer.next(), None);
    }
}
