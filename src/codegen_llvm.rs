use crate::ast::{BlockStatement, Expression, IfAlternative, Program, Statement, Type};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct ValueRef {
    ty: String,
    op: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn parse_program(input: &str) -> Program {
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );
        program
    }

    #[test]
    fn llvm_codegen_uses_registered_tuple_return_signatures() {
        let program = parse_program(
            r#"
            loke safe_div(kain a, kain b) -> (kain, amhar) {
                hlyin (b == 0) {
                    pyan (0, amhar("division by zero"));
                }
                pyan (a / b, bhala);
            }

            loke main() -> kain {
                kain result, amhar err = safe_div(10, 0);
                hlyin (err != bhala) {
                    pya(err);
                }
                pyan result;
            }
            "#,
        );

        let ir = generate_llvm_ir(&program, "tuple_sig").expect("llvm ir");
        assert!(ir.contains("define { i64, i8* } @safe_div(i64 %a, i64 %b)"));
        assert!(ir.contains("call { i64, i8* } @safe_div(i64 10, i64 0)"));
        assert!(ir.contains("extractvalue { i64, i8* }"));
        assert!(ir.contains("icmp ne i8*"));
    }

    #[test]
    fn llvm_codegen_supports_function_values_and_closure_lowering() {
        let program = parse_program(
            r#"
            loke apply_fee(loke(kain) -> kain transform, kain amount) -> kain {
                pyan transform(amount);
            }

            loke main() -> kain {
                kain fee = 250;
                kain after_fee = apply_fee(loke(kain amount) -> kain {
                    pyan amount - fee;
                }, 5000);
                pyan after_fee;
            }
            "#,
        );

        let ir = generate_llvm_ir(&program, "closure_sig").expect("llvm ir");
        assert!(ir.contains("define i64 @apply_fee(i64 (i64)* %transform, i64 %amount)"));
        assert!(ir.contains("load i64 (i64)*, i64 (i64)** %transform.addr"));
        assert!(ir.contains("@closure_env_1 = internal global %closure_env_1 zeroinitializer"));
        assert!(ir.contains("define i64 @mlang_closure_1(i64 %amount)"));
    }

    #[test]
    fn llvm_codegen_supports_array_push_and_print_helpers() {
        let program = parse_program(
            r#"
            loke main() -> kain {
                su<sar> items = ["tea", "coffee"];
                items.push("cake");
                pya(items);
                pyan 0;
            }
            "#,
        );

        let ir = generate_llvm_ir(&program, "array_push").expect("llvm ir");
        assert!(ir.contains("call i8* @mlang_realloc"));
        assert!(ir.contains("call void @mlang_print_array_i8ptr"));
        assert!(ir.contains("alloca i64, align 8"));
    }
}

#[derive(Clone)]
struct FunctionSig {
    ret_ty: String,
    param_tys: Vec<String>,
}

#[derive(Clone)]
struct ArrayMeta {
    elem_ty: String,
    len_slot: String,
    cap_slot: String,
}

#[derive(Clone)]
struct StructMeta {
    llvm_name: String,
    fields: Vec<(String, String)>,
}

#[derive(Clone)]
struct MapMeta {
    map_base: String,
    key_ty: String,
    value_ty: String,
}

#[derive(Clone)]
struct CapturedVar {
    name: String,
    llvm_ty: String,
}

#[derive(Clone)]
struct PendingClosure {
    name: String,
    parameters: Vec<(String, Type, crate::ast::Span)>,
    return_type: Type,
    body: BlockStatement,
    captures: Vec<CapturedVar>,
    env_type: Option<String>,
    env_slot: Option<String>,
}

pub fn generate_llvm_ir(program: &Program, module_name: &str) -> Result<String, String> {
    let mut gen = IrGenerator::new(module_name);
    gen.generate_program(program)?;
    Ok(gen.out)
}

struct IrGenerator {
    out: String,
    globals_insert_at: usize,
    temp_counter: u64,
    label_counter: u64,
    scopes: Vec<HashMap<String, (String, String)>>,
    fn_scopes: Vec<HashMap<String, FunctionSig>>,
    array_scopes: Vec<HashMap<String, ArrayMeta>>,
    map_scopes: Vec<HashMap<String, MapMeta>>,
    struct_registry: HashMap<String, StructMeta>,
    function_registry: HashMap<String, FunctionSig>,
    declared_map_bases: HashSet<String>,
    pending_closures: Vec<PendingClosure>,
    closure_counter: u64,
    current_fn_ret: Option<String>,
    loop_stack: Vec<(String, String)>,
    block_terminated: bool,
}

impl IrGenerator {
    fn new(module_name: &str) -> Self {
        let mut out = String::new();
        out.push_str(&format!("; ModuleID = '{}'\n", module_name));
        out.push_str("source_filename = \"mlang\"\n\n");
        out.push_str("declare i32 @printf(i8*, ...)\n\n");
        out.push_str("declare i8* @mlang_alloc(i64)\n");
        out.push_str("declare i8* @mlang_realloc(i8*, i64)\n");
        out.push_str("declare void @mlang_print_array_i64(i64*, i64)\n");
        out.push_str("declare void @mlang_print_array_i8ptr(i8**, i64)\n\n");
        out.push_str(
            "@.fmt_i64 = private unnamed_addr constant [6 x i8] c\"%lld\\0A\\00\", align 1\n",
        );
        out.push_str(
            "@.fmt_f64 = private unnamed_addr constant [4 x i8] c\"%f\\0A\\00\", align 1\n",
        );
        out.push_str(
            "@.fmt_str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\", align 1\n\n",
        );
        out.push_str(
            "@.fmt_ptr = private unnamed_addr constant [4 x i8] c\"%p\\0A\\00\", align 1\n\n",
        );

        let globals_insert_at = out.len();

        Self {
            out,
            globals_insert_at,
            temp_counter: 0,
            label_counter: 0,
            scopes: vec![HashMap::new()],
            fn_scopes: vec![HashMap::new()],
            array_scopes: vec![HashMap::new()],
            map_scopes: vec![HashMap::new()],
            struct_registry: HashMap::new(),
            function_registry: HashMap::new(),
            declared_map_bases: HashSet::new(),
            pending_closures: Vec::new(),
            closure_counter: 0,
            current_fn_ret: None,
            loop_stack: Vec::new(),
            block_terminated: false,
        }
    }

    fn emit_global(&mut self, line: &str) {
        let mut text = String::from(line);
        if !text.ends_with('\n') {
            text.push('\n');
        }
        self.out.insert_str(self.globals_insert_at, &text);
        self.globals_insert_at += text.len();
    }

    fn generate_program(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.statements {
            if let Statement::StructDecl { name, fields, .. } = stmt {
                self.register_struct(name, fields)?;
            }
        }
        for stmt in &program.statements {
            if let Statement::FunctionDecl {
                name,
                parameters,
                return_type,
                ..
            } = stmt
            {
                self.register_function_signature(name, parameters, return_type)?;
            }
        }

        for stmt in &program.statements {
            match stmt {
                Statement::FunctionDecl {
                    name,
                    parameters,
                    return_type,
                    body,
                    ..
                } => {
                    self.generate_function(name, parameters, return_type, body)?;
                    self.out.push('\n');
                }
                Statement::PackageDecl { .. }
                | Statement::Import { .. }
                | Statement::StructDecl { .. } => {}
                other => {
                    return Err(format!(
                        "LLVM backend currently expects top-level functions only. Unsupported: {:?}",
                        other
                    ));
                }
            }
        }

        let mut idx = 0;
        while idx < self.pending_closures.len() {
            let closure = self.pending_closures[idx].clone();
            idx += 1;
            self.generate_pending_closure(&closure)?;
            self.out.push('\n');
        }
        Ok(())
    }

    fn generate_function(
        &mut self,
        name: &str,
        parameters: &[(String, Type, crate::ast::Span)],
        return_type: &Type,
        body: &BlockStatement,
    ) -> Result<(), String> {
        let ret_ty = self.type_to_llvm(return_type)?;
        self.current_fn_ret = Some(ret_ty.clone());

        let mut params_sig = Vec::with_capacity(parameters.len());
        for (pname, pty, _) in parameters {
            params_sig.push(format!(
                "{} %{}",
                self.type_to_llvm(pty)?,
                self.sanitize_ident(pname)
            ));
        }

        self.out.push_str(&format!(
            "define {} @{}({}) {{\n",
            ret_ty,
            self.sanitize_ident(name),
            params_sig.join(", ")
        ));
        self.out.push_str("entry:\n");
        self.block_terminated = false;

        self.push_scope();
        for (pname, pty, _) in parameters {
            let pty_llvm = self.type_to_llvm(pty)?;
            let clean = self.sanitize_ident(pname);
            let ptr = format!("%{}.addr", clean);
            self.out
                .push_str(&format!("  {} = alloca {}, align 8\n", ptr, pty_llvm));
            self.out.push_str(&format!(
                "  store {} %{}, {}* {}, align 8\n",
                pty_llvm, clean, pty_llvm, ptr
            ));
            self.insert_var(pname, pty_llvm, ptr);
            if let Some(sig) = self.function_sig_from_type(pty)? {
                self.insert_fn_value(pname, sig);
            }
        }

        self.generate_block(body)?;

        if !self.current_block_has_terminator() {
            self.emit_default_return()?;
        }

        self.pop_scope();
        self.current_fn_ret = None;
        self.out.push_str("}\n");
        Ok(())
    }

    fn generate_block(&mut self, block: &BlockStatement) -> Result<(), String> {
        self.push_scope();
        for stmt in &block.statements {
            self.generate_stmt(stmt)?;
            if self.current_block_has_terminator() {
                break;
            }
        }
        self.pop_scope();
        Ok(())
    }

