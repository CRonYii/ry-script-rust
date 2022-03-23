#[cfg(test)]
mod simple_script_tests {
    use std::{collections::HashMap, hash::Hash};

    use ry_script::{
        ast::{never_reducer, value_reducer, ASTNode, RuntimeValue},
        error::{RuntimeError, ScriptError},
        grammar::TerminalSymbolDef,
        runner::{GrammarRule, ReducerArg, ScriptRunner},
        token::{LexerTokenMap, ParserToken, Token},
    };

    /* Defines the types of token that will be used */
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    enum TokenType {
        Identifier,
        Assignment,
        Integer,
        Float,
        String,
        True,
        False,
        Plus,
        Minus,
        Multiply,
        LeftParenthese,
        RightParenthese,
        EOF,
    }

    impl ParserToken<TokenType> for TokenType {
        fn entity(self, value: String) -> Token<TokenType> {
            Token {
                r#type: self,
                value,
            }
        }
    }

    /* Define runtime errors */
    type RuntimeResult<T> = std::result::Result<T, ScriptRuntimeError>;

    impl RuntimeError for ScriptRuntimeError {}

    enum ScriptRuntimeError {
        CannotCast(&'static str, String),
        NotImplemented(&'static str, String),
    }

    impl std::fmt::Display for ScriptRuntimeError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ScriptRuntimeError::CannotCast(name, val) => {
                    write!(f, "Cannot cast {:?} to {}", val, name)
                }
                ScriptRuntimeError::NotImplemented(name, val) => {
                    write!(f, "{:?} does not implemented {}", val, name)
                }
            }
        }
    }

    /* Defines runtime */
    struct RuntimeEnvironment {
        variables: HashMap<String, Value>,
    }

    impl RuntimeEnvironment {
        fn new() -> Self {
            Self {
                variables: HashMap::new(),
            }
        }
    }

    impl RuntimeValue<TokenType> for Value {}

    #[derive(Debug, PartialEq, Clone)]
    enum Value {
        Identifier(String),
        String(String),
        Integer(i64),
        Float(f64),
        Bool(bool),
    }

    impl std::convert::From<Token<TokenType>> for Value {
        fn from(token: Token<TokenType>) -> Self {
            match token.r#type {
                TokenType::String => Value::String(token.value),
                TokenType::Integer => Value::Integer(token.value.parse().unwrap()),
                TokenType::Float => Value::Float(token.value.parse().unwrap()),
                TokenType::True => Value::Bool(true),
                TokenType::False => Value::Bool(false),
                TokenType::Identifier => Value::Identifier(token.value),
                _ => panic!(
                    "Unexpected token {:?} cannot be converted to a value",
                    token.r#type
                ),
            }
        }
    }

    impl std::fmt::Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Value::Integer(num) => write!(f, "{}", num)?,
                Value::Float(num) => write!(f, "{}", num)?,
                Value::String(str) => write!(f, "{}", str)?,
                Value::Bool(bool) => write!(f, "{}", bool)?,
                Value::Identifier(name) => write!(f, "id({})", name)?,
            }
            Ok(())
        }
    }

    /* Value getters */
    impl Value {
        fn value<'a>(&'a self, env: &'a RuntimeEnvironment) -> &'a Value {
            match self {
                Value::Identifier(name) => match env.variables.get(name) {
                    Some(value) => value,
                    None => self,
                },
                _ => self,
            }
        }

        fn float(&self) -> RuntimeResult<f64> {
            match self {
                Value::Integer(val) => Ok(*val as f64),
                Value::Float(val) => Ok(*val),
                _ => Err(ScriptRuntimeError::CannotCast("float", format!("{}", self))),
            }
        }
    }

    impl RuntimeEnvironment {
        fn assign(&mut self, lhs: Value, rhs: Value) -> RuntimeResult<Value> {
            let rhs = rhs.value(self).clone();
            match lhs {
                Value::Identifier(key) => {
                    self.variables.insert(key.clone(), rhs);
                    Ok(Value::Identifier(key))
                }
                _ => Err(ScriptRuntimeError::NotImplemented(
                    "Assignment",
                    format!("{}", lhs),
                )),
            }
        }

        fn mul(&self, lhs: &Value, rhs: &Value) -> RuntimeResult<Value> {
            let lhs = lhs.value(self);
            let rhs = rhs.value(self);
            match lhs {
                Value::Integer(lhs) => match rhs {
                    Value::Integer(rhs) => Ok(Value::Integer(lhs * rhs)),
                    Value::Float(rhs) => Ok(Value::Float((*lhs as f64) * rhs)),
                    _ => Err(ScriptRuntimeError::NotImplemented(
                        "Multiplication",
                        format!("{}", rhs),
                    )),
                },
                Value::Float(lhs) => {
                    let val = lhs * rhs.float()?;
                    Ok(Value::Float(val))
                }
                _ => Err(ScriptRuntimeError::NotImplemented(
                    "Multiplication",
                    format!("{}", lhs),
                )),
            }
        }

        fn add(&self, lhs: &Value, rhs: &Value) -> RuntimeResult<Value> {
            let lhs = lhs.value(self);
            let rhs = rhs.value(self);
            match lhs {
                Value::Integer(lhs) => match rhs {
                    Value::Integer(rhs) => Ok(Value::Integer(lhs + rhs)),
                    Value::Float(rhs) => Ok(Value::Float((*lhs as f64) + rhs)),
                    _ => Err(ScriptRuntimeError::NotImplemented(
                        "Addition",
                        format!("{}", rhs),
                    )),
                },
                Value::Float(lhs) => {
                    let val = lhs + rhs.float()?;
                    Ok(Value::Float(val))
                }
                _ => Err(ScriptRuntimeError::NotImplemented(
                    "Addition",
                    format!("{}", lhs),
                )),
            }
        }

        fn negative(&self, val: &Value) -> RuntimeResult<Value> {
            let val = val.value(self);
            match val {
                Value::Integer(val) => Ok(Value::Integer(-val)),
                Value::Float(val) => Ok(Value::Float(-val)),
                Value::Bool(val) => Ok(Value::Bool(!val)),
                _ => Err(ScriptRuntimeError::NotImplemented(
                    "Negative",
                    format!("{}", val),
                )),
            }
        }
    }

    /* Grammar reducers that takes advantage of RuntimeValue */
    fn assignment_reducer(
        mut args: ReducerArg<RuntimeEnvironment, TokenType, Value, ScriptRuntimeError>,
    ) -> ASTNode<RuntimeEnvironment, TokenType, Value, ScriptRuntimeError> {
        ASTNode::ActionExpression(
            "id = val",
            Box::new(
                move |env| match (args.eval_skip(env, 1)?, args.eval(env)?) {
                    (ASTNode::Value(lhs), ASTNode::Value(rhs)) => {
                        println!("eval {:?} = {:?}", lhs, rhs);
                        Ok(ASTNode::Value(env.assign(lhs, rhs)?))
                    }
                    _ => panic!("Parse Error: Reducer expected value but non-value were given"),
                },
            ),
        )
    }

    fn multiply_reducer(
        mut args: ReducerArg<RuntimeEnvironment, TokenType, Value, ScriptRuntimeError>,
    ) -> ASTNode<RuntimeEnvironment, TokenType, Value, ScriptRuntimeError> {
        ASTNode::ActionExpression(
            "a * b",
            Box::new(
                move |env| match (args.eval_skip(env, 1)?, args.eval(env)?) {
                    (ASTNode::Value(lhs), ASTNode::Value(rhs)) => {
                        #[cfg(feature = "debug_ast")]
                        println!("eval {:?} * {:?}", lhs, rhs);
                        Ok(ASTNode::Value(env.mul(&lhs, &rhs)?))
                    }
                    _ => panic!("Parse Error: Reducer expected value but non-value were given"),
                },
            ),
        )
    }

    fn add_reducer(
        mut args: ReducerArg<RuntimeEnvironment, TokenType, Value, ScriptRuntimeError>,
    ) -> ASTNode<RuntimeEnvironment, TokenType, Value, ScriptRuntimeError> {
        ASTNode::ActionExpression(
            "a + b",
            Box::new(
                move |env| match (args.eval_skip(env, 1)?, args.eval(env)?) {
                    (ASTNode::Value(lhs), ASTNode::Value(rhs)) => {
                        #[cfg(feature = "debug_ast")]
                        println!("eval {:?} + {:?}", lhs, rhs);
                        Ok(ASTNode::Value(env.add(&lhs, &rhs)?))
                    }
                    _ => panic!("Parse Error: Reducer expected value but non-value were given"),
                },
            ),
        )
    }

    fn negative_number_reducer(
        mut args: ReducerArg<RuntimeEnvironment, TokenType, Value, ScriptRuntimeError>,
    ) -> ASTNode<RuntimeEnvironment, TokenType, Value, ScriptRuntimeError> {
        ASTNode::ActionExpression(
            "a + b",
            Box::new(move |env| match args.nth_eval(env, 1)? {
                ASTNode::Value(val) => {
                    #[cfg(feature = "debug_ast")]
                    println!("eval - {:?}", val);
                    Ok(ASTNode::Value(env.negative(&val)?))
                }
                _ => panic!("Parse Error: Reducer expected value but non-value were given"),
            }),
        )
    }

    fn init_simple_script_parser() -> ry_script::error::Result<
        ScriptRunner<RuntimeEnvironment, TokenType, Value, ScriptRuntimeError>,
        ScriptRuntimeError,
    > {
        /* These construct the Lexer */
        let token_map = LexerTokenMap {
            eof: TokenType::EOF,
            identifier: TokenType::Identifier,
            integer: TokenType::Integer,
            float: TokenType::Float,
            string: TokenType::String,
        };
        let operator = [
            /* Specify the possible operator that the lexer will recognize */
            TerminalSymbolDef("=", TokenType::Assignment),
            TerminalSymbolDef("+", TokenType::Plus),
            TerminalSymbolDef("-", TokenType::Minus),
            TerminalSymbolDef("*", TokenType::Multiply),
            TerminalSymbolDef("(", TokenType::LeftParenthese),
            TerminalSymbolDef(")", TokenType::RightParenthese),
        ];
        let keyword = [
            /* Reserved keyword that are automatically converted from Identifer */
            TerminalSymbolDef("true", TokenType::True),
            TerminalSymbolDef("false", TokenType::False),
        ];

        /* These construct the LR Parser */
        let grammars: Vec<GrammarRule<RuntimeEnvironment, TokenType, Value, ScriptRuntimeError>> = vec![
            GrammarRule("B -> S EOF", never_reducer),
            GrammarRule("S -> A1", value_reducer),
            GrammarRule("S -> id = A1", assignment_reducer),
            GrammarRule("A1 -> A2", value_reducer),
            GrammarRule("A1 -> A1 + A2", add_reducer),
            GrammarRule("A2 -> A3", value_reducer),
            GrammarRule("A2 -> A2 * A3", multiply_reducer),
            GrammarRule("A3 -> Val", value_reducer),
            GrammarRule("Val -> str", value_reducer),
            GrammarRule("Val -> num", value_reducer),
            GrammarRule("Val -> + num", |mut args| args.nth_val(1)),
            GrammarRule("Val -> - num", negative_number_reducer),
            GrammarRule("num -> id", value_reducer),
            GrammarRule("num -> int", value_reducer),
            GrammarRule("num -> float", value_reducer),
            GrammarRule("num -> true", value_reducer),
            GrammarRule("num -> false", value_reducer),
            GrammarRule("Val -> ( A1 )", |mut args| args.nth_val(1)),
        ];
        ScriptRunner::new(grammars, token_map, &operator, &keyword)
    }

    #[test]
    fn test_addition() -> Result<(), ScriptError<ScriptRuntimeError>> {
        let mut runner = init_simple_script_parser()?;
        let mut env = RuntimeEnvironment::new();
        match runner.run(&mut env, &"1+1")? {
            ASTNode::Value(value) => assert_eq!(value, Value::Integer(2)),
            _ => panic!(),
        };
        match runner.run(&mut env, &"1+2.5")? {
            ASTNode::Value(value) => assert_eq!(value, Value::Float(3.5)),
            _ => panic!(),
        };
        match runner.run(&mut env, &"1.5+40")? {
            ASTNode::Value(value) => assert_eq!(value, Value::Float(41.5)),
            _ => panic!(),
        };
        match runner.run(&mut env, &"1.5+5.4")? {
            ASTNode::Value(value) => assert_eq!(value, Value::Float(6.9)),
            _ => panic!(),
        };
        Ok(())
    }

    #[test]
    fn test_multiplication_and_addition() -> Result<(), ScriptError<ScriptRuntimeError>> {
        let mut runner = init_simple_script_parser()?;
        let mut env = RuntimeEnvironment::new();
        match runner.run(&mut env, &"1+2*3")? {
            ASTNode::Value(value) => assert_eq!(value, Value::Integer(7)),
            _ => panic!(),
        };
        match runner.run(&mut env, &"2*3+4")? {
            ASTNode::Value(value) => assert_eq!(value, Value::Integer(10)),
            _ => panic!(),
        };
        Ok(())
    }

    #[test]
    fn test_parenthesis_priority() -> Result<(), ScriptError<ScriptRuntimeError>> {
        let mut runner = init_simple_script_parser()?;
        let mut env = RuntimeEnvironment::new();
        match runner.run(&mut env, &"(1+2)*3")? {
            ASTNode::Value(value) => assert_eq!(value, Value::Integer(9)),
            _ => panic!(),
        };
        match runner.run(&mut env, &"2*(3+4)")? {
            ASTNode::Value(value) => assert_eq!(value, Value::Integer(14)),
            _ => panic!(),
        };
        Ok(())
    }

    #[test]
    fn test_assignment() -> Result<(), ScriptError<ScriptRuntimeError>> {
        let mut runner = init_simple_script_parser()?;
        let mut env = RuntimeEnvironment::new();
        match runner.run(&mut env, &"foo = 10")? {
            ASTNode::Value(value) => assert_eq!(value, Value::Identifier("foo".to_string())),
            _ => panic!(),
        };
        let id = Value::Identifier("foo".to_string());
        let value = id.value(&mut env);
        assert_eq!(value, &Value::Integer(10));
        match runner.run(&mut env, &"foo = foo * foo")? {
            ASTNode::Value(value) => match value {
                Value::Identifier(_) => {
                    let value = value.value(&mut env);
                    assert_eq!(value, &Value::Integer(100));
                }
                _ => panic!(),
            },
            _ => panic!(),
        };
        match runner.run(&mut env, &"-foo + -20")? {
            ASTNode::Value(value) => assert_eq!(value, Value::Integer(-120)),
            _ => panic!(),
        };
        Ok(())
    }
}
