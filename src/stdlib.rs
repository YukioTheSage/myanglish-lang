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
pub struct StdlibMethod {
    pub receiver: &'static str,
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
    pub methods: Vec<StdlibMethod>,
}

pub fn resolve_stdlib_module(import_name: &str) -> Option<StdlibModule> {
    let normalized = import_name.trim_matches('"');
    match normalized {
        "kainn/http" => Some(http_module()),
        "kainn" => Some(kainn_module()),
        "json" => Some(json_module()),
        "file" => Some(file_module()),
        "su_nit" => Some(su_nit_module()),
        "pone_set" => Some(pone_set_module()),
        "in_ote" => Some(in_ote_module()),
        "hmat" => Some(hmat_module()),
        "database" => Some(database_module()),
        _ => None,
    }
}

fn http_module() -> StdlibModule {
    let req_ty = Type::Struct("http.Request".to_string());
    let writer_ty = Type::Struct("http.ResponseWriter".to_string());
    let ctx_ty = Type::Baung;
    let http_handler_ty = Type::Function {
        params: vec![req_ty.clone(), writer_ty.clone()],
        return_type: Box::new(Type::Error),
    };
    let http_ctx_handler_ty = Type::Function {
        params: vec![req_ty.clone(), writer_ty.clone(), ctx_ty.clone()],
        return_type: Box::new(Type::Error),
    };
    StdlibModule {
        mlang_name: "kainn/http",
        alias: "http",
        go_imports: vec![
            "net/http",
            "io",
            "strings",
            "encoding/json",
            "net/url",
            "context",
            "time",
        ],
        structs: vec![
            StdlibStruct {
                name: "http.Response",
                fields: vec![
                    ("status".to_string(), Type::Kain),
                    ("body".to_string(), Type::Sar),
                    (
                        "headers".to_string(),
                        Type::Map(Box::new(Type::Sar), Box::new(Type::Sar)),
                    ),
                ],
            },
            StdlibStruct {
                name: "http.Request",
                fields: vec![
                    ("method".to_string(), Type::Sar),
                    ("path".to_string(), Type::Sar),
                    ("body".to_string(), Type::Sar),
                    (
                        "headers".to_string(),
                        Type::Map(Box::new(Type::Sar), Box::new(Type::Sar)),
                    ),
                ],
            },
            StdlibStruct {
                name: "http.ResponseWriter",
                fields: vec![],
            },
        ],
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
            StdlibFunction {
                name: "handle",
                params: vec![Type::Sar, http_handler_ty],
                return_type: Type::Error,
            },
            StdlibFunction {
                name: "handle_ctx",
                params: vec![Type::Sar, ctx_ty.clone(), http_ctx_handler_ty.clone()],
                return_type: Type::Error,
            },
            StdlibFunction {
                name: "handle_timeout",
                params: vec![Type::Sar, Type::Kain, http_ctx_handler_ty],
                return_type: Type::Error,
            },
            StdlibFunction {
                name: "listen",
                params: vec![Type::Sar],
                return_type: Type::Error,
            },
        ],
        methods: vec![
            StdlibMethod {
                receiver: "http.Request",
                name: "header",
                params: vec![Type::Sar],
                return_type: Type::Sar,
            },
            StdlibMethod {
                receiver: "http.Request",
                name: "query",
                params: vec![Type::Sar],
                return_type: Type::Sar,
            },
            StdlibMethod {
                receiver: "http.ResponseWriter",
                name: "write",
                params: vec![Type::Sar],
                return_type: Type::Error,
            },
            StdlibMethod {
                receiver: "http.ResponseWriter",
                name: "status",
                params: vec![Type::Kain],
                return_type: Type::Error,
            },
            StdlibMethod {
                receiver: "http.ResponseWriter",
                name: "header",
                params: vec![Type::Sar, Type::Sar],
                return_type: Type::Error,
            },
            StdlibMethod {
                receiver: "http.ResponseWriter",
                name: "json",
                params: vec![Type::Map(Box::new(Type::Sar), Box::new(Type::Kain))],
                return_type: Type::Error,
            },
        ],
    }
}

