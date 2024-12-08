use std::collections::VecDeque;
use crate::lexer::structs::{Direction, OperatorType, SignType, Token};
use crate::lexer::tokenize;
use crate::parser::structs::{ASTNode, BinaryExpression, ExpressionType, Operand};

pub mod structs;

pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new(source: String) -> Self {
        let tokens = tokenize(source);

        dbg!(tokens.clone());

        Self {
            tokens
        }
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

    pub fn gen_ast(&mut self) -> ASTNode {
        let mut body: Vec<ASTNode> = vec![];

        while !self.is_end() {
            dbg!(self.curr());
            body.push(self.parse_expressions());
        }

        ASTNode::Program(body)
    }

    fn parse_expressions(&mut self) -> ASTNode {
        self.parse_add_expressions()
    }

    fn parse_primary_expressions(&mut self) -> ASTNode {
        // self.parse_add_expressions()

        let token = self.go();

        let mut res: ASTNode = ASTNode::Expression(ExpressionType::Primary);

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
        } else {

        }

        res
    }

    fn parse_add_expressions(&mut self) -> ASTNode {
        let mut left = self.parse_multiply_expressions();
        let token = self.curr();

        dbg!(token.clone());

        while token == Token::Operator(OperatorType::Plus) || token == Token::Operator(OperatorType::Minus) {
            let operator = self.go();
            dbg!(operator.clone());
            if operator != Token::Operator(OperatorType::Plus) && operator != Token::Operator(OperatorType::Minus) {
                break;
            }
            let operand = if operator == Token::Operator(OperatorType::Plus) {
                Operand::Plus
            } else {
                Operand::Minus
            };
            let right = self.parse_multiply_expressions();

            left = ASTNode::Expression(ExpressionType::Binary(
                Box::new(BinaryExpression {
                    left: Box::new(left), right: Box::new(right), operand
                })
            ))
        }

        left
    }

    fn parse_multiply_expressions(&mut self) -> ASTNode {
        let mut left = self.parse_primary_expressions();
        let token = self.curr();

        dbg!(token.clone());

        while token == Token::Operator(OperatorType::Multiply) || token == Token::Operator(OperatorType::Divide) {
            let operator = self.go();
            dbg!(operator.clone());
            if operator != Token::Operator(OperatorType::Multiply) && operator != Token::Operator(OperatorType::Divide) {
                break;
            }
            let operand = if operator == Token::Operator(OperatorType::Multiply) {
                Operand::Multiply
            } else {
                Operand::Divide
            };
            let right = self.parse_primary_expressions();

            left = ASTNode::Expression(ExpressionType::Binary(
                Box::new(BinaryExpression {
                    left: Box::new(left), right: Box::new(right), operand
                })
            ))
        }

        left
    }
}