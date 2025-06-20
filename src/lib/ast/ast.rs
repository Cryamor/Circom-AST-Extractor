use std::ops::Range;
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub enum Pragma {
    Version(Meta, FileID, Version),
    CustomGates(Meta ,FileID),
    Unrecognized,
}

pub trait FillMeta {
    fn fill(&mut self, file_id: usize, elem_id: &mut usize);
}

pub type MainComponent = (Vec<String>, Expression);
pub fn build_main_component(public: Vec<String>, call: Expression) -> MainComponent {
    (public, call)
}

pub type Version = (usize, usize, usize);
pub type FileID = usize;
pub type FileLocation = Range<usize>;

#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone, Debug, Hash, Serialize)]
pub enum Sign {
    Minus,
    NoSign,
    Plus,
}
#[derive(Clone, Debug, Serialize)]
pub struct Num {
    pub sign: Sign,
    pub data: Vec<i128>,
}
impl Num {
    pub fn new(sign: Sign, d: Vec<i128>) -> Num {
        Num {sign, data: d}
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Meta {
    pub elem_id: usize,
    pub start: usize,
    pub end: usize,
    pub location: FileLocation,
    pub file_id: Option<usize>,
    pub component_inference: Option<String>,
    type_knowledge: TypeKnowledge,
    memory_knowledge: MemoryKnowledge,
}
impl Meta {
    pub fn new(start: usize, end: usize) -> Meta {
        Meta {
            end,
            start,
            elem_id: 0,
            location: start..end,
            file_id: Option::None,
            component_inference: None,
            type_knowledge: TypeKnowledge::default(),
            memory_knowledge: MemoryKnowledge::default(),
        }
    }
    pub fn change_location(&mut self, location: FileLocation, file_id: Option<usize>) {
        self.location = location;
        self.file_id = file_id;
    }
    pub fn get_start(&self) -> usize {
        self.location.start
    }
    pub fn get_end(&self) -> usize {
        self.location.end
    }
    pub fn get_file_id(&self) -> usize {
        if let Option::Some(id) = self.file_id {
            id
        } else {
            panic!("Empty file id accessed")
        }
    }
    pub fn get_memory_knowledge(&self) -> &MemoryKnowledge {
        &self.memory_knowledge
    }
    pub fn get_type_knowledge(&self) -> &TypeKnowledge {
        &self.type_knowledge
    }
    pub fn get_mut_memory_knowledge(&mut self) -> &mut MemoryKnowledge {
        &mut self.memory_knowledge
    }
    pub fn get_mut_type_knowledge(&mut self) -> &mut TypeKnowledge {
        &mut self.type_knowledge
    }
    pub fn file_location(&self) -> FileLocation {
        self.location.clone()
    }
    pub fn set_file_id(&mut self, file_id: usize) {
        self.file_id = Option::Some(file_id);
    }
}

#[derive(Clone, Serialize)]
pub struct AST {
    pub meta: Meta,
    pub compiler_version: Option<Version>,
    pub custom_gates: bool,
    pub custom_gates_declared: bool,
    pub includes: Vec<String>,
    pub definitions: Vec<Definition>,
    pub main_component: Option<MainComponent>,
}

impl AST {
    pub fn new(
        meta: Meta,
        pragmas: Vec<Pragma>,
        includes: Vec<String>,
        definitions: Vec<Definition>,
        main_component: Option<MainComponent>,
    ) -> AST {
        let mut custom_gates = None;
        let mut compiler_version = None;
        let custom_gates_declared = definitions.iter().any(|definition| {
            matches!(definition, Definition::Template { is_custom_gate: true, .. })
        });
        for p in pragmas {
            match p {
                Pragma::Version(location, file_id, ver) => match compiler_version {
                    // Some(_) => reports.push(produce_report(
                    //     ReportCode::MultiplePragma,location.start..location.end, file_id)),
                    Some(_) => {},
                    None => compiler_version = Some(ver),
                },
                Pragma::CustomGates(location, file_id ) => match custom_gates {
                    // Some(_) => reports.push(produce_report(
                    //     ReportCode::MultiplePragma, location.start..location.end, file_id)),
                    Some(_) => {},
                    None => custom_gates = Some(true),
                },
                Pragma::Unrecognized => {},
            }
        }

        AST {
            meta,
            compiler_version,
            custom_gates: custom_gates.unwrap_or(false),
            custom_gates_declared,
            includes,
            definitions,
            main_component,
        }
    }
}

#[derive(Clone, Serialize)]
pub enum Definition {
    Template {
        meta: Meta,
        name: String,
        args: Vec<String>,
        arg_location: FileLocation,
        body: Statement,
        parallel: bool,
        is_custom_gate: bool,
    },
    Function {
        meta: Meta,
        name: String,
        args: Vec<String>,
        arg_location: FileLocation,
        body: Statement,
    },
}

pub fn build_template(
    meta: Meta,
    name: String,
    args: Vec<String>,
    arg_location: FileLocation,
    body: Statement,
    parallel: bool,
    is_custom_gate: bool,
) -> Definition {
    Definition::Template { meta, name, args, arg_location, body, parallel, is_custom_gate }
}
pub fn build_function(
    meta: Meta,
    name: String,
    args: Vec<String>,
    arg_location: FileLocation,
    body: Statement,
) -> Definition {
    Definition::Function { meta, name, args, arg_location, body }
}

#[derive(Clone, Debug, Serialize)]
pub enum Statement {
    IfThenElse {
        meta: Meta,
        cond: Expression,
        if_case: Box<Statement>,
        else_case: Option<Box<Statement>>,
    },
    While {
        meta: Meta,
        cond: Expression,
        stmt: Box<Statement>,
    },
    Return {
        meta: Meta,
        value: Expression,
    },
    InitializationBlock {
        meta: Meta,
        xtype: VariableType,
        initializations: Vec<Statement>,
    },
    Declaration {
        meta: Meta,
        xtype: VariableType,
        name: String,
        dimensions: Vec<Expression>,
        is_constant: bool,
    },
    Substitution {
        meta: Meta,
        var: String,
        access: Vec<Access>,
        op: AssignOp,
        rhe: Expression,
    },
    MultSubstitution {
        meta: Meta,
        lhe: Expression,
        op: AssignOp,
        rhe: Expression,
    },
    UnderscoreSubstitution{
        meta: Meta,
        op: AssignOp,
        rhe: Expression,
    },
    ConstraintEquality {
        meta: Meta,
        lhe: Expression,
        rhe: Expression,
    },
    LogCall {
        meta: Meta,
        args: Vec<LogArgument>,
    },
    Block {
        meta: Meta,
        stmts: Vec<Statement>,
    },
    Assert {
        meta: Meta,
        arg: Expression,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug)]
pub enum SignalType {
    Output,
    Input,
    Intermediate,
}

pub type TagList = Vec<String>;

#[derive(Clone, PartialEq, Ord, PartialOrd, Eq, Debug, Serialize)]
pub enum VariableType {
    Var,
    Signal(SignalType, TagList),
    Component,
    AnonymousComponent,
}

#[derive(Clone, Debug, Serialize)]
pub enum Expression {
    InfixOp {
        meta: Meta,
        lhe: Box<Expression>,
        infix_op: ExpressionInfixOpcode,
        rhe: Box<Expression>,
    },
    PrefixOp {
        meta: Meta,
        prefix_op: ExpressionPrefixOpcode,
        rhe: Box<Expression>,
    },
    InlineSwitchOp {
        meta: Meta,
        cond: Box<Expression>,
        if_true: Box<Expression>,
        if_false: Box<Expression>,
    },
    ParallelOp {
        meta: Meta,
        rhe: Box<Expression>,
    },
    Variable {
        meta: Meta,
        name: String,
        access: Vec<Access>,
    },
    // Number(Meta, i128),
    Number(Meta, Num),
    Call {
        meta: Meta,
        id: String,
        args: Vec<Expression>,
    },
    AnonymousComp{
        meta: Meta,
        id: String,
        is_parallel: bool,
        params: Vec<Expression>,
        signals: Vec<Expression>,
        names: Option<Vec<(AssignOp, String)>>,
    },
    ArrayInLine {
        meta: Meta,
        values: Vec<Expression>,
    },
    Tuple {
        meta: Meta,
        values: Vec<Expression>,
    },
    UniformArray {
        meta: Meta,
        value: Box<Expression>,
        dimension: Box<Expression>,
    },
}

#[derive(Clone, Debug, Serialize)]
pub enum Access {
    ComponentAccess(String),
    ArrayAccess(Expression),
}
pub fn build_component_access(acc: String) -> Access {
    Access::ComponentAccess(acc)
}
pub fn build_array_access(expr: Expression) -> Access {
    Access::ArrayAccess(expr)
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize)]
pub enum AssignOp {
    AssignVar,
    AssignSignal,
    AssignConstraintSignal,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize)]
