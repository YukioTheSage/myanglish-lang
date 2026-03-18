use crate::ast::{BlockStatement, Expression, IfAlternative, Program, Statement, Type};

pub struct GoCodeGenerator {
    pub output: String,
    indent_level: usize,
    environment: std::collections::HashMap<String, Type>,
    array_lengths: std::collections::HashMap<String, usize>,
    loop_counter: usize,
    /// Track which helper functions are needed
    needs_read_input: bool,
    /// Track needed imports
    needs_strconv: bool,
    needs_strings: bool,
    needs_errors: bool,
    needs_fmt: bool,
    needs_context: bool,
    needs_time: bool,
    needs_baung_helper: bool,
    needs_net_http: bool,
    needs_gojson: bool,
    /// Track if we're currently inside main() (Go main has no return value)
    in_main: bool,
    /// Registry of struct name -> fields (name, type) for type inference
    struct_fields: std::collections::HashMap<String, Vec<(String, Type)>>,
}

impl GoCodeGenerator {
    pub fn new() -> Self {
        GoCodeGenerator {
            output: String::new(),
            indent_level: 0,
            environment: std::collections::HashMap::new(),
            array_lengths: std::collections::HashMap::new(),
            loop_counter: 0,
            needs_read_input: false,
            needs_strconv: false,
            needs_strings: false,
            needs_errors: false,
            needs_fmt: true,
            needs_context: false,
            needs_time: false,
            needs_baung_helper: false,
            needs_net_http: false,
            needs_gojson: false,
            in_main: false,
            struct_fields: std::collections::HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        // First pass: scan for features that need helpers
        self.scan_program(program);

        // Package declaration
        self.output.push_str("package main\n\n");

        // Imports
        let mut imports: Vec<&str> = Vec::new();
        if self.needs_fmt {
            imports.push("\"fmt\"");
        }
        if self.needs_context {
            imports.push("\"context\"");
        }
        if self.needs_time {
            imports.push("\"time\"");
        }
        if self.needs_net_http {
            imports.push("\"net/http\"");
        }
        if self.needs_gojson {
            imports.push("gojson \"encoding/json\"");
        }

        // Check if we need bufio/os for read input
        if self.needs_read_input {
            imports.push("\"bufio\"");
            imports.push("\"os\"");
        }
        if self.needs_strconv {
            imports.push("\"strconv\"");
        }
        if self.needs_strings {
            imports.push("\"strings\"");
        }
        if self.needs_errors {
            imports.push("\"errors\"");
        }

        if !imports.is_empty() {
            self.output.push_str("import (\n");
            imports.sort();
            for imp in &imports {
                self.output.push_str(&format!("\t{}\n", imp));
            }
            self.output.push_str(")\n\n");
        }

        if self.needs_baung_helper {
            self.output.push_str("type mlangBaung struct {\n");
            self.output.push_str("\tctx context.Context\n");
            self.output.push_str("\tcancel context.CancelFunc\n");
            self.output.push_str("}\n\n");
            self.output
                .push_str("func mlangNewBaung(timeoutMs int64) mlangBaung {\n");
            self.output.push_str("\tctx, cancel := context.WithTimeout(context.Background(), time.Duration(timeoutMs)*time.Millisecond)\n");
            self.output
                .push_str("\treturn mlangBaung{ctx: ctx, cancel: cancel}\n");
            self.output.push_str("}\n\n");
            self.output
                .push_str("func (b mlangBaung) Close() error {\n");
            self.output.push_str("\tif b.cancel != nil {\n");
            self.output.push_str("\t\tb.cancel()\n");
            self.output.push_str("\t}\n");
            self.output.push_str("\treturn nil\n");
            self.output.push_str("}\n\n");
        }

        // Emit read input helper if needed
        if self.needs_read_input {
            self.output
                .push_str("func mlangReadInput(prompt string) string {\n");
            self.output.push_str("\tif prompt != \"\" {\n");
            self.output.push_str("\t\tfmt.Print(prompt)\n");
            self.output.push_str("\t}\n");
            self.output
                .push_str("\tscanner := bufio.NewScanner(os.Stdin)\n");
            self.output.push_str("\tscanner.Scan()\n");
            self.output.push_str("\treturn scanner.Text()\n");
            self.output.push_str("}\n\n");
        }

        // Generate all statements
        // We need to separate: function declarations go at top level,
        // and any top-level statements outside functions are not valid Go,
        // but M-Lang requires a main() function anyway.
        for stmt in &program.statements {
            self.generate_statement(stmt);
        }

        self.output.clone()
    }

    /// Scan the program to determine which helpers/imports are needed
    fn scan_program(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.scan_statement(stmt);
        }
    }

    fn scan_type(&mut self, ty: &Type) {
        match ty {
            Type::Baung => {
                self.needs_baung_helper = true;
                self.needs_context = true;
                self.needs_time = true;
            }
            Type::Array(inner) | Type::Channel(inner) => self.scan_type(inner),
            Type::Map(key, value) => {
                self.scan_type(key);
                self.scan_type(value);
            }
            Type::Tuple(types) => {
                for ty in types {
                    self.scan_type(ty);
                }
            }
            Type::Function {
                params,
                return_type,
            } => {
                for param in params {
                    self.scan_type(param);
                }
                self.scan_type(return_type);
            }
            Type::Struct(name) | Type::Interface(name) => {
                if matches!(
                    name.as_str(),
                    "http.Request" | "http.ResponseWriter" | "http.Response"
                ) {
                    self.needs_net_http = true;
                }
            }
            Type::Kain | Type::Sar | Type::Sit | Type::DaTha | Type::Nil | Type::Error => {}
        }
    }

