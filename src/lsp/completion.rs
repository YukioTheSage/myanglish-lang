use ropey::Rope;
use tower_lsp::lsp_types::*;

use super::analysis::{AnalysisResult, SymbolKindInfo, format_type};

/// All M-Lang keywords with their descriptions.
fn keyword_completions() -> Vec<CompletionItem> {
    vec![
        make_keyword("kain", "Integer type (int)", "kain ${1:name} = ${2:0};"),
        make_keyword("sar", "String type (string)", "sar ${1:name} = \"${2}\";"),
        make_keyword("sit", "Boolean type (bool)", "sit ${1:name} = ${2:hman};"),
        make_keyword("hman", "Boolean true", "hman"),
        make_keyword("hmar", "Boolean false", "hmar"),
        make_keyword(
            "hlyin",
            "If statement",
            "hlyin (${1:condition}) {\n\t${2}\n}",
        ),
        make_keyword("mo", "Else clause", "mo {\n\t${1}\n}"),
        make_keyword(
            "pat",
            "Loop statement (while / for-in)",
            "pat (${1:condition}) {\n\t${2}\n}",
        ),
        make_keyword("kyoe", "Spawn concurrent call", "kyoe ${1:fn_call}();"),
        make_keyword(
            "naut_sone",
            "Defer cleanup call",
            "naut_sone ${1:fn_call}();",
        ),
        make_keyword(
            "set_sae",
            "Test declaration",
            "set_sae ${1:test_name} {\n\t${2:pyan bhala;}\n}",
        ),
        make_keyword("htae", "For-in connector", "htae"),
        CompletionItem {
            label: "pat htae (for-in loop)".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("For-in loop template".to_string()),
            insert_text: Some(
                "pat ${1:item} htae ${2:numbers} {\n\tpya(${1:item});\n}".to_string(),
            ),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        make_keyword(
            "loke",
            "Function declaration",
            "loke ${1:name}(${2}) -> ${3:kain} {\n\t${4}\n}",
        ),
        make_keyword("pyan", "Return statement", "pyan ${1:0};"),
        make_keyword("pya", "Print statement", "pya(${1});"),
        make_keyword("phat", "Read input", "phat(\"${1:prompt}\")"),
        make_keyword("su", "Array type", "su<${1:kain}> ${2:name} = [${3}];"),
        make_keyword(
            "laung",
            "Channel type/make",
            "laung<${1:kain}> ${2:ch} = laung<${1:kain}>();",
        ),
        make_keyword(
            "baung",
            "Context type/make",
            "baung ${1:ctx} = baung(${2:5000});",
        ),
        make_keyword("yu", "Import module", "yu \"${1:module}\";"),
        make_keyword("atote", "Package declaration", "atote ${1:main};"),
        make_keyword("pay", "Export declaration", "pay ${1:declaration}"),
        make_keyword(
            "twe",
            "HashMap type",
            "twe<${1:sar}, ${2:kain}> ${3:name} = {${4}};",
        ),
        // Common snippet: main function
        CompletionItem {
            label: "main (main function)".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Main function template".to_string()),
            insert_text: Some("loke main() -> kain {\n\t${1}\n\tpyan 0;\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

fn make_keyword(label: &str, detail: &str, snippet: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        detail: Some(detail.to_string()),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    }
}

pub fn get_completions(
    _rope: Option<&Rope>,
    _pos: Position,
    analysis: Option<&AnalysisResult>,
) -> Vec<CompletionItem> {
    let mut items = keyword_completions();

    // Add symbols from the analysis as completions
    if let Some(analysis) = analysis {
        for sym in &analysis.symbols {
            let (kind, detail) = match sym.kind {
                SymbolKindInfo::Variable => {
                    let type_str = sym
                        .ty
                        .as_ref()
                        .map(|t| format_type(t))
                        .unwrap_or_else(|| "unknown".to_string());
                    (
                        CompletionItemKind::VARIABLE,
                        format!("variable: {}", type_str),
                    )
                }
                SymbolKindInfo::Function => {
                    let params: Vec<String> = sym
                        .parameters
                        .iter()
                        .map(|(n, t)| format!("{} {}", format_type(t), n))
                        .collect();
                    let ret = sym
                        .ty
                        .as_ref()
                        .map(|t| format_type(t))
                        .unwrap_or_else(|| "unknown".to_string());
                    (
                        CompletionItemKind::FUNCTION,
                        format!("fn({}) -> {}", params.join(", "), ret),
                    )
                }
                SymbolKindInfo::Parameter => {
                    let type_str = sym
                        .ty
                        .as_ref()
                        .map(|t| format_type(t))
                        .unwrap_or_else(|| "unknown".to_string());
                    (CompletionItemKind::VARIABLE, format!("param: {}", type_str))
                }
                SymbolKindInfo::Struct => {
                    let fields: Vec<String> = sym
                        .parameters
                        .iter()
                        .map(|(n, t)| format!("{} {}", format_type(t), n))
                        .collect();
                    (
                        CompletionItemKind::STRUCT,
                        format!("struct {{ {} }}", fields.join(", ")),
                    )
                }
                SymbolKindInfo::Method => {
                    let params: Vec<String> = sym
                        .parameters
                        .iter()
                        .map(|(n, t)| format!("{} {}", format_type(t), n))
                        .collect();
                    let ret = sym
                        .ty
                        .as_ref()
                        .map(|t| format_type(t))
                        .unwrap_or_else(|| "unknown".to_string());
                    (
                        CompletionItemKind::METHOD,
                        format!("method({}) -> {}", params.join(", "), ret),
                    )
                }
                SymbolKindInfo::Interface => {
                    (CompletionItemKind::INTERFACE, "interface".to_string())
                }
            };

            items.push(CompletionItem {
                label: sym.name.clone(),
                kind: Some(kind),
                detail: Some(detail),
                ..Default::default()
            });
        }
    }

    items
}
