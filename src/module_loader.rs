use crate::ast::{BlockStatement, Expression, Program, Statement, Type};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::stdlib::resolve_stdlib_module;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ModuleLoadError {
    pub message: String,
    pub path: Option<PathBuf>,
}

impl std::fmt::Display for ModuleLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{}: {}", path.display(), self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadedProgram {
    pub program: Program,
    pub uses_local_modules: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SymbolKind {
    Function,
    Struct,
    Interface,
    Variable,
}

#[derive(Debug, Clone)]
struct SymbolMeta {
    exported: bool,
    kind: SymbolKind,
}

#[derive(Debug, Clone)]
struct ModuleInfo {
    path: PathBuf,
    package: String,
    program: Program,
    local_imports: Vec<PathBuf>,
    symbols: HashMap<String, SymbolMeta>,
    has_package_decl: bool,
    has_export_decl: bool,
    is_entry: bool,
}

struct Ctx {
    visit_state: HashMap<PathBuf, VisitState>,
    modules: HashMap<PathBuf, ModuleInfo>,
    package_to_path: HashMap<String, PathBuf>,
    order: Vec<PathBuf>,
    errors: Vec<ModuleLoadError>,
    lock_imports: HashMap<String, PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LockFile {
    deps: Vec<LockDependency>,
}

#[derive(Debug, Deserialize)]
struct LockDependency {
    import: String,
    entry: String,
    cache_dir: String,
}

pub fn load_entry_program(entry: &Path) -> Result<LoadedProgram, Vec<ModuleLoadError>> {
    let entry = canonical(entry)?;
    let project_root = find_project_root(&entry);
    let lock_imports = load_lock_imports(&project_root);
    let mut ctx = Ctx {
        visit_state: HashMap::new(),
        modules: HashMap::new(),
        package_to_path: HashMap::new(),
        order: Vec::new(),
        errors: Vec::new(),
        lock_imports,
    };
    visit(&mut ctx, &entry, true);
    if !ctx.errors.is_empty() {
        return Err(ctx.errors);
    }

    let mut mangles = HashMap::<(String, String), String>::new();
    let mut exports = HashMap::<String, HashMap<String, SymbolMeta>>::new();
    for module in ctx.modules.values() {
        exports.insert(module.package.clone(), module.symbols.clone());
        for (name, meta) in &module.symbols {
            let keep_main = module.is_entry
                && module.package == "main"
                && name == "main"
                && meta.kind == SymbolKind::Function;
            let mangled = if keep_main {
                "main".to_string()
            } else {
                format!("{}__{}", module.package, name)
            };
            mangles.insert((module.package.clone(), name.clone()), mangled);
        }
    }

    let mut flattened = Vec::new();
    let mut uses_local_modules = ctx.modules.len() > 1;
    let mut rewrite_errors = Vec::new();

    for path in &ctx.order {
        let module = ctx.modules.get(path).expect("module exists");
        if module.has_package_decl || module.has_export_decl || !module.local_imports.is_empty() {
            uses_local_modules = true;
        }

        let import_aliases: HashMap<String, String> = module
            .local_imports
            .iter()
            .filter_map(|p| {
                ctx.modules
                    .get(p)
                    .map(|m| (m.package.clone(), m.package.clone()))
            })
            .collect();
        let local_mangles: HashMap<String, String> = module
            .symbols
            .keys()
            .filter_map(|name| {
                mangles
                    .get(&(module.package.clone(), name.clone()))
                    .map(|m| (name.clone(), m.clone()))
            })
            .collect();

        for stmt in &module.program.statements {
            match stmt {
                Statement::PackageDecl { .. } => {}
                Statement::Import { module, .. } => {
                    if resolve_stdlib_module(module).is_some() {
                        flattened.push(stmt.clone());
                    }
                }
                Statement::Export { statement, .. } => {
                    if let Some(out) = rewrite_statement(
                        statement,
                        true,
                        &module.path,
                        &local_mangles,
                        &import_aliases,
                        &exports,
                        &mangles,
                        &mut Vec::new(),
                        &mut rewrite_errors,
                    ) {
                        flattened.push(out);
                    }
                }
                _ => {
                    if let Some(out) = rewrite_statement(
                        stmt,
                        true,
                        &module.path,
                        &local_mangles,
                        &import_aliases,
                        &exports,
                        &mangles,
                        &mut Vec::new(),
                        &mut rewrite_errors,
                    ) {
                        flattened.push(out);
                    }
                }
            }
        }
    }

    if !rewrite_errors.is_empty() {
        return Err(rewrite_errors);
    }

    Ok(LoadedProgram {
        program: Program {
            statements: flattened,
        },
        uses_local_modules,
    })
}

fn visit(ctx: &mut Ctx, path: &Path, is_entry: bool) {
    let path = match canonical(path) {
        Ok(p) => p,
        Err(mut errs) => {
            ctx.errors.append(&mut errs);
            return;
        }
    };
    if matches!(ctx.visit_state.get(&path), Some(VisitState::Done)) {
        return;
    }
    if matches!(ctx.visit_state.get(&path), Some(VisitState::Visiting)) {
        ctx.errors.push(ModuleLoadError {
            message: "Import cycle detected".to_string(),
            path: Some(path),
        });
        return;
    }
    ctx.visit_state.insert(path.clone(), VisitState::Visiting);

    let source = match fs::read_to_string(&path) {
        Ok(v) => v,
        Err(e) => {
            ctx.errors.push(ModuleLoadError {
                message: format!("Failed to read module: {}", e),
                path: Some(path),
            });
            return;
        }
    };

    let mut lexer = Lexer::new(&source);
    let mut parser = Parser::new(&mut lexer);
    let Some(program) = parser.parse_program() else {
        ctx.errors.push(ModuleLoadError {
            message: "Failed to parse module".to_string(),
            path: Some(path),
        });
        return;
    };
    if !parser.errors.is_empty() {
        for e in parser.errors {
            ctx.errors.push(ModuleLoadError {
                message: format!("Syntax error: {}", e),
                path: Some(path.clone()),
            });
        }
        return;
    }

    let mut package_name = None;
    let mut has_package_decl = false;
    let mut has_export_decl = false;
    for stmt in &program.statements {
        match stmt {
            Statement::PackageDecl { name, .. } => {
                if package_name.is_some() {
                    ctx.errors.push(ModuleLoadError {
                        message: "Multiple `atote` declarations".to_string(),
                        path: Some(path.clone()),
                    });
                    return;
                }
                has_package_decl = true;
                package_name = Some(name.clone());
            }
            Statement::Export { .. } => has_export_decl = true,
            _ => {}
        }
    }
    let package = if let Some(p) = package_name {
        p
    } else if is_entry {
        "main".to_string()
    } else {
        ctx.errors.push(ModuleLoadError {
            message: "Imported module must declare `atote <name>;`".to_string(),
            path: Some(path.clone()),
        });
        return;
    };

    if let Some(existing) = ctx.package_to_path.get(&package) {
        if existing != &path {
            ctx.errors.push(ModuleLoadError {
                message: format!("Duplicate package `{}`", package),
                path: Some(path.clone()),
            });
            return;
        }
    } else {
        ctx.package_to_path.insert(package.clone(), path.clone());
    }

    let symbols = collect_symbols(&program, &path, &mut ctx.errors);
    let mut local_imports = Vec::new();
    for stmt in &program.statements {
        if let Statement::Import { module, .. } = stmt {
            if resolve_stdlib_module(module).is_some() {
                continue;
            }
            match resolve_import(&path, module, &ctx.lock_imports) {
                Ok(p) => local_imports.push(p),
                Err(msg) => ctx.errors.push(ModuleLoadError {
                    message: msg,
                    path: Some(path.clone()),
                }),
            }
        }
    }
    for imp in &local_imports {
        visit(ctx, imp, false);
    }
    if !ctx.errors.is_empty() {
        return;
    }

    ctx.modules.insert(
        path.clone(),
        ModuleInfo {
            path: path.clone(),
            package,
            program,
            local_imports,
            symbols,
            has_package_decl,
            has_export_decl,
            is_entry,
        },
    );
    ctx.visit_state.insert(path.clone(), VisitState::Done);
    ctx.order.push(path);
}

fn collect_symbols(
    program: &Program,
    path: &Path,
    errors: &mut Vec<ModuleLoadError>,
) -> HashMap<String, SymbolMeta> {
    let mut out = HashMap::new();
    for stmt in &program.statements {
        collect_symbol(stmt, false, path, &mut out, errors);
    }
    out
}

fn collect_symbol(
    stmt: &Statement,
    exported: bool,
    path: &Path,
    out: &mut HashMap<String, SymbolMeta>,
    errors: &mut Vec<ModuleLoadError>,
) {
    let mut insert = |name: &str, kind: SymbolKind, exported: bool| {
        if out.contains_key(name) {
            errors.push(ModuleLoadError {
                message: format!("Duplicate top-level symbol `{}`", name),
                path: Some(path.to_path_buf()),
            });
            return;
        }
        out.insert(name.to_string(), SymbolMeta { exported, kind });
    };

    match stmt {
        Statement::Export { statement, .. } => collect_symbol(statement, true, path, out, errors),
        Statement::FunctionDecl { name, .. } => insert(name, SymbolKind::Function, exported),
        Statement::StructDecl { name, .. } => insert(name, SymbolKind::Struct, exported),
        Statement::InterfaceDecl { name, .. } => insert(name, SymbolKind::Interface, exported),
        Statement::Let { name, .. } => insert(name, SymbolKind::Variable, exported),
        _ => {}
    }
}

fn resolve_import(
    current: &Path,
    import_name: &str,
    lock_imports: &HashMap<String, PathBuf>,
) -> Result<PathBuf, String> {
    let base = current
        .parent()
        .ok_or_else(|| "Cannot resolve import without parent".to_string())?;
    let raw = import_name.trim_matches('"');

    let is_relative_like = raw.starts_with("./") || raw.starts_with("../");
    if is_relative_like || Path::new(raw).is_absolute() {
        let mut candidate = PathBuf::from(raw);
        if candidate.extension().is_none() {
            candidate.set_extension("ml");
        }
        if candidate.is_relative() {
            candidate = base.join(candidate);
        }
        return candidate
            .canonicalize()
            .map_err(|e| format!("Cannot resolve import `{}`: {}", import_name, e));
    }

    if let Some(candidate) = lock_imports.get(raw) {
        return candidate
            .canonicalize()
            .map_err(|e| format!("Cannot resolve lock import `{}`: {}", import_name, e));
    }

    Err(format!(
        "Cannot resolve import `{}`. Use relative import (`./...`) or add dependency via `mlang get`.",
        import_name
    ))
}

fn find_project_root(entry: &Path) -> PathBuf {
    let mut dir = entry
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    loop {
        if dir.join("mlang.lock").exists() {
            return dir;
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => {
                return entry
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf()
            }
        }
    }
}

fn load_lock_imports(project_root: &Path) -> HashMap<String, PathBuf> {
    let lock_path = project_root.join("mlang.lock");
    if !lock_path.exists() {
        return HashMap::new();
    }

    let source = match fs::read_to_string(&lock_path) {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };

    let parsed: LockFile = match serde_json::from_str(&source) {
        Ok(v) => v,
        Err(_) => return HashMap::new(),
    };

    let mut out = HashMap::new();
    for dep in parsed.deps {
        let entry_path = project_root.join(dep.cache_dir).join(dep.entry);
        out.insert(dep.import, entry_path);
    }
    out
}

fn canonical(path: &Path) -> Result<PathBuf, Vec<ModuleLoadError>> {
    path.canonicalize().map_err(|e| {
        vec![ModuleLoadError {
            message: format!("Cannot find file: {}", e),
            path: Some(path.to_path_buf()),
        }]
    })
}

fn rewrite_statement(
    stmt: &Statement,
    top_level: bool,
    path: &Path,
    local_mangles: &HashMap<String, String>,
    import_aliases: &HashMap<String, String>,
    exports: &HashMap<String, HashMap<String, SymbolMeta>>,
    mangles: &HashMap<(String, String), String>,
    locals: &mut Vec<HashSet<String>>,
    errors: &mut Vec<ModuleLoadError>,
) -> Option<Statement> {
    match stmt {
        Statement::FunctionDecl {
            name,
            parameters,
            return_type,
            body,
            name_span,
        } => {
            let mut fn_locals = locals.clone();
            fn_locals.push(parameters.iter().map(|(n, _, _)| n.clone()).collect());
            Some(Statement::FunctionDecl {
                name: if top_level {
                    local_mangles
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| name.clone())
                } else {
                    name.clone()
                },
                parameters: parameters
                    .iter()
                    .map(|(n, t, s)| {
                        (
                            n.clone(),
                            rewrite_type(
                                t,
                                path,
                                local_mangles,
                                import_aliases,
                                exports,
                                mangles,
                                errors,
                            ),
                            s.clone(),
                        )
                    })
                    .collect(),
                return_type: rewrite_type(
                    return_type,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    errors,
                ),
                body: rewrite_block(
                    body,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    &mut fn_locals,
                    errors,
                ),
                name_span: name_span.clone(),
            })
        }
        Statement::TestDecl {
            name,
            body,
            name_span,
        } => Some(Statement::TestDecl {
            name: if top_level {
                local_mangles
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| name.clone())
            } else {
                name.clone()
            },
            body: rewrite_block(
                body,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                &mut locals.clone(),
                errors,
            ),
            name_span: name_span.clone(),
        }),
        Statement::StructDecl {
            name,
            fields,
            name_span,
        } => Some(Statement::StructDecl {
            name: if top_level {
                local_mangles
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| name.clone())
            } else {
                name.clone()
            },
            fields: fields
                .iter()
                .map(|(n, t)| {
                    (
                        n.clone(),
                        rewrite_type(
                            t,
                            path,
                            local_mangles,
                            import_aliases,
                            exports,
                            mangles,
                            errors,
                        ),
                    )
                })
                .collect(),
            name_span: name_span.clone(),
        }),
        Statement::InterfaceDecl {
            name,
            methods,
            name_span,
        } => Some(Statement::InterfaceDecl {
            name: if top_level {
                local_mangles
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| name.clone())
            } else {
                name.clone()
            },
            methods: methods
                .iter()
                .map(|(n, params, ret)| {
                    (
                        n.clone(),
                        params
                            .iter()
                            .map(|(pn, pt)| {
                                (
                                    pn.clone(),
                                    rewrite_type(
                                        pt,
                                        path,
                                        local_mangles,
                                        import_aliases,
                                        exports,
                                        mangles,
                                        errors,
                                    ),
                                )
                            })
                            .collect(),
                        rewrite_type(
                            ret,
                            path,
                            local_mangles,
                            import_aliases,
                            exports,
                            mangles,
                            errors,
                        ),
                    )
                })
                .collect(),
            name_span: name_span.clone(),
        }),
        Statement::Let {
            name,
            value,
            ty,
            name_span,
        } => {
            if !top_level {
                if locals.is_empty() {
                    locals.push(HashSet::new());
                }
                locals
                    .last_mut()
                    .expect("scope exists")
                    .insert(name.clone());
            }
            Some(Statement::Let {
                name: if top_level {
                    local_mangles
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| name.clone())
                } else {
                    name.clone()
                },
                value: rewrite_expr(
                    value,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    locals,
                    errors,
                ),
                ty: rewrite_type(
                    ty,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    errors,
                ),
                name_span: name_span.clone(),
            })
        }
        Statement::LetDestructured { names, value } => {
            if locals.is_empty() {
                locals.push(HashSet::new());
            }
            let rewritten_names = names
                .iter()
                .map(|(n, t, s)| {
                    if !top_level {
                        locals.last_mut().expect("scope exists").insert(n.clone());
                    }
                    (
                        if top_level {
                            local_mangles.get(n).cloned().unwrap_or_else(|| n.clone())
                        } else {
                            n.clone()
                        },
                        rewrite_type(
                            t,
                            path,
                            local_mangles,
                            import_aliases,
                            exports,
                            mangles,
                            errors,
                        ),
                        s.clone(),
                    )
                })
                .collect();
            Some(Statement::LetDestructured {
                names: rewritten_names,
                value: rewrite_expr(
                    value,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    locals,
                    errors,
                ),
            })
        }
        Statement::ForClassic {
            init,
            condition,
            post,
            body,
        } => {
            let mut loop_locals = locals.clone();
            loop_locals.push(HashSet::new());
            Some(Statement::ForClassic {
                init: init
                    .as_ref()
                    .and_then(|s| {
                        rewrite_statement(
                            s,
                            false,
                            path,
                            local_mangles,
                            import_aliases,
                            exports,
                            mangles,
                            &mut loop_locals,
                            errors,
                        )
                    })
                    .map(Box::new),
                condition: condition.as_ref().map(|e| {
                    rewrite_expr(
                        e,
                        path,
                        local_mangles,
                        import_aliases,
                        exports,
                        mangles,
                        &mut loop_locals,
                        errors,
                    )
                }),
                post: post
                    .as_ref()
                    .and_then(|s| {
                        rewrite_statement(
                            s,
                            false,
                            path,
                            local_mangles,
                            import_aliases,
                            exports,
                            mangles,
                            &mut loop_locals,
                            errors,
                        )
                    })
                    .map(Box::new),
                body: rewrite_block(
                    body,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    &mut loop_locals,
                    errors,
                ),
            })
        }
        Statement::ForIn {
            index,
            iterator,
            collection,
            body,
            name_span,
        } => {
            let mut loop_locals = locals.clone();
            loop_locals.push(HashSet::new());
            if let Some(i) = index {
                loop_locals
                    .last_mut()
                    .expect("scope exists")
                    .insert(i.clone());
            }
            loop_locals
                .last_mut()
                .expect("scope exists")
                .insert(iterator.clone());
            Some(Statement::ForIn {
                index: index.clone(),
                iterator: iterator.clone(),
                collection: rewrite_expr(
                    collection,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    locals,
                    errors,
                ),
                body: rewrite_block(
                    body,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    &mut loop_locals,
                    errors,
                ),
                name_span: name_span.clone(),
            })
        }
        Statement::While { condition, body } => Some(Statement::While {
            condition: rewrite_expr(
                condition,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            ),
            body: rewrite_block(
                body,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                &mut locals.clone(),
                errors,
            ),
        }),
        Statement::If {
            condition,
            consequence,
            alternative,
        } => Some(Statement::If {
            condition: rewrite_expr(
                condition,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            ),
            consequence: rewrite_block(
                consequence,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                &mut locals.clone(),
                errors,
            ),
            alternative: alternative.as_ref().map(|alt| match alt {
                crate::ast::IfAlternative::Else(block) => {
                    crate::ast::IfAlternative::Else(rewrite_block(
                        block,
                        path,
                        local_mangles,
                        import_aliases,
                        exports,
                        mangles,
                        &mut locals.clone(),
                        errors,
                    ))
                }
                crate::ast::IfAlternative::ElseIf(stmt) => {
                    crate::ast::IfAlternative::ElseIf(Box::new(
                        rewrite_statement(
                            stmt,
                            false,
                            path,
                            local_mangles,
                            import_aliases,
                            exports,
                            mangles,
                            &mut locals.clone(),
                            errors,
                        )
                        .unwrap_or_else(|| Statement::ExpressionStatement(Expression::NilLiteral)),
                    ))
                }
            }),
        }),
        Statement::Return { value } => Some(Statement::Return {
            value: rewrite_expr(
                value,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            ),
        }),
        Statement::Print { value } => Some(Statement::Print {
            value: rewrite_expr(
                value,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            ),
        }),
        Statement::Assign {
            name,
            value,
            name_span,
        } => Some(Statement::Assign {
            name: if is_local(locals, name) {
                name.clone()
            } else {
                local_mangles
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| name.clone())
            },
            value: rewrite_expr(
                value,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            ),
            name_span: name_span.clone(),
        }),
        Statement::ExpressionStatement(expr) => Some(Statement::ExpressionStatement(rewrite_expr(
            expr,
            path,
            local_mangles,
            import_aliases,
            exports,
            mangles,
            locals,
            errors,
        ))),
        Statement::Go { call } => Some(Statement::Go {
            call: rewrite_expr(
                call,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            ),
        }),
        Statement::Defer { call } => Some(Statement::Defer {
            call: rewrite_expr(
                call,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            ),
        }),
        Statement::FieldAssign {
            object,
            field,
            value,
            name_span,
        } => Some(Statement::FieldAssign {
            object: if is_local(locals, object) {
                object.clone()
            } else {
                local_mangles
                    .get(object)
                    .cloned()
                    .unwrap_or_else(|| object.clone())
            },
            field: field.clone(),
            value: rewrite_expr(
                value,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            ),
            name_span: name_span.clone(),
        }),
        Statement::IndexAssign {
            object,
            index,
            value,
            name_span,
        } => Some(Statement::IndexAssign {
            object: rewrite_expr(
                object,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            ),
            index: rewrite_expr(
                index,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            ),
            value: rewrite_expr(
                value,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            ),
            name_span: name_span.clone(),
        }),
        Statement::MethodDecl {
            receiver_type,
            receiver_name,
            name,
            parameters,
            return_type,
            body,
            name_span,
        } => {
            let mut method_locals = locals.clone();
            method_locals.push(HashSet::new());
            method_locals
                .last_mut()
                .expect("scope exists")
                .insert(receiver_name.clone());
            for (p, _, _) in parameters {
                method_locals
                    .last_mut()
                    .expect("scope exists")
                    .insert(p.clone());
            }
            Some(Statement::MethodDecl {
                receiver_type: rewrite_type_name(
                    receiver_type,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    errors,
                ),
                receiver_name: receiver_name.clone(),
                name: name.clone(),
                parameters: parameters
                    .iter()
                    .map(|(n, t, s)| {
                        (
                            n.clone(),
                            rewrite_type(
                                t,
                                path,
                                local_mangles,
                                import_aliases,
                                exports,
                                mangles,
                                errors,
                            ),
                            s.clone(),
                        )
                    })
                    .collect(),
                return_type: rewrite_type(
                    return_type,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    errors,
                ),
                body: rewrite_block(
                    body,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    &mut method_locals,
                    errors,
                ),
                name_span: name_span.clone(),
            })
        }
        Statement::Break | Statement::Continue => Some(stmt.clone()),
        Statement::PackageDecl { .. } => None,
        Statement::Import { .. } => Some(stmt.clone()),
        Statement::Export { statement, .. } => rewrite_statement(
            statement,
            top_level,
            path,
            local_mangles,
            import_aliases,
            exports,
            mangles,
            locals,
            errors,
        ),
    }
}