    fn scan_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::PackageDecl { .. } => {}
            Statement::Let { value, ty, .. } => {
                self.scan_type(ty);
                self.scan_expression(value);
            }
            Statement::LetDestructured { names, value } => {
                for (_, ty, _) in names {
                    self.scan_type(ty);
                }
                self.scan_expression(value);
            }
            Statement::Assign { value, .. } => self.scan_expression(value),
            Statement::FieldAssign { value, .. } => self.scan_expression(value),
            Statement::IndexAssign {
                object,
                index,
                value,
                ..
            } => {
                self.scan_expression(object);
                self.scan_expression(index);
                self.scan_expression(value);
            }
            Statement::Print { value } => self.scan_expression(value),
            Statement::Return { value } => self.scan_expression(value),
            Statement::If {
                condition,
                consequence,
                alternative,
            } => {
                self.scan_expression(condition);
                self.scan_block(consequence);
                if let Some(alt) = alternative {
                    match alt {
                        IfAlternative::Else(block) => self.scan_block(block),
                        IfAlternative::ElseIf(elif) => self.scan_statement(elif),
                    }
                }
            }
            Statement::While { condition, body } => {
                self.scan_expression(condition);
                self.scan_block(body);
            }
            Statement::Break | Statement::Continue => {}
            Statement::Go { call } | Statement::Defer { call } => self.scan_expression(call),
            Statement::TestDecl { body, .. } => self.scan_block(body),
            Statement::ForIn {
                collection, body, ..
            } => {
                self.scan_expression(collection);
                self.scan_block(body);
            }
            Statement::ForClassic {
                init,
                condition,
                post,
                body,
            } => {
                if let Some(init) = init {
                    self.scan_statement(init);
                }
                if let Some(condition) = condition {
                    self.scan_expression(condition);
                }
                if let Some(post) = post {
                    self.scan_statement(post);
                }
                self.scan_block(body);
            }
            Statement::FunctionDecl {
                parameters,
                return_type,
                body,
                ..
            } => {
                for (_, ty, _) in parameters {
                    self.scan_type(ty);
                }
                self.scan_type(return_type);
                self.scan_block(body);
            }
            Statement::MethodDecl {
                parameters,
                return_type,
                body,
                ..
            } => {
                for (_, ty, _) in parameters {
                    self.scan_type(ty);
                }
                self.scan_type(return_type);
                self.scan_block(body);
            }
            Statement::ExpressionStatement(expr) => self.scan_expression(expr),
            Statement::Import { module, .. } => {
                if module.trim_matches('"') == "kainn/http" {
                    self.needs_net_http = true;
                }
            }
            Statement::StructDecl { fields, .. } => {
                for (_, ty) in fields {
                    self.scan_type(ty);
                }
            }
            Statement::InterfaceDecl { methods, .. } => {
                for (_, params, return_type) in methods {
                    for (_, ty) in params {
                        self.scan_type(ty);
                    }
                    self.scan_type(return_type);
                }
            }
            Statement::Export { statement, .. } => self.scan_statement(statement),
        }
    }

    fn scan_block(&mut self, block: &BlockStatement) {
        for stmt in &block.statements {
            self.scan_statement(stmt);
        }
    }

    fn scan_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::ReadInput { .. } => {
                self.needs_read_input = true;
            }
            Expression::TypeConversion {
                target_type,
                argument,
            } => {
                self.scan_type(target_type);
                // Check if strconv is needed
                match target_type {
                    Type::Kain | Type::Sar | Type::DaTha => {
                        self.needs_strconv = true;
                    }
                    _ => {}
                }
                self.scan_expression(argument);
            }
            Expression::MethodCall {
                object,
                method,
                arguments,
            } => {
                self.scan_expression(object);
                match method.as_str() {
                    "khwae" | "swal" => {
                        self.needs_strings = true;
                    }
                    "json" => {
                        self.needs_gojson = true;
                    }
                    _ => {}
                }
                if matches!(
                    object.as_ref(),
                    Expression::Identifier(name) if name == "http"
                ) {
                    self.needs_net_http = true;
                }
                for arg in arguments {
                    self.scan_expression(arg);
                }
            }
            Expression::ErrorCreate { message } => {
                self.needs_errors = true;
                self.scan_expression(message);
            }
            Expression::Binary { left, right, .. } => {
                self.scan_expression(left);
                self.scan_expression(right);
            }
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.scan_expression(arg);
                }
            }
            Expression::ArrayLiteral { elements } => {
                for el in elements {
                    self.scan_expression(el);
                }
            }
            Expression::HashLiteral { pairs } => {
                for (k, v) in pairs {
                    self.scan_expression(k);
                    self.scan_expression(v);
                }
            }
            Expression::IndexExpression { left, index } => {
                self.scan_expression(left);
                self.scan_expression(index);
            }
            Expression::SliceExpression { left, low, high } => {
                self.scan_expression(left);
                if let Some(l) = low {
                    self.scan_expression(l);
                }
                if let Some(h) = high {
                    self.scan_expression(h);
                }
            }
            Expression::FieldAccess { object, .. } => {
                self.scan_expression(object);
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, expr) in fields {
                    self.scan_expression(expr);
                }
            }
            Expression::ClosureLiteral {
                parameters,
                return_type,
                body,
            } => {
                for (_, ty, _) in parameters {
                    self.scan_type(ty);
                }
                self.scan_type(return_type);
                self.scan_block(body);
            }
            Expression::TupleLiteral { elements } => {
                for el in elements {
                    self.scan_expression(el);
                }
            }
            Expression::ChannelMake {
                value_type,
                capacity,
            } => {
                self.scan_type(value_type);
                if let Some(capacity) = capacity {
                    self.scan_expression(capacity);
                }
            }
            Expression::BaungCreate { timeout_ms } => {
                self.needs_baung_helper = true;
                self.needs_context = true;
                self.needs_time = true;
                self.scan_expression(timeout_ms);
            }
            Expression::IntegerLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NilLiteral
            | Expression::Identifier(_) => {}
        }
    }

    fn generate_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::PackageDecl { .. } => {}
            Statement::Let {
                name, value, ty, ..
            } => {
                self.environment.insert(name.clone(), ty.clone());
                self.indent();

                // Track array lengths
                if let Type::Array(_) = ty {
                    if let Some(length) = self.infer_array_length(value) {
                        self.array_lengths.insert(name.clone(), length);
                    }
                }

                let go_name = self.clean_identifier(name);
                // If value is bhala (nil) and type is not pointer-like, use Go zero value
                if matches!(value, Expression::NilLiteral) {
                    self.output.push_str(&format!("var {} ", go_name));
                    self.generate_type(ty);
                    self.output.push_str("\n");
                } else {
                    self.output.push_str(&format!("var {} ", go_name));
                    self.generate_type(ty);
                    self.output.push_str(" = ");
                    self.generate_expression(value);
                    self.output.push_str("\n");
                }

                self.indent();
                self.output.push_str(&format!("_ = {}\n", go_name));
            }
            Statement::LetDestructured { names, value } => {
                self.indent();
                let name_strs: Vec<String> = names
                    .iter()
                    .map(|(n, ty, _)| {
                        self.environment.insert(n.clone(), ty.clone());
                        self.clean_identifier(n)
                    })
                    .collect();
                self.output.push_str(&name_strs.join(", "));
                self.output.push_str(" := ");
                self.generate_expression(value);
                self.output.push_str("\n");
                // Suppress unused variable warnings
                for n in &name_strs {
                    self.indent();
                    self.output.push_str(&format!("_ = {}\n", n));
                }
            }
            Statement::Assign { name, value, .. } => {
                self.indent();
                self.output
                    .push_str(&format!("{} = ", self.clean_identifier(name)));
                self.generate_expression(value);
                self.output.push_str("\n");

                if let Some(Type::Array(_)) = self.environment.get(name) {
                    if let Some(length) = self.infer_array_length(value) {
                        self.array_lengths.insert(name.clone(), length);
                    }
                }
            }
            Statement::FieldAssign {
                object,
                field,
                value,
                ..
            } => {
                self.indent();
                self.output.push_str(&self.clean_identifier(object));
                self.output.push_str(&format!(
                    ".{} = ",
                    capitalize_first(&self.clean_identifier(field))
                ));
                self.generate_expression(value);
                self.output.push_str("\n");
            }
            Statement::IndexAssign {
                object,
                index,
                value,
                ..
            } => {
                self.indent();
                self.generate_expression(object);
                self.output.push_str("[");
                self.generate_expression(index);
                self.output.push_str("] = ");
                self.generate_expression(value);
                self.output.push_str("\n");
            }
            Statement::FunctionDecl {
                name,
                parameters,
                return_type,
                body,
                ..
            } => {
                let saved_env = self.environment.clone();
                let saved_array_lengths = self.array_lengths.clone();
                self.indent();
                let is_main = name == "main";
                let go_name = if is_main {
                    "main".to_string()
                } else {
                    self.clean_identifier(name)
                };

                self.output.push_str(&format!("func {}(", go_name));

                for (i, (p_name, p_type, _)) in parameters.iter().enumerate() {
                    self.environment.insert(p_name.clone(), p_type.clone());
                    self.output
                        .push_str(&format!("{} ", self.clean_identifier(p_name)));
                    self.generate_type(p_type);
                    if i < parameters.len() - 1 {
                        self.output.push_str(", ");
                    }
                }

                self.output.push_str(")");

                if !is_main {
                    self.output.push_str(" ");
                    self.generate_return_type(return_type);
                }

                self.output.push_str(" ");
                let prev_in_main = self.in_main;
                self.in_main = is_main;
                self.generate_block(body);
                self.in_main = prev_in_main;
                self.output.push_str("\n\n");
                self.environment = saved_env;
                self.array_lengths = saved_array_lengths;
            }
            Statement::Return { value } => {
                if self.in_main {
                    return;
                }
                self.indent();
                self.output.push_str("return ");
                // For tuple returns, emit comma-separated values
                if let Expression::TupleLiteral { elements } = value {
                    for (i, elem) in elements.iter().enumerate() {
                        self.generate_expression(elem);
                        if i < elements.len() - 1 {
                            self.output.push_str(", ");
                        }
                    }
                } else {
                    self.generate_expression(value);
                }
                self.output.push_str("\n");
            }
            Statement::Print { value } => {
                self.indent();
                self.output.push_str("fmt.Println(");
                self.generate_expression(value);
                self.output.push_str(")\n");
            }
            Statement::If {
                condition,
                consequence,
                alternative,
            } => {
                self.indent();
                self.output.push_str("if ");
                self.generate_expression(condition);
                self.output.push_str(" ");
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
                self.output.push_str("for ");
                self.generate_expression(condition);
                self.output.push_str(" ");
                self.generate_block(body);
                self.output.push_str("\n");
            }
            Statement::Break => {
                self.indent();
                self.output.push_str("break\n");
            }
            Statement::Continue => {
                self.indent();
                self.output.push_str("continue\n");
            }
            Statement::Go { call } => {
                self.indent();
                self.generate_prefixed_call("go", call);
                self.output.push_str("\n");
            }
            Statement::Defer { call } => {
                self.indent();
                self.generate_prefixed_call("defer", call);
                self.output.push_str("\n");
            }
            Statement::TestDecl { name, body, .. } => {
                let saved_env = self.environment.clone();
                let saved_array_lengths = self.array_lengths.clone();
                self.indent();
                self.output
                    .push_str(&format!("func {}() error ", self.test_function_name(name)));
                self.generate_block(body);
                self.output.push_str("\n\n");
                self.environment = saved_env;
                self.array_lengths = saved_array_lengths;
            }
            Statement::ForIn {
                index,
                iterator,
                collection,
                body,
                ..
            } => {
                let saved_env = self.environment.clone();
                let saved_array_lengths = self.array_lengths.clone();
                self.indent();

                let iter_name = self.clean_identifier(iterator);
                let index_name = index
                    .as_ref()
                    .map(|name| self.clean_identifier(name))
                    .unwrap_or_else(|| "_".to_string());

                if let Some(Type::Array(inner)) = self.infer_expression_type(collection) {
                    self.environment.insert(iterator.clone(), *inner);
                }
                if let Some(index_name) = index {
                    self.environment.insert(index_name.clone(), Type::Kain);
                }

                if let Expression::Identifier(name) = collection {
                    if let Some(len) = self.array_lengths.get(name).copied() {
                        self.array_lengths.insert(iterator.clone(), len);
                    }
                }

                self.output
                    .push_str(&format!("for {}, {} := range ", index_name, iter_name));
                self.generate_expression(collection);
                self.output.push_str(" ");
                self.generate_block(body);
                self.output.push_str("\n");
                self.environment = saved_env;
                self.array_lengths = saved_array_lengths;
            }
            Statement::ForClassic {
                init,
                condition,
                post,
                body,
            } => {
                let saved_env = self.environment.clone();
                let saved_array_lengths = self.array_lengths.clone();
                self.indent();
                self.output.push_str("for ");
                if let Some(init) = init {
                    self.generate_for_classic_component(init);
                }
                self.output.push_str("; ");
                if let Some(condition) = condition {
                    self.generate_expression(condition);
                }
                self.output.push_str("; ");
                if let Some(post) = post {
                    self.generate_for_classic_component(post);
                }
                self.output.push_str(" ");
                self.generate_block(body);
                self.output.push_str("\n");
                self.environment = saved_env;
                self.array_lengths = saved_array_lengths;
            }
            Statement::ExpressionStatement(expr) => {
                self.indent();
                self.generate_statement_expression(expr);
                self.output.push_str("\n");
            }
            Statement::Import { module, .. } => {
                self.output.push_str(&format!("// import \"{}\"\n", module));
            }
            Statement::StructDecl { name, fields, .. } => {
                // Register struct fields for type inference
                self.struct_fields.insert(name.clone(), fields.clone());
                self.indent();
                let go_name = self.clean_identifier(name);
                self.output
                    .push_str(&format!("type {} struct {{\n", go_name));
                self.indent_level += 1;
                for (fname, ftype) in fields {
                    self.indent();
                    // Capitalize first letter for Go export
                    let go_field = capitalize_first(&self.clean_identifier(fname));
                    self.output.push_str(&format!("{} ", go_field));
                    self.generate_type(ftype);
                    self.output.push_str("\n");
                }
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}\n\n");
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
                let saved_env = self.environment.clone();
                let saved_array_lengths = self.array_lengths.clone();
                self.indent();
                let go_recv_type = self.clean_identifier(receiver_type);
                let go_recv_name = self.clean_identifier(receiver_name);
                let go_name = self.clean_identifier(name);

                self.environment
                    .insert(receiver_name.clone(), Type::Struct(receiver_type.clone()));

                let go_name = capitalize_first(&go_name);
                self.output.push_str(&format!(
                    "func ({} {}) {}(",
                    go_recv_name, go_recv_type, go_name
                ));
                for (i, (p_name, p_type, _)) in parameters.iter().enumerate() {
                    self.environment.insert(p_name.clone(), p_type.clone());
                    self.output
                        .push_str(&format!("{} ", self.clean_identifier(p_name)));
                    self.generate_type(p_type);
                    if i < parameters.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(") ");
                self.generate_return_type(return_type);
                self.output.push_str(" ");
                self.generate_block(body);
                self.output.push_str("\n\n");
                self.environment = saved_env;
                self.array_lengths = saved_array_lengths;
            }
            Statement::InterfaceDecl { name, methods, .. } => {
                self.indent();
                let go_name = self.clean_identifier(name);
                self.output
                    .push_str(&format!("type {} interface {{\n", go_name));
                self.indent_level += 1;
                for (mname, params, ret_type) in methods {
                    self.indent();
                    let go_mname = capitalize_first(&self.clean_identifier(mname));
                    self.output.push_str(&format!("{}(", go_mname));
                    for (i, (pname, ptype)) in params.iter().enumerate() {
                        self.output
                            .push_str(&format!("{} ", self.clean_identifier(pname)));
                        self.generate_type(ptype);
                        if i < params.len() - 1 {
                            self.output.push_str(", ");
                        }
                    }
                    self.output.push_str(") ");
                    self.generate_return_type(ret_type);
                    self.output.push_str("\n");
                }
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}\n\n");
            }
            Statement::Export { statement, .. } => self.generate_statement(statement),
        }
    }

    fn generate_elif_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::If {
                condition,
                consequence,
                alternative,
            } => {
                self.output.push_str("if ");
                self.generate_expression(condition);
                self.output.push_str(" ");
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

    fn generate_return_type(&mut self, return_type: &Type) {
        if let Type::Tuple(types) = return_type {
            self.output.push_str("(");
            for (i, t) in types.iter().enumerate() {
                self.generate_type(t);
                if i < types.len() - 1 {
                    self.output.push_str(", ");
                }
            }
            self.output.push_str(")");
        } else {
            self.generate_type(return_type);
        }
    }

    fn generate_channel_send(&mut self, object: &Expression, arguments: &[Expression]) {
        self.generate_expression(object);
        self.output.push_str(" <- ");
        if let Some(argument) = arguments.first() {
            self.generate_expression(argument);
        } else {
            self.output.push_str("nil");
        }
    }

    fn generate_statement_expression(&mut self, expr: &Expression) {
        if let Expression::MethodCall {
            object,
            method,
            arguments,
        } = expr
        {
            let obj_type = self.infer_expression_type(object);
            if matches!(obj_type.as_ref(), Some(Type::Channel(_))) {
                match method.as_str() {
                    "send" => {
                        self.generate_channel_send(object, arguments);
                        return;
                    }
                    "close" => {
                        self.output.push_str("close(");
                        self.generate_expression(object);
                        self.output.push_str(")");
                        return;
                    }
                    _ => {}
                }
            }
        }

        self.generate_expression(expr);
    }

    fn generate_prefixed_call(&mut self, prefix: &str, expr: &Expression) {
        self.output.push_str(prefix);
        self.output.push_str(" ");

        if let Expression::MethodCall {
            object,
            method,
            arguments,
        } = expr
        {
            let obj_type = self.infer_expression_type(object);
            if matches!(obj_type.as_ref(), Some(Type::Channel(_))) {
                match method.as_str() {
                    "send" => {
                        self.output.push_str("func() { ");
                        self.generate_channel_send(object, arguments);
                        self.output.push_str(" }()");
                        return;
                    }
                    "close" => {
                        self.output.push_str("close(");
                        self.generate_expression(object);
                        self.output.push_str(")");
                        return;
                    }
                    _ => {}
                }
            }
        }

        self.generate_expression(expr);
    }

    fn generate_for_classic_component(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let {
                name, value, ty, ..
            } => {
                self.environment.insert(name.clone(), ty.clone());
                if let Type::Array(_) = ty {
                    if let Some(length) = self.infer_array_length(value) {
                        self.array_lengths.insert(name.clone(), length);
                    }
                }
                self.output
                    .push_str(&format!("{} := ", self.clean_identifier(name)));
                self.generate_expression(value);
            }
            Statement::LetDestructured { names, value } => {
                let name_strs: Vec<String> = names
                    .iter()
                    .map(|(name, ty, _)| {
                        self.environment.insert(name.clone(), ty.clone());
                        self.clean_identifier(name)
                    })
                    .collect();
                self.output.push_str(&name_strs.join(", "));
                self.output.push_str(" := ");
                self.generate_expression(value);
            }
            Statement::Assign { name, value, .. } => {
                self.output
                    .push_str(&format!("{} = ", self.clean_identifier(name)));
                self.generate_expression(value);
            }
            Statement::FieldAssign {
                object,
                field,
                value,
                ..
            } => {
                self.output.push_str(&self.clean_identifier(object));
                self.output.push_str(&format!(
                    ".{} = ",
                    capitalize_first(&self.clean_identifier(field))
                ));
                self.generate_expression(value);
            }
            Statement::IndexAssign {
                object,
                index,
                value,
                ..
            } => {
                self.generate_expression(object);
                self.output.push_str("[");
                self.generate_expression(index);
                self.output.push_str("] = ");
                self.generate_expression(value);
            }
            Statement::ExpressionStatement(expr) => self.generate_statement_expression(expr),
            _ => {}
        }
    }

    fn generate_http_module_call(&mut self, method: &str, arguments: &[Expression]) -> bool {
        match method {
            "handle" if arguments.len() >= 2 => {
                self.output.push_str("func() error { http.HandleFunc(");
                self.generate_expression(&arguments[0]);
                self.output
                    .push_str(", func(w http.ResponseWriter, r *http.Request) { if err := ");
                self.generate_expression(&arguments[1]);
                self.output.push_str(
                    "(r, w); err != nil { http.Error(w, err.Error(), http.StatusInternalServerError) } }); return nil }()",
                );
                true
            }
            "listen" if !arguments.is_empty() => {
                self.output.push_str("http.ListenAndServe(");
                self.generate_expression(&arguments[0]);
                self.output.push_str(", nil)");
                true
            }
            _ => false,
        }
    }

    fn generate_http_request_method_call(
        &mut self,
        object: &Expression,
        method: &str,
        arguments: &[Expression],
    ) -> bool {
        match method {
            "header" if !arguments.is_empty() => {
                self.generate_expression(object);
                self.output.push_str(".Header.Get(");
                self.generate_expression(&arguments[0]);
                self.output.push_str(")");
                true
            }
            "query" if !arguments.is_empty() => {
                self.generate_expression(object);
                self.output.push_str(".URL.Query().Get(");
                self.generate_expression(&arguments[0]);
                self.output.push_str(")");
                true
            }
            _ => false,
        }
    }

    fn generate_http_writer_method_call(
        &mut self,
        object: &Expression,
        method: &str,
        arguments: &[Expression],
    ) -> bool {
        match method {
            "write" if !arguments.is_empty() => {
                self.output.push_str("func() error { _, err := ");
                self.generate_expression(object);
                self.output.push_str(".Write([]byte(");
                self.generate_expression(&arguments[0]);
                self.output.push_str(")); return err }()");
                true
            }
            "status" if !arguments.is_empty() => {
                self.output.push_str("func() error { ");
                self.generate_expression(object);
                self.output.push_str(".WriteHeader(int(");
                self.generate_expression(&arguments[0]);
                self.output.push_str(")); return nil }()");
                true
            }
            "header" if arguments.len() >= 2 => {
                self.output.push_str("func() error { ");
                self.generate_expression(object);
                self.output.push_str(".Header().Set(");
                self.generate_expression(&arguments[0]);
                self.output.push_str(", ");
                self.generate_expression(&arguments[1]);
                self.output.push_str("); return nil }()");
                true
            }
            "json" if !arguments.is_empty() => {
                self.output.push_str("func() error { ");
                self.generate_expression(object);
                self.output
                    .push_str(".Header().Set(\"Content-Type\", \"application/json\"); return gojson.NewEncoder(");
                self.generate_expression(object);
                self.output.push_str(").Encode(");
                self.generate_expression(&arguments[0]);
                self.output.push_str(") }()");
                true
            }
            _ => false,
        }
    }

    fn generate_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::IntegerLiteral(val) => {
                self.output.push_str(&val.to_string());
            }
            Expression::FloatLiteral(val) => {
                // Format float ensuring decimal point
                if val.fract() == 0.0 {
                    self.output.push_str(&format!("{:.1}", val));
                } else {
                    self.output.push_str(&format!("{}", val));
                }
            }
            Expression::StringLiteral(val) => {
                self.output.push_str(&format!("\"{}\"", val));
            }
            Expression::BooleanLiteral(val) => {
                self.output.push_str(if *val { "true" } else { "false" });
            }
            Expression::NilLiteral => {
                self.output.push_str("nil");
            }
            Expression::Identifier(name) => {
                self.output.push_str(&self.clean_identifier(name));
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                self.output.push_str("(");
                self.generate_expression(left);
                self.output.push_str(&format!(" {} ", operator));
                self.generate_expression(right);
                self.output.push_str(")");
            }
            Expression::FunctionCall {
                function,
                arguments,
            } => {
                // Built-in function mappings
                match function.as_str() {
                    "htae" => {
                        // htae(arr, elem) -> append(arr, elem)
                        self.output.push_str("append(");
                        for (i, arg) in arguments.iter().enumerate() {
                            self.generate_expression(arg);
                            if i < arguments.len() - 1 {
                                self.output.push_str(", ");
                            }
                        }
                        self.output.push_str(")");
                    }
                    "ashay" => {
                        // ashay(expr) -> int64(len(expr))
                        self.output.push_str("int64(len(");
                        for (i, arg) in arguments.iter().enumerate() {
                            self.generate_expression(arg);
                            if i < arguments.len() - 1 {
                                self.output.push_str(", ");
                            }
                        }
                        self.output.push_str("))");
                    }
                    _ => {
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
                }
            }
            Expression::ArrayLiteral { elements } => {
                let go_type = if elements.is_empty() {
                    "int64".to_string()
                } else {
                    self.infer_go_type(&elements[0])
                };

                self.output.push_str(&format!("[]{}{{", go_type));
                for (i, elem) in elements.iter().enumerate() {
                    self.generate_expression(elem);
                    if i < elements.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str("}");
            }
            Expression::HashLiteral { pairs } => {
                if pairs.is_empty() {
                    self.output.push_str("map[string]int64{}");
                } else {
                    let key_type = self.infer_go_type(&pairs[0].0);
                    let val_type = self.infer_go_type(&pairs[0].1);
                    self.output
                        .push_str(&format!("map[{}{}]{}{}{{", "", key_type, "", val_type));
                    for (i, (key, val)) in pairs.iter().enumerate() {
                        self.generate_expression(key);
                        self.output.push_str(": ");
                        self.generate_expression(val);
                        if i < pairs.len() - 1 {
                            self.output.push_str(", ");
                        }
                    }
                    self.output.push_str("}");
                }
            }
            Expression::IndexExpression { left, index } => {
                self.generate_expression(left);
                self.output.push_str("[");
                self.generate_expression(index);
                self.output.push_str("]");
            }
            Expression::SliceExpression { left, low, high } => {
                self.generate_expression(left);
                self.output.push_str("[");
                if let Some(l) = low {
                    self.generate_expression(l);
                }
                self.output.push_str(":");
                if let Some(h) = high {
                    self.generate_expression(h);
                }
                self.output.push_str("]");
            }
            Expression::ReadInput { prompt } => {
                self.output.push_str("mlangReadInput(");
                self.generate_expression(prompt);
                self.output.push_str(")");
            }
            Expression::TypeConversion {
                target_type,
                argument,
            } => {
                // Determine source type from argument
                let src_type = self.infer_expression_type(argument);
                match (src_type.as_ref(), target_type) {
                    (Some(Type::Sar), Type::Kain) => {
                        // strconv.Atoi returns (int, error), wrap in func
                        self.output.push_str("func() int64 { v, _ := strconv.Atoi(");
                        self.generate_expression(argument);
                        self.output.push_str("); return int64(v) }()");
                    }
                    (Some(Type::Kain), Type::Sar) => {
                        self.output.push_str("strconv.FormatInt(");
                        self.generate_expression(argument);
                        self.output.push_str(", 10)");
                    }
                    (Some(Type::Kain), Type::DaTha) => {
                        self.output.push_str("float64(");
                        self.generate_expression(argument);
                        self.output.push_str(")");
                    }
                    (Some(Type::DaTha), Type::Kain) => {
                        self.output.push_str("int64(");
                        self.generate_expression(argument);
                        self.output.push_str(")");
                    }
                    (Some(Type::DaTha), Type::Sar) => {
                        self.output.push_str("strconv.FormatFloat(");
                        self.generate_expression(argument);
                        self.output.push_str(", 'f', -1, 64)");
                    }
                    (Some(Type::Sar), Type::DaTha) => {
                        self.output
                            .push_str("func() float64 { v, _ := strconv.ParseFloat(");
                        self.generate_expression(argument);
                        self.output.push_str(", 64); return v }()");
                    }
                    _ => {
                        // Fallback: Go type cast
                        self.generate_type(target_type);
                        self.output.push_str("(");
                        self.generate_expression(argument);
                        self.output.push_str(")");
                    }
                }
            }
            Expression::MethodCall {
                object,
                method,
                arguments,
            } => {
                if matches!(object.as_ref(), Expression::Identifier(name) if name == "http")
                    && self.generate_http_module_call(method, arguments)
                {
                    return;
                }

                let obj_type = self.infer_expression_type(object);
                match (obj_type.as_ref(), method.as_str()) {
                    (Some(Type::Channel(_)), "recv") => {
                        self.output.push_str("<-");
                        self.generate_expression(object);
                    }
                    (Some(Type::Channel(_)), "send") => {
                        self.output.push_str("func() interface{} { ");
                        self.generate_channel_send(object, arguments);
                        self.output.push_str("; return nil }()");
                    }
                    (Some(Type::Channel(_)), "close") => {
                        self.output.push_str("func() interface{} { close(");
                        self.generate_expression(object);
                        self.output.push_str("); return nil }()");
                    }
                    (Some(Type::Struct(struct_name)), _)
                        if struct_name == "http.Request"
                            && self
                                .generate_http_request_method_call(object, method, arguments) => {}
                    (Some(Type::Struct(struct_name)), _)
                        if struct_name == "http.ResponseWriter"
                            && self.generate_http_writer_method_call(object, method, arguments) => {
                    }
                    (Some(Type::Sar), "khwae") => {
                        self.output.push_str("strings.Split(");
                        self.generate_expression(object);
                        self.output.push_str(", ");
                        if !arguments.is_empty() {
                            self.generate_expression(&arguments[0]);
                        }
                        self.output.push_str(")");
                    }
                    (Some(Type::Sar), "swal") => {
                        self.output.push_str("strings.Contains(");
                        self.generate_expression(object);
                        self.output.push_str(", ");
                        if !arguments.is_empty() {
                            self.generate_expression(&arguments[0]);
                        }
                        self.output.push_str(")");
                    }
                    _ => {
                        // Regular method call: obj.method(args)
                        self.generate_expression(object);
                        self.output.push_str(&format!(
                            ".{}(",
                            capitalize_first(&self.clean_identifier(method))
                        ));
                        for (i, arg) in arguments.iter().enumerate() {
                            self.generate_expression(arg);
                            if i < arguments.len() - 1 {
                                self.output.push_str(", ");
                            }
                        }
                        self.output.push_str(")");
                    }
                }
            }
            Expression::FieldAccess { object, field } => {
                self.generate_expression(object);
                self.output.push_str(&format!(
                    ".{}",
                    capitalize_first(&self.clean_identifier(field))
                ));
            }
            Expression::StructLiteral { name, fields } => {
                let go_name = self.clean_identifier(name);
                self.output.push_str(&format!("{}{{", go_name));
                for (i, (fname, fval)) in fields.iter().enumerate() {
                    self.output.push_str(&format!(
                        "{}: ",
                        capitalize_first(&self.clean_identifier(fname))
                    ));
                    self.generate_expression(fval);
                    if i < fields.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str("}");
            }
            Expression::ClosureLiteral {
                parameters,
                return_type,
                body,
            } => {
                let saved_env = self.environment.clone();
                let saved_array_lengths = self.array_lengths.clone();
                self.output.push_str("func(");
                for (i, (p_name, p_type, _)) in parameters.iter().enumerate() {
                    self.environment.insert(p_name.clone(), p_type.clone());
                    self.output
                        .push_str(&format!("{} ", self.clean_identifier(p_name)));
                    self.generate_type(p_type);
                    if i < parameters.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(") ");
                self.generate_return_type(return_type);
                self.output.push_str(" ");
                self.generate_block(body);
                self.environment = saved_env;
                self.array_lengths = saved_array_lengths;
            }
            Expression::ErrorCreate { message } => {
                self.output.push_str("errors.New(");
                self.generate_expression(message);
                self.output.push_str(")");
            }
            Expression::TupleLiteral { elements } => {
                // This shouldn't appear in expression context outside of return
                // but handle it anyway
                for (i, elem) in elements.iter().enumerate() {
                    self.generate_expression(elem);
                    if i < elements.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
            }
            Expression::ChannelMake {
                value_type,
                capacity,
            } => {
                self.output.push_str("make(chan ");
                self.generate_type(value_type);
                if let Some(capacity) = capacity {
                    self.output.push_str(", ");
                    self.generate_expression(capacity);
                }
                self.output.push_str(")");
            }
            Expression::BaungCreate { timeout_ms } => {
                self.output.push_str("mlangNewBaung(");
                self.generate_expression(timeout_ms);
                self.output.push_str(")");
            }
        }
    }

    fn generate_type(&mut self, ty: &Type) {
        match ty {
            Type::Kain => self.output.push_str("int64"),
            Type::Sar => self.output.push_str("string"),
            Type::Sit => self.output.push_str("bool"),
            Type::DaTha => self.output.push_str("float64"),
            Type::Baung => self.output.push_str("mlangBaung"),
            Type::Nil => self.output.push_str("interface{}"),
            Type::Error => self.output.push_str("error"),
            Type::Array(inner) => {
                self.output.push_str("[]");
                self.generate_type(inner);
            }
            Type::Channel(inner) => {
                self.output.push_str("chan ");
                self.generate_type(inner);
            }
            Type::Map(key, val) => {
                self.output.push_str("map[");
                self.generate_type(key);
                self.output.push_str("]");
                self.generate_type(val);
            }
            Type::Struct(name) => {
                self.output.push_str(&self.named_go_type(name));
            }
            Type::Interface(name) => {
                self.output.push_str(&self.named_go_type(name));
            }
            Type::Tuple(types) => {
                // Tuples in Go are represented as multiple return values
                self.output.push_str("(");
                for (i, t) in types.iter().enumerate() {
                    self.generate_type(t);
                    if i < types.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(")");
            }
            Type::Function {
                params,
                return_type,
            } => {
                self.output.push_str("func(");
                for (i, param) in params.iter().enumerate() {
                    self.generate_type(param);
                    if i < params.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(") ");
                self.generate_return_type(return_type);
            }
        }
    }

    fn infer_go_type(&self, expr: &Expression) -> String {
        match expr {
            Expression::IntegerLiteral(_) => "int64".to_string(),
            Expression::FloatLiteral(_) => "float64".to_string(),
            Expression::StringLiteral(_) => "string".to_string(),
            Expression::BooleanLiteral(_) => "bool".to_string(),
            Expression::NilLiteral => "interface{}".to_string(),
            Expression::Identifier(name) => {
                if let Some(ty) = self.environment.get(name) {
                    self.type_to_go_string(ty)
                } else {
                    "int64".to_string()
                }
            }
            Expression::StructLiteral { name, .. } => self.clean_identifier(name),
            Expression::ClosureLiteral {
                parameters,
                return_type,
                ..
            } => self.type_to_go_string(&Type::Function {
                params: parameters.iter().map(|(_, ty, _)| ty.clone()).collect(),
                return_type: Box::new(return_type.clone()),
            }),
            Expression::ErrorCreate { .. } => "error".to_string(),
            Expression::ChannelMake { value_type, .. } => {
                self.type_to_go_string(&Type::Channel(value_type.clone()))
            }
            Expression::BaungCreate { .. } => "mlangBaung".to_string(),
            _ => "int64".to_string(),
        }
    }

    fn type_to_go_string(&self, ty: &Type) -> String {
        match ty {
            Type::Kain => "int64".to_string(),
            Type::Sar => "string".to_string(),
            Type::Sit => "bool".to_string(),
            Type::DaTha => "float64".to_string(),
            Type::Baung => "mlangBaung".to_string(),
            Type::Nil => "interface{}".to_string(),
            Type::Error => "error".to_string(),
            Type::Array(inner) => format!("[]{}", self.type_to_go_string(inner)),
            Type::Channel(inner) => format!("chan {}", self.type_to_go_string(inner)),
            Type::Map(k, v) => format!(
                "map[{}]{}",
                self.type_to_go_string(k),
                self.type_to_go_string(v)
            ),
            Type::Struct(name) => self.named_go_type(name),
            Type::Interface(name) => self.named_go_type(name),
            Type::Tuple(types) => {
                let ts: Vec<String> = types.iter().map(|t| self.type_to_go_string(t)).collect();
                format!("({})", ts.join(", "))
            }
            Type::Function {
                params,
                return_type,
            } => {
                let params = params
                    .iter()
                    .map(|param| self.type_to_go_string(param))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("func({}) {}", params, self.type_to_go_string(return_type))
            }
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
            Expression::FloatLiteral(_) => Some(Type::DaTha),
            Expression::StringLiteral(_) => Some(Type::Sar),
            Expression::BooleanLiteral(_) => Some(Type::Sit),
            Expression::NilLiteral => Some(Type::Nil),
            Expression::Identifier(name) => self.environment.get(name).cloned(),
            Expression::ArrayLiteral { elements } => {
                if elements.is_empty() {
                    None
                } else {
                    self.infer_expression_type(&elements[0])
                        .map(|t| Type::Array(Box::new(t)))
                }
            }
            Expression::ChannelMake { value_type, .. } => Some(Type::Channel(value_type.clone())),
            Expression::BaungCreate { .. } => Some(Type::Baung),
            Expression::ReadInput { .. } => Some(Type::Sar),
            Expression::FieldAccess { object, field } => {
                // Try to infer field type from struct
                if let Some(Type::Struct(struct_name)) = self.infer_expression_type(object) {
                    if let Some(fields) = self.struct_fields.get(&struct_name) {
                        fields
                            .iter()
                            .find(|(fname, _)| fname == field)
                            .map(|(_, ftype)| ftype.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Expression::MethodCall {
                object,
                method,
                arguments,
            } => {
                let obj_ty = self.infer_expression_type(object)?;
                match (obj_ty, method.as_str(), arguments.len()) {
                    (Type::Channel(inner), "recv", 0) => Some(*inner),
                    (Type::Channel(_), "send" | "close", _) => Some(Type::Nil),
                    (Type::Baung, "close", 0) => Some(Type::Error),
                    (Type::Sar, "khwae", 1) => Some(Type::Array(Box::new(Type::Sar))),
                    (Type::Sar, "swal", 1) => Some(Type::Sit),
                    _ => None,
                }
            }
            Expression::StructLiteral { name, .. } => Some(Type::Struct(name.clone())),
            Expression::ClosureLiteral {
                parameters,
                return_type,
                ..
            } => Some(Type::Function {
                params: parameters.iter().map(|(_, ty, _)| ty.clone()).collect(),
                return_type: Box::new(return_type.clone()),
            }),
            Expression::ErrorCreate { .. } => Some(Type::Error),
            Expression::TupleLiteral { elements } => {
                let mut types = Vec::with_capacity(elements.len());
                for element in elements {
                    types.push(self.infer_expression_type(element)?);
                }
                Some(Type::Tuple(types))
            }
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
                        } else if left_ty == Type::DaTha && right_ty == Type::DaTha {
                            Some(Type::DaTha)
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

    fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("\t");
        }
    }

    /// Clean identifier — Go supports Unicode identifiers natively!
    /// No hex-encoding needed unlike C. We just ensure it's a valid Go identifier.
    fn clean_identifier(&self, name: &str) -> String {
        // Go allows Unicode letters in identifiers (since Go 1.0).
        // Myanmar script characters are valid Go identifiers.
        // We just need to make sure it doesn't clash with Go keywords.
        let go_keywords = [
            "break",
            "case",
            "chan",
            "const",
            "continue",
            "default",
            "defer",
            "else",
            "fallthrough",
            "for",
            "func",
            "go",
            "goto",
            "if",
            "import",
            "interface",
            "map",
            "package",
            "range",
            "return",
            "select",
            "struct",
            "switch",
            "type",
            "var",
        ];

        if go_keywords.contains(&name) {
            format!("ml_{}", name)
        } else {
            name.to_string()
        }
    }

    fn named_go_type(&self, name: &str) -> String {
        match name {
            "http.Request" => "*http.Request".to_string(),
            "http.ResponseWriter" => "http.ResponseWriter".to_string(),
            _ => self.clean_identifier(name),
        }
    }

    fn test_function_name(&self, name: &str) -> String {
        go_exported_name(&format!("mlang_test_{}", self.clean_identifier(name)))
    }
}

/// Capitalize the first letter of a string (for Go exported names).
fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn go_exported_name(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(capitalize_first)
        .collect::<Vec<String>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_go_codegen_basic() {
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

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("package main"));
        assert!(go_code.contains("import ("));
        assert!(go_code.contains("\"fmt\""));
        assert!(go_code.contains("func main()"));
        assert!(go_code.contains("var age int64 = 20"));
        assert!(go_code.contains("fmt.Println(\"adult\")"));
    }

    #[test]
    fn test_go_codegen_string_concat() {
        let input = r#"
            loke main() -> kain {
                sar name = "Aung";
                sar greeting = "Hello, " + name + "!";
                pya(greeting);
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("var name string = \"Aung\""));
        // Go string concat is just +, no special function needed
        assert!(go_code.contains("+"));
        assert!(!go_code.contains("mlang_concat")); // Should NOT have C-style concat
    }

    #[test]
    fn test_go_codegen_for_in_loop() {
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

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("[]int64{1, 2, 3}"));
        assert!(go_code.contains("for _, item := range numbers"));
    }

    #[test]
    fn test_go_codegen_while_loop() {
        let input = r#"
            loke main() -> kain {
                kain i = 0;
                pat (i < 5) {
                    pya(i);
                    i = i + 1;
                }
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        // Go: `for i < 5 { ... }` — no "while" keyword
        assert!(go_code.contains("for (i < 5)"));
    }

    #[test]
    fn test_go_codegen_hashmap() {
        let input = r#"
            loke main() -> kain {
                twe<sar, kain> ages = {"alice": 30, "bob": 25};
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("map[string]int64"));
    }

    #[test]
    fn test_go_codegen_function_with_params() {
        let input = r#"
            loke add(kain a, kain b) -> kain {
                pyan a + b;
            }

            loke main() -> kain {
                kain result = add(10, 20);
                pya(result);
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("func add(a int64, b int64) int64"));
        assert!(go_code.contains("return (a + b)"));
        assert!(go_code.contains("add(10, 20)"));
    }

    #[test]
    fn test_go_codegen_unicode_identifiers() {
        // Go supports Unicode identifiers natively — no hex encoding needed
        let input = r#"
            loke main() -> kain {
                kain age = 25;
                pya(age);
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        // Should use clean names, not hex-encoded
        assert!(!go_code.contains("mlang_")); // No hex encoding!
        assert!(go_code.contains("age"));
    }

    #[test]
    fn test_go_codegen_read_input() {
        let input = r#"
            loke main() -> kain {
                sar name = phat("Enter name: ");
                pya(name);
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("mlangReadInput(\"Enter name: \")"));
        assert!(go_code.contains("\"bufio\""));
        assert!(go_code.contains("\"os\""));
        assert!(go_code.contains("func mlangReadInput(prompt string) string"));
    }

    #[test]
    fn test_go_codegen_if_elif_else() {
        let input = r#"
            loke main() -> kain {
                kain age = 20;
                hlyin (age > 18) {
                    pya("adult");
                } mo hlyin (age == 18) {
                    pya("just 18");
                } mo {
                    pya("young");
                }
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("if (age > 18)"));
        assert!(go_code.contains("} else if (age == 18)"));
        assert!(go_code.contains("} else {"));
    }

    #[test]
    fn test_go_codegen_phase3_channel_ops() {
        let input = r#"
            loke producer(laung<kain> ch) -> amhar {
                naut_sone ch.close();
                ch.send(100);
                pyan bhala;
            }

            loke main() -> kain {
                laung<kain> ch = laung<kain>(2);
                kyoe producer(ch);
                kain first = ch.recv();
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("func producer(ch chan int64) error"));
        assert!(go_code.contains("defer close(ch)"));
        assert!(go_code.contains("ch <- 100"));
        assert!(go_code.contains("var ch chan int64 = make(chan int64, 2)"));
        assert!(go_code.contains("go producer(ch)"));
        assert!(go_code.contains("var first int64 = <-ch"));
    }

    #[test]
    fn test_go_codegen_indexed_and_classic_for() {
        let input = r#"
            loke main() -> kain {
                su<kain> costs = [1, 2, 3];
                pat (idx, cost) htae costs {
                    hlyin (idx == 0) {
                        shar;
                    }
                    hlyin (cost > 2) {
                        yut;
                    }
                    pya(cost);
                }
                pat (kain i = 0; i < 3; i = i + 1) {
                    pya(i);
                }
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("for idx, cost := range costs"));
        assert!(go_code.contains("continue"));
        assert!(go_code.contains("break"));
        assert!(go_code.contains("for i := 0; (i < 3); i = (i + 1)"));
    }

    #[test]
    fn test_go_codegen_test_decl_baung_and_closure() {
        let input = r#"
            set_sae smoke_test {
                baung ctx = baung(1000);
                amhar close_err = ctx.close();
                hlyin (close_err != bhala) {
                    pyan close_err;
                }
                pyan bhala;
            }

            loke main() -> kain {
                loke(kain) -> kain inc = loke(kain v) -> kain {
                    pyan v + 1;
                };
                kain out = inc(41);
                pya(out);
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("\"context\""));
        assert!(go_code.contains("\"time\""));
        assert!(go_code.contains("type mlangBaung struct"));
        assert!(go_code.contains("func mlangNewBaung(timeoutMs int64) mlangBaung"));
        assert!(go_code.contains("func MlangTestSmokeTest() error"));
        assert!(go_code.contains("var ctx mlangBaung = mlangNewBaung(1000)"));
        assert!(go_code.contains("ctx.Close()"));
        assert!(go_code.contains("var inc"));
        assert!(go_code.contains("func(v int64) int64"));
    }

    #[test]
    fn test_go_codegen_http_server_lowering() {
        let input = r#"
            yu "kainn/http";

            loke hello(http.Request req, http.ResponseWriter w) -> amhar {
                pyan w.write("Mingalabar from M-Lang!");
            }

            loke main() -> kain {
                amhar handle_err = http.handle("/", hello);
                amhar listen_err = http.listen(":18080");
                pya(handle_err);
                pya(listen_err);
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("\"net/http\""));
        assert!(go_code.contains("func hello(req *http.Request, w http.ResponseWriter) error"));
        assert!(go_code.contains("http.HandleFunc(\"/\""));
        assert!(go_code.contains("hello(r, w)"));
        assert!(go_code.contains("w.Write([]byte(\"Mingalabar from M-Lang!\"))"));
        assert!(go_code.contains("http.ListenAndServe(\":18080\", nil)"));
    }
}
