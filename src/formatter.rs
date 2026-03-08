use crate::ast::{BlockStatement, Expression, IfAlternative, Program, Statement, Type};
use crate::lexer::{tokenize_all, Lexer};
use crate::parser::Parser;
use crate::token::TokenKind;

/// Information about a comment extracted from the token stream.
#[derive(Debug, Clone)]
struct CommentInfo {
    _line: usize,
    _column: usize,
    _inline: bool,
    text: String, // the text after '//'
}

/// Information about a number literal's original representation.
#[derive(Debug, Clone)]
struct NumberInfo {
    value: i64,
    original: String, // the original text (Myanmar or ASCII digits)
}

const INDENT_UNIT: &str = "    ";
const MAX_INLINE_WIDTH: usize = 80;
const COLLECTION_WRAP_THRESHOLD: usize = 3;

/// Main entry point: formats M-Lang source code.
/// Returns Ok(formatted) or Err(list of error messages).
pub fn format_source(source: &str) -> Result<String, Vec<String>> {
    // Step 1: Parse to AST to validate correctness
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(&mut lexer);
    let program = match parser.parse_program() {
        Some(p) => p,
        None => return Err(vec!["Failed to parse program.".to_string()]),
    };

    if !parser.errors.is_empty() {
        return Err(parser.errors.iter().map(|e| format!("{}", e)).collect());
    }

    // Step 2: Tokenize with comments to get comment positions and number formats
    let tokens = tokenize_all(source);

    let mut comments: Vec<CommentInfo> = Vec::new();
    let mut numbers: Vec<NumberInfo> = Vec::new();

    for tok in &tokens {
        match &tok.kind {
            TokenKind::Comment(text) => {
                comments.push(CommentInfo {
                    _line: tok.line,
                    _column: tok.column,
                    _inline: has_non_whitespace_before_column(source, tok.line, tok.column),
                    text: text.clone(),
                });
            }
            TokenKind::Number(val) => {
                // Extract original text from source
                let original = extract_number_text(source, tok.line, tok.column);
                numbers.push(NumberInfo {
                    value: *val,
                    original,
                });
            }
            _ => {}
        }
    }

    // Step 3: Pretty-print from AST
    let formatted = format_program(&program, &comments, &numbers);
    Ok(formatted)
}

fn has_non_whitespace_before_column(source: &str, line: usize, column: usize) -> bool {
    let lines: Vec<&str> = source.lines().collect();
    if line == 0 || line > lines.len() || column == 0 {
        return false;
    }
    let src_line = lines[line - 1];
    src_line
        .chars()
        .take(column.saturating_sub(1))
        .any(|ch| !ch.is_whitespace())
}

/// Extract the original number text from source at the given line and column.
fn extract_number_text(source: &str, line: usize, column: usize) -> String {
    let lines: Vec<&str> = source.lines().collect();
    if line == 0 || line > lines.len() {
        return String::new();
    }
    let src_line = lines[line - 1];
    let chars: Vec<char> = src_line.chars().collect();
    // column is 1-based
    let start = column - 1;
    let mut end = start;
    while end < chars.len() && (chars[end].is_ascii_digit() || is_myanmar_digit(chars[end])) {
        end += 1;
    }
    chars[start..end].iter().collect()
}

fn is_myanmar_digit(ch: char) -> bool {
    '\u{1040}' <= ch && ch <= '\u{1049}'
}

/// Convert an i64 to Myanmar numeral string.
fn to_myanmar_numeral(n: i64) -> String {
    if n == 0 {
        return '\u{1040}'.to_string();
    }
    let negative = n < 0;
    let mut val = if negative { -n } else { n } as u64;
    let mut digits = Vec::new();
    while val > 0 {
        let d = (val % 10) as u32;
        digits.push(char::from_u32(0x1040 + d).unwrap());
        val /= 10;
    }
    digits.reverse();
    let mut result = String::new();
    if negative {
        result.push('-');
    }
    for d in digits {
        result.push(d);
    }
    result
}

