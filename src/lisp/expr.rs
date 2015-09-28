use std::fmt;
use std::collections::HashMap;

#[derive(Debug)]
pub enum EvalError {
    UndefinedName(String)
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EvalError::UndefinedName(ref err) => write!(f, "No such name in environment: {}", err)
        }
    }
}

pub trait Function : fmt::Debug {
    fn call(&self, args: &Vec<Box<Expression>>, env: &Environment) -> Result<i64, EvalError>;
}


#[derive(Debug)]
pub struct Add;

impl Add {
    pub fn new() -> Add {
        return Add;
    }
}

impl Function for Add {
    fn call(&self, args: &Vec<Box<Expression>>, env: &Environment) -> Result<i64, EvalError> {
        args.iter().fold(Ok(0), |acc, expr| { Ok(try!(acc) + try!(expr.eval(env))) })
    }
}

#[derive(Debug)]
pub struct Environment {
    vars: HashMap<String, i64>
}

impl Environment {
    pub fn new() -> Environment {
        Environment {vars: HashMap::new()}
    }

    pub fn get(&self, name: &str) -> Result<i64, EvalError> {
        self.vars.get(name).map(|v| { *v }).ok_or(EvalError::UndefinedName(String::from(name)))
    }
}


pub trait Expression : fmt::Debug {
    fn eval(&self, &Environment) -> Result<i64, EvalError>;
}

#[derive(Debug)]
pub struct Literal {
    val: i64,
}

impl Literal {
    pub fn new(val: i64) -> Literal {
        Literal {val: val}
    }
}

impl Expression for Literal {
    fn eval(&self, env: &Environment) -> Result<i64, EvalError> {
        Ok(self.val)
    }
}

#[derive(Debug)]
pub struct Call {
    function: Box<Function>,
    args: Vec<Box<Expression>>,
}

impl Call {
    pub fn new(function: Box<Function>, args: Vec<Box<Expression>>) -> Call {
        Call {function: function, args:args}
    }
}

impl Expression for Call {
    fn eval(&self, env: &Environment) -> Result<i64, EvalError> {
        self.function.call(&self.args, env)
    }
}

#[derive(Debug)]
pub struct If;

impl If {
    pub fn new() -> If {
        If
    }
}

impl Function for If {
    fn call(&self, args: &Vec<Box<Expression>>, env: &Environment) -> Result<i64, EvalError> {
        let result = try!(args[0].eval(env));
        if result != 0 {
            args[1].eval(env)
        } else {
            args[2].eval(env)
        }
    }
}

#[derive(Debug)]
pub struct Reference<'a> {
    name: &'a str
}

impl<'a> Reference<'a> {
    pub fn new(name: &'a str) -> Reference {
        Reference { name: name }
    }
}

impl<'a> Expression for Reference<'a> {
    fn eval(&self, env: &Environment) -> Result<i64, EvalError> {
        env.get(self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::Environment;
    use super::Function;
    use super::Add;
    use super::If;
    use super::Call;
    use super::Expression;
    use super::Literal;

    #[test]
    fn test_add_two_and_two() {
        let env = Environment::new();
        let add = super::Add;
        let result = add.call(&vec![Box::new(Literal {val:2}), Box::new(Literal{val:2})], &env);
        assert_eq!(4, result.unwrap());
    }

    #[test]
    fn test_add_three_values() {
        let env = Environment::new();
        let add = super::Add;
        assert_eq!(6, add.call(&vec![Box::new(Literal {val:1}),
                                     Box::new(Literal{val:2}),
                                     Box::new(Literal{val:3})],
                               &env)
                   .unwrap());
    }

    #[test]
    fn test_eval_call() {
        let env = Environment::new();
        let add = super::Add;
        let one = Box::new(Literal {val:1});
        let two = Box::new(Literal {val:2});
        let three = Box::new(Literal {val:3});
        let expr = Call {function: Box::new(add), args: vec![one, two, three]};
        assert_eq!(6, expr.eval(&env).unwrap());
    }

    #[test]
    fn test_eval_recursive() {
        let env = Environment::new();
        let expr = Call {function: Box::new(Add),
                         args: vec![Box::new(Literal {val:1}),
                                    Box::new(Call {function: Box::new(Add),
                                                   args: vec![Box::new(Literal {val:2}),
                                                              Box::new(Literal {val:3})]})]};
        assert_eq!(6, expr.eval(&env).unwrap());
    }

    #[test]
    fn test_if_nonzero() {
        let env = Environment::new();
        assert_eq!(4,
                   If.call(&vec![ Box::new(Literal {val:1}),
                                  Box::new(Call {function: Box::new(Add),
                                                 args: vec![Box::new(Literal {val:1}),
                                                            Box::new(Literal {val:3})]}),
                                  Box::new(Literal {val:2})],
                           &env)
                   .unwrap());
    }

    #[test]
    fn test_if_zero() {
        let env = Environment::new();
        assert_eq!(2, If.call(&vec![ Box::new(Literal {val:0}),
                                      Box::new(Call {function: Box::new(Add),
                                                     args: vec![Box::new(Literal {val:1}),
                                                                Box::new(Literal {val:3})]}),
                                      Box::new(Literal {val:2})],
                              &env)
                   .unwrap());
    }
}
