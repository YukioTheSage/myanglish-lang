use mlang::lexer::Lexer;
use mlang::token::TokenKind;
use tower_lsp::lsp_types::*;

/// Semantic token types used by the server.
/// Must match the legend order exactly.
const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::KEYWORD,  // 0
    SemanticTokenType::VARIABLE, // 1
    SemanticTokenType::FUNCTION, // 2
    SemanticTokenType::NUMBER,   // 3
    SemanticTokenType::STRING,   // 4
    SemanticTokenType::OPERATOR, // 5
    SemanticTokenType::TYPE,     // 6
    SemanticTokenType::COMMENT,  // 7
];

pub fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TOKEN_TYPES.to_vec(),
        token_modifiers: vec![],
    }
}

fn token_type_index(kind: &TokenKind) -> Option<u32> {
    match kind {
        // Keywords
        TokenKind::Hlyin
        | TokenKind::Mo
        | TokenKind::Pat
        | TokenKind::Htae
        | TokenKind::Kyoe
        | TokenKind::NautSone
        | TokenKind::SetSae
        | TokenKind::Loke
        | TokenKind::Pyan
        | TokenKind::Pya
        | TokenKind::Phat
        | TokenKind::Yu
        | TokenKind::Atote
        | TokenKind::Pay
        | TokenKind::Hman
        | TokenKind::Hmar
        | TokenKind::Bhala
        | TokenKind::Pone
        | TokenKind::Nee
        | TokenKind::Myat => Some(0), // KEYWORD

        // Types
        TokenKind::Kain
        | TokenKind::Sar
        | TokenKind::Sit
        | TokenKind::Su
        | TokenKind::Laung
        | TokenKind::Baung
        | TokenKind::Twe
        | TokenKind::DaTha
        | TokenKind::Amhar => {
            Some(6) // TYPE
        }

        // Literals
        TokenKind::Number(_) => Some(3),        // NUMBER
        TokenKind::FloatLiteral(_) => Some(3),  // NUMBER
        TokenKind::StringLiteral(_) => Some(4), // STRING

        // Identifiers (could be variable or function — we mark as variable here;
        // a richer pass could disambiguate)
        TokenKind::Identifier(_) => Some(1), // VARIABLE

        // Operators
        TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::Slash
        | TokenKind::Assign
        | TokenKind::Equals
        | TokenKind::NotEquals
        | TokenKind::GreaterThan
        | TokenKind::LessThan
        | TokenKind::GreaterEquals
        | TokenKind::LessEquals
        | TokenKind::Arrow => Some(5), // OPERATOR

        TokenKind::Dot => Some(5), // OPERATOR

        _ => None,
    }
}

/// Estimate the display length of a token.
fn token_length(kind: &TokenKind) -> u32 {
    match kind {
        TokenKind::Kain => 4,     // "kain"
        TokenKind::Sar => 3,      // "sar"
        TokenKind::Sit => 3,      // "sit"
        TokenKind::Hman => 4,     // "hman"
        TokenKind::Hmar => 4,     // "hmar"
        TokenKind::Hlyin => 5,    // "hlyin"
        TokenKind::Mo => 2,       // "mo"
        TokenKind::Pat => 3,      // "pat"
        TokenKind::Htae => 4,     // "htae"
        TokenKind::Kyoe => 5,     // "kyoe"
        TokenKind::NautSone => 9, // "naut_sone"
        TokenKind::SetSae => 7,   // "set_sae"
        TokenKind::Loke => 4,     // "loke"
        TokenKind::Pyan => 4,     // "pyan"
        TokenKind::Pya => 3,      // "pya"
        TokenKind::Phat => 4,     // "phat"
        TokenKind::Su => 2,       // "su"
        TokenKind::Laung => 5,    // "laung"
        TokenKind::Baung => 5,    // "baung"
        TokenKind::Yu => 2,       // "yu"
        TokenKind::Atote => 5,    // "atote"
        TokenKind::Pay => 3,      // "pay"
        TokenKind::Twe => 3,      // "twe"
        TokenKind::DaTha => 6,    // "da_tha"
        TokenKind::Bhala => 5,    // "bhala"
        TokenKind::Pone => 4,     // "pone"
        TokenKind::Nee => 3,      // "nee"
        TokenKind::Myat => 4,     // "myat"
        TokenKind::Amhar => 5,    // "amhar"
        TokenKind::FloatLiteral(f) => format!("{}", f).len() as u32,
        TokenKind::Dot => 1,
        TokenKind::Identifier(s) => s.chars().count() as u32,
        TokenKind::Number(n) => format!("{}", n).len() as u32, // rough
        TokenKind::StringLiteral(s) => (s.chars().count() + 2) as u32, // +2 for quotes
        TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::Slash
        | TokenKind::Assign
        | TokenKind::GreaterThan
        | TokenKind::LessThan => 1,
        TokenKind::Equals
        | TokenKind::NotEquals
        | TokenKind::GreaterEquals
        | TokenKind::LessEquals
        | TokenKind::Arrow => 2,
        _ => 1,
    }
}

pub fn get_semantic_tokens(source: &str) -> Vec<SemanticToken> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    let mut prev_line: u32 = 0;
    let mut prev_col: u32 = 0;

    loop {
        let tok = lexer.next_token();
        if tok.kind == TokenKind::Eof {
            break;
        }

        if let Some(type_index) = token_type_index(&tok.kind) {
            let line = (tok.line as u32).saturating_sub(1); // 0-based
            let col = (tok.column as u32).saturating_sub(1); // 0-based

            let delta_line = line - prev_line;
            let delta_start = if delta_line == 0 { col - prev_col } else { col };

            tokens.push(SemanticToken {
                delta_line,
                delta_start,
                length: token_length(&tok.kind),
                token_type: type_index,
                token_modifiers_bitset: 0,
            });

            prev_line = line;
            prev_col = col;
        }
    }

    tokens
}