    fn generate_stmt(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Let {
                name, value, ty, ..
            } => {
                if let Type::Array(inner) = ty {
                    return self.generate_array_let(name, inner, value);
                }
                if let Type::Map(key, val) = ty {
                    return self.generate_map_let(name, key, val, value);
                }
                let ty_llvm = self.type_to_llvm(ty)?;
                let ptr = format!(
                    "%{}.addr.{}",
                    self.sanitize_ident(name),
                    self.next_temp_id()
                );
                self.out
                    .push_str(&format!("  {} = alloca {}, align 8\n", ptr, ty_llvm));
                let rhs = self.generate_expr(value)?;
                self.ensure_type_eq(&ty_llvm, &rhs.ty, "let assignment")?;
                self.out.push_str(&format!(
                    "  store {} {}, {}* {}, align 8\n",
                    rhs.ty, rhs.op, ty_llvm, ptr
                ));
                self.insert_var(name, ty_llvm, ptr);
                if let Some(sig) = self.function_sig_from_type(ty)? {
                    self.insert_fn_value(name, sig);
                }
                Ok(())
            }
            Statement::LetDestructured { names, value } => {
                let tuple = self.generate_expr(value)?;
                for (index, (name, ty, _)) in names.iter().enumerate() {
                    let elem_ty = self.type_to_llvm(ty)?;
                    let ptr = format!(
                        "%{}.addr.{}",
                        self.sanitize_ident(name),
                        self.next_temp_id()
                    );
                    self.out
                        .push_str(&format!("  {} = alloca {}, align 8\n", ptr, elem_ty));
                    let elem = self.next_temp();
                    self.out.push_str(&format!(
                        "  {} = extractvalue {} {}, {}\n",
                        elem, tuple.ty, tuple.op, index
                    ));
                    self.out.push_str(&format!(
                        "  store {} {}, {}* {}, align 8\n",
                        elem_ty, elem, elem_ty, ptr
                    ));
                    self.insert_var(name, elem_ty.clone(), ptr);
                    if let Some(sig) = self.function_sig_from_type(ty)? {
                        self.insert_fn_value(name, sig);
                    }
                }
                Ok(())
            }
            Statement::Assign { name, value, .. } => {
                let (ty, ptr) = self
                    .lookup_var(name)
                    .ok_or_else(|| format!("Undefined variable: {}", name))?;
                let rhs = self.generate_expr(value)?;
                self.ensure_type_eq(&ty, &rhs.ty, "assignment")?;
                self.out.push_str(&format!(
                    "  store {} {}, {}* {}, align 8\n",
                    rhs.ty, rhs.op, ty, ptr
                ));
                Ok(())
            }
            Statement::FieldAssign {
                object,
                field,
                value,
                ..
            } => self.generate_field_assign(object, field, value),
            Statement::Return { value } => {
                let v = self.generate_expr(value)?;
                let expected = self
                    .current_fn_ret
                    .clone()
                    .unwrap_or_else(|| "void".to_string());
                if expected == "void" {
                    self.out.push_str("  ret void\n");
                    self.block_terminated = true;
                } else {
                    self.ensure_type_eq(&expected, &v.ty, "return")?;
                    self.out.push_str(&format!("  ret {} {}\n", v.ty, v.op));
                    self.block_terminated = true;
                }
                Ok(())
            }
            Statement::Print { value } => self.emit_print_expr(value),
            Statement::ExpressionStatement(expr) => {
                let _ = self.generate_expr(expr)?;
                Ok(())
            }
            Statement::If {
                condition,
                consequence,
                alternative,
            } => self.generate_if(condition, consequence, alternative),
            Statement::While { condition, body } => self.generate_while(condition, body),
            Statement::ForClassic {
                init,
                condition,
                post,
                body,
            } => self.generate_for_classic(init, condition, post, body),
            Statement::ForIn {
                index,
                iterator,
                collection,
                body,
                ..
            } => self.generate_for_in(index, iterator, collection, body),
            Statement::IndexAssign {
                object,
                index,
                value,
                ..
            } => self.generate_index_assign(object, index, value),
            Statement::Break => {
                let (break_label, _) = self
                    .loop_stack
                    .last()
                    .cloned()
                    .ok_or_else(|| "break used outside loop".to_string())?;
                self.out.push_str(&format!("  br label %{}\n", break_label));
                self.block_terminated = true;
                Ok(())
            }
            Statement::Continue => {
                let (_, continue_label) = self
                    .loop_stack
                    .last()
                    .cloned()
                    .ok_or_else(|| "continue used outside loop".to_string())?;
                self.out
                    .push_str(&format!("  br label %{}\n", continue_label));
                self.block_terminated = true;
                Ok(())
            }
            Statement::PackageDecl { .. }
            | Statement::Import { .. }
            | Statement::FunctionDecl { .. }
            | Statement::StructDecl { .. }
            | Statement::MethodDecl { .. }
            | Statement::InterfaceDecl { .. }
            | Statement::Export { .. }
            | Statement::TestDecl { .. }
            | Statement::Go { .. }
            | Statement::Defer { .. } => Err(format!(
                "LLVM Phase1 backend does not support this statement yet: {:?}",
                stmt
            )),
        }
    }

    fn generate_index_assign(
        &mut self,
        object: &Expression,
        index: &Expression,
        value: &Expression,
    ) -> Result<(), String> {
        let Expression::Identifier(name) = object else {
            return Err("Index assignment currently supports identifier targets only".to_string());
        };
        if let Some(meta) = self.lookup_map_meta(name) {
            let (map_ty, map_ptr_slot) = self
                .lookup_var(name)
                .ok_or_else(|| format!("Undefined map target: {}", name))?;
            let map_val = self.next_temp();
            self.out.push_str(&format!(
                "  {} = load {}, {}* {}, align 8\n",
                map_val, map_ty, map_ty, map_ptr_slot
            ));
            let k = self.generate_expr(index)?;
            self.ensure_type_eq(&meta.key_ty, &k.ty, "map index assignment key")?;
            let v = self.generate_expr(value)?;
            self.ensure_type_eq(&meta.value_ty, &v.ty, "map index assignment value")?;
            let set_fn = format!(
                "@mlang_map_set_{}",
                self.map_suffix(&meta.key_ty, &meta.value_ty)
            );
            self.out.push_str(&format!(
                "  call void {}({} {}, {} {}, {} {})\n",
                set_fn, map_ty, map_val, meta.key_ty, k.op, meta.value_ty, v.op
            ));
            return Ok(());
        }

        let meta = self
            .lookup_array_meta(name)
            .ok_or_else(|| format!("Index assignment requires array target: {}", name))?;
        let (arr_ptr_ty, arr_ptr_slot) = self
            .lookup_var(name)
            .ok_or_else(|| format!("Undefined index assignment target: {}", name))?;

        let arr_ptr = self.next_temp();
        self.out.push_str(&format!(
            "  {} = load {}, {}* {}, align 8\n",
            arr_ptr, arr_ptr_ty, arr_ptr_ty, arr_ptr_slot
        ));

        let idx_raw = self.generate_expr(index)?;
        let idx = self.ensure_i64(idx_raw, "index assignment")?;

        let rhs = self.generate_expr(value)?;
        self.ensure_type_eq(&meta.elem_ty, &rhs.ty, "index assignment element")?;

        let elem_ptr = self.next_temp();
        self.out.push_str(&format!(
            "  {} = getelementptr inbounds {}, {} {}, i64 {}\n",
            elem_ptr, meta.elem_ty, arr_ptr_ty, arr_ptr, idx.op
        ));
        self.out.push_str(&format!(
            "  store {} {}, {}* {}, align 8\n",
            rhs.ty, rhs.op, rhs.ty, elem_ptr
        ));
        Ok(())
    }

    fn generate_field_assign(
        &mut self,
        object: &str,
        field: &str,
        value: &Expression,
    ) -> Result<(), String> {
        let (obj_ty, obj_ptr) = self
            .lookup_var(object)
            .ok_or_else(|| format!("Undefined object for field assignment: {}", object))?;
        let struct_name = self
            .extract_struct_name_from_llvm_type(&obj_ty)
            .ok_or_else(|| format!("Field assignment target is not a struct: {}", obj_ty))?;
        let meta = self
            .lookup_struct_meta(struct_name)
            .cloned()
            .ok_or_else(|| format!("Unknown struct type: {}", struct_name))?;
        let field_idx = self
            .find_struct_field_index(&meta, field)
            .ok_or_else(|| format!("Unknown field `{}` on struct `{}`", field, struct_name))?;
        let field_ty = meta.fields[field_idx].1.clone();

        let rhs = self.generate_expr(value)?;
        self.ensure_type_eq(&field_ty, &rhs.ty, "field assignment")?;

        let current = self.next_temp();
        self.out.push_str(&format!(
            "  {} = load {}, {}* {}, align 8\n",
            current, obj_ty, obj_ty, obj_ptr
        ));

        let updated = self.next_temp();
        self.out.push_str(&format!(
            "  {} = insertvalue {} {}, {} {}, {}\n",
            updated, obj_ty, current, rhs.ty, rhs.op, field_idx
        ));

        self.out.push_str(&format!(
            "  store {} {}, {}* {}, align 8\n",
            obj_ty, updated, obj_ty, obj_ptr
        ));
        Ok(())
    }

    fn generate_array_let(
        &mut self,
        name: &str,
        inner: &Type,
        value: &Expression,
    ) -> Result<(), String> {
        let elem_ty = self.type_to_llvm(inner)?;
        let Expression::ArrayLiteral { elements } = value else {
            return Err("Array let currently requires array literal initializer".to_string());
        };

        let n = elements.len();
        let cap = n.max(4);
        let elem_size = self.llvm_type_size(&elem_ty)?;
        let alloc_bytes = (cap as u64) * elem_size;
        let raw_ptr = self.next_temp();
        self.out.push_str(&format!(
            "  {} = call i8* @mlang_alloc(i64 {})\n",
            raw_ptr, alloc_bytes
        ));
        let ptr_ty = format!("{}*", elem_ty);
        let data_ptr = self.next_temp();
        self.out.push_str(&format!(
            "  {} = bitcast i8* {} to {}\n",
            data_ptr, raw_ptr, ptr_ty
        ));

        for (i, expr) in elements.iter().enumerate() {
            let v = self.generate_expr(expr)?;
            self.ensure_type_eq(&elem_ty, &v.ty, "array literal element")?;
            let gep = self.next_temp();
            self.out.push_str(&format!(
                "  {} = getelementptr inbounds {}, {} {}, i64 {}\n",
                gep, elem_ty, ptr_ty, data_ptr, i
            ));
            self.out.push_str(&format!(
                "  store {} {}, {}* {}, align 8\n",
                elem_ty, v.op, elem_ty, gep
            ));
        }

        let ptr_slot = format!(
            "%{}.addr.{}",
            self.sanitize_ident(name),
            self.next_temp_id()
        );
        self.out
            .push_str(&format!("  {} = alloca {}, align 8\n", ptr_slot, ptr_ty));
        self.out.push_str(&format!(
            "  store {} {}, {}* {}, align 8\n",
            ptr_ty, data_ptr, ptr_ty, ptr_slot
        ));

        let len_slot = format!("%{}.len.{}", self.sanitize_ident(name), self.next_temp_id());
        self.out
            .push_str(&format!("  {} = alloca i64, align 8\n", len_slot));
        self.out
            .push_str(&format!("  store i64 {}, i64* {}, align 8\n", n, len_slot));

        let cap_slot = format!("%{}.cap.{}", self.sanitize_ident(name), self.next_temp_id());
        self.out
            .push_str(&format!("  {} = alloca i64, align 8\n", cap_slot));
        self.out.push_str(&format!(
            "  store i64 {}, i64* {}, align 8\n",
            cap, cap_slot
        ));

        self.insert_var(name, ptr_ty, ptr_slot);
        self.insert_array_meta(
            name,
            ArrayMeta {
                elem_ty,
                len_slot,
                cap_slot,
            },
        );
        Ok(())
    }

    fn generate_map_let(
        &mut self,
        name: &str,
        key: &Type,
        val: &Type,
        value: &Expression,
    ) -> Result<(), String> {
        let key_ty = self.type_to_llvm(key)?;
        let value_ty = self.type_to_llvm(val)?;
        let map_ty = self.type_to_llvm(&Type::Map(Box::new(key.clone()), Box::new(val.clone())))?;
        let map_base = map_ty.trim_end_matches('*').to_string();
        self.ensure_map_runtime_decls(&map_base, &key_ty, &value_ty);

        let suffix = self.map_suffix(&key_ty, &value_ty);
        let new_fn = format!("@mlang_map_new_{}", suffix);
        let set_fn = format!("@mlang_map_set_{}", suffix);

        let map_val = self.next_temp();
        self.out
            .push_str(&format!("  {} = call {} {}()\n", map_val, map_ty, new_fn));

        let Expression::HashLiteral { pairs } = value else {
            return Err("Map let currently requires hash literal initializer".to_string());
        };

        for (kexpr, vexpr) in pairs {
            let k = self.generate_expr(kexpr)?;
            self.ensure_type_eq(&key_ty, &k.ty, "map key")?;
            let v = self.generate_expr(vexpr)?;
            self.ensure_type_eq(&value_ty, &v.ty, "map value")?;
            self.out.push_str(&format!(
                "  call void {}({} {}, {} {}, {} {})\n",
                set_fn, map_ty, map_val, key_ty, k.op, value_ty, v.op
            ));
        }

        let ptr = format!(
            "%{}.addr.{}",
            self.sanitize_ident(name),
            self.next_temp_id()
        );
        self.out
            .push_str(&format!("  {} = alloca {}, align 8\n", ptr, map_ty));
        self.out.push_str(&format!(
            "  store {} {}, {}* {}, align 8\n",
            map_ty, map_val, map_ty, ptr
        ));
        self.insert_var(name, map_ty.clone(), ptr);
        self.insert_map_meta(
            name,
            MapMeta {
                map_base,
                key_ty,
                value_ty,
            },
        );
        Ok(())
    }

    fn generate_if(
        &mut self,
        condition: &Expression,
        consequence: &BlockStatement,
        alternative: &Option<IfAlternative>,
    ) -> Result<(), String> {
        let then_label = self.next_label("if_then");
        let else_label = self.next_label("if_else");
        let end_label = self.next_label("if_end");

        let cond_raw = self.generate_expr(condition)?;
        let cond = self.ensure_bool(cond_raw)?;
        self.out.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cond.op, then_label, else_label
        ));
        self.block_terminated = true;

        self.out.push_str(&format!("{}:\n", then_label));
        self.block_terminated = false;
        self.generate_block(consequence)?;
        if !self.current_block_has_terminator() {
            self.out.push_str(&format!("  br label %{}\n", end_label));
            self.block_terminated = true;
        }

        self.out.push_str(&format!("{}:\n", else_label));
        self.block_terminated = false;
        if let Some(alt) = alternative {
            match alt {
                IfAlternative::Else(block) => self.generate_block(block)?,
                IfAlternative::ElseIf(stmt) => self.generate_stmt(stmt)?,
            }
        }
        if !self.current_block_has_terminator() {
            self.out.push_str(&format!("  br label %{}\n", end_label));
            self.block_terminated = true;
        }

        self.out.push_str(&format!("{}:\n", end_label));
        self.block_terminated = false;
        Ok(())
    }

    fn generate_while(
        &mut self,
        condition: &Expression,
        body: &BlockStatement,
    ) -> Result<(), String> {
        let cond_label = self.next_label("while_cond");
        let body_label = self.next_label("while_body");
        let end_label = self.next_label("while_end");

        self.out.push_str(&format!("  br label %{}\n", cond_label));
        self.block_terminated = true;
        self.out.push_str(&format!("{}:\n", cond_label));
        self.block_terminated = false;
        let cond_raw = self.generate_expr(condition)?;
        let cond = self.ensure_bool(cond_raw)?;
        self.out.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cond.op, body_label, end_label
        ));
        self.block_terminated = true;

        self.out.push_str(&format!("{}:\n", body_label));
        self.block_terminated = false;
        self.loop_stack
            .push((end_label.clone(), cond_label.clone()));
        self.generate_block(body)?;
        self.loop_stack.pop();
        if !self.current_block_has_terminator() {
            self.out.push_str(&format!("  br label %{}\n", cond_label));
            self.block_terminated = true;
        }

        self.out.push_str(&format!("{}:\n", end_label));
        self.block_terminated = false;
        Ok(())
    }

    fn generate_for_classic(
        &mut self,
        init: &Option<Box<Statement>>,
        condition: &Option<Expression>,
        post: &Option<Box<Statement>>,
        body: &BlockStatement,
    ) -> Result<(), String> {
        if let Some(init_stmt) = init {
            self.generate_stmt(init_stmt)?;
        }

        let cond_label = self.next_label("for_cond");
        let body_label = self.next_label("for_body");
        let post_label = self.next_label("for_post");
        let end_label = self.next_label("for_end");

        self.out.push_str(&format!("  br label %{}\n", cond_label));
        self.block_terminated = true;

        self.out.push_str(&format!("{}:\n", cond_label));
        self.block_terminated = false;
        if let Some(cond_expr) = condition {
            let cond_raw = self.generate_expr(cond_expr)?;
            let cond = self.ensure_bool(cond_raw)?;
            self.out.push_str(&format!(
                "  br i1 {}, label %{}, label %{}\n",
                cond.op, body_label, end_label
            ));
            self.block_terminated = true;
        } else {
            self.out.push_str(&format!("  br label %{}\n", body_label));
            self.block_terminated = true;
        }

        self.out.push_str(&format!("{}:\n", body_label));
        self.block_terminated = false;
        self.loop_stack
            .push((end_label.clone(), post_label.clone()));
        self.generate_block(body)?;
        self.loop_stack.pop();
        if !self.current_block_has_terminator() {
            self.out.push_str(&format!("  br label %{}\n", post_label));
            self.block_terminated = true;
        }

        self.out.push_str(&format!("{}:\n", post_label));
        self.block_terminated = false;
        if let Some(post_stmt) = post {
            self.generate_stmt(post_stmt)?;
        }
        if !self.current_block_has_terminator() {
            self.out.push_str(&format!("  br label %{}\n", cond_label));
            self.block_terminated = true;
        }

        self.out.push_str(&format!("{}:\n", end_label));
        self.block_terminated = false;
        Ok(())
    }

    fn generate_for_in(
        &mut self,
        index: &Option<String>,
        iterator: &str,
        collection: &Expression,
        body: &BlockStatement,
    ) -> Result<(), String> {
        let (arr_ptr_ty, arr_ptr_slot, meta) = match collection {
            Expression::Identifier(name) => {
                let meta = self
                    .lookup_array_meta(name)
                    .ok_or_else(|| format!("for-in requires array collection, got: {}", name))?;
                let (ptr_ty, ptr_slot) = self
                    .lookup_var(name)
                    .ok_or_else(|| format!("Undefined collection variable: {}", name))?;
                (ptr_ty, ptr_slot, meta)
            }
            _ => {
                return Err("for-in currently supports identifier collections only".to_string());
            }
        };

        self.push_scope();

        let idx_ptr = format!("%for_idx.addr.{}", self.next_temp_id());
        self.out
            .push_str(&format!("  {} = alloca i64, align 8\n", idx_ptr));
        self.out
            .push_str(&format!("  store i64 0, i64* {}, align 8\n", idx_ptr));

        let iter_ptr = format!(
            "%{}.addr.{}",
            self.sanitize_ident(iterator),
            self.next_temp_id()
        );
        self.out.push_str(&format!(
            "  {} = alloca {}, align 8\n",
            iter_ptr, meta.elem_ty
        ));
        self.insert_var(iterator, meta.elem_ty.clone(), iter_ptr.clone());

        let idx_user_ptr = if let Some(idx_name) = index {
            let ptr = format!(
                "%{}.addr.{}",
                self.sanitize_ident(idx_name),
                self.next_temp_id()
            );
            self.out
                .push_str(&format!("  {} = alloca i64, align 8\n", ptr));
            self.insert_var(idx_name, "i64".to_string(), ptr.clone());
            Some(ptr)
        } else {
            None
        };

        let cond_label = self.next_label("forin_cond");
        let body_label = self.next_label("forin_body");
        let post_label = self.next_label("forin_post");
        let end_label = self.next_label("forin_end");

        self.out.push_str(&format!("  br label %{}\n", cond_label));
        self.block_terminated = true;

        self.out.push_str(&format!("{}:\n", cond_label));
        self.block_terminated = false;
        let idx_val = self.next_temp();
        self.out.push_str(&format!(
            "  {} = load i64, i64* {}, align 8\n",
            idx_val, idx_ptr
        ));
        let len_val = self.next_temp();
        self.out.push_str(&format!(
            "  {} = load i64, i64* {}, align 8\n",
            len_val, meta.len_slot
        ));
        let cmp = self.next_temp();
        self.out.push_str(&format!(
            "  {} = icmp slt i64 {}, {}\n",
            cmp, idx_val, len_val
        ));
        self.out.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cmp, body_label, end_label
        ));
        self.block_terminated = true;

        self.out.push_str(&format!("{}:\n", body_label));
        self.block_terminated = false;

        let idx_cur = self.next_temp();
        self.out.push_str(&format!(
            "  {} = load i64, i64* {}, align 8\n",
            idx_cur, idx_ptr
        ));
        let arr_ptr = self.next_temp();
        self.out.push_str(&format!(
            "  {} = load {}, {}* {}, align 8\n",
            arr_ptr, arr_ptr_ty, arr_ptr_ty, arr_ptr_slot
        ));
        if let Some(ptr) = idx_user_ptr {
            self.out
                .push_str(&format!("  store i64 {}, i64* {}, align 8\n", idx_cur, ptr));
        }

        let elem_ptr = self.next_temp();
        self.out.push_str(&format!(
            "  {} = getelementptr inbounds {}, {} {}, i64 {}\n",
            elem_ptr, meta.elem_ty, arr_ptr_ty, arr_ptr, idx_cur
        ));
        let elem_val = self.next_temp();
        self.out.push_str(&format!(
            "  {} = load {}, {}* {}, align 8\n",
            elem_val, meta.elem_ty, meta.elem_ty, elem_ptr
        ));
        self.out.push_str(&format!(
            "  store {} {}, {}* {}, align 8\n",
            meta.elem_ty, elem_val, meta.elem_ty, iter_ptr
        ));

        self.loop_stack
            .push((end_label.clone(), post_label.clone()));
        self.generate_block(body)?;
        self.loop_stack.pop();
        if !self.current_block_has_terminator() {
            self.out.push_str(&format!("  br label %{}\n", post_label));
            self.block_terminated = true;
        }

        self.out.push_str(&format!("{}:\n", post_label));
        self.block_terminated = false;
        let idx_now = self.next_temp();
        self.out.push_str(&format!(
            "  {} = load i64, i64* {}, align 8\n",
            idx_now, idx_ptr
        ));
        let idx_next = self.next_temp();
        self.out
            .push_str(&format!("  {} = add i64 {}, 1\n", idx_next, idx_now));
        self.out.push_str(&format!(
            "  store i64 {}, i64* {}, align 8\n",
            idx_next, idx_ptr
        ));
        self.out.push_str(&format!("  br label %{}\n", cond_label));
        self.block_terminated = true;

        self.out.push_str(&format!("{}:\n", end_label));
        self.block_terminated = false;
        self.pop_scope();
        Ok(())
    }

    fn generate_expr(&mut self, expr: &Expression) -> Result<ValueRef, String> {
        match expr {
            Expression::IntegerLiteral(v) => Ok(ValueRef {
                ty: "i64".to_string(),
                op: v.to_string(),
            }),
            Expression::BooleanLiteral(v) => Ok(ValueRef {
                ty: "i1".to_string(),
                op: if *v { "1" } else { "0" }.to_string(),
            }),
            Expression::FloatLiteral(v) => Ok(ValueRef {
                ty: "double".to_string(),
                op: format!("{}", v),
            }),
            Expression::StringLiteral(s) => {
                let (gname, llen) = self.emit_string_constant(s);
                let tmp = self.next_temp();
                self.out.push_str(&format!(
                    "  {} = getelementptr inbounds [{} x i8], [{} x i8]* {}, i64 0, i64 0\n",
                    tmp, llen, llen, gname
                ));
                Ok(ValueRef {
                    ty: "i8*".to_string(),
                    op: tmp,
                })
            }
            Expression::NilLiteral => Ok(ValueRef {
                ty: "i8*".to_string(),
                op: "null".to_string(),
            }),
            Expression::Identifier(name) => {
                if let Some((ty, ptr)) = self.lookup_var(name) {
                    let tmp = self.next_temp();
                    self.out.push_str(&format!(
                        "  {} = load {}, {}* {}, align 8\n",
                        tmp, ty, ty, ptr
                    ));
                    Ok(ValueRef { ty, op: tmp })
                } else if let Some(sig) = self.function_registry.get(name) {
                    Ok(ValueRef {
                        ty: self.fn_ptr_type(sig),
                        op: format!("@{}", self.sanitize_ident(name)),
                    })
                } else {
                    Err(format!("Undefined variable: {}", name))
                }
            }
            Expression::IndexExpression { left, index } => {
                self.generate_index_expression(left, index)
            }
            Expression::MethodCall {
                object,
                method,
                arguments,
            } => self.generate_method_call(object, method, arguments),
            Expression::FieldAccess { object, field } => self.generate_field_access(object, field),
            Expression::StructLiteral { name, fields } => {
                self.generate_struct_literal(name, fields)
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => self.generate_binary(left, operator, right),
            Expression::FunctionCall {
                function,
                arguments,
            } => self.generate_call(function, arguments),
            Expression::TypeConversion {
                target_type,
                argument,
            } => {
                let v = self.generate_expr(argument)?;
                self.generate_conversion(v, target_type)
            }
            Expression::TupleLiteral { elements } => {
                let mut values = Vec::with_capacity(elements.len());
                for e in elements {
                    values.push(self.generate_expr(e)?);
                }
                if values.is_empty() {
                    return Err("Empty tuple literal is not supported".to_string());
                }
                let tuple_ty = format!(
                    "{{ {} }}",
                    values
                        .iter()
                        .map(|v| v.ty.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let ptr = self.next_temp();
                self.out
                    .push_str(&format!("  {} = alloca {}, align 8\n", ptr, tuple_ty));
                for (i, v) in values.iter().enumerate() {
                    let gep = self.next_temp();
                    self.out.push_str(&format!(
                        "  {} = getelementptr inbounds {}, {}* {}, i32 0, i32 {}\n",
                        gep, tuple_ty, tuple_ty, ptr, i
                    ));
                    self.out.push_str(&format!(
                        "  store {} {}, {}* {}, align 8\n",
                        v.ty, v.op, v.ty, gep
                    ));
                }
                let tmp = self.next_temp();
                self.out.push_str(&format!(
                    "  {} = load {}, {}* {}, align 8\n",
                    tmp, tuple_ty, tuple_ty, ptr
                ));
                Ok(ValueRef {
                    ty: tuple_ty,
                    op: tmp,
                })
            }
            Expression::ErrorCreate { message } => {
                let msg = self.generate_expr(message)?;
                self.ensure_type_eq("i8*", &msg.ty, "error value")?;
                Ok(msg)
            }
            Expression::ClosureLiteral {
                parameters,
                return_type,
                body,
            } => self.generate_closure_literal(parameters, return_type, body),
            other => Err(format!(
                "LLVM Phase1 backend does not support expression yet: {:?}",
                other
            )),
        }
    }

    fn generate_struct_literal(
        &mut self,
        name: &str,
        fields: &[(String, Expression)],
    ) -> Result<ValueRef, String> {
        let meta = self
            .lookup_struct_meta(name)
            .cloned()
            .ok_or_else(|| format!("Unknown struct literal type: {}", name))?;
        let struct_ty = meta.llvm_name.clone();

        let ptr = self.next_temp();
        self.out
            .push_str(&format!("  {} = alloca {}, align 8\n", ptr, struct_ty));

        for (fname, fexpr) in fields {
            let idx = self
                .find_struct_field_index(&meta, fname)
                .ok_or_else(|| format!("Unknown field `{}` in struct literal `{}`", fname, name))?;
            let expected_ty = meta.fields[idx].1.clone();
            let v = self.generate_expr(fexpr)?;
            self.ensure_type_eq(&expected_ty, &v.ty, "struct literal field")?;
            let fptr = self.next_temp();
            self.out.push_str(&format!(
                "  {} = getelementptr inbounds {}, {}* {}, i32 0, i32 {}\n",
                fptr, struct_ty, struct_ty, ptr, idx
            ));
            self.out.push_str(&format!(
                "  store {} {}, {}* {}, align 8\n",
                v.ty, v.op, v.ty, fptr
            ));
        }

        let assembled = self.next_temp();
        self.out.push_str(&format!(
            "  {} = load {}, {}* {}, align 8\n",
            assembled, struct_ty, struct_ty, ptr
        ));

        Ok(ValueRef {
            ty: struct_ty,
            op: assembled,
        })
    }

    fn generate_field_access(
        &mut self,
        object: &Expression,
        field: &str,
    ) -> Result<ValueRef, String> {
        let obj = self.generate_expr(object)?;
        let struct_name = self
            .extract_struct_name_from_llvm_type(&obj.ty)
            .ok_or_else(|| format!("Field access target is not a struct: {}", obj.ty))?;
        let meta = self
            .lookup_struct_meta(struct_name)
            .cloned()
            .ok_or_else(|| format!("Unknown struct type: {}", struct_name))?;
        let idx = self
            .find_struct_field_index(&meta, field)
            .ok_or_else(|| format!("Unknown field `{}` on struct `{}`", field, struct_name))?;
        let field_ty = meta.fields[idx].1.clone();

        let tmp = self.next_temp();
        self.out.push_str(&format!(
            "  {} = extractvalue {} {}, {}\n",
            tmp, obj.ty, obj.op, idx
        ));
        Ok(ValueRef {
            ty: field_ty,
            op: tmp,
        })
    }

    fn generate_conversion(&mut self, value: ValueRef, target: &Type) -> Result<ValueRef, String> {
        let target_ty = self.type_to_llvm(target)?;
        if value.ty == target_ty {
            return Ok(value);
        }

        let tmp = self.next_temp();
        match (value.ty.as_str(), target_ty.as_str()) {
            ("i64", "double") => {
                self.out
                    .push_str(&format!("  {} = sitofp i64 {} to double\n", tmp, value.op));
                Ok(ValueRef {
                    ty: "double".to_string(),
                    op: tmp,
                })
            }
            ("double", "i64") => {
                self.out
                    .push_str(&format!("  {} = fptosi double {} to i64\n", tmp, value.op));
                Ok(ValueRef {
                    ty: "i64".to_string(),
                    op: tmp,
                })
            }
            _ => Err(format!(
                "Unsupported conversion: {} -> {}",
                value.ty, target_ty
            )),
        }
    }

    fn generate_call(
        &mut self,
        function: &str,
        arguments: &[Expression],
    ) -> Result<ValueRef, String> {
        if let Some(sig) = self.lookup_fn_value(function) {
            let (fn_ty, fn_ptr_slot) = self
                .lookup_var(function)
                .ok_or_else(|| format!("Undefined function value: {}", function))?;
            let args = self.prepare_call_args(&sig, arguments, function)?;
            let fn_ptr = self.next_temp();
            self.out.push_str(&format!(
                "  {} = load {}, {}* {}, align 8\n",
                fn_ptr, fn_ty, fn_ty, fn_ptr_slot
            ));
            if sig.ret_ty == "void" {
                self.out
                    .push_str(&format!("  call void {}({})\n", fn_ptr, args.join(", ")));
                return Ok(ValueRef {
                    ty: "void".to_string(),
                    op: String::new(),
                });
            }
            let tmp = self.next_temp();
            self.out.push_str(&format!(
                "  {} = call {} {}({})\n",
                tmp,
                sig.ret_ty,
                fn_ptr,
                args.join(", ")
            ));
            return Ok(ValueRef {
                ty: sig.ret_ty,
                op: tmp,
            });
        }

        let sig = self
            .function_registry
            .get(function)
            .cloned()
            .ok_or_else(|| format!("Undefined function: {}", function))?;
        let args = self.prepare_call_args(&sig, arguments, function)?;

        if sig.ret_ty == "void" {
            self.out.push_str(&format!(
                "  call void @{}({})\n",
                self.sanitize_ident(function),
                args.join(", ")
            ));
            return Ok(ValueRef {
                ty: "void".to_string(),
                op: String::new(),
            });
        }
        let tmp = self.next_temp();
        self.out.push_str(&format!(
            "  {} = call {} @{}({})\n",
            tmp,
            sig.ret_ty,
            self.sanitize_ident(function),
            args.join(", ")
        ));
        Ok(ValueRef {
            ty: sig.ret_ty,
            op: tmp,
        })
    }

    fn generate_method_call(
        &mut self,
        object: &Expression,
        method: &str,
        arguments: &[Expression],
    ) -> Result<ValueRef, String> {
        if method == "push" {
            let Expression::Identifier(name) = object else {
                return Err("push currently supports identifier arrays only".to_string());
            };
            let meta = self
                .lookup_array_meta(name)
                .ok_or_else(|| format!("push target is not tracked as array: {}", name))?;
            if arguments.len() != 1 {
                return Err("push expects exactly one argument".to_string());
            }
            let (ptr_ty, ptr_slot) = self
                .lookup_var(name)
                .ok_or_else(|| format!("Undefined push target: {}", name))?;
            let arg = self.generate_expr(&arguments[0])?;
            self.ensure_type_eq(&meta.elem_ty, &arg.ty, "push argument")?;
            let len = self.next_temp();
            self.out.push_str(&format!(
                "  {} = load i64, i64* {}, align 8\n",
                len, meta.len_slot
            ));
            let cap = self.next_temp();
            self.out.push_str(&format!(
                "  {} = load i64, i64* {}, align 8\n",
                cap, meta.cap_slot
            ));

            let needs_grow = self.next_temp();
            self.out.push_str(&format!(
                "  {} = icmp eq i64 {}, {}\n",
                needs_grow, len, cap
            ));

            let grow_label = self.next_label("array_push_grow");
            let cont_label = self.next_label("array_push_cont");
            self.out.push_str(&format!(
                "  br i1 {}, label %{}, label %{}\n",
                needs_grow, grow_label, cont_label
            ));

            self.out.push_str(&format!("{}:\n", grow_label));
            let doubled_cap = self.next_temp();
            self.out
                .push_str(&format!("  {} = mul i64 {}, 2\n", doubled_cap, cap));
            let nonzero_cap = self.next_temp();
            self.out
                .push_str(&format!("  {} = icmp ne i64 {}, 0\n", nonzero_cap, cap));
            let new_cap = self.next_temp();
            self.out.push_str(&format!(
                "  {} = select i1 {}, i64 {}, i64 4\n",
                new_cap, nonzero_cap, doubled_cap
            ));
            let new_bytes = self.next_temp();
            self.out.push_str(&format!(
                "  {} = mul i64 {}, {}\n",
                new_bytes,
                new_cap,
                self.llvm_type_size(&meta.elem_ty)?
            ));
            let old_ptr = self.next_temp();
            self.out.push_str(&format!(
                "  {} = load {}, {}* {}, align 8\n",
                old_ptr, ptr_ty, ptr_ty, ptr_slot
            ));
            let old_raw = self.next_temp();
            self.out.push_str(&format!(
                "  {} = bitcast {} {} to i8*\n",
                old_raw, ptr_ty, old_ptr
            ));
            let grown_raw = self.next_temp();
            self.out.push_str(&format!(
                "  {} = call i8* @mlang_realloc(i8* {}, i64 {})\n",
                grown_raw, old_raw, new_bytes
            ));
            let grown_ptr = self.next_temp();
            self.out.push_str(&format!(
                "  {} = bitcast i8* {} to {}\n",
                grown_ptr, grown_raw, ptr_ty
            ));
            self.out.push_str(&format!(
                "  store {} {}, {}* {}, align 8\n",
                ptr_ty, grown_ptr, ptr_ty, ptr_slot
            ));
            self.out.push_str(&format!(
                "  store i64 {}, i64* {}, align 8\n",
                new_cap, meta.cap_slot
            ));
            self.out.push_str(&format!("  br label %{}\n", cont_label));

            self.out.push_str(&format!("{}:\n", cont_label));
            let cur_ptr = self.next_temp();
            self.out.push_str(&format!(
                "  {} = load {}, {}* {}, align 8\n",
                cur_ptr, ptr_ty, ptr_ty, ptr_slot
            ));
            let cur_len = self.next_temp();
            self.out.push_str(&format!(
                "  {} = load i64, i64* {}, align 8\n",
                cur_len, meta.len_slot
            ));
            let elem_ptr = self.next_temp();
            self.out.push_str(&format!(
                "  {} = getelementptr inbounds {}, {} {}, i64 {}\n",
                elem_ptr, meta.elem_ty, ptr_ty, cur_ptr, cur_len
            ));
            self.out.push_str(&format!(
                "  store {} {}, {}* {}, align 8\n",
                arg.ty, arg.op, arg.ty, elem_ptr
            ));
            let next_len = self.next_temp();
            self.out
                .push_str(&format!("  {} = add i64 {}, 1\n", next_len, cur_len));
            self.out.push_str(&format!(
                "  store i64 {}, i64* {}, align 8\n",
                next_len, meta.len_slot
            ));
            return Ok(ValueRef {
                ty: ptr_ty,
                op: cur_ptr,
            });
        }

        Err(format!(
            "Unsupported method call in LLVM backend: {}",
            method
        ))
    }

    fn generate_index_expression(
        &mut self,
        left: &Expression,
        index: &Expression,
    ) -> Result<ValueRef, String> {
        let Expression::Identifier(name) = left else {
            return Err(
                "Index expression currently supports identifier collections only".to_string(),
            );
        };
        if let Some(meta) = self.lookup_map_meta(name) {
            let (map_ty, map_ptr_slot) = self
                .lookup_var(name)
                .ok_or_else(|| format!("Undefined map index target: {}", name))?;
            let map_val = self.next_temp();
            self.out.push_str(&format!(
                "  {} = load {}, {}* {}, align 8\n",
                map_val, map_ty, map_ty, map_ptr_slot
            ));
            let k = self.generate_expr(index)?;
            self.ensure_type_eq(&meta.key_ty, &k.ty, "map index key")?;
            let get_fn = format!(
                "@mlang_map_get_{}",
                self.map_suffix(&meta.key_ty, &meta.value_ty)
            );
            let out = self.next_temp();
            self.out.push_str(&format!(
                "  {} = call {} {}({} {}, {} {})\n",
                out, meta.value_ty, get_fn, map_ty, map_val, meta.key_ty, k.op
            ));
            return Ok(ValueRef {
                ty: meta.value_ty,
                op: out,
            });
        }

        let meta = self
            .lookup_array_meta(name)
            .ok_or_else(|| format!("Index expression requires array target: {}", name))?;
        let (arr_ptr_ty, arr_ptr_slot) = self
            .lookup_var(name)
            .ok_or_else(|| format!("Undefined index expression target: {}", name))?;

        let arr_ptr = self.next_temp();
        self.out.push_str(&format!(
            "  {} = load {}, {}* {}, align 8\n",
            arr_ptr, arr_ptr_ty, arr_ptr_ty, arr_ptr_slot
        ));

        let idx_raw = self.generate_expr(index)?;
        let idx = self.ensure_i64(idx_raw, "index expression")?;

        let elem_ptr = self.next_temp();
        self.out.push_str(&format!(
            "  {} = getelementptr inbounds {}, {} {}, i64 {}\n",
            elem_ptr, meta.elem_ty, arr_ptr_ty, arr_ptr, idx.op
        ));

        let elem_val = self.next_temp();
        self.out.push_str(&format!(
            "  {} = load {}, {}* {}, align 8\n",
            elem_val, meta.elem_ty, meta.elem_ty, elem_ptr
        ));
        Ok(ValueRef {
            ty: meta.elem_ty,
            op: elem_val,
        })
    }

    fn generate_binary(
        &mut self,
        left: &Expression,
        op: &str,
        right: &Expression,
    ) -> Result<ValueRef, String> {
        let l = self.generate_expr(left)?;
        let r = self.generate_expr(right)?;
        self.ensure_type_eq(&l.ty, &r.ty, "binary expression")?;

        match l.ty.as_str() {
            "i64" => self.generate_i64_binary(l, op, r),
            "double" => self.generate_f64_binary(l, op, r),
            "i1" => self.generate_i1_binary(l, op, r),
            ptr if ptr.ends_with('*') => self.generate_ptr_binary(l, op, r),
            _ => Err(format!("Unsupported binary type: {}", l.ty)),
        }
    }

    fn generate_i64_binary(
        &mut self,
        l: ValueRef,
        op: &str,
        r: ValueRef,
    ) -> Result<ValueRef, String> {
        let tmp = self.next_temp();
        let (inst, out_ty) = match op {
            "+" => (format!("add i64 {}, {}", l.op, r.op), "i64"),
            "-" => (format!("sub i64 {}, {}", l.op, r.op), "i64"),
            "*" => (format!("mul i64 {}, {}", l.op, r.op), "i64"),
            "/" => (format!("sdiv i64 {}, {}", l.op, r.op), "i64"),
            "%" => (format!("srem i64 {}, {}", l.op, r.op), "i64"),
            "==" => (format!("icmp eq i64 {}, {}", l.op, r.op), "i1"),
            "!=" => (format!("icmp ne i64 {}, {}", l.op, r.op), "i1"),
            "<" => (format!("icmp slt i64 {}, {}", l.op, r.op), "i1"),
            "<=" => (format!("icmp sle i64 {}, {}", l.op, r.op), "i1"),
            ">" => (format!("icmp sgt i64 {}, {}", l.op, r.op), "i1"),
            ">=" => (format!("icmp sge i64 {}, {}", l.op, r.op), "i1"),
            _ => return Err(format!("Unsupported i64 operator: {}", op)),
        };

        self.out.push_str(&format!("  {} = {}\n", tmp, inst));
        Ok(ValueRef {
            ty: out_ty.to_string(),
            op: tmp,
        })
    }

    fn generate_f64_binary(
        &mut self,
        l: ValueRef,
        op: &str,
        r: ValueRef,
    ) -> Result<ValueRef, String> {
        let tmp = self.next_temp();
        let (inst, out_ty) = match op {
            "+" => (format!("fadd double {}, {}", l.op, r.op), "double"),
            "-" => (format!("fsub double {}, {}", l.op, r.op), "double"),
            "*" => (format!("fmul double {}, {}", l.op, r.op), "double"),
            "/" => (format!("fdiv double {}, {}", l.op, r.op), "double"),
            "==" => (format!("fcmp oeq double {}, {}", l.op, r.op), "i1"),
            "!=" => (format!("fcmp one double {}, {}", l.op, r.op), "i1"),
            "<" => (format!("fcmp olt double {}, {}", l.op, r.op), "i1"),
            "<=" => (format!("fcmp ole double {}, {}", l.op, r.op), "i1"),
            ">" => (format!("fcmp ogt double {}, {}", l.op, r.op), "i1"),
            ">=" => (format!("fcmp oge double {}, {}", l.op, r.op), "i1"),
            _ => return Err(format!("Unsupported double operator: {}", op)),
        };

        self.out.push_str(&format!("  {} = {}\n", tmp, inst));
        Ok(ValueRef {
            ty: out_ty.to_string(),
            op: tmp,
        })
    }

    fn generate_i1_binary(
        &mut self,
        l: ValueRef,
        op: &str,
        r: ValueRef,
    ) -> Result<ValueRef, String> {
        let tmp = self.next_temp();
        let inst = match op {
            "&&" => format!("and i1 {}, {}", l.op, r.op),
            "||" => format!("or i1 {}, {}", l.op, r.op),
            "==" => format!("icmp eq i1 {}, {}", l.op, r.op),
            "!=" => format!("icmp ne i1 {}, {}", l.op, r.op),
            _ => return Err(format!("Unsupported i1 operator: {}", op)),
        };
        self.out.push_str(&format!("  {} = {}\n", tmp, inst));
        Ok(ValueRef {
            ty: "i1".to_string(),
            op: tmp,
        })
    }

    fn generate_ptr_binary(
        &mut self,
        l: ValueRef,
        op: &str,
        r: ValueRef,
    ) -> Result<ValueRef, String> {
        let tmp = self.next_temp();
        let inst = match op {
            "==" => format!("icmp eq {} {}, {}", l.ty, l.op, r.op),
            "!=" => format!("icmp ne {} {}, {}", l.ty, l.op, r.op),
            _ => return Err(format!("Unsupported pointer operator: {}", op)),
        };
        self.out.push_str(&format!("  {} = {}\n", tmp, inst));
        Ok(ValueRef {
            ty: "i1".to_string(),
            op: tmp,
        })
    }

    fn emit_print_expr(&mut self, expr: &Expression) -> Result<(), String> {
        if let Expression::Identifier(name) = expr {
            if let Some(meta) = self.lookup_array_meta(name) {
                return self.emit_print_array(name, &meta);
            }
        }
        let v = self.generate_expr(expr)?;
        self.emit_print(v)
    }

    fn emit_print_array(&mut self, name: &str, meta: &ArrayMeta) -> Result<(), String> {
        let (ptr_ty, ptr_slot) = self
            .lookup_var(name)
            .ok_or_else(|| format!("Undefined array variable: {}", name))?;
        let arr_ptr = self.next_temp();
        self.out.push_str(&format!(
            "  {} = load {}, {}* {}, align 8\n",
            arr_ptr, ptr_ty, ptr_ty, ptr_slot
        ));
        let len_val = self.next_temp();
        self.out.push_str(&format!(
            "  {} = load i64, i64* {}, align 8\n",
            len_val, meta.len_slot
        ));

        match meta.elem_ty.as_str() {
            "i64" => {
                self.out.push_str(&format!(
                    "  call void @mlang_print_array_i64({} {}, i64 {})\n",
                    ptr_ty, arr_ptr, len_val
                ));
                Ok(())
            }
            "i8*" => {
                self.out.push_str(&format!(
                    "  call void @mlang_print_array_i8ptr({} {}, i64 {})\n",
                    ptr_ty, arr_ptr, len_val
                ));
                Ok(())
            }
            _ => Err(format!(
                "Unsupported array print element type for LLVM backend: {}",
                meta.elem_ty
            )),
        }
    }

    fn ensure_bool(&mut self, v: ValueRef) -> Result<ValueRef, String> {
        if v.ty == "i1" {
            return Ok(v);
        }
        if v.ty == "i64" {
            let tmp = self.next_temp();
            self.out
                .push_str(&format!("  {} = icmp ne i64 {}, 0\n", tmp, v.op));
            return Ok(ValueRef {
                ty: "i1".to_string(),
                op: tmp,
            });
        }
        Err(format!("Condition must be i1 or i64, found {}", v.ty))
    }

    fn ensure_i64(&mut self, v: ValueRef, context: &str) -> Result<ValueRef, String> {
        if v.ty == "i64" {
            return Ok(v);
        }
        if v.ty == "i1" {
            let tmp = self.next_temp();
            self.out
                .push_str(&format!("  {} = zext i1 {} to i64\n", tmp, v.op));
            return Ok(ValueRef {
                ty: "i64".to_string(),
                op: tmp,
            });
        }
        Err(format!("{} expects integer index, got {}", context, v.ty))
    }

    fn emit_print(&mut self, v: ValueRef) -> Result<(), String> {
        match v.ty.as_str() {
            "i64" => {
                let fmt_ptr = self.next_temp();
                self.out.push_str(&format!(
                    "  {} = getelementptr inbounds [6 x i8], [6 x i8]* @.fmt_i64, i64 0, i64 0\n",
                    fmt_ptr
                ));
                self.out.push_str(&format!(
                    "  call i32 (i8*, ...) @printf(i8* {}, i64 {})\n",
                    fmt_ptr, v.op
                ));
                Ok(())
            }
            "double" => {
                let fmt_ptr = self.next_temp();
                self.out.push_str(&format!(
                    "  {} = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_f64, i64 0, i64 0\n",
                    fmt_ptr
                ));
                self.out.push_str(&format!(
                    "  call i32 (i8*, ...) @printf(i8* {}, double {})\n",
                    fmt_ptr, v.op
                ));
                Ok(())
            }
            "i8*" => {
                let fmt_ptr = self.next_temp();
                self.out.push_str(&format!(
                    "  {} = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0\n",
                    fmt_ptr
                ));
                self.out.push_str(&format!(
                    "  call i32 (i8*, ...) @printf(i8* {}, i8* {})\n",
                    fmt_ptr, v.op
                ));
                Ok(())
            }
            t if t.ends_with('*') => {
                let fmt_ptr = self.next_temp();
                self.out.push_str(&format!(
                    "  {} = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_ptr, i64 0, i64 0\n",
                    fmt_ptr
                ));
                let casted = self.next_temp();
                self.out.push_str(&format!(
                    "  {} = bitcast {} {} to i8*\n",
                    casted, v.ty, v.op
                ));
                self.out.push_str(&format!(
                    "  call i32 (i8*, ...) @printf(i8* {}, i8* {})\n",
                    fmt_ptr, casted
                ));
                Ok(())
            }
            "i1" => {
                let zext = self.next_temp();
                self.out
                    .push_str(&format!("  {} = zext i1 {} to i64\n", zext, v.op));
                self.emit_print(ValueRef {
                    ty: "i64".to_string(),
                    op: zext,
                })
            }
            _ => Err(format!("Unsupported print type: {}", v.ty)),
        }
    }

    fn type_to_llvm(&self, ty: &Type) -> Result<String, String> {
        match ty {
            Type::Kain => Ok("i64".to_string()),
            Type::Sit => Ok("i1".to_string()),
            Type::DaTha => Ok("double".to_string()),
            Type::Sar | Type::Error => Ok("i8*".to_string()),
            Type::Nil => Ok("void".to_string()),
            Type::Array(inner) => Ok(format!("{}*", self.type_to_llvm(inner)?)),
            Type::Map(key, val) => {
                let key_ty = self.type_to_llvm(key)?;
                let val_ty = self.type_to_llvm(val)?;
                Ok(format!("%map.{}*", self.map_suffix(&key_ty, &val_ty)))
            }
            Type::Struct(name) => {
                let meta = self
                    .lookup_struct_meta(name)
                    .ok_or_else(|| format!("Unknown struct type in type lowering: {}", name))?;
                Ok(meta.llvm_name.clone())
            }
            Type::Tuple(items) => {
                if items.is_empty() {
                    return Err("Empty tuple type is not supported".to_string());
                }
                let mut ts = Vec::with_capacity(items.len());
                for t in items {
                    ts.push(self.type_to_llvm(t)?);
                }
                Ok(format!("{{ {} }}", ts.join(", ")))
            }
            Type::Function {
                params,
                return_type,
            } => Ok(self.fn_ptr_type(&self.function_sig_from_parts(params, return_type)?)),
            other => Err(format!("LLVM Phase1 type not supported yet: {:?}", other)),
        }
    }

    fn register_struct(&mut self, name: &str, fields: &[(String, Type)]) -> Result<(), String> {
        let clean = self.sanitize_ident(name);
        let llvm_name = format!("%struct.{}", clean);
        let mut lowered = Vec::with_capacity(fields.len());
        let mut field_tys = Vec::with_capacity(fields.len());
        for (fname, fty) in fields {
            let llvm_ty = self.type_to_llvm(fty)?;
            lowered.push((fname.clone(), llvm_ty.clone()));
            field_tys.push(llvm_ty);
        }

        self.out.push_str(&format!(
            "{} = type {{ {} }}\n",
            llvm_name,
            field_tys.join(", ")
        ));
        self.struct_registry.insert(
            name.to_string(),
            StructMeta {
                llvm_name,
                fields: lowered,
            },
        );
        Ok(())
    }

    fn lookup_struct_meta(&self, name: &str) -> Option<&StructMeta> {
        if let Some(meta) = self.struct_registry.get(name) {
            return Some(meta);
        }
        self.struct_registry.values().find(|m| m.llvm_name == name)
    }

    fn find_struct_field_index(&self, meta: &StructMeta, field: &str) -> Option<usize> {
        meta.fields.iter().position(|(n, _)| n == field)
    }

    fn extract_struct_name_from_llvm_type<'a>(&self, llvm_ty: &'a str) -> Option<&'a str> {
        if llvm_ty.starts_with("%struct.") {
            if let Some(space) = llvm_ty.find(' ') {
                return Some(&llvm_ty[..space]);
            }
            return Some(llvm_ty);
        }
        None
    }

    fn emit_default_return(&mut self) -> Result<(), String> {
        match self.current_fn_ret.as_deref().unwrap_or("void") {
            "void" => self.out.push_str("  ret void\n"),
            "i64" => self.out.push_str("  ret i64 0\n"),
            "i1" => self.out.push_str("  ret i1 0\n"),
            "double" => self.out.push_str("  ret double 0.0\n"),
            "i8*" => self.out.push_str("  ret i8* null\n"),
            other => return Err(format!("No default return for type {}", other)),
        }
        self.block_terminated = true;
        Ok(())
    }

    fn emit_string_constant(&mut self, s: &str) -> (String, usize) {
        let escaped = self.escape_llvm_bytes(s);
        let len = s.as_bytes().len() + 1;
        let gname = format!("@.str.{}", self.next_temp_id());
        self.emit_global(&format!(
            "{} = private unnamed_addr constant [{} x i8] c\"{}\\00\", align 1",
            gname, len, escaped
        ));
        (gname, len)
    }

    fn escape_llvm_bytes(&self, s: &str) -> String {
        let mut out = String::new();
        for b in s.as_bytes() {
            match *b {
                b'\\' => out.push_str("\\5C"),
                b'"' => out.push_str("\\22"),
                32..=126 => out.push(*b as char),
                _ => out.push_str(&format!("\\{:02X}", b)),
            }
        }
        out
    }

    fn next_temp_id(&mut self) -> u64 {
        self.temp_counter += 1;
        self.temp_counter
    }

    fn next_temp(&mut self) -> String {
        format!("%t{}", self.next_temp_id())
    }

    fn next_label(&mut self, prefix: &str) -> String {
        self.label_counter += 1;
        format!("{}.{}", prefix, self.label_counter)
    }

    fn ensure_type_eq(&self, expected: &str, actual: &str, context: &str) -> Result<(), String> {
        if expected == actual {
            Ok(())
        } else {
            Err(format!(
                "Type mismatch in {}: expected {}, got {}",
                context, expected, actual
            ))
        }
    }

    fn sanitize_ident(&self, raw: &str) -> String {
        let mut out = String::new();
        for (i, ch) in raw.chars().enumerate() {
            if (i == 0 && (ch.is_ascii_alphabetic() || ch == '_'))
                || (i > 0 && (ch.is_ascii_alphanumeric() || ch == '_'))
            {
                out.push(ch);
            } else {
                out.push('_');
            }
        }
        if out.is_empty() {
            "unnamed".to_string()
        } else {
            out
        }
    }

    fn insert_var(&mut self, name: &str, ty: String, ptr: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), (ty, ptr));
        }
    }

    fn insert_fn_value(&mut self, name: &str, sig: FunctionSig) {
        if let Some(scope) = self.fn_scopes.last_mut() {
            scope.insert(name.to_string(), sig);
        }
    }

    fn insert_array_meta(&mut self, name: &str, meta: ArrayMeta) {
        if let Some(scope) = self.array_scopes.last_mut() {
            scope.insert(name.to_string(), meta);
        }
    }

    fn insert_map_meta(&mut self, name: &str, meta: MapMeta) {
        if let Some(scope) = self.map_scopes.last_mut() {
            scope.insert(name.to_string(), meta);
        }
    }

    fn lookup_var(&self, name: &str) -> Option<(String, String)> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    fn lookup_fn_value(&self, name: &str) -> Option<FunctionSig> {
        for scope in self.fn_scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    fn lookup_array_meta(&self, name: &str) -> Option<ArrayMeta> {
        for scope in self.array_scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    fn lookup_map_meta(&self, name: &str) -> Option<MapMeta> {
        for scope in self.map_scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.fn_scopes.push(HashMap::new());
        self.array_scopes.push(HashMap::new());
        self.map_scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        let _ = self.scopes.pop();
        let _ = self.fn_scopes.pop();
        let _ = self.array_scopes.pop();
        let _ = self.map_scopes.pop();
    }

    fn map_suffix(&self, key_ty: &str, value_ty: &str) -> String {
        format!("{}_{}", self.type_tag(key_ty), self.type_tag(value_ty))
    }

    fn type_tag(&self, llvm_ty: &str) -> String {
        llvm_ty
            .replace('%', "")
            .replace('*', "ptr")
            .replace('{', "")
            .replace('}', "")
            .replace(' ', "")
            .replace(',', "_")
            .replace('.', "_")
    }

    fn llvm_type_size(&self, llvm_ty: &str) -> Result<u64, String> {
        match llvm_ty {
            "i1" => Ok(1),
            "i64" | "double" | "i8*" => Ok(8),
            ty if ty.ends_with('*') => Ok(8),
            other => Err(format!(
                "LLVM backend does not yet know array element size for {}",
                other
            )),
        }
    }

    fn ensure_map_runtime_decls(&mut self, map_base: &str, key_ty: &str, value_ty: &str) {
        if self.declared_map_bases.contains(map_base) {
            return;
        }
        let suffix = self.map_suffix(key_ty, value_ty);
        self.emit_global(&format!("{} = type opaque", map_base));
        self.emit_global(&format!(
            "declare {}* @mlang_map_new_{}()",
            map_base, suffix
        ));
        self.emit_global(&format!(
            "declare void @mlang_map_set_{}({}*, {}, {})",
            suffix, map_base, key_ty, value_ty
        ));
        self.emit_global(&format!(
            "declare {} @mlang_map_get_{}({}*, {})",
            value_ty, suffix, map_base, key_ty
        ));
        self.emit_global("");
        self.declared_map_bases.insert(map_base.to_string());
    }

    fn current_block_has_terminator(&self) -> bool {
        self.block_terminated
    }

    fn register_function_signature(
        &mut self,
        name: &str,
        parameters: &[(String, Type, crate::ast::Span)],
        return_type: &Type,
    ) -> Result<(), String> {
        let param_tys = parameters
            .iter()
            .map(|(_, ty, _)| self.type_to_llvm(ty))
            .collect::<Result<Vec<_>, _>>()?;
        let ret_ty = self.type_to_llvm(return_type)?;
        self.function_registry
            .insert(name.to_string(), FunctionSig { ret_ty, param_tys });
        Ok(())
    }

    fn function_sig_from_parts(
        &self,
        params: &[Type],
        return_type: &Type,
    ) -> Result<FunctionSig, String> {
        let param_tys = params
            .iter()
            .map(|ty| self.type_to_llvm(ty))
            .collect::<Result<Vec<_>, _>>()?;
        let ret_ty = self.type_to_llvm(return_type)?;
        Ok(FunctionSig { ret_ty, param_tys })
    }

    fn function_sig_from_type(&self, ty: &Type) -> Result<Option<FunctionSig>, String> {
        match ty {
            Type::Function {
                params,
                return_type,
            } => Ok(Some(self.function_sig_from_parts(params, return_type)?)),
            _ => Ok(None),
        }
    }

    fn fn_ptr_type(&self, sig: &FunctionSig) -> String {
        format!("{} ({})*", sig.ret_ty, sig.param_tys.join(", "))
    }

    fn prepare_call_args(
        &mut self,
        sig: &FunctionSig,
        arguments: &[Expression],
        function: &str,
    ) -> Result<Vec<String>, String> {
        if sig.param_tys.len() != arguments.len() {
            return Err(format!(
                "Function `{}` expects {} arguments, got {}",
                function,
                sig.param_tys.len(),
                arguments.len()
            ));
        }

        let mut args = Vec::with_capacity(arguments.len());
        for (index, arg) in arguments.iter().enumerate() {
            let value = self.generate_expr(arg)?;
            self.ensure_type_eq(
                &sig.param_tys[index],
                &value.ty,
                &format!("call argument {} for {}", index, function),
            )?;
            args.push(format!("{} {}", value.ty, value.op));
        }
        Ok(args)
    }

    fn generate_closure_literal(
        &mut self,
        parameters: &[(String, Type, crate::ast::Span)],
        return_type: &Type,
        body: &BlockStatement,
    ) -> Result<ValueRef, String> {
        let id = self.next_closure_id();
        let name = format!("mlang_closure_{}", id);
        let signature_params = parameters
            .iter()
            .map(|(_, ty, _)| ty.clone())
            .collect::<Vec<_>>();
        let sig = self.function_sig_from_parts(&signature_params, return_type)?;
        let captures = self.collect_closure_captures(parameters, body)?;

        let (env_type, env_slot) = if captures.is_empty() {
            (None, None)
        } else {
            let env_type = format!("%closure_env_{}", id);
            let env_slot = format!("@closure_env_{}", id);
            self.emit_global(&format!(
                "{} = type {{ {} }}",
                env_type,
                captures
                    .iter()
                    .map(|cap| cap.llvm_ty.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            self.emit_global(&format!(
                "{} = internal global {} zeroinitializer, align 8",
                env_slot, env_type
            ));

            for (index, capture) in captures.iter().enumerate() {
                let (capture_ty, capture_ptr) = self
                    .lookup_var(&capture.name)
                    .ok_or_else(|| format!("Undefined captured variable: {}", capture.name))?;
                let value = self.next_temp();
                self.out.push_str(&format!(
                    "  {} = load {}, {}* {}, align 8\n",
                    value, capture_ty, capture_ty, capture_ptr
                ));
                let gep = self.next_temp();
                self.out.push_str(&format!(
                    "  {} = getelementptr inbounds {}, {}* {}, i32 0, i32 {}\n",
                    gep, env_type, env_type, env_slot, index
                ));
                self.out.push_str(&format!(
                    "  store {} {}, {}* {}, align 8\n",
                    capture_ty, value, capture_ty, gep
                ));
            }
            (Some(env_type), Some(env_slot))
        };

        self.pending_closures.push(PendingClosure {
            name: name.clone(),
            parameters: parameters.to_vec(),
            return_type: return_type.clone(),
            body: body.clone(),
            captures,
            env_type,
            env_slot,
        });

        Ok(ValueRef {
            ty: self.fn_ptr_type(&sig),
            op: format!("@{}", name),
        })
    }

    fn generate_pending_closure(&mut self, closure: &PendingClosure) -> Result<(), String> {
        let ret_ty = self.type_to_llvm(&closure.return_type)?;
        self.current_fn_ret = Some(ret_ty.clone());

        let mut params_sig = Vec::with_capacity(closure.parameters.len());
        for (pname, pty, _) in &closure.parameters {
            params_sig.push(format!(
                "{} %{}",
                self.type_to_llvm(pty)?,
                self.sanitize_ident(pname)
            ));
        }

        self.out.push_str(&format!(
            "define {} @{}({}) {{\n",
            ret_ty,
            closure.name,
            params_sig.join(", ")
        ));
        self.out.push_str("entry:\n");
        self.block_terminated = false;

        self.push_scope();

        if let (Some(env_type), Some(env_slot)) = (&closure.env_type, &closure.env_slot) {
            for (index, capture) in closure.captures.iter().enumerate() {
                let gep = self.next_temp();
                self.out.push_str(&format!(
                    "  {} = getelementptr inbounds {}, {}* {}, i32 0, i32 {}\n",
                    gep, env_type, env_type, env_slot, index
                ));
                let loaded = self.next_temp();
                self.out.push_str(&format!(
                    "  {} = load {}, {}* {}, align 8\n",
                    loaded, capture.llvm_ty, capture.llvm_ty, gep
                ));
                let ptr = format!(
                    "%{}.captured.{}",
                    self.sanitize_ident(&capture.name),
                    self.next_temp_id()
                );
                self.out.push_str(&format!(
                    "  {} = alloca {}, align 8\n",
                    ptr, capture.llvm_ty
                ));
                self.out.push_str(&format!(
                    "  store {} {}, {}* {}, align 8\n",
                    capture.llvm_ty, loaded, capture.llvm_ty, ptr
                ));
                self.insert_var(&capture.name, capture.llvm_ty.clone(), ptr);
            }
        }

        for (pname, pty, _) in &closure.parameters {
            let pty_llvm = self.type_to_llvm(pty)?;
            let clean = self.sanitize_ident(pname);
            let ptr = format!("%{}.addr", clean);
            self.out
                .push_str(&format!("  {} = alloca {}, align 8\n", ptr, pty_llvm));
            self.out.push_str(&format!(
                "  store {} %{}, {}* {}, align 8\n",
                pty_llvm, clean, pty_llvm, ptr
            ));
            self.insert_var(pname, pty_llvm, ptr);
            if let Some(sig) = self.function_sig_from_type(pty)? {
                self.insert_fn_value(pname, sig);
            }
        }

        self.generate_block(&closure.body)?;
        if !self.current_block_has_terminator() {
            self.emit_default_return()?;
        }

        self.pop_scope();
        self.current_fn_ret = None;
        self.out.push_str("}\n");
        Ok(())
    }

    fn next_closure_id(&mut self) -> u64 {
        self.closure_counter += 1;
        self.closure_counter
    }

    fn collect_closure_captures(
        &self,
        parameters: &[(String, Type, crate::ast::Span)],
        body: &BlockStatement,
    ) -> Result<Vec<CapturedVar>, String> {
        let mut locals = vec![HashSet::new()];
        for (name, _, _) in parameters {
            if let Some(scope) = locals.last_mut() {
                scope.insert(name.clone());
            }
        }
        let mut seen = HashSet::new();
        let mut captures = Vec::new();
        self.collect_capture_block(body, &mut locals, &mut seen, &mut captures)?;
        Ok(captures)
    }

    fn collect_capture_block(
        &self,
        block: &BlockStatement,
        locals: &mut Vec<HashSet<String>>,
        seen: &mut HashSet<String>,
        captures: &mut Vec<CapturedVar>,
    ) -> Result<(), String> {
        locals.push(HashSet::new());
        for stmt in &block.statements {
            self.collect_capture_stmt(stmt, locals, seen, captures)?;
        }
        locals.pop();
        Ok(())
    }

    fn collect_capture_stmt(
        &self,
        stmt: &Statement,
        locals: &mut Vec<HashSet<String>>,
        seen: &mut HashSet<String>,
        captures: &mut Vec<CapturedVar>,
    ) -> Result<(), String> {
        match stmt {
            Statement::Let { name, value, .. } => {
                self.collect_capture_expr(value, locals, seen, captures)?;
                if let Some(scope) = locals.last_mut() {
                    scope.insert(name.clone());
                }
            }
            Statement::LetDestructured { names, value } => {
                self.collect_capture_expr(value, locals, seen, captures)?;
                if let Some(scope) = locals.last_mut() {
                    for (name, _, _) in names {
                        scope.insert(name.clone());
                    }
                }
            }
            Statement::Assign { name, value, .. } => {
                self.collect_capture_name(name, locals, seen, captures)?;
                self.collect_capture_expr(value, locals, seen, captures)?;
            }
            Statement::FieldAssign { object, value, .. } => {
                self.collect_capture_name(object, locals, seen, captures)?;
                self.collect_capture_expr(value, locals, seen, captures)?;
            }
            Statement::IndexAssign {
                object,
                index,
                value,
                ..
            } => {
                self.collect_capture_expr(object, locals, seen, captures)?;
                self.collect_capture_expr(index, locals, seen, captures)?;
                self.collect_capture_expr(value, locals, seen, captures)?;
            }
            Statement::If {
                condition,
                consequence,
                alternative,
            } => {
                self.collect_capture_expr(condition, locals, seen, captures)?;
                self.collect_capture_block(consequence, locals, seen, captures)?;
                if let Some(alt) = alternative {
                    match alt {
                        IfAlternative::Else(block) => {
                            self.collect_capture_block(block, locals, seen, captures)?;
                        }
                        IfAlternative::ElseIf(stmt) => {
                            self.collect_capture_stmt(stmt, locals, seen, captures)?;
                        }
                    }
                }
            }
            Statement::While { condition, body } => {
                self.collect_capture_expr(condition, locals, seen, captures)?;
                self.collect_capture_block(body, locals, seen, captures)?;
            }
            Statement::ForClassic {
                init,
                condition,
                post,
                body,
            } => {
                locals.push(HashSet::new());
                if let Some(init) = init {
                    self.collect_capture_stmt(init, locals, seen, captures)?;
                }
                if let Some(condition) = condition {
                    self.collect_capture_expr(condition, locals, seen, captures)?;
                }
                if let Some(post) = post {
                    self.collect_capture_stmt(post, locals, seen, captures)?;
                }
                self.collect_capture_block(body, locals, seen, captures)?;
                locals.pop();
            }
            Statement::ForIn {
                index,
                iterator,
                collection,
                body,
                ..
            } => {
                self.collect_capture_expr(collection, locals, seen, captures)?;
                locals.push(HashSet::new());
                if let Some(scope) = locals.last_mut() {
                    scope.insert(iterator.clone());
                    if let Some(index) = index {
                        scope.insert(index.clone());
                    }
                }
                self.collect_capture_block(body, locals, seen, captures)?;
                locals.pop();
            }
            Statement::Return { value }
            | Statement::Print { value }
            | Statement::ExpressionStatement(value) => {
                self.collect_capture_expr(value, locals, seen, captures)?;
            }
            Statement::Break
            | Statement::Continue
            | Statement::PackageDecl { .. }
            | Statement::Import { .. }
            | Statement::StructDecl { .. }
            | Statement::MethodDecl { .. }
            | Statement::InterfaceDecl { .. }
            | Statement::Export { .. }
            | Statement::TestDecl { .. }
            | Statement::Go { .. }
            | Statement::Defer { .. }
            | Statement::FunctionDecl { .. } => {}
        }
        Ok(())
    }

    fn collect_capture_expr(
        &self,
        expr: &Expression,
        locals: &mut Vec<HashSet<String>>,
        seen: &mut HashSet<String>,
        captures: &mut Vec<CapturedVar>,
    ) -> Result<(), String> {
        match expr {
            Expression::Identifier(name) => self.collect_capture_name(name, locals, seen, captures),
            Expression::Binary { left, right, .. } => {
                self.collect_capture_expr(left, locals, seen, captures)?;
                self.collect_capture_expr(right, locals, seen, captures)
            }
            Expression::FunctionCall {
                function,
                arguments,
            } => {
                self.collect_capture_name(function, locals, seen, captures)?;
                for arg in arguments {
                    self.collect_capture_expr(arg, locals, seen, captures)?;
                }
                Ok(())
            }
            Expression::ArrayLiteral { elements } => {
                for elem in elements {
                    self.collect_capture_expr(elem, locals, seen, captures)?;
                }
                Ok(())
            }
            Expression::HashLiteral { pairs } => {
                for (key, value) in pairs {
                    self.collect_capture_expr(key, locals, seen, captures)?;
                    self.collect_capture_expr(value, locals, seen, captures)?;
                }
                Ok(())
            }
            Expression::IndexExpression { left, index } => {
                self.collect_capture_expr(left, locals, seen, captures)?;
                self.collect_capture_expr(index, locals, seen, captures)
            }
            Expression::SliceExpression { left, low, high } => {
                self.collect_capture_expr(left, locals, seen, captures)?;
                if let Some(low) = low {
                    self.collect_capture_expr(low, locals, seen, captures)?;
                }
                if let Some(high) = high {
                    self.collect_capture_expr(high, locals, seen, captures)?;
                }
                Ok(())
            }
            Expression::ReadInput { prompt } => {
                self.collect_capture_expr(prompt, locals, seen, captures)
            }
            Expression::TypeConversion { argument, .. } => {
                self.collect_capture_expr(argument, locals, seen, captures)
            }
            Expression::MethodCall {
                object, arguments, ..
            } => {
                self.collect_capture_expr(object, locals, seen, captures)?;
                for arg in arguments {
                    self.collect_capture_expr(arg, locals, seen, captures)?;
                }
                Ok(())
            }
            Expression::FieldAccess { object, .. } => {
                self.collect_capture_expr(object, locals, seen, captures)
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.collect_capture_expr(value, locals, seen, captures)?;
                }
                Ok(())
            }
            Expression::ErrorCreate { message } => {
                self.collect_capture_expr(message, locals, seen, captures)
            }
            Expression::TupleLiteral { elements } => {
                for elem in elements {
                    self.collect_capture_expr(elem, locals, seen, captures)?;
                }
                Ok(())
            }
            Expression::ClosureLiteral { .. }
            | Expression::IntegerLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NilLiteral
            | Expression::ChannelMake { .. }
            | Expression::BaungCreate { .. } => Ok(()),
        }
    }

    fn collect_capture_name(
        &self,
        name: &str,
        locals: &[HashSet<String>],
        seen: &mut HashSet<String>,
        captures: &mut Vec<CapturedVar>,
    ) -> Result<(), String> {
        if locals.iter().rev().any(|scope| scope.contains(name)) {
            return Ok(());
        }
        if self.function_registry.contains_key(name) {
            return Ok(());
        }
        if let Some((llvm_ty, _)) = self.lookup_var(name) {
            if seen.insert(name.to_string()) {
                captures.push(CapturedVar {
                    name: name.to_string(),
                    llvm_ty,
                });
            }
        }
        Ok(())
    }
}
