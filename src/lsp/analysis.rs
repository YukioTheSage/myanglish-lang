use mlang::ast::{Program, Statement, Type};
use mlang::lexer::Lexer;
use mlang::parser::{ParseError, Parser};
use mlang::typecheck::{Environment, TypeCheckError, TypeChecker};
use tower_lsp::lsp_types::*;
use ropey::Rope;

/// Represents a symbol found during analysis (variable, function, etc.)
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKindInfo,
    pub ty: Option<Type>,
    pub line: usize,   // 0-based
    pub column: usize,  // 0-based
    pub parameters: Vec<(String, Type)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKindInfo {
    Variable,
    Function,
    Parameter,
    Struct,
    Method,
    Interface,
}

/// Result of analyzing a single document.
#[derive(Debug)]
pub struct AnalysisResult {
    pub parse_errors: Vec<ParseError>,
    pub type_errors: Vec<TypeCheckError>,
    pub symbols: Vec<SymbolInfo>,
    pub program: Option<Program>,
}

/// Run lexer, parser, and type-checker on the source text.
pub fn analyze(source: &str) -> AnalysisResult {
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer);

    let program = parser.parse_program();
    let parse_errors = parser.errors.clone();

    let mut symbols = Vec::new();
    let mut type_errors = Vec::new();

    if let Some(ref prog) = program {
        // Collect symbols from AST
        collect_symbols(prog, source, &mut symbols);

        // Run type checker
        let mut type_checker = TypeChecker::new();
        let mut env = Environment::new();
        type_checker.check_program(prog, &mut env);
        type_errors = type_checker.errors;
    }

    AnalysisResult {
        parse_errors,
        type_errors,
        symbols,
        program,
    }
}

/// Walk the AST and collect declared symbols with their positions.
fn collect_symbols(program: &Program, _source: &str, symbols: &mut Vec<SymbolInfo>) {
    for stmt in &program.statements {
        collect_statement_symbols(stmt, symbols);
    }
}

fn collect_statement_symbols(stmt: &Statement, symbols: &mut Vec<SymbolInfo>) {
    match stmt {
        Statement::PackageDecl { .. } => {}
        Statement::Let { name, ty, name_span, .. } => {
            // Convert 1-based lexer lines to 0-based LSP lines
            symbols.push(SymbolInfo {
                name: name.clone(),
                kind: SymbolKindInfo::Variable,
                ty: Some(ty.clone()),
                line: name_span.line.saturating_sub(1),
                column: name_span.column.saturating_sub(1),
                parameters: vec![],
            });
        }
        Statement::FunctionDecl {
            name,
            parameters,
            return_type,
            body,
            name_span,
        } => {
            symbols.push(SymbolInfo {
                name: name.clone(),
                kind: SymbolKindInfo::Function,
                ty: Some(return_type.clone()),
                line: name_span.line.saturating_sub(1),
                column: name_span.column.saturating_sub(1),
                parameters: parameters.iter().map(|(n, t, _)| (n.clone(), t.clone())).collect(),
            });

            // Parameters as symbols
            for (param_name, param_type, param_span) in parameters {
                symbols.push(SymbolInfo {
                    name: param_name.clone(),
                    kind: SymbolKindInfo::Parameter,
                    ty: Some(param_type.clone()),
                    line: param_span.line.saturating_sub(1),
                    column: param_span.column.saturating_sub(1),
                    parameters: vec![],
                });
            }

            // Recurse into body
            for s in &body.statements {
                collect_statement_symbols(s, symbols);
            }
        }
        Statement::If {
            consequence,
            alternative,
            ..
        } => {
            for s in &consequence.statements {
                collect_statement_symbols(s, symbols);
            }
            if let Some(alt) = alternative {
                match alt {
                    mlang::ast::IfAlternative::Else(block) => {
                        for s in &block.statements {
                            collect_statement_symbols(s, symbols);
                        }
                    }
                    mlang::ast::IfAlternative::ElseIf(elif) => {
                        collect_statement_symbols(elif, symbols);
                    }
                }
            }
        }
        Statement::While { body, .. } => {
            for s in &body.statements {
                collect_statement_symbols(s, symbols);
            }
        }
        Statement::ForIn {
            index,
            iterator,
            collection,
            body,
            name_span,
        } => {
            let inferred_type = match collection {
                mlang::ast::Expression::Identifier(_) => None,
                mlang::ast::Expression::ArrayLiteral { elements } => {
                    if elements.is_empty() {
                        None
                    } else {
                        match &elements[0] {
                            mlang::ast::Expression::IntegerLiteral(_) => Some(Type::Kain),
                            mlang::ast::Expression::StringLiteral(_) => Some(Type::Sar),
                            mlang::ast::Expression::BooleanLiteral(_) => Some(Type::Sit),
                            _ => None,
                        }
                    }
                }
                _ => None,
            };

            symbols.push(SymbolInfo {
                name: iterator.clone(),
                kind: SymbolKindInfo::Variable,
                ty: inferred_type,
                line: name_span.line.saturating_sub(1),
                column: name_span.column.saturating_sub(1),
                parameters: vec![],
            });

            if let Some(index_name) = index {
                symbols.push(SymbolInfo {
                    name: index_name.clone(),
                    kind: SymbolKindInfo::Variable,
                    ty: Some(Type::Kain),
                    line: name_span.line.saturating_sub(1),
                    column: name_span.column.saturating_sub(1),
                    parameters: vec![],
                });
            }

            for s in &body.statements {
                collect_statement_symbols(s, symbols);
            }
        }
        Statement::TestDecl {
            name,
            body,
            name_span,
        } => {
            symbols.push(SymbolInfo {
                name: name.clone(),
                kind: SymbolKindInfo::Function,
                ty: Some(Type::Error),
                line: name_span.line.saturating_sub(1),
                column: name_span.column.saturating_sub(1),
                parameters: vec![],
            });
            for s in &body.statements {
                collect_statement_symbols(s, symbols);
            }
        }
        Statement::ForClassic {
            init,
            condition: _,
            post,
            body,
        } => {
            if let Some(init_stmt) = init {
                collect_statement_symbols(init_stmt, symbols);
            }
            if let Some(post_stmt) = post {
                collect_statement_symbols(post_stmt, symbols);
            }
            for s in &body.statements {
                collect_statement_symbols(s, symbols);
            }
        }
        Statement::LetDestructured { names, value, .. } => {
            for (name, ty, span) in names {
                symbols.push(SymbolInfo {
                    name: name.clone(),
                    kind: SymbolKindInfo::Variable,
                    ty: Some(ty.clone()),
                    line: span.line.saturating_sub(1),
                    column: span.column.saturating_sub(1),
                    parameters: vec![],
                });
            }
        }
        Statement::StructDecl { name, fields, name_span } => {
            symbols.push(SymbolInfo {
                name: name.clone(),
                kind: SymbolKindInfo::Struct,
                ty: Some(Type::Struct(name.clone())),
                line: name_span.line.saturating_sub(1),
                column: name_span.column.saturating_sub(1),
                parameters: fields.clone(),
            });
        }
        Statement::MethodDecl { receiver_type, name, parameters, return_type, body, name_span, .. } => {
            symbols.push(SymbolInfo {
                name: format!("{}.{}", receiver_type, name),
                kind: SymbolKindInfo::Method,
                ty: Some(return_type.clone()),
                line: name_span.line.saturating_sub(1),
                column: name_span.column.saturating_sub(1),
                parameters: parameters.iter().map(|(n, t, _)| (n.clone(), t.clone())).collect(),
            });
            for s in &body.statements {
                collect_statement_symbols(s, symbols);
            }
        }
        Statement::InterfaceDecl { name, name_span, .. } => {
            symbols.push(SymbolInfo {
                name: name.clone(),
                kind: SymbolKindInfo::Interface,
                ty: Some(Type::Interface(name.clone())),
                line: name_span.line.saturating_sub(1),
                column: name_span.column.saturating_sub(1),
                parameters: vec![],
            });
        }
        Statement::Export { statement, .. } => {
            collect_statement_symbols(statement, symbols);
        }
        _ => {}
    }
}