pub enum ExpressionInfixOpcode {
    Mul,
    Div,
    Add,
    Sub,
    Pow,
    IntDiv,
    Mod,
    ShiftL,
    ShiftR,
    LesserEq,
    GreaterEq,
    Lesser,
    Greater,
    Eq,
    NotEq,
    BoolOr,
    BoolAnd,
    BitOr,
    BitAnd,
    BitXor,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize)]
pub enum ExpressionPrefixOpcode {
    Sub,
    BoolNot,
    Complement,
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug, Serialize)]
pub enum TypeReduction {
    Variable,
    Component,
    Signal,
    Tag,
}

#[derive(Clone, Debug, Serialize)]
pub enum LogArgument {
    LogStr(String),
    LogExp(Expression),
}
pub fn build_log_string(acc: String) -> LogArgument {
    LogArgument::LogStr(acc)
}
pub fn build_log_expression(expr: Expression) -> LogArgument {
    LogArgument::LogExp(expr)
}

#[derive(Default, Clone, Debug, Serialize)]
pub struct TypeKnowledge {
    reduces_to: Option<TypeReduction>,
}
impl TypeKnowledge {
    pub fn new() -> TypeKnowledge {
        TypeKnowledge::default()
    }
    pub fn set_reduces_to(&mut self, reduces_to: TypeReduction) {
        self.reduces_to = Option::Some(reduces_to);
    }
    pub fn get_reduces_to(&self) -> TypeReduction {
        if let Option::Some(t) = &self.reduces_to {
            *t
        } else {
            panic!("reduces_to knowledge is been look at without being initialized");
        }
    }
    pub fn is_var(&self) -> bool {
        self.get_reduces_to() == TypeReduction::Variable
    }
    pub fn is_component(&self) -> bool {
        self.get_reduces_to() == TypeReduction::Component
    }
    pub fn is_signal(&self) -> bool {
        self.get_reduces_to() == TypeReduction::Signal
    }
    pub fn is_tag(&self) -> bool {
        self.get_reduces_to() == TypeReduction::Tag
    }
}

