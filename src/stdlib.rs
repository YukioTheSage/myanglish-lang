use crate::ast::Type;

#[derive(Debug, Clone)]
pub struct StdlibStruct {
    pub name: &'static str,
    pub fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone)]
pub struct StdlibFunction {
    pub name: &'static str,
    pub params: Vec<Type>,
    pub return_type: Type,
}

#[derive(Debug, Clone)]
pub struct StdlibModule {
    pub mlang_name: &'static str,
    pub alias: &'static str,
    pub go_imports: Vec<&'static str>,
    pub structs: Vec<StdlibStruct>,
    pub functions: Vec<StdlibFunction>,
}

pub fn resolve_stdlib_module(import_name: &str) -> Option<StdlibModule> {
    let normalized = import_name.trim_matches('"');
    match normalized {
        "kainn/http" => Some(http_module()),
        "json" => Some(json_module()),
        "file" => Some(file_module()),
        "su_nit" => Some(su_nit_module()),
        _ => None,
    }
}

fn http_module() -> StdlibModule {
    StdlibModule {
        mlang_name: "kainn/http",
        alias: "http",
        go_imports: vec!["net/http", "io", "strings"],
        structs: vec![StdlibStruct {
            name: "http.Response",
            fields: vec![
                ("status".to_string(), Type::Kain),
                ("body".to_string(), Type::Sar),
                (
                    "headers".to_string(),
                    Type::Map(Box::new(Type::Sar), Box::new(Type::Sar)),
                ),
            ],
        }],
        functions: vec![
            StdlibFunction {
                name: "get",
                params: vec![Type::Sar],
                return_type: Type::Tuple(vec![
                    Type::Struct("http.Response".to_string()),
                    Type::Error,
                ]),
            },
            StdlibFunction {
                name: "post",
                params: vec![Type::Sar, Type::Sar],
                return_type: Type::Tuple(vec![
                    Type::Struct("http.Response".to_string()),
                    Type::Error,
                ]),
            },
        ],
    }
}

fn json_module() -> StdlibModule {
    StdlibModule {
        mlang_name: "json",
        alias: "json",
        go_imports: vec!["encoding/json", "fmt"],
        structs: vec![],
        functions: vec![
            StdlibFunction {
                name: "encode",
                params: vec![Type::Map(Box::new(Type::Sar), Box::new(Type::Kain))],
                return_type: Type::Tuple(vec![Type::Sar, Type::Error]),
            },
            StdlibFunction {
                name: "decode",
                params: vec![Type::Sar],
                return_type: Type::Tuple(vec![
                    Type::Map(Box::new(Type::Sar), Box::new(Type::Kain)),
                    Type::Error,
                ]),
            },
        ],
    }
}

fn file_module() -> StdlibModule {
    StdlibModule {
        mlang_name: "file",
        alias: "file",
        go_imports: vec!["os"],
        structs: vec![],
        functions: vec![
            StdlibFunction {
                name: "read",
                params: vec![Type::Sar],
                return_type: Type::Tuple(vec![Type::Sar, Type::Error]),
            },
            StdlibFunction {
                name: "write",
                params: vec![Type::Sar, Type::Sar],
                return_type: Type::Error,
            },
        ],
    }
}

fn su_nit_module() -> StdlibModule {
    StdlibModule {
        mlang_name: "su_nit",
        alias: "su_nit",
        go_imports: vec!["os"],
        structs: vec![],
        functions: vec![
            StdlibFunction {
                name: "env",
                params: vec![Type::Sar],
                return_type: Type::Sar,
            },
            StdlibFunction {
                name: "args",
                params: vec![],
                return_type: Type::Array(Box::new(Type::Sar)),
            },
        ],
    }
}