/// Find the definition location of the word at the given position.
pub fn find_definition(
    rope: &Rope,
    pos: Position,
    analysis: &AnalysisResult,
    uri: &Url,
) -> Option<Location> {
    let word = get_word_at_position(rope, pos)?;

    // Search symbols for a matching definition
    for sym in &analysis.symbols {
        if sym.name == word
            && (sym.kind == SymbolKindInfo::Variable
                || sym.kind == SymbolKindInfo::Function
                || sym.kind == SymbolKindInfo::Parameter)
        {
            return Some(Location {
                uri: uri.clone(),
                range: Range {
                    start: Position::new(sym.line as u32, sym.column as u32),
                    end: Position::new(
                        sym.line as u32,
                        (sym.column + sym.name.chars().count()) as u32,
                    ),
                },
            });
        }
    }

    None
}

/// Extract the word (identifier) at the cursor position.
pub fn get_word_at_position(rope: &Rope, pos: Position) -> Option<String> {
    let line_idx = pos.line as usize;
    if line_idx >= rope.len_lines() {
        return None;
    }
    let line = rope.line(line_idx).to_string();
    let col = pos.character as usize;

    // Find word boundaries
    let chars: Vec<char> = line.chars().collect();
    if col >= chars.len() {
        return None;
    }

    let is_ident_char = |c: char| c.is_alphanumeric() || c == '_' || ('\u{1000}' <= c && c <= '\u{109F}');

    if !is_ident_char(chars[col]) {
        return None;
    }

    let mut start = col;
    while start > 0 && is_ident_char(chars[start - 1]) {
        start -= 1;
    }

    let mut end = col;
    while end < chars.len() && is_ident_char(chars[end]) {
        end += 1;
    }

    Some(chars[start..end].iter().collect())
}

/// Format a Type for display.
pub fn format_type(ty: &Type) -> String {
    match ty {
        Type::Kain => "kain (int)".to_string(),
        Type::Sar => "sar (string)".to_string(),
        Type::Sit => "sit (bool)".to_string(),
        Type::Array(inner) => format!("su<{}> (array)", format_type(inner)),
        Type::Channel(inner) => format!("laung<{}> (channel)", format_type(inner)),
        Type::Map(k, v) => format!("twe<{}, {}> (map)", format_type(k), format_type(v)),
        Type::DaTha => "da_tha (float64)".to_string(),
        Type::Baung => "baung (context)".to_string(),
        Type::Nil => "bhala (nil)".to_string(),
        Type::Error => "amhar (error)".to_string(),
        Type::Struct(name) => format!("pone {} (struct)", name),
        Type::Interface(name) => format!("myat {} (interface)", name),
        Type::Tuple(types) => {
            let ts: Vec<String> = types.iter().map(|t| format_type(t)).collect();
            format!("({}) (tuple)", ts.join(", "))
        }
        Type::Function {
            params,
            return_type,
        } => {
            let ps: Vec<String> = params.iter().map(format_type).collect();
            format!("loke({}) -> {} (function)", ps.join(", "), format_type(return_type))
        }
    }
}
