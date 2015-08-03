use std::fmt::Debug;

pub trait Function : Debug {
    fn call(&self, args: &Vec<Box<Expression>>) -> i64;
}


#[derive(Debug)]
pub struct Add;

impl Add {
    pub fn new() -> Add {
        return Add;
    }
}

impl Function for Add {
    fn call(&self, args: &Vec<Box<Expression>>) -> i64 {
        args.iter().fold(0, |acc, expr| { acc + expr.eval() })
    }
}

pub trait Expression : Debug {
    fn eval(&self) -> i64;
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
    fn eval(&self) -> i64 {
        self.val
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
    fn eval(&self) -> i64 {
        self.function.call(&self.args)
    }
}

#[cfg(test)]
mod tests {
    use super::Function;
    use super::Add;
    use super::Call;
    use super::Expression;
    use super::Literal;

    #[test]
    fn test_add_two_and_two() {
        let add = super::Add;
        assert_eq!(4, add.call(&vec![Box::new(Literal {val:2}), Box::new(Literal{val:2})]));
    }

    #[test]
    fn test_add_three_values() {
        let add = super::Add;
        assert_eq!(6, add.call(&vec![Box::new(Literal {val:1}), Box::new(Literal{val:2}), Box::new(Literal{val:3})]));
    }

    #[test]
    fn test_eval_call() {
        let add = super::Add;
        let one = Box::new(Literal {val:1});
        let two = Box::new(Literal {val:2});
        let three = Box::new(Literal {val:3});
        let expr = Call {function: Box::new(add), args: vec![one, two, three]};
        assert_eq!(6, expr.eval());
    }

    #[test]
    fn test_eval_recursive() {
        let expr = Call {function: Box::new(Add),
                         args: vec![Box::new(Literal {val:1}),
                                    Box::new(Call {function: Box::new(Add),
                                                   args: vec![Box::new(Literal {val:2}),
                                                              Box::new(Literal {val:3})]})]};
        assert_eq!(6, expr.eval());
    }
}
