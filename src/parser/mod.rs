use crate::global::DataType;
use crate::lexer::structs::{Direction, KeywordType, OperatorType, SignType, Token};
use crate::lexer::tokenize;
use crate::parser::structs::ASTNode::Expression;
use crate::parser::structs::MiscNodeType;
use crate::parser::structs::{ASTNode, BinaryExpression, ExpressionType, Operand};
use std::collections::{HashMap, VecDeque};
use std::env::args;

pub mod structs;

pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new(source: String) -> Self {
        let tokens = tokenize(source);

        dbg!(tokens.clone());

        Self { tokens }
    }

    fn is_end(&self) -> bool {
        if self.tokens.is_empty() {
            true
        } else {
            self.tokens[0] == Token::End
        }
    }

    fn curr(&self) -> Token {
        if self.tokens.is_empty() {
            Token::End
        } else {
            self.tokens[0].clone()
        }
    }

    fn go(&mut self) -> Token {
        self.tokens.pop_front().unwrap_or(Token::End).clone()
    }

    fn next(&mut self) -> Token {
        self.tokens.get(1).cloned().unwrap_or(Token::End)
    }

    pub fn gen_ast(&mut self) -> ASTNode {
        let mut body: Vec<ASTNode> = vec![];

        while !self.is_end() {
            dbg!(self.curr());
            body.push(self.parse_expressions())
        }

        dbg!(body.clone());

        ASTNode::Program(body)
    }

    fn parse_expressions(&mut self) -> ASTNode {
        match self.curr() {
            Token::Keyword(keyword) => match keyword {
                KeywordType::Let => self.parse_variable_declaration(),
                KeywordType::Const => self.parse_variable_declaration(),
                KeywordType::Immut => self.parse_variable_declaration(),
                KeywordType::Fn => self.parse_fn_declaration(),
                _ => ASTNode::Expression(ExpressionType::Null),
            },
            Token::Identifier(_) => {
                if self.next() == Token::Operator(OperatorType::Equal) {
                    self.parse_variable_assignment()
                } else if self.next() == Token::Sign(SignType::Paren(Direction::Open)) {
                    self.parse_function_call()
                } else {
                    self.parse_add_expressions()
                }
            }
            Token::Boolean(value) => {
                self.go();
                ASTNode::Boolean(value)
            }
            Token::Operator(operator_type) => {
                if operator_type == OperatorType::SelfAssign {
                    self.parse_self_assign_expression()
                } else {
                    self.parse_add_expressions()
                }
            }
            Token::Sign(sign_type) => {
                if sign_type == SignType::Paren(Direction::Open) {
                    self.parse_repeat_expression()
                } else {
                    self.parse_add_expressions()
                }
            }
            _ => self.parse_add_expressions(),
        }
    }

    fn parse_primary_expressions(&mut self) -> Option<ASTNode> {
        // self.parse_add_expressions()

        let token = self.go();

        let mut res = Some(ASTNode::Expression(ExpressionType::Null));

        if let Token::Sign(sign_type) = &token {
            if let SignType::Paren(direction) = sign_type {
                if *direction == Direction::Open {
                    res = Some(self.parse_expressions());
                    // dbg!("Removed", self.go());
                } else {
                    res = None
                }
            }
        }

        if let Token::String(v) = token.clone() {
            res = Some(ASTNode::String(v));
        } else if let Token::Number(v) = token.clone() {
            res = Some(ASTNode::Number(v));
        } else if let Token::Identifier(v) = token.clone() {
            res = Some(ASTNode::Identifier(v));
        } else if let Token::Sign(sign_type) = token.clone() {
            if let SignType::Semicolon = sign_type {
                res = Some(ASTNode::Expression(ExpressionType::Null));
            }
        }

        res
    }

    fn parse_add_expressions(&mut self) -> ASTNode {
        let mut left = self.parse_multiply_expressions();
        let token = self.curr();

        dbg!(token.clone());

        while token == Token::Operator(OperatorType::Plus)
            || token == Token::Operator(OperatorType::Minus)
        {
            let operator = self.curr();
            dbg!(operator.clone());
            if operator != Token::Operator(OperatorType::Plus)
                && operator != Token::Operator(OperatorType::Minus)
            {
                break;
            }
            self.go();
            let operand = if operator == Token::Operator(OperatorType::Plus) {
                Operand::Plus
            } else {
                Operand::Minus
            };
            let right = self.parse_multiply_expressions();

            left = ASTNode::Expression(ExpressionType::Binary(Box::new(BinaryExpression {
                left: Box::new(left),
                right: Box::new(right),
                operand,
            })))
        }

        left
    }

    fn parse_variable_declaration(&mut self) -> ASTNode {
        let let_or_const = self.go();
        let is_immut = let_or_const == Token::Keyword(KeywordType::Immut);
        if is_immut {
            self.expect_token(
                Token::Keyword(KeywordType::Let),
                "After immut should follow the let keyword",
            );
        }

        let identifier: String;

        if let Token::Identifier(ident) = self.go() {
            identifier = ident;
        } else {
            panic!(
                "Cannot parse a variable declaration, as an identifier is not passed after `let`."
            );
        }

        if self.go() != Token::Operator(OperatorType::Equal) {
            if is_immut {
                panic!("Declaring immutables requires a value.")
            }
        }

        ASTNode::VariableDeclaration(is_immut, identifier, Box::new(self.parse_expressions()))
    }

    fn parse_multiply_expressions(&mut self) -> ASTNode {
        let mut left = self.parse_equality_expressions();
        let token = self.curr();

        dbg!(token.clone());

        while token == Token::Operator(OperatorType::Multiply)
            || token == Token::Operator(OperatorType::Divide)
        {
            let operator = self.curr();
            dbg!(operator.clone());
            if operator != Token::Operator(OperatorType::Multiply)
                && operator != Token::Operator(OperatorType::Divide)
            {
                break;
            }
            self.go();
            let operand = if operator == Token::Operator(OperatorType::Multiply) {
                Operand::Multiply
            } else {
                Operand::Divide
            };
            let right = self.parse_equality_expressions();

            left = ASTNode::Expression(ExpressionType::Binary(Box::new(BinaryExpression {
                left: Box::new(left),
                right: Box::new(right),
                operand,
            })))
        }

        left
    }

    fn parse_equality_expressions(&mut self) -> ASTNode {
        let mut left = self.parse_comparison_expressions();
        let token = self.curr();

        dbg!(token.clone());

        while token == Token::Sign(SignType::Equality) || token == Token::Sign(SignType::Inequality)
        {
            let operator = self.curr();
            dbg!(operator.clone());
            if operator != Token::Sign(SignType::Equality)
                && operator != Token::Sign(SignType::Inequality)
            {
                break;
            }
            self.go();
            let operand = if operator == Token::Sign(SignType::Equality) {
                Operand::Equality
            } else {
                Operand::Inequality
            };
            let right = self.parse_comparison_expressions();

            left = ASTNode::Expression(ExpressionType::Binary(Box::new(BinaryExpression {
                left: Box::new(left),
                right: Box::new(right),
                operand,
            })))
        }

        left
    }

    fn parse_comparison_expressions(&mut self) -> ASTNode {
        let mut left = self.parse_double_arrow_call();
        let token = self.curr();

        dbg!(token.clone());

        while token == Token::Operator(OperatorType::Bigger)
            || token == Token::Operator(OperatorType::Smaller)
            || token == Token::Operator(OperatorType::BiggerEqual)
            || token == Token::Operator(OperatorType::SmallerEqual)
        {
            let operator = self.curr();
            dbg!(operator.clone());
            if !(operator == Token::Operator(OperatorType::Bigger)
                || operator == Token::Operator(OperatorType::Smaller)
                || operator == Token::Operator(OperatorType::BiggerEqual)
                || operator == Token::Operator(OperatorType::SmallerEqual))
            {
                break;
            }
            self.go();
            let operand = match operator {
                Token::Operator(t) => match t {
                    OperatorType::Bigger => Operand::Bigger,
                    OperatorType::Smaller => Operand::Smaller,
                    OperatorType::BiggerEqual => Operand::BiggerEqual,
                    OperatorType::SmallerEqual => Operand::SmallerEqual,
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            };
            let right = self.parse_double_arrow_call();

            left = ASTNode::Expression(ExpressionType::Binary(Box::new(BinaryExpression {
                left: Box::new(left),
                right: Box::new(right),
                operand,
            })))
        }

        left
    }

    fn parse_variable_assignment(&mut self) -> ASTNode {
        let identifier_token = self.go();
        let _ = self.go(); // equals sign goes here
        let value = self.parse_function_call();

        if let Token::Identifier(identifier) = identifier_token {
            ASTNode::VariableAssignment(identifier, Box::new(value))
        } else {
            unreachable!()
        }
    }

    fn parse_self_assign_expression(&mut self) -> ASTNode {
        let _ = self.go(); // the self-assign operator;
        let identifier_token = self.curr();

        if let Token::Identifier(identifier) = identifier_token {
            ASTNode::VariableAssignment(identifier, Box::new(self.parse_expressions()))
        } else {
            panic!("Cannot use self-assign operator without an identifier.");
        }
    }

    fn parse_repeat_expression(&mut self) -> ASTNode {
        self.go();
        let operation = self.parse_expressions();
        if self.curr() == Token::Sign(SignType::Paren(Direction::Close)) {
            self.go();
        }
        let operator = self.curr();

        dbg!(operator.clone());

        if operator == Token::Operator(OperatorType::Repeat) {
            self.go(); // operator
            let num_token = self.parse_expressions();
            ASTNode::RepeatOperation(Box::new(num_token), Box::new(operation))
        } else {
            operation
        }
    }

    fn parse_fn_declaration(&mut self) -> ASTNode {
        self.go(); // fn keyword
        let identifier_token = self.curr();
        if let Token::Identifier(identifier) = identifier_token {
            self.go(); // identifier
            if self.curr() == Token::Sign(SignType::DoubleArrow) {
                self.parse_double_arrow_signature(identifier)
            } else if self.curr() == Token::Sign(SignType::Paren(Direction::Open)) {
                let args_list = self.parse_fn_args_list();

                self.expect_token(
                    Token::Sign(SignType::Arrow),
                    "Expected an arrow (->) after the arguments.",
                );

                let data_type_token = self.go();

                dbg!(data_type_token.clone());

                if let Token::Identifier(_data_type) = data_type_token {
                    self.expect_token(
                        Token::Sign(SignType::CurlyBrace(Direction::Open)),
                        "Expected a code block.",
                    );

                    let body = self.parse_code_block();

                    ASTNode::FunctionDeclaration(identifier, args_list, Box::new(body))
                } else {
                    panic!("Expected a data-type");
                }
            } else {
                panic!("Expected an opening parentheses.")
            }
        } else {
            panic!("Expecting an identifier after the `fn` keyword.")
        }
    }

    fn parse_fn_args_list(&mut self) -> HashMap<String, String> {
        self.go(); // paren
        let mut args_map: HashMap<String, String> = HashMap::new();
        while self.curr() != Token::Sign(SignType::Paren(Direction::Close)) {
            let (arg_name, data_type) = self.parse_fn_arg();
            if self.curr() == Token::Sign(SignType::Comma) {
                self.go();
            }
            args_map.insert(arg_name, data_type);
        }
        dbg!(self.go()); // paren
        args_map
    }

    fn parse_fn_arg(&mut self) -> (String, String) {
        let identifier_token = self.go();
        dbg!(identifier_token.clone());
        if let Token::Identifier(identifier) = identifier_token {
            if self.curr() == Token::Sign(SignType::Colon) {
                self.go(); // colon

                let data_type_token = self.go();
                if let Token::Identifier(data_type) = data_type_token {
                    (identifier, data_type)
                } else {
                    panic!("Expecting a data type identifier after the color (:)");
                }
            } else {
                panic!("Expecting a colon (:) after the argument name.")
            }
        } else {
            panic!("Expecting an identifier in the arguments list.")
        }
    }

    fn expect_token(&mut self, token: Token, reason: &str) {
        let tk = self.go();
        dbg!("Just removed {}", tk.clone());
        if token != tk {
            panic!("{}", reason)
        }
    }

    fn parse_code_block(&mut self) -> ASTNode {
        let mut nodes: Vec<ASTNode> = Vec::new();

        while self.curr() != Token::Sign(SignType::CurlyBrace(Direction::Close)) {
            nodes.push(self.parse_expressions());
        }
        self.go();

        ASTNode::FunctionBody(nodes)
    }

    fn parse_function_call(&mut self) -> ASTNode {
        if let Token::Identifier(identifier) = self.curr() {
            if self.next() == Token::Sign(SignType::Paren(Direction::Open)) {
                self.go();
                // self.go();
                let arg_list = self.parse_fn_call_arg_list();

                ASTNode::FunctionCall(identifier, arg_list)
            } else {
                self.parse_primary_expressions()
                    .unwrap_or(Expression(ExpressionType::Null))
            }
        } else {
            self.parse_primary_expressions()
                .unwrap_or(Expression(ExpressionType::Null))
        }
    }

    fn parse_fn_call_arg_list(&mut self) -> Vec<ASTNode> {
        self.expect_token(
            Token::Sign(SignType::Paren(Direction::Open)),
            "Expected an opening paren.",
        );

        let mut list: Vec<ASTNode> = vec![self.parse_expressions()];
        let mut tk = self.go();

        while tk == Token::Sign(SignType::Comma) && !self.is_end() {
            list.push(self.parse_expressions());
            tk = self.go();
        }

        // self.expect_token(Token::Sign(SignType::Paren(Direction::Close)), "Expected a closing paren.");

        list
    }

    // fn parse_if_statement(&mut self) -> ASTNode {
    //
    // }

    fn parse_double_arrow_signature(&mut self, identifier: String) -> ASTNode {
        self.go(); // ->>
        if let Token::Identifier(arg_type) = self.go() {
            self.expect_token(
                Token::Sign(SignType::Arrow),
                "Expected an arrow, then a data-type",
            );

            if let Token::Identifier(data_type) = self.go() {
                let mut args = HashMap::new();

                args.insert(String::from("it"), arg_type);

                self.expect_token(
                    Token::Sign(SignType::CurlyBrace(Direction::Open)),
                    "Expected a code block.",
                );

                let body = self.parse_code_block();

                ASTNode::FunctionDeclaration(identifier, args, Box::new(body))
            } else {
                panic!("Expected an identifier for the result's data-type!")
            }
        } else {
            panic!("Expected an identifier for the argument's data-type!")
        }
    }

    fn parse_double_arrow_call(&mut self) -> ASTNode {
        let mut left = self.parse_function_call();
        let token = self.curr();

        dbg!(token.clone());

        while token == Token::Sign(SignType::DoubleArrow) {
            let operator = self.curr();
            dbg!(operator.clone());
            if operator != Token::Sign(SignType::DoubleArrow) {
                break;
            }
            self.go();
            let right = self.parse_function_call();

            // left = ASTNode::Expression(ExpressionType::Binary(Box::new(BinaryExpression {
            //     left: Box::new(left),
            //     right: Box::new(right),
            //     operand,
            // })))
            
            if let ASTNode::Identifier(id) = right {
                left = ASTNode::FunctionCall(
                    id,
                    vec![left.clone()]
                )
            }
        }

        left
    }

    // fn parse_double_arrow_calls(body: Vec<ASTNode>) -> Vec<ASTNode> {
    //     let mut new_nodes = vec![];
    //     let mut hm: HashMap<usize, bool> = HashMap::new();
    //
    //     for (i, el) in body.iter().enumerate() {
    //         if el == &ASTNode::Misc(MiscNodeType::DoubleArrow) {
    //             let expr_idx = i - 1;
    //             let fn_idx = i + 1;
    //             hm.insert(
    //                 expr_idx,
    //                 true
    //             );
    //             hm.insert(
    //                 fn_idx,
    //                 true
    //             );
    //             let exp = body.get(expr_idx).expect("Expected an expression").clone();
    //             let fn_v = body.get(fn_idx).expect("Expected a function name identifier").clone();
    //
    //             if let ASTNode::Identifier(name) = fn_v {
    //                 new_nodes.pop();
    //                 new_nodes.push(ASTNode::FunctionCall(
    //                     name,
    //                     vec![exp]
    //                 ))
    //             }
    //         } else {
    //             if !hm.get(&i).unwrap_or(&false) {
    //                 new_nodes.push(el.clone())
    //             }
    //         }
    //     }
    //
    //     new_nodes
    // }
}
