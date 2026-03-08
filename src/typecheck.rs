use std::collections::HashMap;
use crate::ast::{BlockStatement, Expression, IfAlternative, Program, Statement, Type};
use crate::stdlib::resolve_stdlib_module;

#[derive(Clone)]
pub struct Symbol {
    pub ty: Type,
    pub is_function: bool,
    pub parameters: Vec<Type>, // Only used if is_function = true
}

#[derive(Clone)]
pub struct Environment {
    pub store: HashMap<String, Symbol>,
    pub outer: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn new_enclosed(outer: Environment) -> Self {
        Environment {
            store: HashMap::new(),
            outer: Some(Box::new(outer)),
        }
    }

    pub fn get(&self, name: &str) -> Option<Symbol> {
        match self.store.get(name) {
            Some(symbol) => Some(symbol.clone()),
            None => match &self.outer {
                Some(outer) => outer.get(name),
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: String, symbol: Symbol) {
        self.store.insert(name, symbol);
    }
}

#[derive(Debug, Clone)]
pub struct TypeCheckError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}] {}", self.line, self.column, self.message)
    }
}

pub struct TypeChecker {
    pub errors: Vec<TypeCheckError>,
    /// Registry of struct types: struct_name -> Vec<(field_name, field_type)>
    pub struct_registry: HashMap<String, Vec<(String, Type)>>,
    /// Registry of methods: (type_name, method_name) -> (param_types, return_type)
    pub method_registry: HashMap<(String, String), (Vec<Type>, Type)>,
    /// Registry of interfaces: interface_name -> Vec<(method_name, param_types, return_type)>
    pub interface_registry: HashMap<String, Vec<(String, Vec<Type>, Type)>>,
    loop_depth: usize,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            errors: Vec::new(),
            struct_registry: HashMap::new(),
            method_registry: HashMap::new(),
            interface_registry: HashMap::new(),
            loop_depth: 0,
        }
    }

    fn is_assignable(&self, expected: &Type, actual: &Type) -> bool {
        if actual == &Type::Nil {
            return true;
        }
        match (expected, actual) {
            (Type::Tuple(expected_items), Type::Tuple(actual_items)) => {
                expected_items.len() == actual_items.len()
                    && expected_items
                        .iter()
                        .zip(actual_items.iter())
                        .all(|(e, a)| self.is_assignable(e, a))
            }
            (
                Type::Function {
                    params: expected_params,
                    return_type: expected_ret,
                },
                Type::Function {
                    params: actual_params,
                    return_type: actual_ret,
                },
            ) => {
                expected_params.len() == actual_params.len()
                    && expected_params
                        .iter()
                        .zip(actual_params.iter())
                        .all(|(e, a)| self.is_assignable(e, a))
                    && self.is_assignable(expected_ret, actual_ret)
            }
            _ => expected == actual,
        }
    }

    fn push_error(&mut self, message: String) {
        self.errors.push(TypeCheckError {
            message,
            line: 0,
            column: 0,
        });
    }

    pub fn check_program(&mut self, program: &Program, env: &mut Environment) {
        // First pass: register all structs, methods, interfaces, and top-level functions
        // so they can be referenced before their declaration (forward references)
        for stmt in &program.statements {
            match stmt {
                Statement::StructDecl { name, fields, .. } => {
                    self.struct_registry.insert(name.clone(), fields.clone());
                }
                Statement::MethodDecl { receiver_type, name, parameters, return_type, .. } => {
                    let param_types: Vec<Type> = parameters.iter().map(|(_, ty, _)| ty.clone()).collect();
                    self.method_registry.insert(
                        (receiver_type.clone(), name.clone()),
                        (param_types, return_type.clone()),
                    );
                }
                Statement::InterfaceDecl { name, methods, .. } => {
                    let method_sigs: Vec<(String, Vec<Type>, Type)> = methods.iter().map(|(mname, params, ret)| {
                        let ptypes: Vec<Type> = params.iter().map(|(_, ty)| ty.clone()).collect();
                        (mname.clone(), ptypes, ret.clone())
                    }).collect();
                    self.interface_registry.insert(name.clone(), method_sigs);
                }
                Statement::FunctionDecl { name, parameters, return_type, .. } => {
                    let param_types: Vec<Type> = parameters.iter().map(|(_, ty, _)| ty.clone()).collect();
                    env.set(name.clone(), Symbol {
                        ty: Type::Function {
                            params: param_types.clone(),
                            return_type: Box::new(return_type.clone()),
                        },
                        is_function: true,
                        parameters: param_types,
                    });
                }
                Statement::Import { module, .. } => {
                    self.register_stdlib_import(module, env);
                }
                _ => {}
            }
        }

        // Second pass: full type checking
        for stmt in &program.statements {
            self.check_statement(stmt, env);
        }
    }

    fn check_statement(&mut self, stmt: &Statement, env: &mut Environment) {
        match stmt {
            Statement::Let { name, value, ty, .. } => {
                let value_type = self.check_expression(value, env);
                if let Some(vt) = value_type {
                    if !self.is_assignable(ty, &vt) {
                        self.push_error(format!("Type mismatch: cannot assign `{:?}` to variable `{}` of type `{:?}`", vt, name, ty));
                    } else {
                        env.set(name.clone(), Symbol {
                            ty: ty.clone(),
                            is_function: false,
                            parameters: vec![],
                        });
                    }
                }
            }
            Statement::LetDestructured { names, value } => {
                let value_type = self.check_expression(value, env);
                if let Some(Type::Tuple(types)) = value_type {
                    if types.len() != names.len() {
                        self.push_error(format!("Destructuring mismatch: expected {} values, got {}", names.len(), types.len()));
                    } else {
                        for (i, (name, ty, _)) in names.iter().enumerate() {
                            if !self.is_assignable(ty, &types[i]) {
                                self.push_error(format!("Type mismatch in destructuring: expected {:?} for `{}`, got {:?}", ty, name, types[i]));
                            }
                            env.set(name.clone(), Symbol {
                                ty: ty.clone(),
                                is_function: false,
                                parameters: vec![],
                            });
                        }
                    }
                }
            }
            Statement::Assign { name, value, .. } => {
                let var_symbol = env.get(name);
                if let Some(sym) = var_symbol {
                    let value_type = self.check_expression(value, env);
                    if let Some(vt) = value_type {
                        if !self.is_assignable(&sym.ty, &vt) {
                            self.push_error(format!("Type mismatch: cannot assign `{:?}` to variable `{}` of type `{:?}`", vt, name, sym.ty));
                        }
                    }
                } else {
                    self.push_error(format!("Undeclared variable `{}`", name));
                }
            }
            Statement::FieldAssign {
                object,
                field,
                value,
                ..
            } => {
                let Some(object_symbol) = env.get(object) else {
                    self.push_error(format!("Undeclared variable `{}`", object));
                    return;
                };

                let Type::Struct(struct_name) = &object_symbol.ty else {
                    self.push_error(format!(
                        "Cannot assign field `{}` on non-struct variable `{}`",
                        field, object
                    ));
                    return;
                };

                let Some(fields) = self.struct_registry.get(struct_name).cloned() else {
                    self.push_error(format!("Unknown struct type `{}`", struct_name));
                    return;
                };

                let Some((_, field_ty)) = fields.iter().find(|(fname, _)| fname == field) else {
                    self.push_error(format!(
                        "Struct `{}` has no field `{}`",
                        struct_name, field
                    ));
                    return;
                };

                let value_type = self.check_expression(value, env);
                if let Some(vt) = value_type {
                    if !self.is_assignable(field_ty, &vt) {
                        self.push_error(format!(
                            "Type mismatch: cannot assign `{:?}` to field `{}.{}` of type `{:?}`",
                            vt, object, field, field_ty
                        ));
                    }
                }
            }
            Statement::IndexAssign {
                object,
                index,
                value,
                ..
            } => {
                let Some(object_ty) = self.check_expression(object, env) else {
                    return;
                };
                let Some(index_ty) = self.check_expression(index, env) else {
                    return;
                };
                let Some(value_ty) = self.check_expression(value, env) else {
                    return;
                };

                match object_ty {
                    Type::Array(inner) => {
                        if index_ty != Type::Kain {
                            self.push_error("Array index assignment requires kain index".to_string());
                        }
                        if !self.is_assignable(&inner, &value_ty) {
                            self.push_error(format!(
                                "Array assignment type mismatch: expected `{:?}`, got `{:?}`",
                                inner, value_ty
                            ));
                        }
                    }
                    Type::Map(key_ty, val_ty) => {
                        if index_ty != *key_ty {
                            self.push_error(format!(
                                "Map assignment key type mismatch: expected `{:?}`, got `{:?}`",
                                key_ty, index_ty
                            ));
                        }
                        if !self.is_assignable(&val_ty, &value_ty) {
                            self.push_error(format!(
                                "Map assignment value type mismatch: expected `{:?}`, got `{:?}`",
                                val_ty, value_ty
                            ));
                        }
                    }
                    other => {
                        self.push_error(format!(
                            "Index assignment requires array or map target, got `{:?}`",
                            other
                        ));
                    }
                }
            }
            Statement::FunctionDecl { name, parameters, return_type, body, .. } => {
                let mut param_types = Vec::new();
                for (_, ty, _) in parameters {
                    param_types.push(ty.clone());
                }
                
                env.set(name.clone(), Symbol {
                    ty: Type::Function {
                        params: param_types.clone(),
                        return_type: Box::new(return_type.clone()),
                    },
                    is_function: true,
                    parameters: param_types,
                });

                let mut enclosed_env = Environment::new_enclosed(env.clone());
                
                for (param_name, param_type, _) in parameters {
                    enclosed_env.set(param_name.clone(), Symbol {
                        ty: param_type.clone(),
                        is_function: false,
                        parameters: vec![],
                    });
                }
                
                self.check_block_statement(body, &mut enclosed_env, Some(return_type));
            }
            Statement::Return { value } => {
                self.check_expression(value, env);
            }
            Statement::Print { value } => {
                self.check_expression(value, env);
            }
            Statement::If { condition, consequence, alternative } => {
                let cond_ty = self.check_expression(condition, env);
                if cond_ty != Some(Type::Sit) {
                    self.push_error(format!("If condition must be a boolean (sit), got {:?}", cond_ty));
                }
                self.check_block_statement(consequence, env, None);
                if let Some(alt) = alternative {
                    match alt {
                        IfAlternative::Else(block) => {
                            self.check_block_statement(block, env, None);
                        }
                        IfAlternative::ElseIf(elif_stmt) => {
                            self.check_statement(elif_stmt, env);
                        }
                    }
                }
            }
            Statement::While { condition, body } => {
                let cond_ty = self.check_expression(condition, env);
                if cond_ty != Some(Type::Sit) {
                    self.push_error(format!("While condition must be a boolean (sit), got {:?}", cond_ty));
                }
                self.loop_depth += 1;
                self.check_block_statement(body, env, None);
                self.loop_depth -= 1;
            }
            Statement::Break => {
                if self.loop_depth == 0 {
                    self.push_error("`yut` can only be used inside loops".to_string());
                }
            }
            Statement::Continue => {
                if self.loop_depth == 0 {
                    self.push_error("`shar` can only be used inside loops".to_string());
                }
            }
            Statement::ForIn {
                index,
                iterator,
                collection,
                body,
                ..
            } => {
                let collection_type = self.check_expression(collection, env);
                match collection_type {
                    Some(Type::Array(inner)) => {
                        if let Some(index_name) = index {
                            env.set(index_name.clone(), Symbol {
                                ty: Type::Kain,
                                is_function: false,
                                parameters: vec![],
                            });
                        }
                        env.set(iterator.clone(), Symbol {
                            ty: (*inner).clone(),
                            is_function: false,
                            parameters: vec![],
                        });
                        self.loop_depth += 1;
                        self.check_block_statement(body, env, None);
                        self.loop_depth -= 1;
                    }
                    Some(other) => {
                        self.push_error(format!("For-in collection must be an array (su<...>), got {:?}", other));
                    }
                    None => {}
                }
            }
            Statement::ExpressionStatement(expr) => {
                self.check_expression(expr, env);
            }
            Statement::Import { module, .. } => {
                self.register_stdlib_import(module, env);
            }
            Statement::StructDecl { name, fields, .. } => {
                self.struct_registry.insert(name.clone(), fields.clone());
            }
            Statement::MethodDecl { receiver_type, receiver_name, name, parameters, return_type, body, .. } => {
                let param_types: Vec<Type> = parameters.iter().map(|(_, ty, _)| ty.clone()).collect();
                self.method_registry.insert(
                    (receiver_type.clone(), name.clone()),
                    (param_types, return_type.clone()),
                );

                let mut enclosed_env = Environment::new_enclosed(env.clone());
                enclosed_env.set(receiver_name.clone(), Symbol {
                    ty: Type::Struct(receiver_type.clone()),
                    is_function: false,
                    parameters: vec![],
                });
                for (param_name, param_type, _) in parameters {
                    enclosed_env.set(param_name.clone(), Symbol {
                        ty: param_type.clone(),
                        is_function: false,
                        parameters: vec![],
                    });
                }
                self.check_block_statement(body, &mut enclosed_env, Some(return_type));
            }
            Statement::InterfaceDecl { name, methods, .. } => {
                let method_sigs: Vec<(String, Vec<Type>, Type)> = methods.iter().map(|(mname, params, ret)| {
                    let ptypes: Vec<Type> = params.iter().map(|(_, ty)| ty.clone()).collect();
                    (mname.clone(), ptypes, ret.clone())
                }).collect();
                self.interface_registry.insert(name.clone(), method_sigs);
            }
        }
    }

    fn check_block_statement(&mut self, block: &BlockStatement, env: &mut Environment, expected_return: Option<&Type>) {
        for stmt in &block.statements {
            self.check_statement(stmt, env);
            
            if let Statement::Return { value } = stmt {
                if let Some(ret_type) = expected_return {
                    let actual_type = self.check_expression(value, env);
                    if let Some(act) = actual_type {
                        if !self.is_assignable(ret_type, &act) {
                            self.push_error(format!("Return type mismatch: expected {:?}, got {:?}", ret_type, act));
                        }
                    }
                }
            }
        }
    }

    fn register_stdlib_import(&mut self, module_name: &str, env: &mut Environment) {
        let Some(module) = resolve_stdlib_module(module_name) else {
            return;
        };

        let module_type_name = format!("__module::{}", module.alias);
        env.set(
            module.alias.to_string(),
            Symbol {
                ty: Type::Struct(module_type_name.clone()),
                is_function: false,
                parameters: vec![],
            },
        );

        for module_struct in module.structs {
            self.struct_registry
                .insert(module_struct.name.to_string(), module_struct.fields);
        }

        for module_func in module.functions {
            self.method_registry.insert(
                (module_type_name.clone(), module_func.name.to_string()),
                (module_func.params, module_func.return_type),
            );
        }
    }

    fn check_expression(&mut self, expr: &Expression, env: &mut Environment) -> Option<Type> {
        match expr {
            Expression::IntegerLiteral(_) => Some(Type::Kain),
            Expression::FloatLiteral(_) => Some(Type::DaTha),
            Expression::StringLiteral(_) => Some(Type::Sar),
            Expression::BooleanLiteral(_) => Some(Type::Sit),
            Expression::NilLiteral => Some(Type::Nil),
            Expression::Identifier(name) => {
                match env.get(name) {
                    Some(sym) => Some(sym.ty),
                    None => {
                        self.push_error(format!("Undeclared identifier `{}`", name));
                        None
                    }
                }
            }
            Expression::ArrayLiteral { elements } => {
                if elements.is_empty() {
                    return None;
                }
                let first_ty = self.check_expression(&elements[0], env)?;
                for el in elements.iter().skip(1) {
                    let ty = self.check_expression(el, env)?;
                    if ty != first_ty {
                        self.push_error(format!("Array elements must have the same type. Expected {:?}, got {:?}", first_ty, ty));
                    }
                }
                Some(Type::Array(Box::new(first_ty)))
            }
            Expression::HashLiteral { pairs } => {
                if pairs.is_empty() { return None; }
                let key_ty = self.check_expression(&pairs[0].0, env)?;
                let val_ty = self.check_expression(&pairs[0].1, env)?;
                
                for (k, v) in pairs.iter().skip(1) {
                    let k_t = self.check_expression(k, env)?;
                    let v_t = self.check_expression(v, env)?;
                    if k_t != key_ty || v_t != val_ty {
                        self.push_error(format!("HashMap elements must have consistent types."));
                    }
                }
                Some(Type::Map(Box::new(key_ty), Box::new(val_ty)))
            }
            Expression::IndexExpression { left, index } => {
                let left_ty = self.check_expression(left, env)?;
                let index_ty = self.check_expression(index, env)?;
                
                match left_ty {
                    Type::Array(inner) => {
                        if index_ty != Type::Kain {
                            self.push_error(format!("Array index must be an integer (kain)"));
                        }
                        Some(*inner)
                    }
                    Type::Map(key, val) => {
                        if index_ty != *key {
                            self.push_error(format!("Map key type mismatch. Expected {:?}", key));
                        }
                        Some(*val)
                    }
                    _ => {
                        self.push_error(format!("Cannot index into non-collection type {:?}", left_ty));
                        None
                    }
                }
            }
            Expression::SliceExpression { left, low, high } => {
                let left_ty = self.check_expression(left, env)?;
                if let Type::Array(inner) = &left_ty {
                    if let Some(l) = low {
                        let lt = self.check_expression(l, env)?;
                        if lt != Type::Kain {
                            self.push_error("Slice low index must be kain".to_string());
                        }
                    }
                    if let Some(h) = high {
                        let ht = self.check_expression(h, env)?;
                        if ht != Type::Kain {
                            self.push_error("Slice high index must be kain".to_string());
                        }
                    }
                    Some(Type::Array(inner.clone()))
                } else {
                    self.push_error(format!("Cannot slice non-array type {:?}", left_ty));
                    None
                }
            }
            Expression::ReadInput { prompt } => {
                let prompt_ty = self.check_expression(prompt, env)?;
                if prompt_ty != Type::Sar {
                    self.push_error(format!("Prompt must be a string (sar)"));
                }
                Some(Type::Sar)
            }
            Expression::TypeConversion { target_type, argument } => {
                let _arg_type = self.check_expression(argument, env)?;
                // Allow conversions between Kain, Sar, DaTha
                Some(target_type.clone())
            }
            Expression::Binary { left, operator, right } => {
                let left_ty = self.check_expression(left, env)?;
                let right_ty = self.check_expression(right, env)?;
                
                match operator.as_str() {
                    "+" | "-" | "*" | "/" => {
                        if left_ty == Type::Kain && right_ty == Type::Kain {
                            Some(Type::Kain)
                        } else if left_ty == Type::DaTha && right_ty == Type::DaTha {
                            Some(Type::DaTha)
                        } else if operator == "+" && left_ty == Type::Sar && right_ty == Type::Sar {
                            Some(Type::Sar)
                        } else {
                            self.push_error(format!("Operator `{}` requires matching numeric types or strings", operator));
                            None
                        }
                    }
                    "==" | "!=" => {
                        // Allow nil comparisons with any type
                        if left_ty == Type::Nil || right_ty == Type::Nil {
                            Some(Type::Sit)
                        } else if left_ty == right_ty {
                            Some(Type::Sit)
                        } else {
                            self.push_error(format!("Cannot compare differing types {:?} and {:?}", left_ty, right_ty));
                            None
                        }
                    }
                    ">" | "<" | ">=" | "<=" => {
                        if left_ty == Type::Kain && right_ty == Type::Kain {
                            Some(Type::Sit)
                        } else if left_ty == Type::DaTha && right_ty == Type::DaTha {
                            Some(Type::Sit)
                        } else {
                            self.push_error(format!("Operator `{}` requires two integers (kain) or two floats (da_tha)", operator));
                            None
                        }
                    }
                    _ => None,
                }
            }
            Expression::FunctionCall { function, arguments } => {
                // Check for built-in functions
                match function.as_str() {
                    "htae" => {
                        // append: htae(array, elem) -> array
                        if arguments.len() != 2 {
                            self.push_error("htae() expects 2 arguments".to_string());
                            return None;
                        }
                        let arr_ty = self.check_expression(&arguments[0], env)?;
                        let elem_ty = self.check_expression(&arguments[1], env)?;
                        if let Type::Array(inner) = &arr_ty {
                            if !self.is_assignable(inner, &elem_ty) {
                                self.push_error(format!("htae element type mismatch: expected {:?}, got {:?}", inner, elem_ty));
                            }
                            Some(arr_ty.clone())
                        } else {
                            self.push_error("htae() first argument must be an array".to_string());
                            None
                        }
                    }
                    "ashay" => {
                        // length: ashay(collection) -> kain
                        if arguments.len() != 1 {
                            self.push_error("ashay() expects 1 argument".to_string());
                            return None;
                        }
                        let arg_ty = self.check_expression(&arguments[0], env)?;
                        match arg_ty {
                            Type::Array(_) | Type::Sar | Type::Map(_, _) => Some(Type::Kain),
                            _ => {
                                self.push_error(format!("ashay() argument must be array, string, or map, got {:?}", arg_ty));
                                None
                            }
                        }
                    }
                    _ => {
                        match env.get(function) {
                            Some(sym) => {
                                let (expected_params, return_type) = match &sym.ty {
                                    Type::Function {
                                        params,
                                        return_type,
                                    } => (params.clone(), (*return_type.clone()).clone()),
                                    _ if sym.is_function => {
                                        (sym.parameters.clone(), sym.ty.clone())
                                    }
                                    _ => {
                                        self.push_error(format!("`{}` is not a function", function));
                                        return None;
                                    }
                                };

                                if expected_params.len() != arguments.len() {
                                    self.push_error(format!(
                                        "Function `{}` expects {} arguments, got {}",
                                        function,
                                        expected_params.len(),
                                        arguments.len()
                                    ));
                                } else {
                                    for (i, arg) in arguments.iter().enumerate() {
                                        let arg_ty = self.check_expression(arg, env)?;
                                        if !self.is_assignable(&expected_params[i], &arg_ty) {
                                            self.push_error(format!(
                                                "Argument {} to `{}` expected type {:?}, got {:?}",
                                                i, function, expected_params[i], arg_ty
                                            ));
                                        }
                                    }
                                }

                                Some(return_type)
                            }
                            None => {
                                self.push_error(format!("Undeclared function `{}`", function));
                                None
                            }
                        }
                    }
                }
            }
            Expression::MethodCall { object, method, arguments } => {
                let obj_ty = self.check_expression(object, env)?;
                // Built-in string methods
                if obj_ty == Type::Sar {
                    match method.as_str() {
                        "khwae" => {
                            // split: sar.khwae(separator) -> su<sar>
                            if arguments.len() != 1 {
                                self.push_error("khwae() expects 1 argument".to_string());
                                return None;
                            }
                            self.check_expression(&arguments[0], env);
                            return Some(Type::Array(Box::new(Type::Sar)));
                        }
                        "swal" => {
                            // contains: sar.swal(substring) -> sit
                            if arguments.len() != 1 {
                                self.push_error("swal() expects 1 argument".to_string());
                                return None;
                            }
                            self.check_expression(&arguments[0], env);
                            return Some(Type::Sit);
                        }
                        "ashay" => {
                            return Some(Type::Kain);
                        }
                        _ => {}
                    }
                }
                // Built-in array methods
                if let Type::Array(inner) = &obj_ty {
                    match method.as_str() {
                        "push" => {
                            if arguments.len() != 1 {
                                self.push_error("push() expects 1 argument".to_string());
                                return None;
                            }
                            let arg_ty = self.check_expression(&arguments[0], env)?;
                            if !self.is_assignable(inner, &arg_ty) {
                                self.push_error(format!(
                                    "push() type mismatch: expected `{:?}`, got `{:?}`",
                                    inner, arg_ty
                                ));
                            }
                            return Some(Type::Array(inner.clone()));
                        }
                        "remove" => {
                            if arguments.len() != 1 {
                                self.push_error("remove() expects 1 argument".to_string());
                                return None;
                            }
                            let idx_ty = self.check_expression(&arguments[0], env)?;
                            if idx_ty != Type::Kain {
                                self.push_error("remove() index must be kain".to_string());
                            }
                            return Some(Type::Array(inner.clone()));
                        }
                        "len" => {
                            if !arguments.is_empty() {
                                self.push_error("len() expects no arguments".to_string());
                            }
                            return Some(Type::Kain);
                        }
                        _ => {}
                    }
                }
                // Built-in map methods
                if let Type::Map(_, _) = &obj_ty {
                    if method == "len" {
                        if !arguments.is_empty() {
                            self.push_error("len() expects no arguments".to_string());
                        }
                        return Some(Type::Kain);
                    }
                }
                // Struct methods
                if let Type::Struct(type_name) = &obj_ty {
                    let registry_key = (type_name.clone(), method.clone());
                    if let Some((param_types, ret_type)) = self.method_registry.get(&registry_key).cloned() {
                        if param_types.len() != arguments.len() {
                            self.push_error(format!("Method `{}` expects {} arguments, got {}", method, param_types.len(), arguments.len()));
                        }
                        for (i, arg) in arguments.iter().enumerate() {
                            let Some(arg_ty) = self.check_expression(arg, env) else {
                                continue;
                            };
                            if let Some(expected) = param_types.get(i) {
                                if !self.is_assignable(expected, &arg_ty) {
                                    self.push_error(format!(
                                        "Method `{}` argument {} expected `{:?}`, got `{:?}`",
                                        method, i, expected, arg_ty
                                    ));
                                }
                            }
                        }
                        return Some(ret_type);
                    }
                }
                self.push_error(format!("Unknown method `{}` on type {:?}", method, obj_ty));
                None
            }
            Expression::FieldAccess { object, field } => {
                let obj_ty = self.check_expression(object, env)?;
                if let Type::Struct(type_name) = &obj_ty {
                    if let Some(fields) = self.struct_registry.get(type_name) {
                        for (fname, ftype) in fields {
                            if fname == field {
                                return Some(ftype.clone());
                            }
                        }
                        self.push_error(format!("Struct `{}` has no field `{}`", type_name, field));
                    } else {
                        self.push_error(format!("Unknown struct type `{}`", type_name));
                    }
                } else {
                    self.push_error(format!("Cannot access field on non-struct type {:?}", obj_ty));
                }
                None
            }
            Expression::StructLiteral { name, fields } => {
                if let Some(decl_fields) = self.struct_registry.get(name).cloned() {
                    for (fname, fexpr) in fields {
                        self.check_expression(fexpr, env);
                        let expected = decl_fields.iter().find(|(n, _)| n == fname);
                        if expected.is_none() {
                            self.push_error(format!("Struct `{}` has no field `{}`", name, fname));
                        }
                    }
                    Some(Type::Struct(name.clone()))
                } else {
                    self.push_error(format!("Unknown struct `{}`", name));
                    None
                }
            }
            Expression::ClosureLiteral {
                parameters,
                return_type,
                body,
            } => {
                let mut closure_env = Environment::new_enclosed(env.clone());
                for (param_name, param_type, _) in parameters {
                    closure_env.set(
                        param_name.clone(),
                        Symbol {
                            ty: param_type.clone(),
                            is_function: false,
                            parameters: vec![],
                        },
                    );
                }
                self.check_block_statement(body, &mut closure_env, Some(return_type));
                Some(Type::Function {
                    params: parameters.iter().map(|(_, ty, _)| ty.clone()).collect(),
                    return_type: Box::new(return_type.clone()),
                })
            }
            Expression::ErrorCreate { message } => {
                self.check_expression(message, env);
                Some(Type::Error)
            }
            Expression::TupleLiteral { elements } => {
                let types: Vec<Type> = elements.iter().filter_map(|e| self.check_expression(e, env)).collect();
                Some(Type::Tuple(types))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_type_checker_success() {
        let input = r#"
            loke main() -> kain {
                kain age = ၂၀;
                hlyin (age > ၁၈) {
                    pyan 0;
                }
                pyan 1;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut checker = TypeChecker::new();
        let mut env = Environment::new();
        checker.check_program(&program, &mut env);

        assert_eq!(checker.errors.len(), 0);
    }

    #[test]
    fn test_type_checker_type_mismatch() {
        let input = r#"
            loke main() -> kain {
                kain age = "Hello";
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();

        let mut checker = TypeChecker::new();
        let mut env = Environment::new();
        checker.check_program(&program, &mut env);

        assert_eq!(checker.errors.len(), 1);
        assert!(checker.errors[0].message.contains("Type mismatch: cannot assign `Sar` to variable `age` of type `Kain`"));
    }

    #[test]
    fn test_type_checker_for_in_success() {
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

        let mut checker = TypeChecker::new();
        let mut env = Environment::new();
        checker.check_program(&program, &mut env);

        assert!(checker.errors.is_empty(), "Type checker errors: {:?}", checker.errors);
    }

    #[test]
    fn test_type_checker_error_nil_comparison() {
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

        let mut checker = TypeChecker::new();
        let mut env = Environment::new();
        checker.check_program(&program, &mut env);

        assert!(checker.errors.is_empty(), "Type checker errors: {:?}", checker.errors);
    }

    #[test]
    fn test_type_checker_break_continue_outside_loop() {
        let input = r#"
            loke main() -> kain {
                shar;
                yut;
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        let mut checker = TypeChecker::new();
        let mut env = Environment::new();
        checker.check_program(&program, &mut env);

        assert_eq!(checker.errors.len(), 2);
        assert!(checker.errors[0].message.contains("inside loops"));
        assert!(checker.errors[1].message.contains("inside loops"));
    }

    #[test]
    fn test_type_checker_for_in_with_index() {
        let input = r#"
            loke main() -> kain {
                su<kain> numbers = [1, 2, 3];
                pat (kain i, kain item) htae numbers {
                    kain next = i + 1;
                    pya(item);
                    pya(next);
                }
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        let mut checker = TypeChecker::new();
        let mut env = Environment::new();
        checker.check_program(&program, &mut env);

        assert!(checker.errors.is_empty(), "Type checker errors: {:?}", checker.errors);
    }

    #[test]
    fn test_type_checker_closure_callback() {
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

        let mut checker = TypeChecker::new();
        let mut env = Environment::new();
        checker.check_program(&program, &mut env);

        assert!(checker.errors.is_empty(), "Type checker errors: {:?}", checker.errors);
    }

    #[test]
    fn test_type_checker_stdlib_http_usage() {
        let input = r#"
            yu "kainn/http";

            loke main() -> kain {
                http.Response res, amhar err = http.get("https://example.com");
                hlyin (err == bhala) {
                    pya(res.body);
                    pya(res.status);
                }
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        let mut checker = TypeChecker::new();
        let mut env = Environment::new();
        checker.check_program(&program, &mut env);

        assert!(
            checker.errors.is_empty(),
            "Type checker errors: {:?}",
            checker.errors
        );
    }

    #[test]
    fn test_type_checker_unknown_import_is_noop() {
        let input = r#"
            yu unknown_module;

            loke main() -> kain {
                unknown_module.call();
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        let mut checker = TypeChecker::new();
        let mut env = Environment::new();
        checker.check_program(&program, &mut env);

        assert!(!checker.errors.is_empty());
        assert!(
            checker.errors[0]
                .message
                .contains("Undeclared identifier `unknown_module`")
        );
    }

    #[test]
    fn test_type_checker_json_decode_type_lock() {
        let input = r#"
            yu "json";

            loke main() -> kain {
                twe<sar, sar> parsed, amhar err = json.decode("{}");
                pya(err);
                pya(parsed);
                pyan 0;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(parser.errors.is_empty(), "Parse errors: {:?}", parser.errors);

        let mut checker = TypeChecker::new();
        let mut env = Environment::new();
        checker.check_program(&program, &mut env);

        assert!(!checker.errors.is_empty());
        assert!(
            checker.errors
                .iter()
                .any(|e| e.message.contains("Type mismatch in destructuring"))
        );
    }
}