fn kainn_module() -> StdlibModule {
    let listener_ty = Type::Struct("kainn.TCPListener".to_string());
    let conn_ty = Type::Struct("kainn.TCPConn".to_string());
    let udp_ty = Type::Struct("kainn.UDPConn".to_string());
    StdlibModule {
        mlang_name: "kainn",
        alias: "kainn",
        go_imports: vec!["net", "bufio", "io", "strings"],
        structs: vec![
            StdlibStruct {
                name: "kainn.TCPListener",
                fields: vec![],
            },
            StdlibStruct {
                name: "kainn.TCPConn",
                fields: vec![],
            },
            StdlibStruct {
                name: "kainn.UDPConn",
                fields: vec![],
            },
        ],
        functions: vec![
            StdlibFunction {
                name: "tcp_listen",
                params: vec![Type::Sar],
                return_type: Type::Tuple(vec![listener_ty.clone(), Type::Error]),
            },
            StdlibFunction {
                name: "tcp_dial",
                params: vec![Type::Sar],
                return_type: Type::Tuple(vec![conn_ty.clone(), Type::Error]),
            },
            StdlibFunction {
                name: "udp_bind",
                params: vec![Type::Sar],
                return_type: Type::Tuple(vec![udp_ty.clone(), Type::Error]),
            },
        ],
        methods: vec![
            StdlibMethod {
                receiver: "kainn.TCPListener",
                name: "accept",
                params: vec![],
                return_type: Type::Tuple(vec![conn_ty.clone(), Type::Error]),
            },
            StdlibMethod {
                receiver: "kainn.TCPListener",
                name: "close",
                params: vec![],
                return_type: Type::Error,
            },
            StdlibMethod {
                receiver: "kainn.TCPConn",
                name: "read",
                params: vec![],
                return_type: Type::Tuple(vec![Type::Sar, Type::Error]),
            },
            StdlibMethod {
                receiver: "kainn.TCPConn",
                name: "write",
                params: vec![Type::Sar],
                return_type: Type::Error,
            },
            StdlibMethod {
                receiver: "kainn.TCPConn",
                name: "close",
                params: vec![],
                return_type: Type::Error,
            },
            StdlibMethod {
                receiver: "kainn.UDPConn",
                name: "recv",
                params: vec![],
                return_type: Type::Tuple(vec![Type::Sar, Type::Sar, Type::Error]),
            },
            StdlibMethod {
                receiver: "kainn.UDPConn",
                name: "send_to",
                params: vec![Type::Sar, Type::Sar],
                return_type: Type::Error,
            },
            StdlibMethod {
                receiver: "kainn.UDPConn",
                name: "close",
                params: vec![],
                return_type: Type::Error,
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
        methods: vec![],
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
        methods: vec![],
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
        methods: vec![],
    }
}

fn pone_set_module() -> StdlibModule {
    StdlibModule {
        mlang_name: "pone_set",
        alias: "pone_set",
        go_imports: vec!["fmt"],
        structs: vec![],
        functions: vec![StdlibFunction {
            name: "pon_san",
            params: vec![Type::Sar, Type::Sar],
            return_type: Type::Sar,
        }],
        methods: vec![],
    }
}

fn in_ote_module() -> StdlibModule {
    StdlibModule {
        mlang_name: "in_ote",
        alias: "in_ote",
        go_imports: vec!["bufio", "os", "io"],
        structs: vec![],
        functions: vec![
            StdlibFunction {
                name: "twin_phat",
                params: vec![],
                return_type: Type::Tuple(vec![Type::Sar, Type::Error]),
            },
            StdlibFunction {
                name: "htote_yay",
                params: vec![Type::Sar],
                return_type: Type::Error,
            },
        ],
        methods: vec![],
    }
}

fn hmat_module() -> StdlibModule {
    StdlibModule {
        mlang_name: "hmat",
        alias: "hmat",
        go_imports: vec!["log"],
        structs: vec![],
        functions: vec![
            StdlibFunction {
                name: "mhat_chet",
                params: vec![Type::Sar],
                return_type: Type::Error,
            },
            StdlibFunction {
                name: "mhat_thati",
                params: vec![Type::Sar],
                return_type: Type::Error,
            },
            StdlibFunction {
                name: "mhat_amhar",
                params: vec![Type::Sar],
                return_type: Type::Error,
            },
        ],
        methods: vec![],
    }
}

fn database_module() -> StdlibModule {
    let conn_ty = Type::Struct("database.Conn".to_string());
    let row_ty = Type::Map(Box::new(Type::Sar), Box::new(Type::Sar));
    StdlibModule {
        mlang_name: "database",
        alias: "database",
        go_imports: vec![
            "database/sql",
            "github.com/lib/pq",
            "fmt",
            "context",
            "time",
        ],
        structs: vec![StdlibStruct {
            name: "database.Conn",
            fields: vec![],
        }],
        functions: vec![StdlibFunction {
            name: "open",
            params: vec![Type::Sar],
            return_type: Type::Tuple(vec![conn_ty.clone(), Type::Error]),
        }],
        methods: vec![
            StdlibMethod {
                receiver: "database.Conn",
                name: "exec",
                params: vec![Type::Baung, Type::Sar],
                return_type: Type::Error,
            },
            StdlibMethod {
                receiver: "database.Conn",
                name: "query_one",
                params: vec![Type::Baung, Type::Sar],
                return_type: Type::Tuple(vec![row_ty.clone(), Type::Error]),
            },
            StdlibMethod {
                receiver: "database.Conn",
                name: "query_all",
                params: vec![Type::Baung, Type::Sar],
                return_type: Type::Tuple(vec![Type::Array(Box::new(row_ty)), Type::Error]),
            },
            StdlibMethod {
                receiver: "database.Conn",
                name: "close",
                params: vec![],
                return_type: Type::Error,
            },
        ],
    }
}