fn rewrite_block(
    block: &BlockStatement,
    path: &Path,
    local_mangles: &HashMap<String, String>,
    import_aliases: &HashMap<String, String>,
    exports: &HashMap<String, HashMap<String, SymbolMeta>>,
    mangles: &HashMap<(String, String), String>,
    locals: &mut Vec<HashSet<String>>,
    errors: &mut Vec<ModuleLoadError>,
) -> BlockStatement {
    locals.push(HashSet::new());
    let mut statements = Vec::new();
    for stmt in &block.statements {
        if let Some(s) = rewrite_statement(
            stmt,
            false,
            path,
            local_mangles,
            import_aliases,
            exports,
            mangles,
            locals,
            errors,
        ) {
            statements.push(s);
        }
    }
    locals.pop();
    BlockStatement { statements }
}

fn rewrite_type(
    ty: &Type,
    path: &Path,
    local_mangles: &HashMap<String, String>,
    import_aliases: &HashMap<String, String>,
    exports: &HashMap<String, HashMap<String, SymbolMeta>>,
    mangles: &HashMap<(String, String), String>,
    errors: &mut Vec<ModuleLoadError>,
) -> Type {
    match ty {
        Type::Struct(n) => Type::Struct(rewrite_type_name(
            n,
            path,
            local_mangles,
            import_aliases,
            exports,
            mangles,
            errors,
        )),
        Type::Array(t) => Type::Array(Box::new(rewrite_type(
            t,
            path,
            local_mangles,
            import_aliases,
            exports,
            mangles,
            errors,
        ))),
        Type::Map(k, v) => Type::Map(
            Box::new(rewrite_type(
                k,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                errors,
            )),
            Box::new(rewrite_type(
                v,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                errors,
            )),
        ),
        Type::Tuple(items) => Type::Tuple(
            items
                .iter()
                .map(|t| {
                    rewrite_type(
                        t,
                        path,
                        local_mangles,
                        import_aliases,
                        exports,
                        mangles,
                        errors,
                    )
                })
                .collect(),
        ),
        Type::Function {
            params,
            return_type,
        } => Type::Function {
            params: params
                .iter()
                .map(|p| {
                    rewrite_type(
                        p,
                        path,
                        local_mangles,
                        import_aliases,
                        exports,
                        mangles,
                        errors,
                    )
                })
                .collect(),
            return_type: Box::new(rewrite_type(
                return_type,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                errors,
            )),
        },
        Type::Channel(value) => Type::Channel(Box::new(rewrite_type(
            value,
            path,
            local_mangles,
            import_aliases,
            exports,
            mangles,
            errors,
        ))),
        _ => ty.clone(),
    }
}

