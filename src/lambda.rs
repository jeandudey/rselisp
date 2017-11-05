#[macro_use]
use super::*;

/// Whether function arguments are self quoting or evaluated
#[derive(Clone)]
pub enum EvalOption {
    Evaluated,
    Unevaluated,
}

/// Something which can be called with arguments
pub trait Func {
    /// Return whether the arguments are evaluated
    fn eval_args(&self) -> EvalOption;
    /// The canonical name of this function
    fn name(&self) -> &str;
    /// Evaluate this function
    fn call(&self, &mut Lsp, &mut Iter<LispObj>) -> Result<LispObj, String>;
}

#[derive(Clone, Debug)]
pub struct ArgSpec {
    name: String,
}

impl ArgSpec {
    fn new(name: String) -> ArgSpec {
        ArgSpec {
            name: name,
        }
    }
}

impl fmt::Display for ArgSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.name)
    }
}

#[derive(Clone, Debug)]
pub struct ArgSpecs(Vec<ArgSpec>);

impl std::ops::Deref for ArgSpecs {
    type Target = Vec<ArgSpec>;

    fn deref(&self) -> &Vec<ArgSpec> {
        &self.0
    }
}

impl fmt::Display for ArgSpecs {
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_iter('[', &mut self.iter(), f)
    }
}

/// A function created by the user with the lambda builtin
#[derive(Clone, Debug)]
pub struct UserFunc {
    args: ArgSpecs,
    body: LispObjRef,
}

impl UserFunc {
    fn new(args: Vec<ArgSpec>, body: LispObjRef) -> UserFunc {
        UserFunc {
            args: ArgSpecs(args),
            body: body,
        }
    }

    pub fn lambda(args: &mut Iter<LispObj>) -> Result<LispObj, String> {
        match take2!(args) {
            (Some(&LispObj::Sxp(ref args_sxp)), Some(body)) => {
                let largs: Result<Vec<ArgSpec>, String> = args_sxp.lst.iter().map(
                    |arg| -> Result<ArgSpec, String> {
                        match arg {
                            &LispObj::Sym(ref name) => Ok(ArgSpec::new(name.to_owned())),
                            _ => Err(format!("Lambda arguments must be symbols")),
                        }
                    }
                ).collect();

                Ok(LispObj::Lambda(
                    UserFunc::new(largs?, match body {
                        &LispObj::Ref(ref iref) => iref.clone(),
                        _ => body.clone().into_ref(),
                    })
                ))
            },
            _ => Err(format!("(lambda ([args]) [body])")),
        }
    }
}

impl Func for UserFunc {
    fn eval_args(&self) -> EvalOption { EvalOption::Evaluated }
    fn name(&self) -> &str { "" }

    fn call(&self, lsp: &mut Lsp, args: &mut Iter<LispObj>) -> Result<LispObj, String> {
        let mut ns = Namespace::new();
        for spec in self.args.iter() {
            if let Some(arg) = args.next() {
                ns.reg_var_s(spec.name.clone(), arg);
            } else {
                return Err(format!("'{}' expected '{}' argument", self.name(), spec.name));
            }
        }
        lsp.locals.push(ns);
        let ret = lsp.eval_ref(&self.body);
        lsp.locals.pop();
        ret
    }
}

impl fmt::Display for UserFunc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "('{} . (lambda {} {}))", &self.name(), &self.args, LispObj::Ref(self.body.clone()))
    }
}