/// Format the entire program.
fn format_program(
    program: &Program,
    comments: &[CommentInfo],
    numbers: &[NumberInfo],
) -> String {
    let mut ctx = FormatContext {
        numbers,
        number_index: 0,
    };
    let mut output = String::new();

    for comment in comments {
        output.push_str(&format!("//{}\n", comment.text));
    }

    if !comments.is_empty() && !program.statements.is_empty() {
        output.push('\n');
    }

    for (i, stmt) in program.statements.iter().enumerate() {
        if i > 0 {
            let prev = &program.statements[i - 1];
            if statement_is_block_like(prev) || statement_is_block_like(stmt)
            {
                output.push('\n');
            }
        }

        let formatted = format_statement(stmt, 0, &mut ctx);
        output.push_str(&formatted);
        output.push('\n');
    }

    // Ensure single trailing newline
    while output.ends_with("\n\n") {
        output.pop();
    }
    if !output.ends_with('\n') {
        output.push('\n');
    }

    output
}

struct FormatContext<'a> {
    numbers: &'a [NumberInfo],
    number_index: usize,
}

impl<'a> FormatContext<'a> {
    /// Get the original representation of the next number literal.
    fn next_number(&mut self, value: i64) -> String {
        // Try to find a matching number in our list
        while self.number_index < self.numbers.len() {
            let info = &self.numbers[self.number_index];
            if info.value == value {
                self.number_index += 1;
                if !info.original.is_empty() {
                    return info.original.clone();
                }
                return to_myanmar_numeral(value);
            }
            self.number_index += 1;
        }
        // Fallback: use Myanmar numerals
        to_myanmar_numeral(value)
    }
}

fn indent_str(level: usize) -> String {
    INDENT_UNIT.repeat(level)
}