fn rewrite_type_name(
    name: &str,
    path: &Path,
    local_mangles: &HashMap<String, String>,
    import_aliases: &HashMap<String, String>,
    exports: &HashMap<String, HashMap<String, SymbolMeta>>,
    mangles: &HashMap<(String, String), String>,
    errors: &mut Vec<ModuleLoadError>,
) -> String {
    if let Some((alias, sym)) = name.split_once('.') {
        if let Some(pkg) = import_aliases.get(alias) {
            if let Some(m) = resolve_imported_symbol(pkg, sym, path, exports, mangles, errors) {
                return m;
            }
        }
    }
    local_mangles
        .get(name)
        .cloned()
        .unwrap_or_else(|| name.to_string())
}

fn rewrite_expr(
    expr: &Expression,
    path: &Path,
    local_mangles: &HashMap<String, String>,
    import_aliases: &HashMap<String, String>,
    exports: &HashMap<String, HashMap<String, SymbolMeta>>,
    mangles: &HashMap<(String, String), String>,
    locals: &mut Vec<HashSet<String>>,
    errors: &mut Vec<ModuleLoadError>,
) -> Expression {
    match expr {
        Expression::Identifier(name) => {
            if is_local(locals, name) {
                Expression::Identifier(name.clone())
            } else {
                Expression::Identifier(
                    local_mangles
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| name.clone()),
                )
            }
        }
        Expression::FunctionCall {
            function,
            arguments,
        } => Expression::FunctionCall {
            function: if is_local(locals, function) {
                function.clone()
            } else {
                local_mangles
                    .get(function)
                    .cloned()
                    .unwrap_or_else(|| function.clone())
            },
            arguments: arguments
                .iter()
                .map(|a| {
                    rewrite_expr(
                        a,
                        path,
                        local_mangles,
                        import_aliases,
                        exports,
                        mangles,
                        locals,
                        errors,
                    )
                })
                .collect(),
        },
        Expression::MethodCall {
            object,
            method,
            arguments,
        } => {
            if let Expression::Identifier(alias) = object.as_ref() {
                if !is_local(locals, alias) {
                    if let Some(pkg) = import_aliases.get(alias) {
                        if let Some(m) =
                            resolve_imported_symbol(pkg, method, path, exports, mangles, errors)
                        {
                            return Expression::FunctionCall {
                                function: m,
                                arguments: arguments
                                    .iter()
                                    .map(|a| {
                                        rewrite_expr(
                                            a,
                                            path,
                                            local_mangles,
                                            import_aliases,
                                            exports,
                                            mangles,
                                            locals,
                                            errors,
                                        )
                                    })
                                    .collect(),
                            };
                        }
                    }
                }
            }
            Expression::MethodCall {
                object: Box::new(rewrite_expr(
                    object,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    locals,
                    errors,
                )),
                method: method.clone(),
                arguments: arguments
                    .iter()
                    .map(|a| {
                        rewrite_expr(
                            a,
                            path,
                            local_mangles,
                            import_aliases,
                            exports,
                            mangles,
                            locals,
                            errors,
                        )
                    })
                    .collect(),
            }
        }
        Expression::FieldAccess { object, field } => {
            if let Expression::Identifier(alias) = object.as_ref() {
                if !is_local(locals, alias) {
                    if let Some(pkg) = import_aliases.get(alias) {
                        if let Some(m) =
                            resolve_imported_symbol(pkg, field, path, exports, mangles, errors)
                        {
                            return Expression::Identifier(m);
                        }
                    }
                }
            }
            Expression::FieldAccess {
                object: Box::new(rewrite_expr(
                    object,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    locals,
                    errors,
                )),
                field: field.clone(),
            }
        }
        Expression::Binary {
            left,
            operator,
            right,
        } => Expression::Binary {
            left: Box::new(rewrite_expr(
                left,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            )),
            operator: operator.clone(),
            right: Box::new(rewrite_expr(
                right,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            )),
        },
        Expression::ArrayLiteral { elements } => Expression::ArrayLiteral {
            elements: elements
                .iter()
                .map(|e| {
                    rewrite_expr(
                        e,
                        path,
                        local_mangles,
                        import_aliases,
                        exports,
                        mangles,
                        locals,
                        errors,
                    )
                })
                .collect(),
        },
        Expression::HashLiteral { pairs } => Expression::HashLiteral {
            pairs: pairs
                .iter()
                .map(|(k, v)| {
                    (
                        rewrite_expr(
                            k,
                            path,
                            local_mangles,
                            import_aliases,
                            exports,
                            mangles,
                            locals,
                            errors,
                        ),
                        rewrite_expr(
                            v,
                            path,
                            local_mangles,
                            import_aliases,
                            exports,
                            mangles,
                            locals,
                            errors,
                        ),
                    )
                })
                .collect(),
        },
        Expression::IndexExpression { left, index } => Expression::IndexExpression {
            left: Box::new(rewrite_expr(
                left,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            )),
            index: Box::new(rewrite_expr(
                index,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            )),
        },
        Expression::SliceExpression { left, low, high } => Expression::SliceExpression {
            left: Box::new(rewrite_expr(
                left,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            )),
            low: low.as_ref().map(|e| {
                Box::new(rewrite_expr(
                    e,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    locals,
                    errors,
                ))
            }),
            high: high.as_ref().map(|e| {
                Box::new(rewrite_expr(
                    e,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    locals,
                    errors,
                ))
            }),
        },
        Expression::TypeConversion {
            target_type,
            argument,
        } => Expression::TypeConversion {
            target_type: rewrite_type(
                target_type,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                errors,
            ),
            argument: Box::new(rewrite_expr(
                argument,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            )),
        },
        Expression::StructLiteral { name, fields } => Expression::StructLiteral {
            name: local_mangles
                .get(name)
                .cloned()
                .unwrap_or_else(|| name.clone()),
            fields: fields
                .iter()
                .map(|(n, v)| {
                    (
                        n.clone(),
                        rewrite_expr(
                            v,
                            path,
                            local_mangles,
                            import_aliases,
                            exports,
                            mangles,
                            locals,
                            errors,
                        ),
                    )
                })
                .collect(),
        },
        Expression::ClosureLiteral {
            parameters,
            return_type,
            body,
        } => {
            let mut closure_locals = locals.clone();
            closure_locals.push(parameters.iter().map(|(n, _, _)| n.clone()).collect());
            Expression::ClosureLiteral {
                parameters: parameters
                    .iter()
                    .map(|(n, t, s)| {
                        (
                            n.clone(),
                            rewrite_type(
                                t,
                                path,
                                local_mangles,
                                import_aliases,
                                exports,
                                mangles,
                                errors,
                            ),
                            s.clone(),
                        )
                    })
                    .collect(),
                return_type: rewrite_type(
                    return_type,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    errors,
                ),
                body: rewrite_block(
                    body,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    &mut closure_locals,
                    errors,
                ),
            }
        }
        Expression::ReadInput { prompt } => Expression::ReadInput {
            prompt: Box::new(rewrite_expr(
                prompt,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            )),
        },
        Expression::ErrorCreate { message } => Expression::ErrorCreate {
            message: Box::new(rewrite_expr(
                message,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            )),
        },
        Expression::TupleLiteral { elements } => Expression::TupleLiteral {
            elements: elements
                .iter()
                .map(|e| {
                    rewrite_expr(
                        e,
                        path,
                        local_mangles,
                        import_aliases,
                        exports,
                        mangles,
                        locals,
                        errors,
                    )
                })
                .collect(),
        },
        Expression::ChannelMake {
            value_type,
            capacity,
        } => Expression::ChannelMake {
            value_type: Box::new(rewrite_type(
                value_type,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                errors,
            )),
            capacity: capacity.as_ref().map(|expr| {
                Box::new(rewrite_expr(
                    expr,
                    path,
                    local_mangles,
                    import_aliases,
                    exports,
                    mangles,
                    locals,
                    errors,
                ))
            }),
        },
        Expression::BaungCreate { timeout_ms } => Expression::BaungCreate {
            timeout_ms: Box::new(rewrite_expr(
                timeout_ms,
                path,
                local_mangles,
                import_aliases,
                exports,
                mangles,
                locals,
                errors,
            )),
        },
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NilLiteral => expr.clone(),
    }
}

