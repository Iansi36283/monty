use std::fmt::Debug;
use num_traits::{ToPrimitive};
use python_parser::ast::*;

fn main() {
    monty();
}


#[derive(Debug, Clone)]
enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<Value>),
    Range(i64),
    True,
    False,
    None,
}

#[derive(Debug, Clone)]
enum Expr {
    Assign {
        target: String,
        value: Box<Expr>,
    },
    Constant(Value),
    Name(String),
    Call {
        func: String,
        args: Vec<Expr>,
    },
    Op {
        left: Box<Expr>,
        op: Bop,
        right: Box<Expr>,
    },
    List(Vec<Expr>),
}


#[derive(Debug, Clone)]
enum Node {
    Pass,
    Expression(Expr),
    For {
        item: Expr,
        iter: Expr,
        body: Vec<Node>,
        // this could be just `Vec<Node>`, is that faster?
        or_else: Option<Vec<Node>>,
    },
    If {
        test: Expr,
        body: Vec<Node>,
        // this could be just `Vec<Node>`, is that faster?
        or_else: Option<Vec<Node>>,
    },
}


fn monty() {
    let code = "for i in y:\n    i = 1\n";
    let (_, ast) = python_parser::file_input(python_parser::make_strspan(code)).unwrap();
    dbg!(&ast);
    let nodes = parse_statements(ast).unwrap();
    dbg!(nodes);
}

type ParseResult<T> = Result<T, String>;

fn parse_statements(statements: Vec<Statement>) -> ParseResult<Vec<Node>> {
    statements.into_iter().map(|e| parse_statement(e)).collect()
}

fn parse_statement(statement: Statement) -> ParseResult<Node> {
    match statement {
        Statement::Pass => Ok(Node::Pass),
        Statement::Del(_expr) => todo!("Del"),
        Statement::Break => todo!("Break"),
        Statement::Continue => todo!("Continue"),
        Statement::Return(_expr) => todo!("Return"),
        Statement::RaiseExcFrom(_expr1, _expr2) => todo!("RaiseExcFrom"),
        Statement::RaiseExc(_expr) => todo!("RaiseExc"),
        Statement::Raise => todo!("Raise"),
        Statement::Global(_names) => todo!("Global"),
        Statement::Nonlocal(_names) => todo!("Nonlocal"),
        Statement::Assert(_expr, _op_exp) => todo!("Assert"),
        Statement::Import(_import) => todo!("Import"),
        Statement::Expressions(_expressions) => todo!("Expressions"),
        // `lhs = rhs1 = rhs2` -> `lhs, vec![rhs1, rhs2]`
        Statement::Assignment(lhs, rhs) => {
            assert_eq!(rhs.len(), 1);
            let rhs1 = first(rhs)?;
            parse_assignment(lhs, rhs1)
        },
        // `lhs: type` -> `lhs, type`
        Statement::TypeAnnotation(_lhs, _rhs) => Ok(Node::Pass),
        // `lhs: type = rhs` -> `lhs, type, rhs`
        Statement::TypedAssignment(lhs, _tp, rhs) => parse_assignment(lhs, rhs),
        // `lhs += rhs` -> `lhs, AugAssignOp::Add, rhs`
        Statement::AugmentedAssignment(_lhs, _op, _rhs) => todo!("AugmentedAssignment"),

        Statement::Compound(compound_statement) => parse_compound(*compound_statement),
    }
}

