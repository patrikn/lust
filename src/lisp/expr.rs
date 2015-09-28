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
    fn call(&self, args: &Vec<Box<Expression>>, env: &mut Environment) -> Result<i64, EvalError>;
}


#[derive(Debug)]
pub struct Add;

impl Add {
    pub fn new() -> Add {
        return Add;
    }
}

impl Function for Add {
    fn call(&self, args: &Vec<Box<Expression>>, env: &mut Environment) -> Result<i64, EvalError> {
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

    pub fn set(&mut self, name: &str, val: i64) -> i64 {
        self.vars.insert(String::from(name), val);
        val
    }
}


pub trait Expression : fmt::Debug {
    fn eval(&self, &mut Environment) -> Result<i64, EvalError>;

    fn lvalue(&self, &mut Environment) -> Result<&str, EvalError>;
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
    fn eval(&self, env: &mut Environment) -> Result<i64, EvalError> {
        Ok(self.val)
    }

    fn lvalue(&self, env: &mut Environment) -> Result<&str, EvalError> {
        Err(EvalError::UndefinedName(format!("{:?}", self.val)))
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
    fn eval(&self, env: &mut Environment) -> Result<i64, EvalError> {
        self.function.call(&self.args, env)
    }

    fn lvalue(&self, env: &mut Environment) -> Result<&str, EvalError> {
        Err(EvalError::UndefinedName(format!("({:?} {:?})", self.function, self.args)))
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
    fn call(&self, args: &Vec<Box<Expression>>, env: &mut Environment) -> Result<i64, EvalError> {
        let result = try!(args[0].eval(env));
        if result != 0 {
            args[1].eval(env)
        } else {
            args[2].eval(env)
        }
    }
}


#[derive(Debug)]
pub struct Reference {
    name: String
}

impl Reference {
    pub fn new(name: &str) -> Reference {
        Reference { name: String::from(name) }
    }
}

impl Expression for Reference {
    fn eval(&self, env: &mut Environment) -> Result<i64, EvalError> {
        env.get(&self.name)
    }


    fn lvalue(&self, env: &mut Environment) -> Result<&str, EvalError> {
        Ok(&*self.name)
    }
}


#[derive(Debug)]
pub struct Set;

impl Set {
    pub fn new() -> Set {
        Set
    }
}

impl Function for Set {
    fn call(&self, args: &Vec<Box<Expression>>, env: &mut Environment) -> Result<i64, EvalError> {
        let lvalue = try!(args[0].lvalue(env));
        let val = try!(args[1].eval(env));
        Ok(env.set(lvalue.as_ref(), val))
    }
}


#[cfg(test)]
mod tests {
    use super::Environment;
    use super::Set;
    use super::Function;
    use super::Add;
    use super::If;
    use super::Call;
    use super::Expression;
    use super::Literal;
    use super::Reference;

    #[test]
    fn test_add_two_and_two() {
        let mut env = Environment::new();
        let add = super::Add;
        let result = add.call(&vec![Box::new(Literal {val:2}), Box::new(Literal{val:2})], &mut env);
        assert_eq!(4, result.unwrap());
    }

    #[test]
    fn test_add_three_values() {
        let mut env = Environment::new();
        let add = super::Add;
        assert_eq!(6, add.call(&vec![Box::new(Literal {val:1}),
                                     Box::new(Literal{val:2}),
                                     Box::new(Literal{val:3})],
                               &mut env)
                   .unwrap());
    }

    #[test]
    fn test_eval_call() {
        let mut env = Environment::new();
        let add = super::Add;
        let one = Box::new(Literal {val:1});
        let two = Box::new(Literal {val:2});
        let three = Box::new(Literal {val:3});
        let expr = Call {function: Box::new(add), args: vec![one, two, three]};
        assert_eq!(6, expr.eval(&mut env).unwrap());
    }

    #[test]
    fn test_eval_recursive() {
        let mut env = Environment::new();
        let expr = Call {function: Box::new(Add),
                         args: vec![Box::new(Literal {val:1}),
                                    Box::new(Call {function: Box::new(Add),
                                                   args: vec![Box::new(Literal {val:2}),
                                                              Box::new(Literal {val:3})]})]};
        assert_eq!(6, expr.eval(&mut env).unwrap());
    }

    #[test]
    fn test_if_nonzero() {
        let mut env = Environment::new();
        assert_eq!(4,
                   If.call(&vec![ Box::new(Literal {val:1}),
                                  Box::new(Call {function: Box::new(Add),
                                                 args: vec![Box::new(Literal {val:1}),
                                                            Box::new(Literal {val:3})]}),
                                  Box::new(Literal {val:2})],
                           &mut env)
                   .unwrap());
    }

    #[test]
    fn test_if_zero() {
        let mut env = Environment::new();
        assert_eq!(2, If.call(&vec![ Box::new(Literal {val:0}),
                                      Box::new(Call {function: Box::new(Add),
                                                     args: vec![Box::new(Literal {val:1}),
                                                                Box::new(Literal {val:3})]}),
                                      Box::new(Literal {val:2})],
                              &mut env)
                   .unwrap());
    }

    #[test]
    fn test_missing_variable() {
        let mut env = Environment::new();
        Reference::new("foo").eval(&mut env).unwrap_err();
    }

    #[test]
    fn test_variable() {
        let mut env = Environment::new();
        env.set("foo", 3);
        assert_eq!(3, Reference::new("foo").eval(&mut env).unwrap());
    }

    #[test]
    fn test_variable_argument() {
        let mut env = Environment::new();
        env.set("foo", 123);
        let add = super::Add;
        let one = Box::new(Reference::new("foo"));
        let two = Box::new(Literal {val:2});
        let three = Box::new(Literal {val:3});
        let expr = Call {function: Box::new(add), args: vec![one, two, three]};
        assert_eq!(128, expr.eval(&mut env).unwrap());
    }

    #[test]
    fn test_assign_value() {
        let mut env = Environment::new();
        let expr = Call {function: Box::new(Set::new()), args: vec![Box::new(Reference::new("bar")),
                                                                    Box::new(Literal::new(3))]};
        assert_eq!(3, expr.eval(&mut env).unwrap());
        let read = Reference::new("bar");
        assert_eq!(3, read.eval(&mut env).unwrap());
    }

    #[test]
    fn test_reassign_value() {
        let mut env = Environment::new();
        env.set("bar", 3);
        let expr = Call {function: Box::new(Set::new()), args: vec![Box::new(Reference::new("bar")),
                                                                    Box::new(Literal::new(17))]};
        assert_eq!(17, expr.eval(&mut env).unwrap());
        let read = Reference::new("bar");
        assert_eq!(17, read.eval(&mut env).unwrap());
    }
}