fn resolve_imported_symbol(
    pkg: &str,
    symbol: &str,
    path: &Path,
    exports: &HashMap<String, HashMap<String, SymbolMeta>>,
    mangles: &HashMap<(String, String), String>,
    errors: &mut Vec<ModuleLoadError>,
) -> Option<String> {
    let Some(package_syms) = exports.get(pkg) else {
        errors.push(ModuleLoadError {
            message: format!("Unknown imported package `{}`", pkg),
            path: Some(path.to_path_buf()),
        });
        return None;
    };
    let Some(meta) = package_syms.get(symbol) else {
        errors.push(ModuleLoadError {
            message: format!("Package `{}` has no symbol `{}`", pkg, symbol),
            path: Some(path.to_path_buf()),
        });
        return None;
    };
    if !meta.exported {
        errors.push(ModuleLoadError {
            message: format!("Symbol `{}` is not exported from package `{}`", symbol, pkg),
            path: Some(path.to_path_buf()),
        });
        return None;
    }
    mangles.get(&(pkg.to_string(), symbol.to_string())).cloned()
}

fn is_local(scopes: &[HashSet<String>], name: &str) -> bool {
    scopes.iter().rev().any(|s| s.contains(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("mlang_module_loader_{}", nanos));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn test_module_loader_resolves_local_import_and_mangles() {
        let root = temp_root();
        let main_file = root.join("main.ml");
        let util_file = root.join("util.ml");

        fs::write(
            &util_file,
            r#"
atote util;
pay loke add(kain a, kain b) -> kain {
    pyan a + b;
}
"#,
        )
        .unwrap();
        fs::write(
            &main_file,
            r#"
yu "./util";
loke main() -> kain {
    kain x = util.add(1, 2);
    pyan x;
}
"#,
        )
        .unwrap();

        let loaded = load_entry_program(&main_file).unwrap();
        let dbg = format!("{:?}", loaded.program.statements);
        assert!(dbg.contains("util__add"));
        assert!(loaded.uses_local_modules);
    }

    #[test]
    fn test_module_loader_import_cycle_error() {
        let root = temp_root();
        let main_file = root.join("main.ml");
        let a_file = root.join("a.ml");
        let b_file = root.join("b.ml");

        fs::write(
            &a_file,
            "atote a; yu \"./b\"; pay loke fa() -> kain { pyan 0; }",
        )
        .unwrap();
        fs::write(
            &b_file,
            "atote b; yu \"./a\"; pay loke fb() -> kain { pyan 0; }",
        )
        .unwrap();
        fs::write(&main_file, "yu \"./a\"; loke main() -> kain { pyan 0; }").unwrap();

        let err = load_entry_program(&main_file).unwrap_err();
        assert!(err.iter().any(|e| e.message.contains("cycle")));
    }

    #[test]
    fn test_module_loader_resolves_lockfile_import() {
        let root = temp_root();
        let main_file = root.join("main.ml");
        let dep_dir = root.join(".mlang").join("deps").join("abc123");
        let dep_file = dep_dir.join("lib.ml");
        let lock_file = root.join("mlang.lock");

        fs::create_dir_all(&dep_dir).unwrap();
        fs::write(
            &dep_file,
            r#"
atote dep;
pay loke value() -> kain {
    pyan 42;
}
"#,
        )
        .unwrap();
        fs::write(
            &lock_file,
            r#"
{
  "version": 1,
  "deps": [
    {
      "import": "dep",
      "git": "https://example.invalid/dep.git",
      "ref": "main",
      "commit": "abc123",
      "entry": "lib.ml",
      "cache_dir": ".mlang/deps/abc123"
    }
  ]
}
"#,
        )
        .unwrap();
        fs::write(
            &main_file,
            r#"
yu "dep";
loke main() -> kain {
    pyan dep.value();
}
"#,
        )
        .unwrap();

        let loaded = load_entry_program(&main_file).unwrap();
        let dbg = format!("{:?}", loaded.program.statements);
        assert!(dbg.contains("dep__value"));
        assert!(loaded.uses_local_modules);
    }

    #[test]
    fn test_module_loader_missing_lock_import_has_actionable_error() {
        let root = temp_root();
        let main_file = root.join("main.ml");
        fs::write(
            &main_file,
            r#"
yu "dep";
loke main() -> kain {
    pyan 0;
}
"#,
        )
        .unwrap();

        let err = load_entry_program(&main_file).unwrap_err();
        assert!(err.iter().any(|e| e.message.contains("mlang get")));
    }
}
