use crate::ast::{BlockStatement, Expression, IfAlternative, Program, Statement, Type};

pub struct CodeGenerator {
    pub output: String,
    indent_level: usize,
    environment: std::collections::HashMap<String, Type>,
    array_lengths: std::collections::HashMap<String, usize>,
    loop_counter: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            output: String::new(),
            indent_level: 0,
            environment: std::collections::HashMap::new(),
            array_lengths: std::collections::HashMap::new(),
            loop_counter: 0,
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        // Standard C Library Includes
        self.output.push_str("#include <stdio.h>\n");
        self.output.push_str("#include <stdbool.h>\n");
        self.output.push_str("#include <string.h>\n");
        self.output.push_str("#include <stdlib.h>\n\n");

        self.output
            .push_str("char* mlang_concat(const char* s1, const char* s2) {\n");
        self.output
            .push_str("    char* result = malloc(strlen(s1) + strlen(s2) + 1);\n");
        self.output.push_str("    strcpy(result, s1);\n");
        self.output.push_str("    strcat(result, s2);\n");
        self.output.push_str("    return result;\n");
        self.output.push_str("}\n\n");

        self.output
            .push_str("char* mlang_read_input(const char* prompt) {\n");
        self.output
            .push_str("    if (prompt && strlen(prompt) > 0) {\n");
        self.output.push_str("        printf(\"%s\", prompt);\n");
        self.output.push_str("    }\n");
        self.output.push_str("    char buffer[1024];\n");
        self.output
            .push_str("    if (fgets(buffer, sizeof(buffer), stdin) != NULL) {\n");
        self.output
            .push_str("        size_t len = strlen(buffer);\n");
        self.output
            .push_str("        if (len > 0 && buffer[len-1] == '\\n') buffer[len-1] = '\\0';\n");
        self.output
            .push_str("        char* result = malloc(strlen(buffer) + 1);\n");
        self.output.push_str("        strcpy(result, buffer);\n");
        self.output.push_str("        return result;\n");
        self.output.push_str("    }\n");
        self.output
            .push_str("    char* empty = malloc(1); empty[0] = '\\0'; return empty;\n");
        self.output.push_str("}\n\n");

        for stmt in &program.statements {
            self.generate_statement(stmt);
        }

