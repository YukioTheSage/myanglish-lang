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
    block_depth: usize,
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
            block_depth: 0,
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
            TokenKind::Atote => self.parse_package_statement(),
            TokenKind::Pay => self.parse_export_statement(),
            TokenKind::Kain
            | TokenKind::Sar
            | TokenKind::Sit
            | TokenKind::DaTha
            | TokenKind::Su
            | TokenKind::Twe
            | TokenKind::Laung
            | TokenKind::Baung => self.parse_let_or_destructured(),
            TokenKind::Hlyin => self.parse_if_statement(),
            TokenKind::Pyan => self.parse_return_statement(),
            TokenKind::Pat => self.parse_pat_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            TokenKind::Kyoe => self.parse_go_statement(),
            TokenKind::NautSone => self.parse_defer_statement(),
            TokenKind::SetSae => self.parse_test_declaration(),
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
                if self.peek_is_identifier() {
                    // Could be `StructName varName = expr;` (struct type let)
                    self.parse_let_or_destructured()
                } else {
                    self.parse_reassignment_or_expression_statement()
                }
            }
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
            TokenKind::Laung => self.parse_channel_type(),
            TokenKind::Baung => Some(Type::Baung),
            TokenKind::Twe => self.parse_map_type(),
            TokenKind::LParen => self.parse_tuple_type(),
            TokenKind::Loke => self.parse_function_type(),
            TokenKind::Identifier(name) => self
                .parse_qualified_type_name(name.clone())
                .map(Type::Struct),
            _ => None,
        }
    }

    fn parse_qualified_type_name(&mut self, first: String) -> Option<String> {
        let mut full_name = first;
        while self.peek_token.kind == TokenKind::Dot {
            self.next_token(); // consume '.'
            if !self.expect_peek_identifier() {
                return None;
            }
            let segment = match &self.current_token.kind {
                TokenKind::Identifier(name) => name.clone(),
                _ => return None,
            };
            full_name.push('.');
            full_name.push_str(&segment);
        }
        Some(full_name)
    }

    fn parse_function_type(&mut self) -> Option<Type> {
        // Syntax: loke(type1, type2, ...) -> return_type
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }

        let mut params = Vec::new();
        if self.peek_token.kind == TokenKind::RParen {
            self.next_token();
        } else {
            self.next_token();
            params.push(self.parse_type()?);
            while self.peek_token.kind == TokenKind::Comma {
                self.next_token(); // consume ','
                self.next_token(); // move to next type
                params.push(self.parse_type()?);
            }
            if !self.expect_peek(TokenKind::RParen) {
                return None;
            }
        }

        if !self.expect_peek(TokenKind::Arrow) {
            return None;
        }
        self.next_token();
        let return_type = self.parse_type()?;

        Some(Type::Function {
            params,
            return_type: Box::new(return_type),
        })
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
        if !self.expect_peek(TokenKind::LessThan) {
            return None;
        }
        self.next_token();
        let inner_type = self.parse_type()?;
        if !self.expect_peek(TokenKind::GreaterThan) {
            return None;
        }
        Some(Type::Array(Box::new(inner_type)))
    }

    fn parse_map_type(&mut self) -> Option<Type> {
        // Syntax: twe<sar, kain>
        if !self.expect_peek(TokenKind::LessThan) {
            return None;
        }
        self.next_token();
        let key_type = self.parse_type()?;
        if !self.expect_peek(TokenKind::Comma) {
            return None;
        }
        self.next_token();
        let val_type = self.parse_type()?;
        if !self.expect_peek(TokenKind::GreaterThan) {
            return None;
        }
        Some(Type::Map(Box::new(key_type), Box::new(val_type)))
    }

    fn parse_channel_type(&mut self) -> Option<Type> {
        // Syntax: laung<kain>
        if !self.expect_peek(TokenKind::LessThan) {
            return None;
        }
        self.next_token();
        let inner_type = self.parse_type()?;
        if !self.expect_peek(TokenKind::GreaterThan) {
            return None;
        }
        Some(Type::Channel(Box::new(inner_type)))
    }

    fn parse_package_statement(&mut self) -> Option<Statement> {
        if self.block_depth > 0 {
            self.errors.push(ParseError {
                message: "`atote` is only allowed at top level".to_string(),
                line: self.current_token.line,
                column: self.current_token.column,
            });
            return None;
        }

        if !self.expect_peek_identifier() {
            return None;
        }

        let name_span = Span {
            line: self.current_token.line,
            column: self.current_token.column,
        };
        let name = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };

        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token();
        }

        Some(Statement::PackageDecl { name, name_span })
    }

    fn parse_export_statement(&mut self) -> Option<Statement> {
        let name_span = Span {
            line: self.current_token.line,
            column: self.current_token.column,
        };

        if self.block_depth > 0 {
            self.errors.push(ParseError {
                message: "`pay` is only allowed at top level".to_string(),
                line: self.current_token.line,
                column: self.current_token.column,
            });
            return None;
        }

        self.next_token();
        let inner = self.parse_statement()?;
        let exportable = matches!(
            inner,
            Statement::Let { .. }
                | Statement::FunctionDecl { .. }
                | Statement::StructDecl { .. }
                | Statement::MethodDecl { .. }
                | Statement::InterfaceDecl { .. }
        );

        if !exportable {
            self.errors.push(ParseError {
                message: "`pay` can only export declarations".to_string(),
                line: name_span.line,
                column: name_span.column,
            });
            return None;
        }

        Some(Statement::Export {
            statement: Box::new(inner),
            name_span,
        })
    }

    fn parse_import_statement(&mut self) -> Option<Statement> {
        self.next_token();

        let name_span = Span {
            line: self.current_token.line,
            column: self.current_token.column,
        };
        let module = match &self.current_token.kind {
            TokenKind::Identifier(n) | TokenKind::StringLiteral(n) => n.clone(),
            _ => return None,
        };

        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token();
        }

        Some(Statement::Import { module, name_span })
    }

    fn parse_test_declaration(&mut self) -> Option<Statement> {
        if self.block_depth > 0 {
            self.errors.push(ParseError {
                message: "`set_sae` is only allowed at top level".to_string(),
                line: self.current_token.line,
                column: self.current_token.column,
            });
            return None;
        }

        if !self.expect_peek_identifier() {
            return None;
        }

        let name_span = Span {
            line: self.current_token.line,
            column: self.current_token.column,
        };
        let name = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        let body = self.parse_block_statement();
        Some(Statement::TestDecl {
            name,
            body,
            name_span,
        })
    }

    fn parse_let_or_destructured(&mut self) -> Option<Statement> {
        // Parse first type + name
        let ty1 = self.parse_type()?;
        if !self.expect_peek_identifier() {
            return None;
        }
        let name_span1 = Span {
            line: self.current_token.line,
            column: self.current_token.column,
        };
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
                let span = Span {
                    line: self.current_token.line,
                    column: self.current_token.column,
                };
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
        Some(Statement::Let {
            name: name1,
            value,
            ty: ty1,
            name_span: name_span1,
        })
    }

    fn parse_reassignment_or_expression_statement(&mut self) -> Option<Statement> {
        let name_span = Span {
            line: self.current_token.line,
            column: self.current_token.column,
        };
        let left = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token.kind == TokenKind::Assign {
            self.next_token(); // move to '='
            self.next_token(); // move to assigned value
            let value = self.parse_expression(Precedence::Lowest)?;
            if self.peek_token.kind == TokenKind::Semicolon {
                self.next_token();
            }

            return match left {
                Expression::Identifier(name) => Some(Statement::Assign {
                    name,
                    value,
                    name_span,
                }),
                Expression::FieldAccess { object, field } => match *object {
                    Expression::Identifier(object_name) => Some(Statement::FieldAssign {
                        object: object_name,
                        field,
                        value,
                        name_span,
                    }),
                    _ => {
                        self.errors.push(ParseError {
                            message: "Invalid field assignment target".to_string(),
                            line: name_span.line,
                            column: name_span.column,
                        });
                        None
                    }
                },
                Expression::IndexExpression { left, index } => Some(Statement::IndexAssign {
                    object: *left,
                    index: *index,
                    value,
                    name_span,
                }),
                _ => {
                    self.errors.push(ParseError {
                        message: "Invalid assignment target".to_string(),
                        line: name_span.line,
                        column: name_span.column,
                    });
                    None
                }
            };
        }

        // Qualified type declaration in expression position:
        // `module.Type name = expr;`
        // `module.Type a, amhar b = expr;`
        if let Expression::FieldAccess { object, field } = left.clone() {
            if let Expression::Identifier(type_ns) = *object {
                if self.peek_is_identifier() {
                    self.next_token(); // move to declared variable name
                    let first_name_span = Span {
                        line: self.current_token.line,
                        column: self.current_token.column,
                    };
                    let first_name = match &self.current_token.kind {
                        TokenKind::Identifier(n) => n.clone(),
                        _ => return None,
                    };
                    let first_ty = Type::Struct(format!("{}.{}", type_ns, field));

                    if self.peek_token.kind == TokenKind::Comma {
                        self.next_token(); // consume ','
                        let mut names = vec![(first_name, first_ty, first_name_span)];
                        loop {
                            self.next_token(); // move to next type
                            let ty = self.parse_type()?;
                            if !self.expect_peek_identifier() {
                                return None;
                            }
                            let span = Span {
                                line: self.current_token.line,
                                column: self.current_token.column,
                            };
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

                    if !self.expect_peek(TokenKind::Assign) {
                        return None;
                    }
                    self.next_token();
                    let value = self.parse_expression(Precedence::Lowest)?;
                    if self.peek_token.kind == TokenKind::Semicolon {
                        self.next_token();
                    }
                    return Some(Statement::Let {
                        name: first_name,
                        value,
                        ty: first_ty,
                        name_span: first_name_span,
                    });
                }
            }
        }

        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token();
        }

        Some(Statement::ExpressionStatement(left))
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
                return Some(Statement::Return {
                    value: Expression::TupleLiteral { elements },
                });
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

    fn parse_break_statement(&mut self) -> Option<Statement> {
        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token();
        }
        Some(Statement::Break)
    }

    fn parse_continue_statement(&mut self) -> Option<Statement> {
        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token();
        }
        Some(Statement::Continue)
    }

    fn parse_go_statement(&mut self) -> Option<Statement> {
        self.next_token();
        let call = self.parse_expression(Precedence::Lowest)?;
        if !matches!(
            call,
            Expression::FunctionCall { .. } | Expression::MethodCall { .. }
        ) {
            self.errors.push(ParseError {
                message: "`kyoe` expects a function or method call".to_string(),
                line: self.current_token.line,
                column: self.current_token.column,
            });
            return None;
        }
        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token();
        }
        Some(Statement::Go { call })
    }

    fn parse_defer_statement(&mut self) -> Option<Statement> {
        self.next_token();
        let call = self.parse_expression(Precedence::Lowest)?;
        if !matches!(
            call,
            Expression::FunctionCall { .. } | Expression::MethodCall { .. }
        ) {
            self.errors.push(ParseError {
                message: "`naut_sone` expects a function or method call".to_string(),
                line: self.current_token.line,
                column: self.current_token.column,
            });
            return None;
        }
        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token();
        }
        Some(Statement::Defer { call })
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

    fn parse_pat_statement(&mut self) -> Option<Statement> {
        if self.peek_token.kind == TokenKind::LParen {
            return self.parse_paren_pat_statement();
        }

        self.parse_for_in_statement()
    }

    fn parse_paren_pat_statement(&mut self) -> Option<Statement> {
        // We already know peek is '('
        self.next_token(); // consume '('
        self.next_token(); // move to first token inside parentheses

        if self.current_token.kind == TokenKind::Semicolon
            || (matches!(self.current_token.kind, TokenKind::Identifier(_))
                && self.peek_token.kind == TokenKind::Assign)
        {
            return self.parse_classic_for_statement();
        }

        // For-in with index/typed variables:
        // pat (kain i, kain item) htae collection { ... }
        if self.token_can_start_type(&self.current_token.kind) && self.peek_is_identifier() {
            let first_ty = self.parse_type()?;
            if !self.expect_peek_identifier() {
                return None;
            }
            let first_name = match &self.current_token.kind {
                TokenKind::Identifier(n) => n.clone(),
                _ => return None,
            };
            let first_name_span = Span {
                line: self.current_token.line,
                column: self.current_token.column,
            };

            if self.peek_token.kind == TokenKind::Assign {
                self.next_token(); // '='
                self.next_token(); // first token of init value
                let init_value = self.parse_expression(Precedence::Lowest)?;
                let init_stmt = Statement::Let {
                    name: first_name,
                    value: init_value,
                    ty: first_ty,
                    name_span: first_name_span,
                };
                return self.parse_classic_for_tail(Some(init_stmt));
            }

            let mut name_span = Span {
                line: self.current_token.line,
                column: self.current_token.column,
            };

            let (index, iterator) = if self.peek_token.kind == TokenKind::Comma {
                self.next_token(); // consume ','
                self.next_token(); // move to second type
                let _second_ty = self.parse_type()?;
                if !self.expect_peek_identifier() {
                    return None;
                }
                let second_name = match &self.current_token.kind {
                    TokenKind::Identifier(n) => n.clone(),
                    _ => return None,
                };
                name_span = Span {
                    line: self.current_token.line,
                    column: self.current_token.column,
                };
                (Some(first_name), second_name)
            } else {
                (None, first_name)
            };

            if !self.expect_peek(TokenKind::RParen) {
                return None;
            }
            if !self.expect_peek(TokenKind::Htae) {
                return None;
            }

            self.next_token();
            let collection = self.parse_expression(Precedence::Lowest)?;

            if !self.expect_peek(TokenKind::LBrace) {
                return None;
            }

            let body = self.parse_block_statement();

            return Some(Statement::ForIn {
                index,
                iterator,
                collection,
                body,
                name_span,
            });
        }

        // For-in with untyped index variables:
        // pat (i, item) htae collection { ... }
        if let TokenKind::Identifier(first_name) = &self.current_token.kind {
            if self.peek_token.kind == TokenKind::Comma {
                let index_name = first_name.clone();
                let first_span = Span {
                    line: self.current_token.line,
                    column: self.current_token.column,
                };
                self.next_token(); // consume ','
                if !self.expect_peek_identifier() {
                    return None;
                }
                let iterator = match &self.current_token.kind {
                    TokenKind::Identifier(n) => n.clone(),
                    _ => return None,
                };

                if !self.expect_peek(TokenKind::RParen) {
                    return None;
                }
                if !self.expect_peek(TokenKind::Htae) {
                    return None;
                }

                self.next_token();
                let collection = self.parse_expression(Precedence::Lowest)?;

                if !self.expect_peek(TokenKind::LBrace) {
                    return None;
                }

                let body = self.parse_block_statement();
                return Some(Statement::ForIn {
                    index: Some(index_name),
                    iterator,
                    collection,
                    body,
                    name_span: first_span,
                });
            }
        }

        // Regular while loop form: pat (<condition>) { ... }
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

    fn parse_classic_for_statement(&mut self) -> Option<Statement> {
        let init = if self.current_token.kind == TokenKind::Semicolon {
            None
        } else {
            let stmt = self.parse_for_classic_component(true)?;
            Some(stmt)
        };

        self.parse_classic_for_tail(init)
    }

    fn parse_classic_for_tail(&mut self, init: Option<Statement>) -> Option<Statement> {
        if self.current_token.kind != TokenKind::Semicolon {
            if !self.expect_peek(TokenKind::Semicolon) {
                return None;
            }
        }

        self.next_token(); // token after first ';'
        let condition = if self.current_token.kind == TokenKind::Semicolon {
            None
        } else {
            let cond = self.parse_expression(Precedence::Lowest)?;
            Some(cond)
        };

        if self.current_token.kind != TokenKind::Semicolon {
            if !self.expect_peek(TokenKind::Semicolon) {
                return None;
            }
        }

        self.next_token(); // token after second ';'
        let post = if self.current_token.kind == TokenKind::RParen {
            None
        } else {
            let stmt = self.parse_for_classic_component(false)?;
            Some(Box::new(stmt))
        };

        if self.current_token.kind != TokenKind::RParen {
            if !self.expect_peek(TokenKind::RParen) {
                return None;
            }
        }

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }
        let body = self.parse_block_statement();

        Some(Statement::ForClassic {
            init: init.map(Box::new),
            condition,
            post,
            body,
        })
    }

    fn parse_for_classic_component(&mut self, allow_typed_let: bool) -> Option<Statement> {
        if allow_typed_let
            && self.token_can_start_type(&self.current_token.kind)
            && self.peek_is_identifier()
        {
            let ty = self.parse_type()?;
            if !self.expect_peek_identifier() {
                return None;
            }
            let name_span = Span {
                line: self.current_token.line,
                column: self.current_token.column,
            };
            let name = match &self.current_token.kind {
                TokenKind::Identifier(n) => n.clone(),
                _ => return None,
            };
            if !self.expect_peek(TokenKind::Assign) {
                return None;
            }
            self.next_token();
            let value = self.parse_expression(Precedence::Lowest)?;
            return Some(Statement::Let {
                name,
                value,
                ty,
                name_span,
            });
        }

        if let TokenKind::Identifier(name) = &self.current_token.kind {
            if self.peek_token.kind == TokenKind::Assign {
                let span = Span {
                    line: self.current_token.line,
                    column: self.current_token.column,
                };
                let name = name.clone();
                self.next_token(); // '='
                self.next_token(); // value start
                let value = self.parse_expression(Precedence::Lowest)?;
                return Some(Statement::Assign {
                    name,
                    value,
                    name_span: span,
                });
            }
        }

        let expr = self.parse_expression(Precedence::Lowest)?;
        Some(Statement::ExpressionStatement(expr))
    }

    fn token_can_start_type(&self, kind: &TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Kain
                | TokenKind::Sar
                | TokenKind::Sit
                | TokenKind::DaTha
                | TokenKind::Amhar
                | TokenKind::Su
                | TokenKind::Laung
                | TokenKind::Baung
                | TokenKind::Twe
                | TokenKind::Identifier(_)
                | TokenKind::LParen
        )
    }

    fn parse_for_in_statement(&mut self) -> Option<Statement> {
        if !self.expect_peek_identifier() {
            return None;
        }

        let name_span = Span {
            line: self.current_token.line,
            column: self.current_token.column,
        };
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
            index: None,
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

        let name_span = Span {
            line: self.current_token.line,
            column: self.current_token.column,
        };
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
        let name_span = Span {
            line: self.current_token.line,
            column: self.current_token.column,
        };
        let name = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }
        let mut fields = Vec::new();
        self.next_token(); // move past '{'
        while self.current_token.kind != TokenKind::RBrace
            && self.current_token.kind != TokenKind::Eof
        {
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
        Some(Statement::StructDecl {
            name,
            fields,
            name_span,
        })
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
        let name_span = Span {
            line: self.current_token.line,
            column: self.current_token.column,
        };
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
        let name_span = Span {
            line: self.current_token.line,
            column: self.current_token.column,
        };
        let name = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }
        let mut methods = Vec::new();
        self.next_token(); // move past '{'
        while self.current_token.kind != TokenKind::RBrace
            && self.current_token.kind != TokenKind::Eof
        {
            // Each method: loke methodName(params) -> retType;
            if self.current_token.kind != TokenKind::Loke {
                self.errors.push(ParseError {
                    message: format!(
                        "Expected 'loke' in interface method declaration, got {:?}",
                        self.current_token.kind
                    ),
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
        Some(Statement::InterfaceDecl {
            name,
            methods,
            name_span,
        })
    }

    fn parse_interface_params(&mut self) -> Option<Vec<(String, Type)>> {
        let mut params = Vec::new();
        if self.peek_token.kind == TokenKind::RParen {
            self.next_token();
            return Some(params);
        }
        self.next_token();
        let ty = self.parse_type()?;
        if !self.expect_peek_identifier() {
            return None;
        }
        let name = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        params.push((name, ty));
        while self.peek_token.kind == TokenKind::Comma {
            self.next_token();
            self.next_token();
            let ty = self.parse_type()?;
            if !self.expect_peek_identifier() {
                return None;
            }
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
        if !self.expect_peek_identifier() {
            return None;
        }

        let span = Span {
            line: self.current_token.line,
            column: self.current_token.column,
        };
        match &self.current_token.kind {
            TokenKind::Identifier(n) => identifiers.push((n.clone(), ty, span)),
            _ => return None,
        };

        while self.peek_token.kind == TokenKind::Comma {
            self.next_token();
            self.next_token();

            let ty = self.parse_type()?;
            if !self.expect_peek_identifier() {
                return None;
            }

            let span = Span {
                line: self.current_token.line,
                column: self.current_token.column,
            };
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

        self.block_depth += 1;
        self.next_token();

        while self.current_token.kind != TokenKind::RBrace
            && self.current_token.kind != TokenKind::Eof
        {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        self.block_depth = self.block_depth.saturating_sub(1);
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
                } else if self.peek_token.kind == TokenKind::LBrace
                    && name.chars().next().map_or(false, |c| c.is_uppercase())
                {
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
            TokenKind::Loke => self.parse_closure_literal(),
            TokenKind::Laung => self.parse_channel_make(),
            TokenKind::Baung => self.parse_baung_create(),
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
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Equals
                | TokenKind::NotEquals
                | TokenKind::LessThan
                | TokenKind::GreaterThan
                | TokenKind::LessEquals
                | TokenKind::GreaterEquals => {
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
            return Expression::StructLiteral {
                name,
                fields: vec![],
            };
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
        Some(Expression::ErrorCreate {
            message: Box::new(message),
        })
    }

    fn parse_closure_literal(&mut self) -> Option<Expression> {
        // loke(type name, ...) -> return_type { ... }
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }
        let parameters = self.parse_function_parameters()?;

        let return_type = if self.peek_token.kind == TokenKind::Arrow {
            self.next_token(); // consume '->'
            self.next_token(); // move to return type
            self.parse_type()?
        } else {
            Type::Nil
        };

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }
        let body = self.parse_block_statement();

        Some(Expression::ClosureLiteral {
            parameters,
            return_type,
            body,
        })
    }

    fn parse_dot_expression(&mut self, left: Expression) -> Option<Expression> {
        // left.identifier or left.identifier(args)
        self.next_token(); // move past Dot to identifier
        let method_or_field = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => {
                self.errors.push(ParseError {
                    message: format!(
                        "Expected identifier after '.', got {:?}",
                        self.current_token.kind
                    ),
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
            if !self.expect_peek(TokenKind::RBracket) {
                return None;
            }
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
            if !self.expect_peek(TokenKind::RBracket) {
                return None;
            }
            return Some(Expression::SliceExpression {
                left: Box::new(left),
                low: Some(Box::new(index_or_low)),
                high: Some(Box::new(high)),
            });
        }

        if !self.expect_peek(TokenKind::RBracket) {
            return None;
        }
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
        if !self.expect_peek(TokenKind::Colon) {
            return None;
        }
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest)?;
        pairs.push((key, value));

        while self.peek_token.kind == TokenKind::Comma {
            self.next_token();
            self.next_token();
            let key = self.parse_expression(Precedence::Lowest)?;
            if !self.expect_peek(TokenKind::Colon) {
                return None;
            }
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
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }
        self.next_token();
        let prompt = self.parse_expression(Precedence::Lowest)?;
        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }
        Some(Expression::ReadInput {
            prompt: Box::new(prompt),
        })
    }

    fn parse_channel_make(&mut self) -> Option<Expression> {
        // Syntax:
        //   laung<T>()
        //   laung<T>(capacity)
        if !self.expect_peek(TokenKind::LessThan) {
            return None;
        }
        self.next_token();
        let value_type = self.parse_type()?;
        if !self.expect_peek(TokenKind::GreaterThan) {
            return None;
        }
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }

        let capacity = if self.peek_token.kind == TokenKind::RParen {
            self.next_token();
            None
        } else {
            self.next_token();
            let expr = self.parse_expression(Precedence::Lowest)?;
            if !self.expect_peek(TokenKind::RParen) {
                return None;
            }
            Some(Box::new(expr))
        };

        Some(Expression::ChannelMake {
            value_type: Box::new(value_type),
            capacity,
        })
    }

    fn parse_baung_create(&mut self) -> Option<Expression> {
        // Syntax: baung(timeout_ms)
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }
        self.next_token();
        let timeout_ms = self.parse_expression(Precedence::Lowest)?;
        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }
        Some(Expression::BaungCreate {
            timeout_ms: Box::new(timeout_ms),
        })
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
            TokenKind::LessThan
            | TokenKind::GreaterThan
            | TokenKind::LessEquals
            | TokenKind::GreaterEquals => Precedence::LessGreater,
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
                    message: format!(
                        "Expected next token to be Identifier, got {:?}",
                        self.peek_token.kind
                    ),
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
        // Line 1: loke main() -> kain {
        // Line 2:     kain age = ၂၀;
        // Line 3: }
        let input = "loke main() -> kain {\n    kain age = ၂၀;\n}";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );

        if let Statement::FunctionDecl {
            name,
            name_span,
            body,
            ..
        } = &program.statements[0]
        {
            assert_eq!(name, "main");
            println!(
                "FunctionDecl '{}' span: line={}, col={}",
                name, name_span.line, name_span.column
            );
            // Lexer starts at line=1, so function name should be line 1
            assert_eq!(name_span.line, 1);
            assert!(name_span.column > 0, "column should be > 0");

            if let Statement::Let {
                name, name_span, ..
            } = &body.statements[0]
            {
                assert_eq!(name, "age");
                println!(
                    "Let '{}' span: line={}, col={}",
                    name, name_span.line, name_span.column
                );
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
        if let Statement::FunctionDecl {
            name,
            parameters,
            return_type,
            ..
        } = &program.statements[0]
        {
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
    fn test_import_string_and_legacy_syntax() {
        let input = r#"
            yu "json";
            yu json;
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);

        let program = parser.parse_program().unwrap();
        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );
        assert_eq!(program.statements.len(), 2);

        match &program.statements[0] {
            Statement::Import { module, .. } => assert_eq!(module, "json"),
            other => panic!("Expected import statement, got {:?}", other),
        }
        match &program.statements[1] {
            Statement::Import { module, .. } => assert_eq!(module, "json"),
            other => panic!("Expected import statement, got {:?}", other),
        }
    }

    #[test]
    fn test_qualified_struct_type_in_let() {
        let input = r#"
            loke main() -> kain {
                http.Response res = bhala;
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );

        let Statement::FunctionDecl { body, .. } = &program.statements[0] else {
            panic!("Expected function declaration");
        };
        let Statement::Let { ty, .. } = &body.statements[0] else {
            panic!("Expected let statement");
        };
        assert_eq!(ty, &Type::Struct("http.Response".to_string()));
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
                    if let Statement::If {
                        alternative: inner_alt,
                        ..
                    } = elif_stmt.as_ref()
                    {
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

        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );
        assert_eq!(program.statements.len(), 2);
        assert!(matches!(program.statements[1], Statement::ForIn { .. }));
    }

    #[test]
    fn test_for_in_loop_with_index() {
        let input = r#"
            su<kain> numbers = [၁, ၂, ၃];
            pat (kain i, kain item) htae numbers {
                pya(i);
                pya(item);
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );
        assert_eq!(program.statements.len(), 2);
        match &program.statements[1] {
            Statement::ForIn {
                index, iterator, ..
            } => {
                assert_eq!(index.as_deref(), Some("i"));
                assert_eq!(iterator, "item");
            }
            other => panic!("Expected ForIn, got {:?}", other),
        }
    }

    #[test]
    fn test_struct_field_assignment() {
        let input = r#"
pone Person { sar name; kain age; }
loke main() -> kain {
    Person p = Person { name: "Aung", age: 20 };
    p.name = "Ko Ko";
    p.age = 25;
    pya(p.name);
    pyan 0;
}
"#;
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let _program = parser.parse_program().unwrap();
        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );
    }

    #[test]
    fn test_array_index_assignment() {
        let input = r#"
loke main() -> kain {
    su<kain> nums = [1, 2, 3];
    nums[0] = 10;
    twe<sar, kain> prices = {"tea": 500};
    prices["coffee"] = 800;
    pyan 0;
}
"#;
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let _program = parser.parse_program().unwrap();
        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );
    }

    #[test]
    fn test_break_continue_statements() {
        let input = r#"
loke main() -> kain {
    kain i = 0;
    pat (i < 10) {
        i = i + 1;
        hlyin (i == 3) {
            shar;
        }
        hlyin (i == 7) {
            yut;
        }
    }
    pyan 0;
}
"#;
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let _program = parser.parse_program().unwrap();
        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );
    }

    #[test]
    fn test_closure_literal_and_function_type() {
        let input = r#"
loke on_message(loke(sar) -> kain callback) -> kain {
    pyan callback("hello");
}

loke main() -> kain {
    on_message(loke(sar msg) -> kain {
        pya(msg);
        pyan 0;
    });
    pyan 0;
}
"#;
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let _program = parser.parse_program().unwrap();
        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );
    }

    #[test]
    fn test_package_and_export_declarations() {
        let input = r#"
atote util;
pay loke add(kain a, kain b) -> kain {
    pyan a + b;
}
"#;
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );
        assert!(matches!(
            program.statements[0],
            Statement::PackageDecl { .. }
        ));
        assert!(matches!(program.statements[1], Statement::Export { .. }));
    }

    #[test]
    fn test_classic_for_loop_statement() {
        let input = r#"
loke main() -> kain {
    pat (kain i = 0; i < 10; i = i + 1) {
        pya(i);
    }
    pyan 0;
}
"#;
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );
        let Statement::FunctionDecl { body, .. } = &program.statements[0] else {
            panic!("Expected function declaration");
        };
        assert!(matches!(body.statements[0], Statement::ForClassic { .. }));
    }

    #[test]
    fn test_phase3_channel_go_defer_parsing() {
        let input = r#"
        loke worker(laung<kain> ch) -> kain {
            naut_sone ch.close();
            ch.send(1);
            pyan 0;
        }

        loke main() -> kain {
            laung<kain> ch = laung<kain>(10);
            kyoe worker(ch);
            kain v = ch.recv();
            pya(v);
            pyan 0;
        }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );

        let Statement::FunctionDecl { body, .. } = &program.statements[0] else {
            panic!("Expected first function declaration");
        };
        assert!(matches!(body.statements[0], Statement::Defer { .. }));

        let Statement::FunctionDecl { body, .. } = &program.statements[1] else {
            panic!("Expected second function declaration");
        };
        assert!(matches!(body.statements[1], Statement::Go { .. }));
    }

    #[test]
    fn test_phase3_go_defer_require_call() {
        let input = r#"
        loke main() -> kain {
            kyoe 1;
            naut_sone bhala;
            pyan 0;
        }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let _program = parser.parse_program().unwrap();

        assert!(!parser.errors.is_empty());
        assert!(
            parser
                .errors
                .iter()
                .any(|e| e.message.contains("expects a function or method call"))
        );
    }

    #[test]
    fn test_export_inside_block_is_error() {
        let input = r#"
loke main() -> kain {
    pay kain x = 1;
    pyan 0;
}
"#;
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let _program = parser.parse_program().unwrap();
        assert!(!parser.errors.is_empty());
        assert!(
            parser
                .errors
                .iter()
                .any(|e| e.message.contains("only allowed at top level"))
        );
    }

    #[test]
    fn test_phase4_baung_and_set_sae_parsing() {
        let input = r#"
set_sae timeout_guard {
    baung ctx = baung(5000);
    pyan bhala;
}
"#;
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::TestDecl { name, body, .. } => {
                assert_eq!(name, "timeout_guard");
                assert!(matches!(
                    body.statements[0],
                    Statement::Let {
                        ty: Type::Baung,
                        ..
                    }
                ));
            }
            other => panic!("Expected TestDecl, got {:?}", other),
        }
    }

    #[test]
    fn test_set_sae_inside_block_is_error() {
        let input = r#"
loke main() -> kain {
    set_sae bad {
        pyan bhala;
    }
    pyan 0;
}
"#;
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let _program = parser.parse_program().unwrap();
        assert!(
            parser
                .errors
                .iter()
                .any(|e| e.message.contains("`set_sae` is only allowed at top level"))
        );
    }
}