fn statement_is_block_like(stmt: &Statement) -> bool {
    matches!(
        stmt,
        Statement::If { .. }
            | Statement::While { .. }
            | Statement::ForIn { .. }
            | Statement::FunctionDecl { .. }
            | Statement::StructDecl { .. }
            | Statement::MethodDecl { .. }
            | Statement::InterfaceDecl { .. }
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Assoc {
    Left,
    Non,
}

fn operator_precedence(op: &str) -> (u8, Assoc) {
    match op {
        "==" | "!=" => (2, Assoc::Non),
        "<" | ">" | "<=" | ">=" => (3, Assoc::Non),
        "+" | "-" => (4, Assoc::Left),
        "*" | "/" => (5, Assoc::Left),
        _ => (1, Assoc::Non),
    }
}

fn expression_precedence(expr: &Expression) -> u8 {
    match expr {
        Expression::Binary { operator, .. } => operator_precedence(operator).0,
        _ => 10,
    }
}

fn needs_parentheses(
    child: &Expression,
    parent_prec: u8,
    parent_assoc: Assoc,
    is_right_child: bool,
) -> bool {
    let child_prec = expression_precedence(child);
    if child_prec < parent_prec {
        return true;
    }
    if child_prec > parent_prec {
        return false;
    }
    is_right_child && parent_assoc != Assoc::Left
}

fn format_block_delimited_items(
    opener: &str,
    closer: &str,
    items: &[String],
    indent: usize,
) -> String {
    if items.is_empty() {
        return format!("{}{}", opener, closer);
    }

    let inline = format!("{}{}{}", opener, items.join(", "), closer);
    if inline.len() <= MAX_INLINE_WIDTH && items.len() <= COLLECTION_WRAP_THRESHOLD {
        return inline;
    }

    let mut out = String::new();
    out.push_str(opener);
    out.push('\n');
    let child_indent = indent_str(indent + 1);
    for (idx, item) in items.iter().enumerate() {
        out.push_str(&child_indent);
        out.push_str(item);
        if idx + 1 < items.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(&indent_str(indent));
    out.push_str(closer);
    out
}

fn format_function_params(
    parameters: &[(String, Type, crate::ast::Span)],
    indent: usize,
) -> String {
    if parameters.is_empty() {
        return "()".to_string();
    }

    let items = parameters
        .iter()
        .map(|(pname, pty, _)| format!("{} {}", format_type(pty), pname))
        .collect::<Vec<_>>();
    format_block_delimited_items("(", ")", &items, indent)
}

fn format_statement(stmt: &Statement, indent: usize, ctx: &mut FormatContext) -> String {
    let ind = indent_str(indent);
    match stmt {
        Statement::Let { name, value, ty, .. } => {
            let ty_str = format_type(ty);
            let val_str = format_expression(value, indent, ctx);
            format!("{}{} {} = {};", ind, ty_str, name, val_str)
        }
        Statement::LetDestructured { names, value } => {
            let parts: Vec<String> = names.iter().map(|(n, t, _)| {
                format!("{} {}", format_type(t), n)
            }).collect();
            let val_str = format_expression(value, indent, ctx);
            format!("{}{} = {};", ind, parts.join(", "), val_str)
        }
        Statement::Assign { name, value, .. } => {
            let val_str = format_expression(value, indent, ctx);
            format!("{}{} = {};", ind, name, val_str)
        }
        Statement::FieldAssign {
            object,
            field,
            value,
            ..
        } => {
            let val_str = format_expression(value, indent, ctx);
            format!("{}{}.{} = {};", ind, object, field, val_str)
        }
        Statement::IndexAssign {
            object,
            index,
            value,
            ..
        } => {
            let obj_str = format_expression(object, indent, ctx);
            let idx_str = format_expression(index, indent, ctx);
            let val_str = format_expression(value, indent, ctx);
            format!("{}{}[{}] = {};", ind, obj_str, idx_str, val_str)
        }
        Statement::If {
            condition,
            consequence,
            alternative,
        } => {
            let cond_str = format_expression(condition, indent, ctx);
            let mut result = format!("{}hlyin ({}) {{\n", ind, cond_str);
            result.push_str(&format_block(consequence, indent + 1, ctx));
            match alternative {
                Some(IfAlternative::Else(block)) => {
                    result.push_str(&format!("{}}} mo {{\n", ind));
                    result.push_str(&format_block(block, indent + 1, ctx));
                    result.push_str(&format!("{}}}", ind));
                }
                Some(IfAlternative::ElseIf(elif_stmt)) => {
                    let elif_formatted = format_elif(elif_stmt, indent, ctx);
                    result.push_str(&format!("{}}} mo {}", ind, elif_formatted.trim_start()));
                }
                None => {
                    result.push_str(&format!("{}}}", ind));
                }
            }
            result
        }
        Statement::While { condition, body } => {
            let cond_str = format_expression(condition, indent, ctx);
            let mut result = format!("{}pat ({}) {{\n", ind, cond_str);
            result.push_str(&format_block(body, indent + 1, ctx));
            result.push_str(&format!("{}}}", ind));
            result
        }
        Statement::Break => format!("{}yut;", ind),
        Statement::Continue => format!("{}shar;", ind),
        Statement::ForIn {
            index,
            iterator,
            collection,
            body,
            ..
        } => {
            let collection_str = format_expression(collection, indent, ctx);
            let header = if let Some(idx) = index {
                format!("{}pat ({}, {}) htae {} {{\n", ind, idx, iterator, collection_str)
            } else {
                format!("{}pat {} htae {} {{\n", ind, iterator, collection_str)
            };
            let mut result = header;
            result.push_str(&format_block(body, indent + 1, ctx));
            result.push_str(&format!("{}}}", ind));
            result
        }
        Statement::FunctionDecl {
            name,
            parameters,
            return_type,
            body,
            ..
        } => {
            let params = format_function_params(parameters, indent);
            let ret = format_type(return_type);
            let mut result = format!("{}loke {}{} -> {} {{\n", ind, name, params, ret);
            result.push_str(&format_block(body, indent + 1, ctx));
            result.push_str(&format!("{}}}", ind));
            result
        }
        Statement::Return { value } => {
            if let Expression::TupleLiteral { elements } = value {
                let vals: Vec<String> = elements.iter().map(|e| format_expression(e, indent, ctx)).collect();
                format!("{}pyan ({});", ind, vals.join(", "))
            } else {
                let val_str = format_expression(value, indent, ctx);
                format!("{}pyan {};", ind, val_str)
            }
        }
        Statement::Print { value } => {
            let val_str = format_expression(value, indent, ctx);
            format!("{}pya({});", ind, val_str)
        }
        Statement::Import { module, .. } => {
            format!("{}yu \"{}\";", ind, module.trim_matches('"'))
        }
        Statement::ExpressionStatement(expr) => {
            let expr_str = format_expression(expr, indent, ctx);
            format!("{}{};", ind, expr_str)
        }
        Statement::StructDecl { name, fields, .. } => {
            let mut result = format!("{}pone {} {{\n", ind, name);
            for (fname, ftype) in fields {
                result.push_str(&format!("{}{} {};\n", indent_str(indent + 1), format_type(ftype), fname));
            }
            result.push_str(&format!("{}}}", ind));
            result
        }
        Statement::MethodDecl {
            receiver_type,
            receiver_name,
            name,
            parameters,
            return_type,
            body,
            ..
        } => {
            let params = format_function_params(parameters, indent);
            let ret = format_type(return_type);
            let mut result = format!("{}nee ({} {}) {}{} -> {} {{\n", ind, receiver_type, receiver_name, name, params, ret);
            result.push_str(&format_block(body, indent + 1, ctx));
            result.push_str(&format!("{}}}", ind));
            result
        }
        Statement::InterfaceDecl { name, methods, .. } => {
            let mut result = format!("{}myat {} {{\n", ind, name);
            for (mname, params, ret_type) in methods {
                let param_strs: Vec<String> = params.iter().map(|(pname, ptype)| {
                    format!("{} {}", format_type(ptype), pname)
                }).collect();
                result.push_str(&format!("{}loke {}({}) -> {};\n",
                    indent_str(indent + 1),
                    mname,
                    param_strs.join(", "),
                    format_type(ret_type),
                ));
            }
            result.push_str(&format!("{}}}", ind));
            result
        }
    }
}

fn format_elif(stmt: &Statement, indent: usize, ctx: &mut FormatContext) -> String {
    let ind = indent_str(indent);
    if let Statement::If {
        condition,
        consequence,
        alternative,
    } = stmt
    {
        let cond_str = format_expression(condition, indent, ctx);
        let mut result = format!("{}hlyin ({}) {{\n", ind, cond_str);
        result.push_str(&format_block(consequence, indent + 1, ctx));
        match alternative {
            Some(IfAlternative::Else(block)) => {
                result.push_str(&format!("{}}} mo {{\n", ind));
                result.push_str(&format_block(block, indent + 1, ctx));
                result.push_str(&format!("{}}}", ind));
            }
            Some(IfAlternative::ElseIf(elif_stmt)) => {
                let elif_formatted = format_elif(elif_stmt, indent, ctx);
                result.push_str(&format!("{}}} mo {}", ind, elif_formatted.trim_start()));
            }
            None => {
                result.push_str(&format!("{}}}", ind));
            }
        }
        result
    } else {
        format_statement(stmt, indent, ctx)
    }
}

fn format_block(block: &BlockStatement, indent: usize, ctx: &mut FormatContext) -> String {
    let mut result = String::new();
    for (idx, stmt) in block.statements.iter().enumerate() {
        if idx > 0 {
            let prev = &block.statements[idx - 1];
            if statement_is_block_like(prev) || statement_is_block_like(stmt) {
                result.push('\n');
            }
        }
        result.push_str(&format_statement(stmt, indent, ctx));
        result.push('\n');
    }
    result
}

fn format_expression(expr: &Expression, indent: usize, ctx: &mut FormatContext) -> String {
    match expr {
        Expression::IntegerLiteral(val) => ctx.next_number(*val),
        Expression::StringLiteral(val) => format!("\"{}\"", val),
        Expression::BooleanLiteral(val) => {
            if *val {
                "hman".to_string()
            } else {
                "hmar".to_string()
            }
        }
        Expression::Identifier(name) => name.clone(),
        Expression::Binary {
            left,
            operator,
            right,
        } => {
            let (prec, assoc) = operator_precedence(operator);

            let mut l = format_expression(left, indent, ctx);
            if needs_parentheses(left, prec, assoc, false) {
                l = format!("({})", l);
            }

            let mut r = format_expression(right, indent, ctx);
            if needs_parentheses(right, prec, assoc, true) {
                r = format!("({})", r);
            }
            format!("{} {} {}", l, operator, r)
        }
        Expression::FunctionCall {
            function,
            arguments,
        } => {
            let args = arguments
                .iter()
                .map(|a| format_expression(a, indent + 1, ctx))
                .collect::<Vec<_>>();
            let rendered = format_block_delimited_items("(", ")", &args, indent);
            format!("{}{}", function, rendered)
        }
        Expression::ArrayLiteral { elements } => {
            let elems = elements
                .iter()
                .map(|e| format_expression(e, indent + 1, ctx))
                .collect::<Vec<_>>();
            format_block_delimited_items("[", "]", &elems, indent)
        }
        Expression::HashLiteral { pairs } => {
            let ps = pairs
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}: {}",
                        format_expression(k, indent + 1, ctx),
                        format_expression(v, indent + 1, ctx)
                    )
                })
                .collect::<Vec<_>>();
            format_block_delimited_items("{", "}", &ps, indent)
        }
        Expression::IndexExpression { left, index } => {
            let l = format_expression(left, indent, ctx);
            let i = format_expression(index, indent, ctx);
            format!("{}[{}]", l, i)
        }
        Expression::ReadInput { prompt } => {
            let p = format_expression(prompt, indent, ctx);
            format!("phat({})", p)
        }
        Expression::FloatLiteral(val) => format!("{}", val),
        Expression::NilLiteral => "bhala".to_string(),
        Expression::SliceExpression { left, low, high } => {
            let obj = format_expression(left, indent, ctx);
            let l = low.as_ref().map(|e| format_expression(e, indent, ctx)).unwrap_or_default();
            let h = high.as_ref().map(|e| format_expression(e, indent, ctx)).unwrap_or_default();
            format!("{}[{}:{}]", obj, l, h)
        }
        Expression::TypeConversion { target_type, argument } => {
            let arg = format_expression(argument, indent, ctx);
            format!("pyaung_{}({})", format_type(&target_type), arg)
        }
        Expression::MethodCall { object, method, arguments } => {
            let obj = format_expression(object, indent, ctx);
            let args: Vec<String> = arguments.iter().map(|a| format_expression(a, indent, ctx)).collect();
            format!("{}.{}({})", obj, method, args.join(", "))
        }
        Expression::FieldAccess { object, field } => {
            let obj = format_expression(object, indent, ctx);
            format!("{}.{}", obj, field)
        }
        Expression::StructLiteral { name, fields } => {
            let fs: Vec<String> = fields.iter().map(|(fname, fval)| {
                format!("{}: {}", fname, format_expression(fval, indent, ctx))
            }).collect();
            format!("{} {{ {} }}", name, fs.join(", "))
        }
        Expression::ClosureLiteral {
            parameters,
            return_type,
            body,
        } => {
            let params = format_function_params(parameters, indent);
            let mut result = format!("loke{}", params);
            if *return_type != Type::Nil {
                result.push_str(&format!(" -> {}", format_type(return_type)));
            }
            result.push_str(" {\n");
            result.push_str(&format_block(body, indent + 1, ctx));
            result.push_str(&format!("{}}}", indent_str(indent)));
            result
        }
        Expression::ErrorCreate { message } => {
            let msg = format_expression(message, indent, ctx);
            format!("amhar({})", msg)
        }
        Expression::TupleLiteral { elements } => {
            let vals: Vec<String> = elements.iter().map(|e| format_expression(e, indent, ctx)).collect();
            format!("({})", vals.join(", "))
        }
    }
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::Kain => "kain".to_string(),
        Type::Sar => "sar".to_string(),
        Type::Sit => "sit".to_string(),
        Type::Array(inner) => format!("su<{}>", format_type(inner)),
        Type::Map(key, val) => format!("twe<{}, {}>", format_type(key), format_type(val)),
        Type::DaTha => "da_tha".to_string(),
        Type::Nil => "bhala".to_string(),
        Type::Error => "amhar".to_string(),
        Type::Struct(name) => name.clone(),
        Type::Interface(name) => name.clone(),
        Type::Tuple(types) => {
            let ts: Vec<String> = types.iter().map(|t| format_type(t)).collect();
            format!("({})", ts.join(", "))
        }
        Type::Function {
            params,
            return_type,
        } => {
            let pstr: Vec<String> = params.iter().map(format_type).collect();
            format!("loke({}) -> {}", pstr.join(", "), format_type(return_type))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_program() {
        let input = r#"loke main() -> kain {
kain age = ၂၀;
pyan ၀;
}"#;
        let result = format_source(input).unwrap();
        let expected = "loke main() -> kain {\n    kain age = ၂၀;\n    pyan ၀;\n}\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_if_else() {
        let input = r#"loke main() -> kain {
hlyin (x > ၁၈) {
pya("yes");
}mo{
pya("no");
}
pyan ၀;
}"#;
        let result = format_source(input).unwrap();
        assert!(result.contains("} mo {"));
        assert!(result.contains("    hlyin (x > ၁၈) {"));
    }

    #[test]
    fn test_format_preserves_binary_parentheses() {
        let input = r#"loke main() -> kain {
kain x = (၁ + ၂) * ၃;
pyan x;
}"#;
        let result = format_source(input).unwrap();
        assert!(result.contains("kain x = (၁ + ၂) * ၃;"));
    }

    #[test]
    fn test_format_wraps_long_array() {
        let input = r#"loke main() -> kain {
su<kain> xs = [၁, ၂, ၃, ၄, ၅];
pyan ၀;
}"#;
        let result = format_source(input).unwrap();
        assert!(result.contains("su<kain> xs = [\n"));
        assert!(result.contains("        ၁,"));
    }

    #[test]
    fn test_format_wraps_long_function_params() {
        let input = r#"loke veryLongFunctionName(kain firstArg, kain secondArg, kain thirdArg, kain fourthArg) -> kain {
pyan firstArg;
}"#;
        let result = format_source(input).unwrap();
        assert!(result.contains("loke veryLongFunctionName(\n"));
        assert!(result.contains("    kain firstArg,"));
    }

    #[test]
    fn test_format_preserves_myanmar_numerals() {
        let input = "loke main() -> kain {\nkain x = ၁၂၃;\npyan ၀;\n}";
        let result = format_source(input).unwrap();
        assert!(result.contains("၁၂၃"));
        assert!(result.contains("၀"));
    }

    #[test]
    fn test_format_preserves_ascii_numerals() {
        let input = "loke main() -> kain {\nkain x = 123;\npyan 0;\n}";
        let result = format_source(input).unwrap();
        assert!(result.contains("123"));
        assert!(result.contains("0"));
    }

    #[test]
    fn test_format_comment_preserved() {
        let input = "// this is a comment\nloke main() -> kain {\npyan ၀;\n}";
        let result = format_source(input).unwrap();
        assert!(result.contains("// this is a comment"));
    }

    #[test]
    fn test_format_elif_chain() {
        let input = r#"loke main() -> kain {
hlyin (x == ၁) {
pya("one");
} mo hlyin (x == ၂) {
pya("two");
} mo {
pya("other");
}
pyan ၀;
}"#;
        let result = format_source(input).unwrap();
        assert!(result.contains("} mo hlyin (x == ၂) {"));
        assert!(result.contains("} mo {"));
    }

    #[test]
    fn test_format_trailing_newline() {
        let input = "loke main() -> kain {\npyan ၀;\n}";
        let result = format_source(input).unwrap();
        assert!(result.ends_with('\n'));
        assert!(!result.ends_with("\n\n"));
    }

    #[test]
    fn test_format_imports_to_quoted_style() {
        let input = r#"yu json;
yu "file";
"#;
        let result = format_source(input).unwrap();
        assert!(result.contains("yu \"json\";"));
        assert!(result.contains("yu \"file\";"));
    }
}