        self.output.clone()
    }

    fn generate_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let {
                name, value, ty, ..
            } => {
                self.environment.insert(name.clone(), ty.clone());
                self.indent();
                self.generate_type(ty);
                self.output
                    .push_str(&format!(" {} = ", self.clean_identifier(name)));
                self.generate_expression(value);
                self.output.push_str(";\n");

                if let Type::Array(_) = ty {
                    if let Some(length) = self.infer_array_length(value) {
                        self.array_lengths.insert(name.clone(), length);
                        self.indent();
                        self.output.push_str(&format!(
                            "long long {}_len = {};\n",
                            self.clean_identifier(name),
                            length
                        ));
                    }
                }
            }
            Statement::Assign { name, value, .. } => {
                self.indent();
                self.output
                    .push_str(&format!("{} = ", self.clean_identifier(name)));
                self.generate_expression(value);
                self.output.push_str(";\n");

                if let Some(Type::Array(_)) = self.environment.get(name) {
                    if let Some(length) = self.infer_array_length(value) {
                        self.array_lengths.insert(name.clone(), length);
                        self.indent();
                        self.output.push_str(&format!(
                            "{}_len = {};\n",
                            self.clean_identifier(name),
                            length
                        ));
                    }
                }
            }
            Statement::FunctionDecl {
                name,
                parameters,
                return_type,
                body,
                ..
            } => {
                self.indent();
                let c_name = if name == "main" {
                    "main".to_string()
                } else {
                    self.clean_identifier(name)
                };

                if name == "main" {
                    self.output.push_str("int main(");
                } else {
                    self.generate_type(return_type);
                    self.output.push_str(&format!(" {}(", c_name));
                }

                for (i, (p_name, p_type, _)) in parameters.iter().enumerate() {
                    self.generate_type(p_type);
                    self.output
                        .push_str(&format!(" {}", self.clean_identifier(p_name)));
                    if i < parameters.len() - 1 {
                        self.output.push_str(", ");
                    }
                }

                self.output.push_str(") ");
                self.generate_block(body);
                self.output.push_str("\n");
            }
            Statement::Return { value } => {
                self.indent();
                self.output.push_str("return ");
                self.generate_expression(value);
                self.output.push_str(";\n");
            }
            Statement::Print { value } => {
                self.indent();
                self.output.push_str("printf(");

                crate::ast::Expression::dummy_print_format(&mut self.output, value); // Internal helper logic
                self.generate_print_format(value);

                self.output.push_str(", ");
                self.generate_expression(value);
                self.output.push_str(");\n");
            }
            Statement::If {
                condition,
                consequence,
                alternative,
            } => {
                self.indent();
                self.output.push_str("if (");
                self.generate_expression(condition);
                self.output.push_str(") ");
                self.generate_block(consequence);

                if let Some(alt) = alternative {
                    match alt {
                        IfAlternative::Else(block) => {
                            self.output.push_str(" else ");
                            self.generate_block(block);
                        }
                        IfAlternative::ElseIf(elif_stmt) => {
                            self.output.push_str(" else ");
                            self.generate_elif_statement(elif_stmt);
                        }
                    }
                }
                self.output.push_str("\n");
            }
            Statement::While { condition, body } => {
                self.indent();
                self.output.push_str("while (");
                self.generate_expression(condition);
                self.output.push_str(") ");
                self.generate_block(body);
                self.output.push_str("\n");
            }
            Statement::ForIn {
                iterator,
                collection,
                body,
                ..
            } => {
                let item_type = match self.infer_expression_type(collection) {
                    Some(Type::Array(inner)) => *inner,
                    _ => {
                        self.indent();
                        self.output
                            .push_str("/* Unsupported for-in collection type */\n");
                        return;
                    }
                };

                let (collection_c_expr, array_len) = match collection {
                    Expression::Identifier(name) => {
                        let Some(array_len) = self.array_lengths.get(name).copied() else {
                            self.indent();
                            self.output
                                .push_str("/* Cannot iterate: array length unknown */\n");
                            return;
                        };
                        (self.clean_identifier(name), array_len)
                    }
                    Expression::ArrayLiteral { elements } => {
                        let c_type = if elements.is_empty() {
                            "long long"
                        } else {
                            self.guess_c_type(&elements[0])
                        };
                        let mut literal_expr = format!("({}[]){{", c_type);
                        for (i, expr) in elements.iter().enumerate() {
                            literal_expr.push_str(&self.expression_to_c(expr));
                            if i < elements.len() - 1 {
                                literal_expr.push_str(", ");
                            }
                        }
                        literal_expr.push('}');
                        (literal_expr, elements.len())
                    }
                    _ => {
                        self.indent();
                        self.output
                            .push_str("/* Unsupported for-in collection expression */\n");
                        return;
                    }
                };

                let idx_name = format!("__mlang_i{}", self.loop_counter);
                self.loop_counter += 1;

                self.environment.insert(iterator.clone(), item_type.clone());

                self.indent();
                self.output.push_str(&format!(
                    "for (long long {} = 0; {} < {}; {}++) ",
                    idx_name, idx_name, array_len, idx_name
                ));
                self.output.push_str("{\n");
                self.indent_level += 1;

                self.indent();
                self.generate_type(&item_type);
                self.output.push_str(&format!(
                    " {} = {}[{}];\n",
                    self.clean_identifier(iterator),
                    collection_c_expr,
                    idx_name
                ));

                for stmt in &body.statements {
                    self.generate_statement(stmt);
                }

                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}\n");
            }
            Statement::ExpressionStatement(expr) => {
                self.indent();
                self.generate_expression(expr);
                self.output.push_str(";\n");
            }
            Statement::Import { module, .. } => {
                let clean_mod = module.replace("\"", ""); // If module was parsed as string
                self.output
                    .push_str(&format!("#include \"{}.c\"\n", clean_mod));
            }
            _ => {} // C backend: new statement types not supported
        }
    }

    fn generate_elif_statement(&mut self, stmt: &Statement) {
        // Generate elif as "if (...) { ... } else ..." without leading indent
        match stmt {
            Statement::If {
                condition,
                consequence,
                alternative,
            } => {
                self.output.push_str("if (");
                self.generate_expression(condition);
                self.output.push_str(") ");
                self.generate_block(consequence);

                if let Some(alt) = alternative {
                    match alt {
                        IfAlternative::Else(block) => {
                            self.output.push_str(" else ");
                            self.generate_block(block);
                        }
                        IfAlternative::ElseIf(elif_stmt) => {
                            self.output.push_str(" else ");
                            self.generate_elif_statement(elif_stmt);
                        }
                    }
                }
            }
            _ => self.generate_statement(stmt),
        }
    }

    fn generate_block(&mut self, block: &BlockStatement) {
        self.output.push_str("{\n");
        self.indent_level += 1;
        for stmt in &block.statements {
            self.generate_statement(stmt);
        }
        self.indent_level -= 1;
        self.indent();
        self.output.push_str("}");
    }

    fn generate_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::IntegerLiteral(val) => self.output.push_str(&val.to_string()),
            Expression::StringLiteral(val) => self.output.push_str(&format!("\"{}\"", val)),
            Expression::BooleanLiteral(val) => {
                self.output.push_str(if *val { "true" } else { "false" })
            }
            Expression::Identifier(name) => self.output.push_str(&self.clean_identifier(name)),
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                if operator == "+" && self.is_string_expression(left) {
                    self.output.push_str("mlang_concat(");
                    self.generate_expression(left);
                    self.output.push_str(", ");
                    self.generate_expression(right);
                    self.output.push_str(")");
                } else {
                    self.output.push_str("(");
                    self.generate_expression(left);
                    self.output.push_str(&format!(" {} ", operator));
                    self.generate_expression(right);
                    self.output.push_str(")");
                }
            }
            Expression::FunctionCall {
                function,
                arguments,
            } => {
                self.output
                    .push_str(&format!("{}(", self.clean_identifier(function)));
                for (i, arg) in arguments.iter().enumerate() {
                    self.generate_expression(arg);
                    if i < arguments.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(")");
            }
            Expression::ArrayLiteral { elements } => {
                let c_type = if elements.is_empty() {
                    "long long"
                } else {
                    self.guess_c_type(&elements[0])
                };

                self.output.push_str(&format!("({}[]){{", c_type));
                for (i, expr) in elements.iter().enumerate() {
                    self.generate_expression(expr);
                    if i < elements.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str("}");
            }
            Expression::HashLiteral { pairs: _ } => {
                self.output
                    .push_str("NULL /* true HashMap requires complex C runtime */");
            }
            Expression::IndexExpression { left, index } => {
                self.generate_expression(left);
                self.output.push_str("[");
                self.generate_expression(index);
                self.output.push_str("]");
            }
            Expression::ReadInput { prompt } => {
                self.output.push_str("mlang_read_input(");
                self.generate_expression(prompt);
                self.output.push_str(")");
            }
            _ => {} // C backend: new expression types not supported
        }
    }

    fn is_string_expression(&self, expr: &Expression) -> bool {
        match expr {
            Expression::StringLiteral(_) => true,
            Expression::Identifier(name) => {
                if let Some(Type::Sar) = self.environment.get(name) {
                    true
                } else {
                    false
                }
            }
            Expression::ReadInput { .. } => true,
            Expression::Binary { left, operator, .. } => {
                if operator == "+" {
                    self.is_string_expression(left)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn infer_array_length(&self, expr: &Expression) -> Option<usize> {
        match expr {
            Expression::ArrayLiteral { elements } => Some(elements.len()),
            Expression::Identifier(name) => self.array_lengths.get(name).copied(),
            _ => None,
        }
    }

    fn infer_expression_type(&self, expr: &Expression) -> Option<Type> {
        match expr {
            Expression::IntegerLiteral(_) => Some(Type::Kain),
            Expression::StringLiteral(_) => Some(Type::Sar),
            Expression::BooleanLiteral(_) => Some(Type::Sit),
            Expression::Identifier(name) => self.environment.get(name).cloned(),
            Expression::ArrayLiteral { elements } => {
                if elements.is_empty() {
                    None
                } else {
                    self.infer_expression_type(&elements[0])
                        .map(|t| Type::Array(Box::new(t)))
                }
            }
            Expression::HashLiteral { .. } => None,
            Expression::IndexExpression { .. } => None,
            Expression::ReadInput { .. } => Some(Type::Sar),
            Expression::FunctionCall { .. } => None,
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left_ty = self.infer_expression_type(left)?;
                let right_ty = self.infer_expression_type(right)?;
                match operator.as_str() {
                    "+" | "-" | "*" | "/" => {
                        if left_ty == Type::Kain && right_ty == Type::Kain {
                            Some(Type::Kain)
                        } else if operator == "+" && left_ty == Type::Sar && right_ty == Type::Sar {
                            Some(Type::Sar)
                        } else {
                            None
                        }
                    }
                    "==" | "!=" | ">" | "<" | ">=" | "<=" => Some(Type::Sit),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn expression_to_c(&self, expr: &Expression) -> String {
        match expr {
            Expression::IntegerLiteral(val) => val.to_string(),
            Expression::StringLiteral(val) => format!("\"{}\"", val),
            Expression::BooleanLiteral(val) => {
                if *val {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Expression::Identifier(name) => self.clean_identifier(name),
            _ => "0".to_string(),
        }
    }

    fn guess_c_type(&self, expr: &Expression) -> &str {
        match expr {
            Expression::IntegerLiteral(_) => "long long",
            Expression::StringLiteral(_) => "char*",
            Expression::BooleanLiteral(_) => "bool",
            Expression::Identifier(name) => {
                if let Some(ty) = self.environment.get(name) {
                    match ty {
                        Type::Kain => "long long",
                        Type::Sar => "char*",
                        Type::Sit => "bool",
                        Type::Array(_) => "void*",
                        Type::Map(_, _) => "void*",
                        _ => "void*",
                    }
                } else {
                    "long long"
                }
            }
            _ => "long long",
        }
    }

    fn generate_type(&mut self, ty: &Type) {
        match ty {
            Type::Kain => self.output.push_str("long long"),
            Type::Sar => self.output.push_str("char*"),
            Type::Sit => self.output.push_str("bool"),
            Type::Array(inner) => {
                self.generate_type(inner);
                self.output.push_str("*");
            }
            Type::Map(_, _) => self.output.push_str("void* /* HashMap */"),
            _ => self.output.push_str("void*"),
        }
    }

    fn generate_print_format(&mut self, expr: &Expression) {
        // Need to roughly know the type to give C the correct printf format string
        match expr {
            Expression::IntegerLiteral(_) => self.output.push_str("\"%lld\\n\""),
            Expression::StringLiteral(_) => self.output.push_str("\"%s\\n\""),
            Expression::BooleanLiteral(_) => self.output.push_str("\"%d\\n\""), // map true/false to 1/0
            Expression::Identifier(name) => {
                if let Some(ty) = self.environment.get(name) {
                    match ty {
                        Type::Kain => self.output.push_str("\"%lld\\n\""),
                        Type::Sar => self.output.push_str("\"%s\\n\""),
                        Type::Sit => self.output.push_str("\"%d\\n\""),
                        Type::Array(_) => {
                            self.output.push_str("\"[Array]\\n\""); // Basic fallback for now
                        }
                        Type::Map(_, _) => {
                            self.output.push_str("\"[HashMap]\\n\""); // Basic fallback for now
                        }
                        _ => {
                            self.output.push_str("\"%lld\\n\""); // fallback for new types
                        }
                    }
                } else {
                    self.output.push_str("\"%lld\\n\""); // fallback
                }
            }
            Expression::Binary { .. } => self.output.push_str("\"%lld\\n\""), // assume int math for now
            _ => self.output.push_str("\"%lld\\n\""),                         // default fallback
        }
    }

    fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    // Convert Burmese unicode identifiers into safe ascii (e.g. hex encoded)
    // C variables cannot be in unicode typically across all compilers
    fn clean_identifier(&self, name: &str) -> String {
        // Option 1: Transliteration (Hard)
        // Option 2: Prepending 'mlang_' + hex encoding bytes
        // Option 3: Use gcc/clang Unicode support using UCN (\uXXXX).
        // Let's use simple substitution for testing, replace 'အသက်' with 'var1' etc
        // But for a true compiler, let's prefix it with mlang_ and utf8 hex
        let mut clean = String::from("mlang_");
        for b in name.bytes() {
            clean.push_str(&format!("{:02x}", b));
        }
        clean
    }
}

impl Expression {
    pub fn dummy_print_format(_out: &mut String, _expr: &Expression) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_codegen() {
        let input = r#"
            loke main() -> kain {
                kain age = ၂၀;
                hlyin (age > ၁၈) {
                    pya("adult");
                }
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = CodeGenerator::new();
        let c_code = codegen.generate(&program);

        assert!(c_code.contains("int main()"));
        assert!(c_code.contains("printf(\"%s\\n\", \"adult\")")); // String generation
    }

    #[test]
    fn test_codegen_for_in_loop() {
        let input = r#"
            loke main() -> kain {
                su<kain> numbers = [၁, ၂, ၃];
                pat item htae numbers {
                    pya(item);
                }
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = CodeGenerator::new();
        let c_code = codegen.generate(&program);

        assert!(c_code.contains("for (long long __mlang_i0 = 0; __mlang_i0 < 3; __mlang_i0++)"));
        assert!(c_code.contains("mlang_6974656d = mlang_6e756d62657273[__mlang_i0];"));
    }

    #[test]
    fn test_codegen_for_in_loop_array_alias_length_propagates() {
        let input = r#"
            loke main() -> kain {
                su<kain> numbers = [၁, ၂, ၃];
                su<kain> other = numbers;
                pat item htae other {
                    pya(item);
                }
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = CodeGenerator::new();
        let c_code = codegen.generate(&program);

        assert!(c_code.contains("long long mlang_6f74686572_len = 3;"));
        assert!(c_code.contains("for (long long __mlang_i0 = 0; __mlang_i0 < 3; __mlang_i0++)"));
    }

    #[test]
    fn test_codegen_for_in_loop_array_literal_direct() {
        let input = r#"
            loke main() -> kain {
                pat item htae [၁, ၂, ၃] {
                    pya(item);
                }
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = CodeGenerator::new();
        let c_code = codegen.generate(&program);

        assert!(c_code.contains("for (long long __mlang_i0 = 0; __mlang_i0 < 3; __mlang_i0++)"));
        assert!(c_code.contains("= (long long[]){1, 2, 3}[__mlang_i0];"));
    }
}
