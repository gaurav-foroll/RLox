pub mod environment;

use std::process;

use crate::{
    object::Object,
    parser::{
        expr::{bool, is_truthy, Expr},
        stmt::Stmt,
    },
    token::Tokentype,
};
use environment::Environment;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            self.execute_statement(statement);
        }
    }
}

// for expression
impl Interpreter {
    fn visit_expression(&mut self, expression: Expr) -> Object {
        match expression {
            Expr::Literal { value } => {
                let val = value;
                val.clone()
            }
            Expr::Grouping { expression } => self.evaluate_expression(*expression),
            Expr::Unary { operator, right } => {
                let right = self.evaluate_expression(*right);

                match operator.tokentype {
                    Tokentype::Bang => {
                        let truthy = is_truthy(&right);
                        if truthy == Object::True {
                            Object::False
                        } else {
                            Object::True
                        }
                    }
                    Tokentype::Minus => match right {
                        Object::IntValue(value) => Object::IntValue(-value),
                        Object::FloatValue(value) => Object::FloatValue(-value),
                        _ => Object::Null,
                    },
                    _ => Object::Null,
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate_expression(*left);
                let right = self.evaluate_expression(*right);
                // matchception begins here ;) good luck understanding code
                // changed my mind writing clean code ;)
                match operator.tokentype {
                    Tokentype::Minus => left - right,
                    Tokentype::Plus => left + right,
                    Tokentype::Slash => left / right,
                    Tokentype::Star => left * right,
                    Tokentype::Greater => bool(left > right),
                    Tokentype::GreaterEqual => bool(left >= right),
                    Tokentype::Less => bool(left < right),
                    Tokentype::LessEqual => bool(left <= right),
                    Tokentype::EqualEqual => bool(left == right),
                    Tokentype::BangEqual => bool(left != right),
                    _ => Object::Null,
                }
            }
            Expr::Variable { name } => match self.environment.get(name) {
                Ok(val) => val,
                Err(msg) => {
                    println!("Error : {} ", msg);
                    Object::Null
                }
            },
            Expr::Assign { name, value } => {
                let value = self.evaluate_expression(*value);
                match self.environment.assign(name, &value) {
                    Ok(_) => {}
                    Err(msg) => println!("{}", msg),
                };
                value
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate_expression(*left);

                if operator.tokentype == Tokentype::Or {
                    if is_truthy(&left) == Object::True {
                        left
                    } else {
                        self.evaluate_expression(*right)
                    }
                } else {
                    if !(is_truthy(&left) == Object::True) {
                        left
                    } else {
                        self.evaluate_expression(*right)
                    }
                }
            }
        }
    }

    fn evaluate_expression(&mut self, expr: Expr) -> Object {
        self.visit_expression(expr)
    }
}

// for statements
impl Interpreter {
    fn visit_statement(&mut self, statement: Stmt) {
        match statement {
            Stmt::Expression { expression } => {
                let _ = self.evaluate_expression(expression);
            }
            Stmt::Print { expression } => {
                let val = self.evaluate_expression(expression);
                print!("{}", val);
            }
            Stmt::Var { name, initalizer } => {
                let mut value: Object = Object::Null;
                let null = Expr::Literal {
                    value: Object::Null,
                };
                if initalizer != null {
                    value = self.evaluate_expression(initalizer);
                }

                self.environment.define(name.lexeme, value)
            }
            Stmt::Block { statements } => self.execute_block(statements, self.environment.clone()),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate_expression(condition) == Object::True {
                    self.execute_statement(*then_branch);
                } else {
                    match *else_branch {
                        Some(statement) => {
                            self.execute_statement(statement);
                        }
                        None => (),
                    }
                }
            }
            Stmt::While { condition, body } => {
                while self.evaluate_expression(condition.clone()) == Object::True {
                    self.execute_statement(*body.clone());
                }
            }
        }
    }

    fn execute_statement(&mut self, statement: Stmt) {
        self.visit_statement(statement);
    }

    fn execute_block(&mut self, statements: Vec<Stmt>, environment: Environment) {
        self.environment = Environment::new_with_enclosing(environment.clone());

        for statement in statements {
            self.execute_statement(statement);
        }

        match self.environment.get_enclosing() {
            Some(environment) => self.environment = environment,
            None => {
                println!("Environment Lost . ");
                process::exit(64);
            }
        };
    }
}