#[derive(Default, Clone, Debug, Serialize)]
pub struct MemoryKnowledge {
    concrete_dimensions: Option<Vec<usize>>,
    full_length: Option<usize>,
    abstract_memory_address: Option<usize>,
}
impl MemoryKnowledge {
    pub fn new() -> MemoryKnowledge {
        MemoryKnowledge::default()
    }
    pub fn set_concrete_dimensions(&mut self, value: Vec<usize>) {
        self.full_length = Option::Some(value.iter().fold(1, |p, v| p * (*v)));
        self.concrete_dimensions = Option::Some(value);
    }
    pub fn set_abstract_memory_address(&mut self, value: usize) {
        self.abstract_memory_address = Option::Some(value);
    }
    pub fn get_concrete_dimensions(&self) -> &[usize] {
        if let Option::Some(v) = &self.concrete_dimensions {
            v
        } else {
            panic!("concrete dimensions was look at without being initialized");
        }
    }
    pub fn get_full_length(&self) -> usize {
        if let Option::Some(v) = &self.full_length {
            *v
        } else {
            panic!("full dimension was look at without being initialized");
        }
    }
    pub fn get_abstract_memory_address(&self) -> usize {
        if let Option::Some(v) = &self.abstract_memory_address {
            *v
        } else {
            panic!("abstract memory address was look at without being initialized");
        }
    }
}

