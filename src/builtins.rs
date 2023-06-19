use crate::evaluate::Evaluator;
use std::borrow::Cow;
use std::fmt;

use crate::exceptions::{exc_err, Exception, InternalRunError};
use crate::parse_error::{ParseError, ParseResult};
use crate::run::RunResult;
use crate::types::{ExprLoc, Kwarg};
use crate::Object;

// this is a temporary hack
#[derive(Debug, Clone)]
pub(crate) enum Builtins {
    Print,
    Range,
    Len,
    ValueError,
    TypeError,
    NameError,
}

impl fmt::Display for Builtins {
    // TODO replace with a strum
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Print => write!(f, "print"),
            Self::Range => write!(f, "range"),
            Self::Len => write!(f, "len"),
            Self::ValueError => write!(f, "ValueError"),
            Self::TypeError => write!(f, "TypeError"),
            Self::NameError => write!(f, "NameError"),
        }
    }
}

impl Builtins {
    // TODO replace with a strum
    pub fn find(name: &str) -> ParseResult<'static, Self> {
        match name {
            "print" => Ok(Self::Print),
            "range" => Ok(Self::Range),
            "len" => Ok(Self::Len),
            "ValueError" => Ok(Self::ValueError),
            "TypeError" => Ok(Self::TypeError),
            "NameError" => Ok(Self::NameError),
            _ => exc_err!(ParseError::Internal; "unknown builtin: {name}"),
        }
    }

    /// whether the function has side effects
    pub fn side_effects(&self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match self {
            Self::Print => true,
            _ => false,
        }
    }

    pub fn call_function<'c, 'd>(
        &self,
        eval: &Evaluator<'d>,
        args: &'d [ExprLoc<'c>],
        _kwargs: &'d [Kwarg],
    ) -> RunResult<'c, Cow<'d, Object>> {
        match self {
            Self::Print => {
                for (i, arg) in args.iter().enumerate() {
                    let object = eval.evaluate(arg)?;
                    if i == 0 {
                        print!("{object}");
                    } else {
                        print!(" {object}");
                    }
                }
                println!();
                Ok(Cow::Owned(Object::None))
            }
            Self::Range => {
                if args.len() != 1 {
                    exc_err!(InternalRunError::TodoError; "range() takes exactly one argument")
                } else {
                    let object = eval.evaluate(&args[0])?;
                    let size = object.as_int()?;
                    Ok(Cow::Owned(Object::Range(size)))
                }
            }
            Self::Len => {
                let object = one_arg("len", eval, args)?;
                match object.len() {
                    Some(len) => Ok(Cow::Owned(Object::Int(len as i64))),
                    None => exc_err!(Exception::TypeError; "Object of type {} has no len()", object),
                }
            }
            Self::ValueError => {
                let object = one_arg("ValueError", eval, args)?;
                let str: String = object.into_owned().try_into()?;
                Ok(Cow::Owned(Object::Exc(Exception::ValueError(str.into()))))
            }
            Self::TypeError => {
                let object = one_arg("TypeError", eval, args)?;
                let str: String = object.into_owned().try_into()?;
                Ok(Cow::Owned(Object::Exc(Exception::TypeError(str.into()))))
            }
            Self::NameError => {
                let object = one_arg("NameError", eval, args)?;
                let str: String = object.into_owned().try_into()?;
                Ok(Cow::Owned(Object::Exc(Exception::NameError(str.into()))))
            }
        }
    }
}

fn one_arg<'c, 'd>(
    name: &'static str,
    eval: &Evaluator<'d>,
    args: &'d [ExprLoc<'c>],
) -> RunResult<'c, Cow<'d, Object>> {
    if args.len() != 1 {
        exc_err!(Exception::TypeError; "{}() takes exactly exactly one argument ({} given)", name, args.len())
    } else {
        eval.evaluate(&args[0])
    }
}
