use crate::lexer::structs::{Direction, KeywordType, OperatorType, SignType, Token};
use crate::lexer::tokenize;
use crate::parser::structs::{ASTNode, BinaryExpression, ExpressionType, Operand};
use std::collections::VecDeque;

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
            body.push(self.parse_expressions());
        }

        ASTNode::Program(body)
    }

    fn parse_expressions(&mut self) -> ASTNode {
         match self.curr() {
             Token::Keyword(keyword) => {
                 match keyword {
                     KeywordType::Let => self.parse_variable_declaration(),
                     KeywordType::Const => self.parse_variable_declaration(),
                     _ => ASTNode::Expression(ExpressionType::Null)
                 }
             },
             Token::Identifier(_) => {
                 if self.next() == Token::Operator(OperatorType::Equal) {
                     self.parse_variable_assignment()
                 } else {
                     self.parse_add_expressions()
                 }
             },
             Token::Boolean(value) => {
                 self.go();
                 ASTNode::Boolean(value)
             },
             Token::Operator(operator_type) => {
                 if operator_type == OperatorType::SelfAssign {
                     self.parse_self_assign_expression()
                 } else {
                     self.parse_add_expressions()
                 }
             },
             Token::Sign(sign_type) => {
                 if sign_type == SignType::Paren(Direction::Open) {
                     self.parse_repeat_expression()
                 } else {
                     self.parse_add_expressions()
                 }
             }
             _ => self.parse_add_expressions()
         }
    }

    fn parse_primary_expressions(&mut self) -> ASTNode {
        // self.parse_add_expressions()

        let token = self.go();

        let mut res: ASTNode = ASTNode::Expression(ExpressionType::Null);

        if let Token::Sign(sign_type) = &token {
            if let SignType::Paren(direction) = sign_type {
                if *direction == Direction::Open {
                    res = self.parse_expressions();
                    // dbg!("Removed", self.go());
                }
            }
        }

        if let Token::String(v) = token.clone() {
            res = ASTNode::String(v);
        } else if let Token::Number(v) = token.clone() {
            res = ASTNode::Number(v);
        } else if let Token::Identifier(v) = token.clone() {
            res = ASTNode::Identifier(v);
        } else if let Token::Sign(sign_type) = token.clone() {
            if let SignType::Semicolon = sign_type {
                res = ASTNode::Expression(ExpressionType::Null);
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
        let is_let = let_or_const == Token::Keyword(KeywordType::Let);

        let identifier: String;

        if let Token::Identifier(ident) = self.go() {
            identifier = ident;
        } else {
            panic!("Cannot parse a variable declaration, as an identifier is not passed after `let`.");
        }

        if self.go() != Token::Operator(OperatorType::Equal) {
            if !is_let {
                panic!("Declaring constants requires a value.")
            }
        }

        ASTNode::VariableDeclaration(
            is_let,
            identifier,
            Box::new(self.parse_expressions())
        )
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
        let mut left = self.parse_primary_expressions();
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
            let right = self.parse_primary_expressions();

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
        let value = self.parse_expressions();

        if let Token::Identifier(identifier) = identifier_token {
            ASTNode::VariableAssignment(
                identifier,
                Box::new(value)
            )
        } else {
            unreachable!()
        }
    }

    fn parse_self_assign_expression(&mut self) -> ASTNode {
        let _ = self.go(); // the self-assign operator;
        let identifier_token = self.curr();

        if let Token::Identifier(identifier) = identifier_token {
            ASTNode::VariableAssignment(
                identifier,
                Box::new(self.parse_expressions())
            )
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
            ASTNode::RepeatOperation(
                Box::new(num_token),
                Box::new(operation)
            )
        } else {
            operation
        }
    }
}
