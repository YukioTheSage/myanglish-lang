#[derive(Debug, PartialEq, Clone, Default)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Kain, // int
    Sar,  // string
    Sit,  // bool
    DaTha, // float64
    Nil,  // nil type (type of bhala)
    Error, // error type (amhar)
    Array(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Struct(String),
    Interface(String),
    Tuple(Vec<Type>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BooleanLiteral(bool),
    NilLiteral,
    Identifier(String),
    Binary {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    FunctionCall {
        function: String,
        arguments: Vec<Expression>,
    },
    ArrayLiteral {
        elements: Vec<Expression>,
    },
    HashLiteral {
        pairs: Vec<(Expression, Expression)>,
    },
    IndexExpression {
        left: Box<Expression>,
        index: Box<Expression>,
    },
    SliceExpression {
        left: Box<Expression>,
        low: Option<Box<Expression>>,
        high: Option<Box<Expression>>,
    },
    ReadInput {
        prompt: Box<Expression>,
    },
    TypeConversion {
        target_type: Type,
        argument: Box<Expression>,
    },
    MethodCall {
        object: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
    },
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },
    StructLiteral {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    ErrorCreate {
        message: Box<Expression>,
    },
    TupleLiteral {
        elements: Vec<Expression>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum IfAlternative {
    ElseIf(Box<Statement>),  // Statement::If for elif chains
    Else(BlockStatement),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Let {
        name: String,
        value: Expression,
        ty: Type,
        name_span: Span,
    },
    LetDestructured {
        names: Vec<(String, Type, Span)>,
        value: Expression,
    },
    Assign {
        name: String,
        value: Expression,
        name_span: Span,
    },
    If {
        condition: Expression,
        consequence: BlockStatement,
        alternative: Option<IfAlternative>,
    },
    While {
        condition: Expression,
        body: BlockStatement,
    },
    ForIn {
        iterator: String,
        collection: Expression,
        body: BlockStatement,
        name_span: Span,
    },
    FunctionDecl {
        name: String,
        parameters: Vec<(String, Type, Span)>, // name, type, span
        return_type: Type,
        body: BlockStatement,
        name_span: Span,
    },
    Return {
        value: Expression,
    },
    Print {
        value: Expression,
    },
    Import {
        module: String,
        name_span: Span,
    },
    ExpressionStatement(Expression),
    StructDecl {
        name: String,
        fields: Vec<(String, Type)>,
        name_span: Span,
    },
    MethodDecl {
        receiver_type: String,
        receiver_name: String,
        name: String,
        parameters: Vec<(String, Type, Span)>,
        return_type: Type,
        body: BlockStatement,
        name_span: Span,
    },
    InterfaceDecl {
        name: String,
        methods: Vec<(String, Vec<(String, Type)>, Type)>,
        name_span: Span,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}