fn parse_compound(compound: CompoundStatement) -> ParseResult<Node> {
    match compound {
        CompoundStatement::If(ifs, or_else) => {
            let mut ifs_iter = ifs.into_iter().rev();
            let or_else = match or_else {
                Some(statements) => Some(parse_statements(statements)?),
                None => None,
            };

            let (test, body) = ifs_iter.next().unwrap();
            let mut node = parse_if(test, body, or_else)?;

            for (test, body) in ifs_iter {
                node = parse_if(test, body, Some(vec![node]))?;
            }
            Ok(node)
        }
        CompoundStatement::For { r#async, item, iterator, for_block, else_block, .. } => {
            assert!(!r#async);
            let item = parse_expression(first(item)?)?;
            let iter = parse_expression(first(iterator)?)?;
            let body = parse_statements(for_block)?;
            let or_else = match else_block {
                Some(statements) => Some(parse_statements(statements)?),
                None => None,
            };
            Ok(Node::For { item, iter, body, or_else })
        }
        CompoundStatement::While(_test, _while_block, _else_block) => todo!("while"),
        CompoundStatement::With(_items, _with_block) => todo!("with"),
        CompoundStatement::Funcdef(_funcdef) => todo!("funcdef"),
        CompoundStatement::Classdef(_class_def) => todo!("class_def"),
        CompoundStatement::Try(_try_block) => todo!("try"),
    }
}

fn parse_if(test: Expression, body: Vec<Statement>, or_else: Option<Vec<Node>>) -> ParseResult<Node> {
    let test = parse_expression(test)?;
    let body = parse_statements(body)?;
    Ok(Node::If { test, body, or_else})
}

/// `lhs = rhs` -> `lhs, rhs`
fn parse_assignment(lhs: Vec<Expression>, rhs: Vec<Expression>) -> ParseResult<Node> {
    assert_eq!(rhs.len(), 1);
    let target = first(lhs)?;
    let target = match target {
        Expression::Name(name) => name,
        _ => todo!(),
    };
    assert_eq!(rhs.len(), 1);
    let rhs_expr = first(rhs)?;
    let value = Box::new(parse_expression(rhs_expr)?);
    Ok(Node::Expression(Expr::Assign { target, value }))
}

fn parse_expression(expression: Expression) -> ParseResult<Expr> {
    match expression {
        Expression::Ellipsis => todo!("Ellipsis"),
        Expression::None => Ok(Expr::Constant(Value::None)),
        Expression::True => Ok(Expr::Constant(Value::True)),
        Expression::False => Ok(Expr::Constant(Value::False)),
        Expression::Name(name) => Ok(Expr::Name(name)),
        Expression::Int(int_type) => Ok(Expr::Constant(Value::Int(int_type.to_i64().unwrap()))),
        Expression::ImaginaryInt(_int) => todo!("ImaginaryInt"),
        Expression::Float(float_type) => Ok(Expr::Constant(Value::Float(float_type.to_f64().unwrap()))),
        Expression::ImaginaryFloat(_f) => todo!("ImaginaryFloat"),
        Expression::String(str_vec) => {
            let v = str_vec.into_iter().map(prepare_str).collect::<ParseResult<Vec<_>>>()?;
            Ok(Expr::Constant(Value::Str(v.join(""))))
        }
        Expression::Bytes(_vec) => todo!("Bytes"),
        Expression::DictLiteral(_items) => todo!("DictLiteral"),
        Expression::SetLiteral(_items) => todo!("SetLiteral"),
        Expression::ListLiteral(_items) => todo!("ListLiteral"),
        Expression::TupleLiteral(_items) => todo!("TupleLiteral"),
        Expression::DictComp(_items, _comp) => todo!("DictComp"),
        Expression::SetComp(_items, _comp) => todo!("SetComp"),
        Expression::ListComp(_items, _comp) => todo!("ListComp"),
        Expression::Generator(_items, _comp) => todo!("Generator"),
        Expression::Await(_) => todo!("Await"),
        Expression::Call(_name, _args) => todo!("Call"),
        Expression::Subscript(_, _vec) => todo!("Subscript"),
        Expression::Attribute(_lhs, _name) => todo!("Attribute"),
        Expression::Uop(_uop, _expr) => todo!("Uop"),
        // Binary operator. A simplified version of `MultiBop`, when the
        // expressivity of MultiBop is not needed.
        Expression::Bop(op, lhs, rhs) => {
            let left = Box::new(parse_expression(*lhs)?);
            let right = Box::new(parse_expression(*rhs)?);
            Ok(Expr::Op { left, op, right })
        },
        // Binary operator... but may be applied on more than one expr
        // (eg. `a <= b < c`)
        Expression::MultiBop(lhs, ops) => todo!("MultiBop"),
        // 1 if 2 else 3
        Expression::Ternary(_true, _if, _else) => todo!("Ternary"),
        Expression::Yield(_vec) => todo!("Yield"),
        Expression::YieldFrom(_expr) => todo!("YieldFrom"),
        Expression::Star(_expr) => todo!("Star"),
        Expression::Lambdef(_untyped_args_list, _expr) => todo!("Lambdef"),
        // Walrus operator: 1 := 2
        Expression::Named(_lhs, _rhs) => todo!("Named"),
    }
}

fn first<T: Debug>(v: Vec<T>) -> ParseResult<T> {
    if v.len() != 1 {
        return Err(format!("Expected 1 element, got {} (raw: {v:?})", v.len()));
    }
    v.into_iter().next().ok_or_else(|| "Expected 1 element, got 0".to_string())
}

fn prepare_str(s: PyString) -> ParseResult<String> {
    s.content.into_string().map_err(|e| format!("{:?}", e))
}
