use crate::ast::{BlockStatement, Expression, IfAlternative, Program, Span, Statement, Type};
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest = 1,
    Equals = 2,
    LessGreater = 3,
    Sum = 4,
    Product = 5,
    Call = 6,
    Index = 7,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}] {}", self.line, self.column, self.message)
    }
}

pub struct Parser<'a> {
    lexer: &'a mut Lexer,
    current_token: Token,
    peek_token: Token,
    pub errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer) -> Self {
        let current_token = lexer.next_non_comment_token();
        let peek_token = lexer.next_non_comment_token();
        
        Parser {
            lexer,
            current_token,
            peek_token,
            errors: Vec::new(),
        }
    }

    pub fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_non_comment_token();
    }

    pub fn parse_program(&mut self) -> Option<Program> {
        let mut program = Program {
            statements: Vec::new(),
        };

        while self.current_token.kind != TokenKind::Eof {
            if let Some(stmt) = self.parse_statement() {
                program.statements.push(stmt);
            }
            self.next_token();
        }

        Some(program)
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.current_token.kind {
            TokenKind::Kain | TokenKind::Sar | TokenKind::Sit | TokenKind::DaTha | TokenKind::Su | TokenKind::Twe => self.parse_let_or_destructured(),
            TokenKind::Hlyin => self.parse_if_statement(),
            TokenKind::Pyan => self.parse_return_statement(),
            TokenKind::Pat => self.parse_pat_statement(),
            TokenKind::Phat => {
                // Read statement might be an expression, but if it stands alone we process as expression statement
                self.parse_expression_statement()
            }
            TokenKind::Yu => self.parse_import_statement(),
            TokenKind::Loke => self.parse_function_declaration(),
            TokenKind::Pya => self.parse_print_statement(),
            TokenKind::Pone => self.parse_struct_declaration(),
            TokenKind::Nee => self.parse_method_declaration(),
            TokenKind::Myat => self.parse_interface_declaration(),
            TokenKind::Amhar => {
                // Could be `amhar err = expr;` (let with error type) or expression
                if self.peek_is_identifier() {
                    self.parse_let_or_destructured()
                } else {
                    self.parse_expression_statement()
                }
            }
            TokenKind::Identifier(_) => {
                // Check for destructured let: `Type name, Type name = expr;`
                // or assignment: `name = expr;`
                // or struct type let: `StructName name = expr;`
                if self.peek_token.kind == TokenKind::Assign {
                    self.parse_assign_statement()
                } else if self.peek_is_identifier() {
                    // Could be `StructName varName = expr;` (struct type let)
                    self.parse_let_or_destructured()
                } else {
                    self.parse_expression_statement()
                }
            },
            _ => self.parse_expression_statement(),
        }
    }

    fn peek_is_identifier(&self) -> bool {
        matches!(self.peek_token.kind, TokenKind::Identifier(_))
    }

    fn parse_type(&mut self) -> Option<Type> {
        match &self.current_token.kind {
            TokenKind::Kain => Some(Type::Kain),
            TokenKind::Sar => Some(Type::Sar),
            TokenKind::Sit => Some(Type::Sit),
            TokenKind::DaTha => Some(Type::DaTha),
            TokenKind::Amhar => Some(Type::Error),
            TokenKind::Su => self.parse_array_type(),
            TokenKind::Twe => self.parse_map_type(),
            TokenKind::LParen => self.parse_tuple_type(),
            TokenKind::Identifier(name) => Some(Type::Struct(name.clone())),
            _ => None,
        }
    }

    fn parse_tuple_type(&mut self) -> Option<Type> {
        // Syntax: (Type, Type, ...)
        self.next_token(); // consume '('
        let mut types = Vec::new();
        let first = self.parse_type()?;
        types.push(first);
        while self.peek_token.kind == TokenKind::Comma {
            self.next_token(); // consume ','
            self.next_token(); // move to next type
            let ty = self.parse_type()?;
            types.push(ty);
        }
        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }
        Some(Type::Tuple(types))
    }

    fn parse_array_type(&mut self) -> Option<Type> {
        // Syntax: su<kain>
        if !self.expect_peek(TokenKind::LessThan) { return None; }
        self.next_token();
        let inner_type = self.parse_type()?;
        if !self.expect_peek(TokenKind::GreaterThan) { return None; }
        Some(Type::Array(Box::new(inner_type)))
    }

    fn parse_map_type(&mut self) -> Option<Type> {
        // Syntax: twe<sar, kain>
        if !self.expect_peek(TokenKind::LessThan) { return None; }
        self.next_token();
        let key_type = self.parse_type()?;
        if !self.expect_peek(TokenKind::Comma) { return None; }
        self.next_token();
        let val_type = self.parse_type()?;
        if !self.expect_peek(TokenKind::GreaterThan) { return None; }
        Some(Type::Map(Box::new(key_type), Box::new(val_type)))
    }

    fn parse_import_statement(&mut self) -> Option<Statement> {
        if !self.expect_peek_identifier() {
            return None;
        }

        let name_span = Span { line: self.current_token.line, column: self.current_token.column };
        let module = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };

        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token();
        }

        Some(Statement::Import { module, name_span })
    }

    fn parse_let_or_destructured(&mut self) -> Option<Statement> {
        // Parse first type + name
        let ty1 = self.parse_type()?;
        if !self.expect_peek_identifier() {
            return None;
        }
        let name_span1 = Span { line: self.current_token.line, column: self.current_token.column };
        let name1 = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };

        // Check if this is a destructured let: `Type1 name1, Type2 name2 = expr;`
        if self.peek_token.kind == TokenKind::Comma {
            self.next_token(); // consume ','
            let mut names = vec![(name1, ty1, name_span1)];
            // Parse remaining `Type name` pairs
            loop {
                self.next_token(); // move to next type
                let ty = self.parse_type()?;
                if !self.expect_peek_identifier() {
                    return None;
                }
                let span = Span { line: self.current_token.line, column: self.current_token.column };
                let name = match &self.current_token.kind {
                    TokenKind::Identifier(n) => n.clone(),
                    _ => return None,
                };
                names.push((name, ty, span));
                if self.peek_token.kind != TokenKind::Comma {
                    break;
                }
                self.next_token(); // consume ','
            }
            if !self.expect_peek(TokenKind::Assign) {
                return None;
            }
            self.next_token();
            let value = self.parse_expression(Precedence::Lowest)?;
            if self.peek_token.kind == TokenKind::Semicolon {
                self.next_token();
            }
            return Some(Statement::LetDestructured { names, value });
        }

        // Simple let
        if !self.expect_peek(TokenKind::Assign) {
            return None;
        }
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token();
        }
        Some(Statement::Let { name: name1, value, ty: ty1, name_span: name_span1 })
    }

    fn parse_assign_statement(&mut self) -> Option<Statement> {
        let name_span = Span { line: self.current_token.line, column: self.current_token.column };
        let name = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };

        self.next_token(); // move to =
        self.next_token(); // move past =

        let value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token();
        }

        Some(Statement::Assign { name, value, name_span })
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        self.next_token();

        // Check for tuple return: pyan (expr, expr, ...);
        if self.current_token.kind == TokenKind::LParen {
            // Peek ahead to determine if this is a grouped expression or tuple
            // We parse as grouped, then check if there was a comma
            let first = {
                self.next_token(); // move past '('
                self.parse_expression(Precedence::Lowest)?
            };
            if self.peek_token.kind == TokenKind::Comma {
                // It's a tuple
                let mut elements = vec![first];
                while self.peek_token.kind == TokenKind::Comma {
                    self.next_token(); // consume ','
                    self.next_token();
                    elements.push(self.parse_expression(Precedence::Lowest)?);
                }
                if !self.expect_peek(TokenKind::RParen) {
                    return None;
                }
                if self.peek_token.kind == TokenKind::Semicolon {
                    self.next_token();
                }
                return Some(Statement::Return { value: Expression::TupleLiteral { elements } });
            }
            // Single grouped expression
            if !self.expect_peek(TokenKind::RParen) {
                return None;
            }
            if self.peek_token.kind == TokenKind::Semicolon {
                self.next_token();
            }
            return Some(Statement::Return { value: first });
        }

        let value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token();
        }

        Some(Statement::Return { value })
    }

    fn parse_print_statement(&mut self) -> Option<Statement> {
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }
        
        self.next_token();
        
        let value = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }

        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token();
        }

        Some(Statement::Print { value })
    }

    fn parse_if_statement(&mut self) -> Option<Statement> {
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }

        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        let consequence = self.parse_block_statement();

        let alternative = if self.peek_token.kind == TokenKind::Mo {
            self.next_token(); // consume Mo (else)
            if self.peek_token.kind == TokenKind::Hlyin {
                // elif: mo hlyin (...) { ... }
                self.next_token(); // consume Hlyin (if)
                let elif_stmt = self.parse_if_statement()?;
                Some(IfAlternative::ElseIf(Box::new(elif_stmt)))
            } else {
                // plain else: mo { ... }
                if !self.expect_peek(TokenKind::LBrace) {
                    return None;
                }
                Some(IfAlternative::Else(self.parse_block_statement()))
            }
        } else {
            None
        };

        Some(Statement::If {
            condition,
            consequence,
            alternative,
        })
    }

    fn parse_while_statement(&mut self) -> Option<Statement> {
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }

        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        let body = self.parse_block_statement();

        Some(Statement::While { condition, body })
    }

    fn parse_pat_statement(&mut self) -> Option<Statement> {
        if self.peek_token.kind == TokenKind::LParen {
            return self.parse_while_statement();
        }

        self.parse_for_in_statement()
    }

    fn parse_for_in_statement(&mut self) -> Option<Statement> {
        if !self.expect_peek_identifier() {
            return None;
        }

        let name_span = Span { line: self.current_token.line, column: self.current_token.column };
        let iterator = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };

        if !self.expect_peek(TokenKind::Htae) {
            return None;
        }

        self.next_token();
        let collection = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        let body = self.parse_block_statement();

        Some(Statement::ForIn {
            iterator,
            collection,
            body,
            name_span,
        })
    }

    fn parse_function_declaration(&mut self) -> Option<Statement> {
        if !self.expect_peek_identifier() {
            return None;
        }

        let name_span = Span { line: self.current_token.line, column: self.current_token.column };
        let name = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };

        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }

        let parameters = self.parse_function_parameters()?;

        if !self.expect_peek(TokenKind::Arrow) {
            return None;
        }

        self.next_token();
        let return_type = self.parse_type()?;

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        let body = self.parse_block_statement();

        Some(Statement::FunctionDecl {
            name,
            parameters,
            return_type,
            body,
            name_span,
        })
    }

    fn parse_struct_declaration(&mut self) -> Option<Statement> {
        // pone Name { Type field; ... }
        if !self.expect_peek_identifier() {
            return None;
        }
        let name_span = Span { line: self.current_token.line, column: self.current_token.column };
        let name = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }
        let mut fields = Vec::new();
        self.next_token(); // move past '{'
        while self.current_token.kind != TokenKind::RBrace && self.current_token.kind != TokenKind::Eof {
            let field_type = self.parse_type()?;
            if !self.expect_peek_identifier() {
                return None;
            }
            let field_name = match &self.current_token.kind {
                TokenKind::Identifier(n) => n.clone(),
                _ => return None,
            };
            fields.push((field_name, field_type));
            if self.peek_token.kind == TokenKind::Semicolon {
                self.next_token(); // consume ';'
            }
            self.next_token();
        }
        Some(Statement::StructDecl { name, fields, name_span })
    }

    fn parse_method_declaration(&mut self) -> Option<Statement> {
        // nee (TypeName receiverName) methodName(params) -> retType { body }
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }
        self.next_token(); // move to receiver type
        let receiver_type = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => {
                self.errors.push(ParseError {
                    message: "Expected receiver type name".to_string(),
                    line: self.current_token.line,
                    column: self.current_token.column,
                });
                return None;
            }
        };
        if !self.expect_peek_identifier() {
            return None;
        }
        let receiver_name = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }
        if !self.expect_peek_identifier() {
            return None;
        }
        let name_span = Span { line: self.current_token.line, column: self.current_token.column };
        let name = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }
        let parameters = self.parse_function_parameters()?;
        if !self.expect_peek(TokenKind::Arrow) {
            return None;
        }
        self.next_token();
        let return_type = self.parse_type()?;
        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }
        let body = self.parse_block_statement();
        Some(Statement::MethodDecl {
            receiver_type,
            receiver_name,
            name,
            parameters,
            return_type,
            body,
            name_span,
        })
    }

    fn parse_interface_declaration(&mut self) -> Option<Statement> {
        // myat Name { loke methodName(params) -> retType; ... }
        if !self.expect_peek_identifier() {
            return None;
        }
        let name_span = Span { line: self.current_token.line, column: self.current_token.column };
        let name = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }
        let mut methods = Vec::new();
        self.next_token(); // move past '{'
        while self.current_token.kind != TokenKind::RBrace && self.current_token.kind != TokenKind::Eof {
            // Each method: loke methodName(params) -> retType;
            if self.current_token.kind != TokenKind::Loke {
                self.errors.push(ParseError {
                    message: format!("Expected 'loke' in interface method declaration, got {:?}", self.current_token.kind),
                    line: self.current_token.line,
                    column: self.current_token.column,
                });
                return None;
            }
            if !self.expect_peek_identifier() {
                return None;
            }
            let method_name = match &self.current_token.kind {
                TokenKind::Identifier(n) => n.clone(),
                _ => return None,
            };
            if !self.expect_peek(TokenKind::LParen) {
                return None;
            }
            // Parse parameter types (simplified: Type name pairs)
            let params = self.parse_interface_params()?;
            if !self.expect_peek(TokenKind::Arrow) {
                return None;
            }
            self.next_token();
            let ret_type = self.parse_type()?;
            methods.push((method_name, params, ret_type));
            if self.peek_token.kind == TokenKind::Semicolon {
                self.next_token(); // consume ';'
            }
            self.next_token();
        }
        Some(Statement::InterfaceDecl { name, methods, name_span })
    }

    fn parse_interface_params(&mut self) -> Option<Vec<(String, Type)>> {
        let mut params = Vec::new();
        if self.peek_token.kind == TokenKind::RParen {
            self.next_token();
            return Some(params);
        }
        self.next_token();
        let ty = self.parse_type()?;
        if !self.expect_peek_identifier() { return None; }
        let name = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        params.push((name, ty));
        while self.peek_token.kind == TokenKind::Comma {
            self.next_token();
            self.next_token();
            let ty = self.parse_type()?;
            if !self.expect_peek_identifier() { return None; }
            let name = match &self.current_token.kind {
                TokenKind::Identifier(n) => n.clone(),
                _ => return None,
            };
            params.push((name, ty));
        }
        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }
        Some(params)
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<(String, Type, Span)>> {
        let mut identifiers = Vec::new();

        if self.peek_token.kind == TokenKind::RParen {
            self.next_token();
            return Some(identifiers);
        }

        self.next_token();

        let ty = self.parse_type()?;
        if !self.expect_peek_identifier() { return None; }

        let span = Span { line: self.current_token.line, column: self.current_token.column };
        match &self.current_token.kind {
            TokenKind::Identifier(n) => identifiers.push((n.clone(), ty, span)),
            _ => return None,
        };

        while self.peek_token.kind == TokenKind::Comma {
            self.next_token();
            self.next_token();

            let ty = self.parse_type()?;
            if !self.expect_peek_identifier() { return None; }

            let span = Span { line: self.current_token.line, column: self.current_token.column };
            match &self.current_token.kind {
                TokenKind::Identifier(n) => identifiers.push((n.clone(), ty, span)),
                _ => return None,
            };
        }

        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }

        Some(identifiers)
    }

    fn parse_block_statement(&mut self) -> BlockStatement {
        let mut statements = Vec::new();

        self.next_token();

        while self.current_token.kind != TokenKind::RBrace && self.current_token.kind != TokenKind::Eof {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        BlockStatement { statements }
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token();
        }

        Some(Statement::ExpressionStatement(expr))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left = match &self.current_token.kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                // Check for pyaung_ type conversion functions
                if name.starts_with("pyaung_") {
                    self.parse_type_conversion(&name)
                } else if self.peek_token.kind == TokenKind::LBrace && name.chars().next().map_or(false, |c| c.is_uppercase()) {
                    // Struct literal: Name { field: val, ... } (only if name starts with uppercase)
                    Some(self.parse_struct_literal_or_ident(&name))
                } else {
                    Some(Expression::Identifier(name))
                }
            }
            TokenKind::Number(val) => Some(Expression::IntegerLiteral(*val)),
            TokenKind::FloatLiteral(val) => Some(Expression::FloatLiteral(*val)),
            TokenKind::StringLiteral(val) => Some(Expression::StringLiteral(val.clone())),
            TokenKind::Hman => Some(Expression::BooleanLiteral(true)),
            TokenKind::Hmar => Some(Expression::BooleanLiteral(false)),
            TokenKind::Bhala => Some(Expression::NilLiteral),
            TokenKind::Amhar => self.parse_error_create(),
            // htae and ashay used as built-in function calls
            TokenKind::Htae => {
                if self.peek_token.kind == TokenKind::LParen {
                    Some(Expression::Identifier("htae".to_string()))
                } else {
                    self.errors.push(ParseError {
                        message: format!("Unexpected htae in expression context"),
                        line: self.current_token.line,
                        column: self.current_token.column,
                    });
                    None
                }
            }
            TokenKind::LParen => self.parse_grouped_expression(),
            TokenKind::LBracket => self.parse_array_literal(),
            TokenKind::LBrace => self.parse_hash_literal(),
            TokenKind::Phat => self.parse_read_input(),
            _ => {
                self.errors.push(ParseError {
                    message: format!("No prefix parse function for {:?}", self.current_token.kind),
                    line: self.current_token.line,
                    column: self.current_token.column,
                });
                None
            }
        }?;

        while self.peek_token.kind != TokenKind::Semicolon && precedence < self.peek_precedence() {
            match self.peek_token.kind {
                TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash |
                TokenKind::Equals | TokenKind::NotEquals | TokenKind::LessThan |
                TokenKind::GreaterThan | TokenKind::LessEquals | TokenKind::GreaterEquals => {
                    self.next_token();
                    left = self.parse_infix_expression(left)?;
                }
                TokenKind::LParen => {
                    self.next_token();
                    left = self.parse_call_expression(left)?;
                }
                TokenKind::LBracket => {
                    self.next_token();
                    left = self.parse_index_or_slice_expression(left)?;
                }
                TokenKind::Dot => {
                    self.next_token(); // consume the Dot
                    left = self.parse_dot_expression(left)?;
                }
                _ => return Some(left),
            }
        }

        Some(left)
    }

    fn parse_type_conversion(&mut self, name: &str) -> Option<Expression> {
        let suffix = &name["pyaung_".len()..];
        let target_type = match suffix {
            "kain" => Type::Kain,
            "sar" => Type::Sar,
            "da_tha" => Type::DaTha,
            _ => {
                self.errors.push(ParseError {
                    message: format!("Unknown type conversion: {}", name),
                    line: self.current_token.line,
                    column: self.current_token.column,
                });
                return None;
            }
        };
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }
        self.next_token();
        let argument = self.parse_expression(Precedence::Lowest)?;
        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }
        Some(Expression::TypeConversion {
            target_type,
            argument: Box::new(argument),
        })
    }

    fn parse_struct_literal_or_ident(&mut self, name: &str) -> Expression {
        // Try to parse as struct literal: Name { field: val, ... }
        // We need to peek past LBrace to check if it looks like field: value
        let name = name.to_string();
        self.next_token(); // consume '{'
        
        // Check for empty struct literal or field: pattern
        if self.peek_token.kind == TokenKind::RBrace {
            self.next_token(); // consume '}'
            return Expression::StructLiteral { name, fields: vec![] };
        }
        
        // Check if this looks like `identifier:` pattern (struct literal)
        // If peek after identifier is Colon, it's a struct literal
        self.next_token(); // move to first item
        
        if let TokenKind::Identifier(field_name) = &self.current_token.kind {
            if self.peek_token.kind == TokenKind::Colon {
                let field_name = field_name.clone();
                self.next_token(); // consume ':'
                self.next_token(); // move to value
                let value = match self.parse_expression(Precedence::Lowest) {
                    Some(v) => v,
                    None => return Expression::Identifier(name),
                };
                let mut fields = vec![(field_name, value)];
                while self.peek_token.kind == TokenKind::Comma {
                    self.next_token(); // consume ','
                    self.next_token(); // move to field name
                    let fname = match &self.current_token.kind {
                        TokenKind::Identifier(n) => n.clone(),
                        _ => break,
                    };
                    if self.peek_token.kind != TokenKind::Colon {
                        break;
                    }
                    self.next_token(); // consume ':'
                    self.next_token(); // move to value
                    let fval = match self.parse_expression(Precedence::Lowest) {
                        Some(v) => v,
                        None => break,
                    };
                    fields.push((fname, fval));
                }
                if self.peek_token.kind == TokenKind::RBrace {
                    self.next_token(); // consume '}'
                }
                return Expression::StructLiteral { name, fields };
            }
        }
        
        // Not a struct literal - this was actually a hash literal or something else
        // This shouldn't normally happen in well-formed code
        Expression::Identifier(name)
    }

    fn parse_error_create(&mut self) -> Option<Expression> {
        // amhar("message")
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }
        self.next_token();
        let message = self.parse_expression(Precedence::Lowest)?;
        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }
        Some(Expression::ErrorCreate { message: Box::new(message) })
    }

    fn parse_dot_expression(&mut self, left: Expression) -> Option<Expression> {
        // left.identifier or left.identifier(args)
        self.next_token(); // move past Dot to identifier
        let method_or_field = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => {
                self.errors.push(ParseError {
                    message: format!("Expected identifier after '.', got {:?}", self.current_token.kind),
                    line: self.current_token.line,
                    column: self.current_token.column,
                });
                return None;
            }
        };
        // Check if it's a method call (followed by '(')
        if self.peek_token.kind == TokenKind::LParen {
            self.next_token(); // consume '('
            let arguments = self.parse_expression_list(TokenKind::RParen)?;
            Some(Expression::MethodCall {
                object: Box::new(left),
                method: method_or_field,
                arguments,
            })
        } else {
            Some(Expression::FieldAccess {
                object: Box::new(left),
                field: method_or_field,
            })
        }
    }

    fn parse_index_or_slice_expression(&mut self, left: Expression) -> Option<Expression> {
        self.next_token(); // move past '['
        
        // Check for [:high] form
        if self.current_token.kind == TokenKind::Colon {
            self.next_token();
            let high = self.parse_expression(Precedence::Lowest)?;
            if !self.expect_peek(TokenKind::RBracket) { return None; }
            return Some(Expression::SliceExpression {
                left: Box::new(left),
                low: None,
                high: Some(Box::new(high)),
            });
        }
        
        let index_or_low = self.parse_expression(Precedence::Lowest)?;
        
        // Check for slice: [low:high] or [low:]
        if self.peek_token.kind == TokenKind::Colon {
            self.next_token(); // consume ':'
            // Check for [low:] form (no high)
            if self.peek_token.kind == TokenKind::RBracket {
                self.next_token(); // consume ']'
                return Some(Expression::SliceExpression {
                    left: Box::new(left),
                    low: Some(Box::new(index_or_low)),
                    high: None,
                });
            }
            self.next_token();
            let high = self.parse_expression(Precedence::Lowest)?;
            if !self.expect_peek(TokenKind::RBracket) { return None; }
            return Some(Expression::SliceExpression {
                left: Box::new(left),
                low: Some(Box::new(index_or_low)),
                high: Some(Box::new(high)),
            });
        }
        
        if !self.expect_peek(TokenKind::RBracket) { return None; }
        Some(Expression::IndexExpression {
            left: Box::new(left),
            index: Box::new(index_or_low),
        })
    }

    fn parse_grouped_expression(&mut self) -> Option<Expression> {
        self.next_token();
        let exp = self.parse_expression(Precedence::Lowest)?;
        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }
        Some(exp)
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Option<Expression> {
        let operator = self.token_kind_to_string(&self.current_token.kind);
        let precedence = self.current_precedence();
        self.next_token();
        
        let right = self.parse_expression(precedence)?;
        
        Some(Expression::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    fn parse_call_expression(&mut self, function: Expression) -> Option<Expression> {
        let func_name = match function {
            Expression::Identifier(n) => n,
            _ => return None,
        };

        let arguments = self.parse_expression_list(TokenKind::RParen)?;
        Some(Expression::FunctionCall {
            function: func_name,
            arguments,
        })
    }

    fn parse_array_literal(&mut self) -> Option<Expression> {
        let elements = self.parse_expression_list(TokenKind::RBracket)?;
        Some(Expression::ArrayLiteral { elements })
    }

    fn parse_hash_literal(&mut self) -> Option<Expression> {
        let mut pairs = Vec::new();

        if self.peek_token.kind == TokenKind::RBrace {
            self.next_token();
            return Some(Expression::HashLiteral { pairs });
        }

        self.next_token();

        let key = self.parse_expression(Precedence::Lowest)?;
        if !self.expect_peek(TokenKind::Colon) { return None; }
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest)?;
        pairs.push((key, value));

        while self.peek_token.kind == TokenKind::Comma {
            self.next_token();
            self.next_token();
            let key = self.parse_expression(Precedence::Lowest)?;
            if !self.expect_peek(TokenKind::Colon) { return None; }
            self.next_token();
            let value = self.parse_expression(Precedence::Lowest)?;
            pairs.push((key, value));
        }

        if !self.expect_peek(TokenKind::RBrace) {
            return None;
        }

        Some(Expression::HashLiteral { pairs })
    }

    fn parse_read_input(&mut self) -> Option<Expression> {
        if !self.expect_peek(TokenKind::LParen) { return None; }
        self.next_token();
        let prompt = self.parse_expression(Precedence::Lowest)?;
        if !self.expect_peek(TokenKind::RParen) { return None; }
        Some(Expression::ReadInput { prompt: Box::new(prompt) })
    }

    fn parse_expression_list(&mut self, end: TokenKind) -> Option<Vec<Expression>> {
        let mut args = Vec::new();

        if self.peek_token.kind == end {
            self.next_token();
            return Some(args);
        }

        self.next_token();
        args.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token.kind == TokenKind::Comma {
            self.next_token();
            self.next_token();
            args.push(self.parse_expression(Precedence::Lowest)?);
        }

        if !self.expect_peek(end) {
            return None;
        }

        Some(args)
    }

    fn peek_precedence(&self) -> Precedence {
        self.get_precedence(&self.peek_token.kind)
    }

    fn current_precedence(&self) -> Precedence {
        self.get_precedence(&self.current_token.kind)
    }

    fn get_precedence(&self, kind: &TokenKind) -> Precedence {
        match kind {
            TokenKind::Equals | TokenKind::NotEquals => Precedence::Equals,
            TokenKind::LessThan | TokenKind::GreaterThan | TokenKind::LessEquals | TokenKind::GreaterEquals => Precedence::LessGreater,
            TokenKind::Plus | TokenKind::Minus => Precedence::Sum,
            TokenKind::Star | TokenKind::Slash => Precedence::Product,
            TokenKind::LParen => Precedence::Call,
            TokenKind::Dot => Precedence::Call,
            TokenKind::LBracket => Precedence::Index,
            _ => Precedence::Lowest,
        }
    }

    fn token_kind_to_string(&self, kind: &TokenKind) -> String {
        match kind {
            TokenKind::Plus => "+".to_string(),
            TokenKind::Minus => "-".to_string(),
            TokenKind::Star => "*".to_string(),
            TokenKind::Slash => "/".to_string(),
            TokenKind::Equals => "==".to_string(),
            TokenKind::NotEquals => "!=".to_string(),
            TokenKind::LessThan => "<".to_string(),
            TokenKind::GreaterThan => ">".to_string(),
            TokenKind::LessEquals => "<=".to_string(),
            TokenKind::GreaterEquals => ">=".to_string(),
            _ => "".to_string(),
        }
    }

    fn expect_peek(&mut self, kind: TokenKind) -> bool {
        if self.peek_token.kind == kind {
            self.next_token();
            true
        } else {
            self.peek_error(kind);
            false
        }
    }

    fn expect_peek_identifier(&mut self) -> bool {
        match self.peek_token.kind {
            TokenKind::Identifier(_) => {
                self.next_token();
                true
            }
            _ => {
                self.errors.push(ParseError {
                    message: format!("Expected next token to be Identifier, got {:?}", self.peek_token.kind),
                    line: self.peek_token.line,
                    column: self.peek_token.column,
                });
                false
            }
        }
    }

    fn peek_error(&mut self, _kind: TokenKind) {
        self.errors.push(ParseError {
            message: format!("Unexpected token: {:?}", self.peek_token.kind),
            line: self.peek_token.line,
            column: self.peek_token.column,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_let_statements() {
        let input = r#"
            kain age = ၂၀;
            sar name = "Aung";
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);

        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 2);
    }

    #[test]
    fn test_spans_are_recorded() {
        use crate::ast::Span;
        // Line 1: loke main() -> kain {
        // Line 2:     kain age = ၂၀;
        // Line 3: }
        let input = "loke main() -> kain {\n    kain age = ၂၀;\n}";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        if let Statement::FunctionDecl { name, name_span, body, .. } = &program.statements[0] {
            assert_eq!(name, "main");
            println!("FunctionDecl '{}' span: line={}, col={}", name, name_span.line, name_span.column);
            // Lexer starts at line=1, so function name should be line 1
            assert_eq!(name_span.line, 1);
            assert!(name_span.column > 0, "column should be > 0");

            if let Statement::Let { name, name_span, .. } = &body.statements[0] {
                assert_eq!(name, "age");
                println!("Let '{}' span: line={}, col={}", name, name_span.line, name_span.column);
                // Should be on line 2
                assert_eq!(name_span.line, 2);
                assert!(name_span.column > 0);
            } else {
                panic!("Expected Let statement in body");
            }
        } else {
            panic!("Expected FunctionDecl");
        }
    }

    #[test]
    fn test_function_declaration() {
        let input = r#"
            loke main(kain a, sar b) -> kain {
                pyan a;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);

        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);
        if let Statement::FunctionDecl { name, parameters, return_type, .. } = &program.statements[0] {
            assert_eq!(name, "main");
            assert_eq!(parameters.len(), 2);
            assert_eq!(return_type, &Type::Kain);
        } else {
            panic!("Expected FunctionDecl");
        }
    }

    #[test]
    fn test_arrays_and_hashmaps() {
        let input = r#"
            su<kain> numbers = [၁, ၂, ၃];
            twe<sar, kain> dict = {"a": 1, "b": 2};
            kain first = numbers[၀];
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);

        let program = parser.parse_program().unwrap();

        if parser.errors.len() != 0 {
            for error in &parser.errors {
                println!("Parser Error: {}", error);
            }
        }

        assert_eq!(parser.errors.len(), 0);
        assert_eq!(program.statements.len(), 3);
    }

    #[test]
    fn test_read_input() {
        let input = r#"
            sar name = phat("name?");
        "#;
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);

        let program = parser.parse_program().unwrap();
        assert_eq!(parser.errors.len(), 0);
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_elif() {
        use crate::ast::IfAlternative;

        let input = r#"
            hlyin (x == ၁) {
                pya("one");
            } mo hlyin (x == ၂) {
                pya("two");
            } mo {
                pya("other");
            }
        "#;
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);

        let program = parser.parse_program().unwrap();

        if !parser.errors.is_empty() {
            for error in &parser.errors {
                println!("Parser Error: {}", error);
            }
        }
        assert_eq!(parser.errors.len(), 0);
        assert_eq!(program.statements.len(), 1);

        // Verify the top-level if statement has an ElseIf alternative
        if let Statement::If { alternative, .. } = &program.statements[0] {
            match alternative {
                Some(IfAlternative::ElseIf(elif_stmt)) => {
                    // The elif should itself be an If with an Else alternative
                    if let Statement::If { alternative: inner_alt, .. } = elif_stmt.as_ref() {
                        assert!(matches!(inner_alt, Some(IfAlternative::Else(_))));
                    } else {
                        panic!("Expected inner If statement in elif");
                    }
                }
                _ => panic!("Expected ElseIf alternative, got {:?}", alternative),
            }
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_for_in_loop() {
        let input = r#"
            su<kain> numbers = [၁, ၂, ၃];
            pat item htae numbers {
                pya(item);
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);
        assert_eq!(program.statements.len(), 2);
        assert!(matches!(program.statements[1], Statement::ForIn { .. }));
    }
}
