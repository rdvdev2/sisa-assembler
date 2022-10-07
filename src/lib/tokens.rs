#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Invalid(String),
    Ignore,
    And,
    Or,
    Xor,
    Not,
    Add,
    Sub,
    Sha,
    Shl,
    Cmplt,
    Cmple,
    Cmpeq,
    Cmpltu,
    Cmpleu,
    Addi,
    Ld,
    St,
    Ldb,
    Stb,
    Jalr,
    Bz,
    Bnz,
    Movi,
    Movhi,
    In,
    Out,
    Nop,
    Reg(u8),
    Lit(u16),
    Comma,
    Lpar,
    Rpar,
    Colon,
    Ident(String),
    Lo,
    Hi,
    BeginText,
    BeginData,
    End,
    Byte,
    Word,
    Space,
    Even,
    Equals,
    Set,
}

pub fn parse_reg(tok: &str) -> Token {
    Token::Reg(tok.trim_start_matches('R').parse().unwrap())
}

pub fn parse_int_lit(tok: &str) -> Token {
    Token::Lit(tok.parse::<i16>().unwrap() as u16)
}

pub fn parse_hex_lit(tok: &str) -> Token {
    Token::Lit(
        u16::from_str_radix(tok.trim_start_matches("0x").trim_start_matches("0X"), 16).unwrap(),
    )
}
