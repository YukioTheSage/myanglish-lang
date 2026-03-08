use crate::ast::{BlockStatement, Expression, IfAlternative, Program, Statement, Type};
use crate::stdlib::{resolve_stdlib_module, StdlibModule};
use std::collections::{BTreeSet, HashMap};

pub struct GoCodeGenerator {
    pub output: String,
    indent_level: usize,
    environment: HashMap<String, Type>,
    array_lengths: HashMap<String, usize>,
    loop_counter: usize,
    /// Track which helper functions are needed
    needs_read_input: bool,
    /// Track needed imports
    needs_strconv: bool,
    needs_strings: bool,
    needs_errors: bool,
    needs_fmt: bool,
    /// Track if we're currently inside main() (Go main has no return value)
    in_main: bool,
    /// Registry of struct name -> fields (name, type) for type inference
    struct_fields: HashMap<String, Vec<(String, Type)>>,
    /// Imported hard-coded stdlib modules by alias
    stdlib_modules: HashMap<String, StdlibModule>,
}

impl GoCodeGenerator {
    pub fn new() -> Self {
        GoCodeGenerator {
            output: String::new(),
            indent_level: 0,
            environment: HashMap::new(),
            array_lengths: HashMap::new(),
            loop_counter: 0,
            needs_read_input: false,
            needs_strconv: false,
            needs_strings: false,
            needs_errors: false,
            needs_fmt: true,
            in_main: false,
            struct_fields: HashMap::new(),
            stdlib_modules: HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        // First pass: scan for features that need helpers
        self.scan_program(program);

        // Package declaration
        self.output.push_str("package main\n\n");

        // Imports
        let mut imports: BTreeSet<String> = BTreeSet::new();
        if self.needs_fmt {
            imports.insert("\"fmt\"".to_string());
        }

        // Check if we need bufio/os for read input
        if self.needs_read_input {
            imports.insert("\"bufio\"".to_string());
            imports.insert("\"os\"".to_string());
        }
        if self.needs_strconv {
            imports.insert("\"strconv\"".to_string());
        }
        if self.needs_strings {
            imports.insert("\"strings\"".to_string());
        }
        if self.needs_errors {
            imports.insert("\"errors\"".to_string());
        }

        for module in self.stdlib_modules.values() {
            for go_import in &module.go_imports {
                imports.insert(format!("\"{}\"", go_import));
            }
        }

        if !imports.is_empty() {
            self.output.push_str("import (\n");
            for imp in &imports {
                self.output.push_str(&format!("\t{}\n", imp));
            }
            self.output.push_str(")\n\n");
        }

        // Emit read input helper if needed
        if self.needs_read_input {
            self.output.push_str("func mlangReadInput(prompt string) string {\n");
            self.output.push_str("\tif prompt != \"\" {\n");
            self.output.push_str("\t\tfmt.Print(prompt)\n");
            self.output.push_str("\t}\n");
            self.output.push_str("\tscanner := bufio.NewScanner(os.Stdin)\n");
            self.output.push_str("\tscanner.Scan()\n");
            self.output.push_str("\treturn scanner.Text()\n");
            self.output.push_str("}\n\n");
        }

        self.emit_stdlib_helpers();

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

    fn scan_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let { value, .. } => self.scan_expression(value),
            Statement::LetDestructured { value, .. } => self.scan_expression(value),
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
            Statement::If { condition, consequence, alternative } => {
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
            Statement::ForIn { collection, body, .. } => {
                self.scan_expression(collection);
                self.scan_block(body);
            }
            Statement::FunctionDecl { body, .. } => self.scan_block(body),
            Statement::MethodDecl { body, .. } => self.scan_block(body),
            Statement::ExpressionStatement(expr) => self.scan_expression(expr),
            Statement::Import { module, .. } => {
                if let Some(stdlib_module) = resolve_stdlib_module(module) {
                    for module_struct in &stdlib_module.structs {
                        self.struct_fields
                            .insert(module_struct.name.to_string(), module_struct.fields.clone());
                    }
                    self.stdlib_modules
                        .insert(stdlib_module.alias.to_string(), stdlib_module);
                }
            }
            Statement::StructDecl { .. } => {}
            Statement::InterfaceDecl { .. } => {}
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
            Expression::TypeConversion { target_type, argument } => {
                // Check if strconv is needed
                match target_type {
                    Type::Kain | Type::Sar | Type::DaTha => {
                        self.needs_strconv = true;
                    }
                    _ => {}
                }
                self.scan_expression(argument);
            }
            Expression::MethodCall { object, method, arguments } => {
                self.scan_expression(object);
                match method.as_str() {
                    "khwae" | "swal" => {
                        self.needs_strings = true;
                    }
                    _ => {}
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
                if let Some(l) = low { self.scan_expression(l); }
                if let Some(h) = high { self.scan_expression(h); }
            }
            Expression::FieldAccess { object, .. } => {
                self.scan_expression(object);
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, expr) in fields {
                    self.scan_expression(expr);
                }
            }
            Expression::ClosureLiteral { body, .. } => {
                self.scan_block(body);
            }
            Expression::TupleLiteral { elements } => {
                for el in elements {
                    self.scan_expression(el);
                }
            }
            _ => {}
        }
    }

    fn emit_stdlib_helpers(&mut self) {
        let mut aliases: Vec<String> = self.stdlib_modules.keys().cloned().collect();
        aliases.sort();
        for alias in aliases {
            match alias.as_str() {
                "http" => self.emit_http_helpers(),
                "json" => self.emit_json_helpers(),
                "file" => self.emit_file_helpers(),
                "su_nit" => self.emit_su_nit_helpers(),
                _ => {}
            }
        }
    }

    fn emit_http_helpers(&mut self) {
        self.output.push_str("type MlangHTTPResponse struct {\n");
        self.output.push_str("\tStatus int64\n");
        self.output.push_str("\tBody string\n");
        self.output.push_str("\tHeaders map[string]string\n");
        self.output.push_str("}\n\n");

        self.output.push_str("func mlangHTTPGet(url string) (MlangHTTPResponse, error) {\n");
        self.output.push_str("\tresp, err := http.Get(url)\n");
        self.output.push_str("\tif err != nil {\n");
        self.output.push_str("\t\treturn MlangHTTPResponse{}, err\n");
        self.output.push_str("\t}\n");
        self.output.push_str("\tdefer resp.Body.Close()\n");
        self.output.push_str("\tbody, err := io.ReadAll(resp.Body)\n");
        self.output.push_str("\tif err != nil {\n");
        self.output.push_str("\t\treturn MlangHTTPResponse{}, err\n");
        self.output.push_str("\t}\n");
        self.output.push_str("\theaders := map[string]string{}\n");
        self.output.push_str("\tfor k, v := range resp.Header {\n");
        self.output.push_str("\t\tif len(v) > 0 {\n");
        self.output.push_str("\t\t\theaders[k] = v[0]\n");
        self.output.push_str("\t\t}\n");
        self.output.push_str("\t}\n");
        self.output.push_str(
            "\treturn MlangHTTPResponse{Status: int64(resp.StatusCode), Body: string(body), Headers: headers}, nil\n",
        );
        self.output.push_str("}\n\n");

        self.output.push_str("func mlangHTTPPost(url string, body string) (MlangHTTPResponse, error) {\n");
        self.output
            .push_str("\tresp, err := http.Post(url, \"application/json\", strings.NewReader(body))\n");
        self.output.push_str("\tif err != nil {\n");
        self.output.push_str("\t\treturn MlangHTTPResponse{}, err\n");
        self.output.push_str("\t}\n");
        self.output.push_str("\tdefer resp.Body.Close()\n");
        self.output.push_str("\trespBody, err := io.ReadAll(resp.Body)\n");
        self.output.push_str("\tif err != nil {\n");
        self.output.push_str("\t\treturn MlangHTTPResponse{}, err\n");
        self.output.push_str("\t}\n");
        self.output.push_str("\theaders := map[string]string{}\n");
        self.output.push_str("\tfor k, v := range resp.Header {\n");
        self.output.push_str("\t\tif len(v) > 0 {\n");
        self.output.push_str("\t\t\theaders[k] = v[0]\n");
        self.output.push_str("\t\t}\n");
        self.output.push_str("\t}\n");
        self.output.push_str(
            "\treturn MlangHTTPResponse{Status: int64(resp.StatusCode), Body: string(respBody), Headers: headers}, nil\n",
        );
        self.output.push_str("}\n\n");
    }

    fn emit_json_helpers(&mut self) {
        self.output.push_str("func mlangJSONEncode(data map[string]int64) (string, error) {\n");
        self.output.push_str("\tencoded, err := json.Marshal(data)\n");
        self.output.push_str("\tif err != nil {\n");
        self.output.push_str("\t\treturn \"\", err\n");
        self.output.push_str("\t}\n");
        self.output.push_str("\treturn string(encoded), nil\n");
        self.output.push_str("}\n\n");

        self.output.push_str("func mlangJSONDecodeMapInt(payload string) (map[string]int64, error) {\n");
        self.output.push_str("\traw := map[string]interface{}{}\n");
        self.output.push_str("\tif err := json.Unmarshal([]byte(payload), &raw); err != nil {\n");
        self.output.push_str("\t\treturn nil, err\n");
        self.output.push_str("\t}\n");
        self.output.push_str("\tout := map[string]int64{}\n");
        self.output.push_str("\tfor key, value := range raw {\n");
        self.output.push_str("\t\tnum, ok := value.(float64)\n");
        self.output.push_str("\t\tif !ok {\n");
        self.output.push_str("\t\t\treturn nil, fmt.Errorf(\"json.decode expected number for key %s\", key)\n");
        self.output.push_str("\t\t}\n");
        self.output.push_str("\t\tif num != float64(int64(num)) {\n");
        self.output.push_str("\t\t\treturn nil, fmt.Errorf(\"json.decode expected integer-compatible number for key %s\", key)\n");
        self.output.push_str("\t\t}\n");
        self.output.push_str("\t\tout[key] = int64(num)\n");
        self.output.push_str("\t}\n");
        self.output.push_str("\treturn out, nil\n");
        self.output.push_str("}\n\n");
    }

    fn emit_file_helpers(&mut self) {
        self.output.push_str("func mlangFileRead(path string) (string, error) {\n");
        self.output.push_str("\tdata, err := os.ReadFile(path)\n");
        self.output.push_str("\tif err != nil {\n");
        self.output.push_str("\t\treturn \"\", err\n");
        self.output.push_str("\t}\n");
        self.output.push_str("\treturn string(data), nil\n");
        self.output.push_str("}\n\n");

        self.output.push_str("func mlangFileWrite(path string, content string) error {\n");
        self.output.push_str("\treturn os.WriteFile(path, []byte(content), 0o644)\n");
        self.output.push_str("}\n\n");
    }

    fn emit_su_nit_helpers(&mut self) {
        self.output.push_str("func mlangArgs() []string {\n");
        self.output.push_str("\tif len(os.Args) <= 1 {\n");
        self.output.push_str("\t\treturn []string{}\n");
        self.output.push_str("\t}\n");
        self.output.push_str("\treturn os.Args[1:]\n");
        self.output.push_str("}\n\n");
    }

    fn try_generate_stdlib_call(
        &mut self,
        object: &Expression,
        method: &str,
        arguments: &[Expression],
    ) -> bool {
        let alias = match object {
            Expression::Identifier(name) => name.as_str(),
            _ => return false,
        };

        if !self.stdlib_modules.contains_key(alias) {
            return false;
        }

        match (alias, method) {
            ("http", "get") => {
                self.output.push_str("mlangHTTPGet(");
                if let Some(url) = arguments.first() {
                    self.generate_expression(url);
                }
                self.output.push_str(")");
                true
            }
            ("http", "post") => {
                self.output.push_str("mlangHTTPPost(");
                if let Some(url) = arguments.first() {
                    self.generate_expression(url);
                }
                self.output.push_str(", ");
                if let Some(body) = arguments.get(1) {
                    self.generate_expression(body);
                } else {
                    self.output.push_str("\"\"");
                }
                self.output.push_str(")");
                true
            }
            ("json", "encode") => {
                self.output.push_str("mlangJSONEncode(");
                if let Some(value) = arguments.first() {
                    self.generate_expression(value);
                }
                self.output.push_str(")");
                true
            }
            ("json", "decode") => {
                self.output.push_str("mlangJSONDecodeMapInt(");
                if let Some(value) = arguments.first() {
                    self.generate_expression(value);
                }
                self.output.push_str(")");
                true
            }
            ("file", "read") => {
                self.output.push_str("mlangFileRead(");
                if let Some(path) = arguments.first() {
                    self.generate_expression(path);
                }
                self.output.push_str(")");
                true
            }
            ("file", "write") => {
                self.output.push_str("mlangFileWrite(");
                if let Some(path) = arguments.first() {
                    self.generate_expression(path);
                }
                self.output.push_str(", ");
                if let Some(content) = arguments.get(1) {
                    self.generate_expression(content);
                } else {
                    self.output.push_str("\"\"");
                }
                self.output.push_str(")");
                true
            }
            ("su_nit", "env") => {
                self.output.push_str("os.Getenv(");
                if let Some(key) = arguments.first() {
                    self.generate_expression(key);
                }
                self.output.push_str(")");
                true
            }
            ("su_nit", "args") => {
                self.output.push_str("mlangArgs()");
                true
            }
            _ => false,
        }
    }

    fn generate_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let { name, value, ty, .. } => {
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
                let name_strs: Vec<String> = names.iter().map(|(n, ty, _)| {
                    self.environment.insert(n.clone(), ty.clone());
                    self.clean_identifier(n)
                }).collect();
                self.output.push_str(&name_strs.join(", "));
                self.output.push_str(" := ");
                self.generate_expression(value);
                self.output.push_str("\n");
            }
            Statement::Assign { name, value, .. } => {
                self.indent();
                self.output.push_str(&format!("{} = ", self.clean_identifier(name)));
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
                self.output.push_str(&format!(
                    "{}.{} = ",
                    self.clean_identifier(object),
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
            Statement::FunctionDecl { name, parameters, return_type, body, .. } => {
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
                    self.output.push_str(&format!("{} ", self.clean_identifier(p_name)));
                    self.generate_type(p_type);
                    if i < parameters.len() - 1 {
                        self.output.push_str(", ");
                    }
                }

                self.output.push_str(")");

                if !is_main {
                    self.output.push_str(" ");
                    // Handle tuple return types
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

                self.output.push_str(" ");
                let prev_in_main = self.in_main;
                self.in_main = is_main;
                self.generate_block(body);
                self.in_main = prev_in_main;
                self.output.push_str("\n\n");
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
            Statement::If { condition, consequence, alternative } => {
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
            Statement::ForIn {
                index,
                iterator,
                collection,
                body,
                ..
            } => {
                self.indent();

                let iter_name = self.clean_identifier(iterator);
                let index_name = index.as_ref().map(|n| self.clean_identifier(n));

                if let Some(Type::Array(inner)) = self.infer_expression_type(collection) {
                    self.environment.insert(iterator.clone(), *inner);
                }
                if let Some(index_var) = index {
                    self.environment.insert(index_var.clone(), Type::Kain);
                }

                if let Expression::Identifier(name) = collection {
                    if let Some(len) = self.array_lengths.get(name).copied() {
                        self.array_lengths.insert(iterator.clone(), len);
                    }
                }

                if let Some(idx) = index_name {
                    self.output
                        .push_str(&format!("for {}, {} := range ", idx, iter_name));
                } else {
                    self.output.push_str(&format!("for _, {} := range ", iter_name));
                }
                self.generate_expression(collection);
                self.output.push_str(" ");
                self.generate_block(body);
                self.output.push_str("\n");
            }
            Statement::ExpressionStatement(expr) => {
                if let Expression::MethodCall {
                    object,
                    method,
                    arguments,
                } = expr
                {
                    let obj_type = self.infer_expression_type(object);
                    if let (Some(Type::Array(_)), Expression::Identifier(obj_name), "push") =
                        (obj_type, object.as_ref(), method.as_str())
                    {
                        self.indent();
                        self.output
                            .push_str(&format!("{} = append({}, ", obj_name, obj_name));
                        if let Some(arg) = arguments.first() {
                            self.generate_expression(arg);
                        }
                        self.output.push_str(")\n");
                        return;
                    }
                    if let (Some(Type::Array(_)), Expression::Identifier(obj_name), "remove") =
                        (self.infer_expression_type(object), object.as_ref(), method.as_str())
                    {
                        self.indent();
                        self.output.push_str(&format!(
                            "{} = append({}[0:",
                            obj_name, obj_name
                        ));
                        if let Some(arg) = arguments.first() {
                            self.generate_expression(arg);
                        }
                        self.output.push_str("], ");
                        self.output.push_str(obj_name);
                        self.output.push_str("[");
                        if let Some(arg) = arguments.first() {
                            self.generate_expression(arg);
                        }
                        self.output.push_str("+1:]...)\n");
                        return;
                    }
                }
                self.indent();
                self.generate_expression(expr);
                self.output.push_str("\n");
            }
            Statement::Import { .. } => {}
            Statement::StructDecl { name, fields, .. } => {
                // Register struct fields for type inference
                self.struct_fields.insert(name.clone(), fields.clone());
                self.indent();
                let go_name = self.clean_identifier(name);
                self.output.push_str(&format!("type {} struct {{\n", go_name));
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
            Statement::MethodDecl { receiver_type, receiver_name, name, parameters, return_type, body, .. } => {
                self.indent();
                let go_recv_type = self.clean_identifier(receiver_type);
                let go_recv_name = self.clean_identifier(receiver_name);
                let go_name = self.clean_identifier(name);

                self.environment.insert(receiver_name.clone(), Type::Struct(receiver_type.clone()));

                let go_name = capitalize_first(&go_name);
                self.output.push_str(&format!("func ({} {}) {}(", go_recv_name, go_recv_type, go_name));
                for (i, (p_name, p_type, _)) in parameters.iter().enumerate() {
                    self.environment.insert(p_name.clone(), p_type.clone());
                    self.output.push_str(&format!("{} ", self.clean_identifier(p_name)));
                    self.generate_type(p_type);
                    if i < parameters.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(") ");
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
                self.output.push_str(" ");
                self.generate_block(body);
                self.output.push_str("\n\n");
            }
            Statement::InterfaceDecl { name, methods, .. } => {
                self.indent();
                let go_name = self.clean_identifier(name);
                self.output.push_str(&format!("type {} interface {{\n", go_name));
                self.indent_level += 1;
                for (mname, params, ret_type) in methods {
                    self.indent();
                    let go_mname = capitalize_first(&self.clean_identifier(mname));
                    self.output.push_str(&format!("{}(", go_mname));
                    for (i, (pname, ptype)) in params.iter().enumerate() {
                        self.output.push_str(&format!("{} ", self.clean_identifier(pname)));
                        self.generate_type(ptype);
                        if i < params.len() - 1 {
                            self.output.push_str(", ");
                        }
                    }
                    self.output.push_str(") ");
                    if let Type::Tuple(types) = ret_type {
                        self.output.push_str("(");
                        for (i, t) in types.iter().enumerate() {
                            self.generate_type(t);
                            if i < types.len() - 1 {
                                self.output.push_str(", ");
                            }
                        }
                        self.output.push_str(")");
                    } else {
                        self.generate_type(ret_type);
                    }
                    self.output.push_str("\n");
                }
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}\n\n");
            }
        }
    }

    fn generate_elif_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::If { condition, consequence, alternative } => {
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
            Expression::Binary { left, operator, right } => {
                self.output.push_str("(");
                self.generate_expression(left);
                self.output.push_str(&format!(" {} ", operator));
                self.generate_expression(right);
                self.output.push_str(")");
            }
            Expression::FunctionCall { function, arguments } => {
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
                        self.output.push_str(&format!("{}(", self.clean_identifier(function)));
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
                    self.output.push_str(&format!("map[{}{}]{}{}{{", "", key_type, "", val_type));
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
            Expression::TypeConversion { target_type, argument } => {
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
                        self.output.push_str("func() float64 { v, _ := strconv.ParseFloat(");
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
            Expression::MethodCall { object, method, arguments } => {
                if self.try_generate_stdlib_call(object, method, arguments) {
                    return;
                }

                // Check for built-in string methods
                let obj_type = self.infer_expression_type(object);
                match (obj_type.as_ref(), method.as_str()) {
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
                    (Some(Type::Array(_)), "push") => {
                        self.output.push_str("append(");
                        self.generate_expression(object);
                        self.output.push_str(", ");
                        if let Some(arg) = arguments.first() {
                            self.generate_expression(arg);
                        }
                        self.output.push_str(")");
                    }
                    (Some(Type::Array(_)), "remove") => {
                        self.output.push_str("append(");
                        self.generate_expression(object);
                        self.output.push_str("[:");
                        if let Some(arg) = arguments.first() {
                            self.generate_expression(arg);
                        }
                        self.output.push_str("], ");
                        self.generate_expression(object);
                        self.output.push_str("[");
                        if let Some(arg) = arguments.first() {
                            self.generate_expression(arg);
                        }
                        self.output.push_str("+1:]...)");
                    }
                    (Some(Type::Array(_)), "len") | (Some(Type::Map(_, _)), "len") => {
                        self.output.push_str("int64(len(");
                        self.generate_expression(object);
                        self.output.push_str("))");
                    }
                    _ => {
                        // Regular method call: obj.method(args)
                        self.generate_expression(object);
                        self.output.push_str(&format!(".{}(", capitalize_first(&self.clean_identifier(method))));
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
                self.output.push_str(&format!(".{}", capitalize_first(&self.clean_identifier(field))));
            }
            Expression::StructLiteral { name, fields } => {
                let go_name = self.clean_identifier(name);
                self.output.push_str(&format!("{}{{", go_name));
                for (i, (fname, fval)) in fields.iter().enumerate() {
                    self.output.push_str(&format!("{}: ", capitalize_first(&self.clean_identifier(fname))));
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
                let prev_in_main = self.in_main;
                self.in_main = false;

                self.output.push_str("func(");
                for (i, (name, ty, _)) in parameters.iter().enumerate() {
                    self.environment.insert(name.clone(), ty.clone());
                    self.output
                        .push_str(&format!("{} ", self.clean_identifier(name)));
                    self.generate_type(ty);
                    if i < parameters.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(")");
                if *return_type != Type::Nil {
                    self.output.push_str(" ");
                    self.generate_type(return_type);
                }
                self.output.push_str(" ");
                self.generate_block(body);

                self.environment = saved_env;
                self.in_main = prev_in_main;
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
        }
    }

    fn generate_type(&mut self, ty: &Type) {
        match ty {
            Type::Kain => self.output.push_str("int64"),
            Type::Sar => self.output.push_str("string"),
            Type::Sit => self.output.push_str("bool"),
            Type::DaTha => self.output.push_str("float64"),
            Type::Nil => self.output.push_str("interface{}"),
            Type::Error => self.output.push_str("error"),
            Type::Array(inner) => {
                self.output.push_str("[]");
                self.generate_type(inner);
            }
            Type::Map(key, val) => {
                self.output.push_str("map[");
                self.generate_type(key);
                self.output.push_str("]");
                self.generate_type(val);
            }
            Type::Struct(name) => {
                self.output.push_str(&self.go_struct_name(name));
            }
            Type::Interface(name) => {
                self.output.push_str(&self.clean_identifier(name));
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
            Type::Function { params, return_type } => {
                self.output.push_str("func(");
                for (i, param_ty) in params.iter().enumerate() {
                    self.generate_type(param_ty);
                    if i < params.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(")");
                if **return_type != Type::Nil {
                    self.output.push_str(" ");
                    self.generate_type(return_type);
                }
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
            Expression::StructLiteral { name, .. } => self.go_struct_name(name),
            Expression::ErrorCreate { .. } => "error".to_string(),
            _ => "int64".to_string(),
        }
    }

    fn type_to_go_string(&self, ty: &Type) -> String {
        match ty {
            Type::Kain => "int64".to_string(),
            Type::Sar => "string".to_string(),
            Type::Sit => "bool".to_string(),
            Type::DaTha => "float64".to_string(),
            Type::Nil => "interface{}".to_string(),
            Type::Error => "error".to_string(),
            Type::Array(inner) => format!("[]{}", self.type_to_go_string(inner)),
            Type::Map(k, v) => format!("map[{}]{}", self.type_to_go_string(k), self.type_to_go_string(v)),
            Type::Struct(name) => self.go_struct_name(name),
            Type::Interface(name) => self.clean_identifier(name),
            Type::Tuple(types) => {
                let ts: Vec<String> = types.iter().map(|t| self.type_to_go_string(t)).collect();
                format!("({})", ts.join(", "))
            }
            Type::Function { params, return_type } => {
                let param_strings: Vec<String> =
                    params.iter().map(|p| self.type_to_go_string(p)).collect();
                if **return_type == Type::Nil {
                    format!("func({})", param_strings.join(", "))
                } else {
                    format!(
                        "func({}) {}",
                        param_strings.join(", "),
                        self.type_to_go_string(return_type)
                    )
                }
            }
        }
    }

    fn go_struct_name(&self, name: &str) -> String {
        match name {
            "http.Response" => "MlangHTTPResponse".to_string(),
            _ => self.clean_identifier(name),
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
                    self.infer_expression_type(&elements[0]).map(|t| Type::Array(Box::new(t)))
                }
            }
            Expression::ReadInput { .. } => Some(Type::Sar),
            Expression::MethodCall {
                object,
                method,
                arguments: _,
            } => {
                if let Expression::Identifier(alias) = object.as_ref() {
                    if let Some(module) = self.stdlib_modules.get(alias) {
                        if let Some(module_fn) = module.functions.iter().find(|f| f.name == method) {
                            return Some(module_fn.return_type.clone());
                        }
                    }
                }

                let obj_ty = self.infer_expression_type(object)?;
                match (obj_ty, method.as_str()) {
                    (Type::Sar, "khwae") => Some(Type::Array(Box::new(Type::Sar))),
                    (Type::Sar, "swal") => Some(Type::Sit),
                    (Type::Sar, "ashay") => Some(Type::Kain),
                    (Type::Array(inner), "push") => Some(Type::Array(inner)),
                    (Type::Array(inner), "remove") => Some(Type::Array(inner)),
                    (Type::Array(_), "len") => Some(Type::Kain),
                    (Type::Map(_, _), "len") => Some(Type::Kain),
                    _ => None,
                }
            }
            Expression::FieldAccess { object, field } => {
                // Try to infer field type from struct
                if let Some(Type::Struct(struct_name)) = self.infer_expression_type(object) {
                    if let Some(fields) = self.struct_fields.get(&struct_name) {
                        fields.iter()
                            .find(|(fname, _)| fname == field)
                            .map(|(_, ftype)| ftype.clone())
                    } else {
                        None
                    }
                } else {
                    None
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
            Expression::Binary { left, operator, right } => {
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
            "break", "case", "chan", "const", "continue", "default", "defer",
            "else", "fallthrough", "for", "func", "go", "goto", "if",
            "import", "interface", "map", "package", "range", "return",
            "select", "struct", "switch", "type", "var",
        ];

        if go_keywords.contains(&name) {
            format!("ml_{}", name)
        } else {
            name.to_string()
        }
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
    fn test_go_codegen_for_in_loop_with_index() {
        let input = r#"
            loke main() -> kain {
                su<kain> numbers = [၁, ၂, ၃];
                pat (kain i, kain item) htae numbers {
                    pya(i);
                    pya(item);
                }
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("for i, item := range numbers"));
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
    fn test_go_codegen_struct_field_assignment() {
        let input = r#"
            pone Person { sar name; kain age; }
            loke main() -> kain {
                Person p = Person { name: "Aung", age: 20 };
                p.name = "Ko Ko";
                p.age = 25;
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("p.Name = \"Ko Ko\""));
        assert!(go_code.contains("p.Age = 25"));
    }

    #[test]
    fn test_go_codegen_index_assignment() {
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
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("nums[0] = 10"));
        assert!(go_code.contains("prices[\"coffee\"] = 800"));
    }

    #[test]
    fn test_go_codegen_array_push_statement() {
        let input = r#"
            loke main() -> kain {
                su<kain> nums = [1, 2];
                nums.push(3);
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("nums = append(nums, 3)"));
    }

    #[test]
    fn test_go_codegen_destructured_error_vars_are_usable() {
        let input = r#"
            loke divide(kain a, kain b) -> (kain, amhar) {
                hlyin (b == 0) {
                    pyan (0, amhar("division by zero"));
                }
                pyan (a / b, bhala);
            }
            loke main() -> kain {
                kain result, amhar err = divide(10, 0);
                hlyin (err != bhala) {
                    pya("Error occurred");
                }
                pya(result);
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(!go_code.contains("_ = result"));
        assert!(!go_code.contains("_ = err"));
        assert!(go_code.contains("if (err != nil)"));
    }

    #[test]
    fn test_go_codegen_break_continue() {
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
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("continue"));
        assert!(go_code.contains("break"));
    }

    #[test]
    fn test_go_codegen_closure_callback() {
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
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("func on_message(callback func(string) int64) int64"));
        assert!(go_code.contains("on_message(func(msg string) int64"));
    }

    #[test]
    fn test_go_codegen_stdlib_http_module() {
        let input = r#"
            yu "kainn/http";

            loke main() -> kain {
                http.Response res, amhar err = http.get("https://example.com");
                hlyin (err == bhala) {
                    pya(res.status);
                }
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("\"net/http\""));
        assert!(go_code.contains("\"io\""));
        assert!(go_code.contains("\"strings\""));
        assert!(go_code.contains("type MlangHTTPResponse struct"));
        assert!(go_code.contains("mlangHTTPGet(\"https://example.com\")"));
        assert!(go_code.contains("res, err :="));
    }

    #[test]
    fn test_go_codegen_stdlib_json_file_su_nit_modules() {
        let input = r#"
            yu "json";
            yu "file";
            yu "su_nit";

            loke main() -> kain {
                twe<sar, kain> payload = {"price": 5000};
                sar encoded, amhar encode_err = json.encode(payload);
                twe<sar, kain> parsed, amhar decode_err = json.decode(encoded);
                sar content, amhar read_err = file.read("input.txt");
                amhar write_err = file.write("output.txt", content);
                sar token = su_nit.env("BOT_TOKEN");
                su<sar> args = su_nit.args();
                pya(parsed);
                pya(token);
                pya(args);
                pya(encode_err);
                pya(decode_err);
                pya(read_err);
                pya(write_err);
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        let mut codegen = GoCodeGenerator::new();
        let go_code = codegen.generate(&program);

        assert!(go_code.contains("\"encoding/json\""));
        assert!(go_code.contains("\"os\""));
        assert!(go_code.contains("mlangJSONEncode(payload)"));
        assert!(go_code.contains("mlangJSONDecodeMapInt(encoded)"));
        assert!(go_code.contains("mlangFileRead(\"input.txt\")"));
        assert!(go_code.contains("mlangFileWrite(\"output.txt\", content)"));
        assert!(go_code.contains("os.Getenv(\"BOT_TOKEN\")"));
        assert!(go_code.contains("mlangArgs()"));
    }
}
