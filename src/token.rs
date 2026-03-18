#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    // Keywords (Myanglish)
    Kain,     // kain (int)
    Sar,      // sar (string)
    Sit,      // sit (bool)
    DaTha,    // da_tha (float)
    Hman,     // hman (true)
    Hmar,     // hmar (false)
    Bhala,    // bhala (nil/null)
    Hlyin,    // hlyin (if)
    Mo,       // mo (else)
    Pat,      // pat (while)
    Htae,     // htae (for-in)
    Laung,    // laung (channel type/make)
    Baung,    // baung (context type/make)
    Break,    // yut (break)
    Continue, // shar (continue)
    Kyoe,     // kyoe (go/goroutine-style)
    NautSone, // naut_sone (defer-style)
    SetSae,   // set_sae (test declaration)
    Loke,     // loke (function)
    Pyan,     // pyan (return)
    Pya,      // pya (print)
    Phat,     // phat (read)
    Su,       // su (array)
    Yu,       // yu (import)
    Twe,      // twe (hashmap)
    Atote,    // atote (package)
    Pay,      // pay (export)
    Pone,     // pone (struct)
    Nee,      // nee (method)
    Myat,     // myat (interface)
    Amhar,    // amhar (error type)

    // Identifiers and Literals
    Identifier(String),
    Number(i64),
    FloatLiteral(f64),
    StringLiteral(String),

    // Symbols and Operators
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    Assign,        // =
    Equals,        // ==
    NotEquals,     // !=
    GreaterThan,   // >
    LessThan,      // <
    GreaterEquals, // >=
    LessEquals,    // <=
    Dot,           // .

    // Punctuation
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    LBracket,  // [
    RBracket,  // ]
    Comma,     // ,
    Semicolon, // ;
    Colon,     // :
    Arrow,     // ->

    Comment(String), // // comment text

    Eof,     // End of File
    Illegal, // Unrecognized Character
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}
