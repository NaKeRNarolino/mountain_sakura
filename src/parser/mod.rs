use crate::lexer::structs::{Direction, KeywordType, OperatorType, SignType, Token};
use crate::lexer::tokenize;
use crate::parser::structs::{ASTNode, AssignmentProperty, BinaryExpression, ExpressionType, IfStatement, LayoutCreation, Operand};
use std::collections::{HashMap, VecDeque};
use crate::parser::structs::{FieldParserDescription, LayoutDeclaration};
use crate::parser::structs::ForStatement;
use crate::parser::structs::UseNative;
use crate::parser::structs::OnceStatement;

pub mod structs;

#[derive(Clone)]
pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new(source: String) -> Self {
        let tokens = tokenize(source);

        dbg!(&tokens);

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

    fn peek(&mut self) -> Token {
        self.tokens.get(1).cloned().unwrap_or(Token::End)
    }

    pub fn gen_ast(&mut self) -> ASTNode {
        let mut body: Vec<ASTNode> = vec![];

        while !self.is_end() {
            body.push(self.parse_expressions())
        }

        ASTNode::Program(body)
    }

    fn parse_expressions(&mut self) -> ASTNode {
        dbg!(&&&self.curr());
        match self.curr() {
            Token::Keyword(keyword) => match keyword {
                KeywordType::Let => self.parse_variable_declaration(),
                KeywordType::Const => self.parse_variable_declaration(),
                KeywordType::Immut => self.parse_variable_declaration(),
                KeywordType::Fn => self.parse_fn_declaration(),
                KeywordType::If => self.parse_if_declaration(),
                KeywordType::Once => self.parse_once_declaration(),
                KeywordType::Use => self.parse_use(),
                KeywordType::Block => {
                    self.go(); // `block`
                    self.parse_code_block()
                },
                KeywordType::For => self.parse_for_expression(),
                KeywordType::Enum => self.parse_enum_declaration(),
                KeywordType::Typeof => {
                    self.go(); // `typeof`
                    let v = self.parse_start_expr();
                    
                    ASTNode::Typeof(Box::new(v))
                },
                KeywordType::Layout => self.parse_layout_declaration(),
                _ => ASTNode::Expression(ExpressionType::Null),
            },
            Token::Identifier(_) => {
                self.parse_start_expr()
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
                    self.parse_start_expr()
                }
            }
            _ => self.parse_start_expr(),
        }
    }

    fn parse_primary_expressions(&mut self) -> Option<ASTNode> {
        // self.parse_add_expressions()

        let token = self.curr();

        let res = Some(ASTNode::Expression(ExpressionType::Null));

        if let Token::Sign(sign_type) = &token {
            if let SignType::Paren(direction) = sign_type {
                return if *direction == Direction::Open {
                    self.go();
                    Some(self.parse_expressions())
                    // dbg!("Removed", self.go());
                } else {
                    self.go();
                    None
                }
            }
        }

        if let Token::String(v) = token.clone() {
            self.go();
            return Some(ASTNode::String(v));
        } else if let Token::Number(v) = token.clone() {
            self.go();
            return Some(ASTNode::Number(v));
        } else if let Token::Identifier(v) = token.clone() {
            self.go();
            if self.curr() == Token::Sign(SignType::SlashArrow) {
                return Some(self.parse_enum_access(&v))
            } else if self.curr() == Token::Sign(SignType::CurlyBrace(Direction::Open)) {
                return Some(self.parse_layout_creation(&v))
            } else if self.curr() == Token::Sign(SignType::Dot) {
                return Some(self.parse_layout_property_access(&v))
            }
            return Some(ASTNode::Identifier(v));
        } else if let Token::Sign(sign_type) = token.clone() {
            if let SignType::Semicolon = sign_type {
                self.go();
                return Some(ASTNode::Expression(ExpressionType::Null));
            } else if sign_type == SignType::Caret {
                return Some(self.parse_binding_access())
            } else {
                if sign_type != SignType::CurlyBrace(Direction::Close) {
                    self.go();
                }
            }
        }

        res
    }

    fn parse_start_expr(&mut self) -> ASTNode {
        self.parse_variable_assignment()
    }

    fn parse_double_dot_expressions(&mut self) -> ASTNode {
        let mut left = self.parse_add_expressions();
        let token = self.curr();

        while token == Token::Sign(SignType::DoubleDot)
        {
            let operator = self.curr();
            if operator != Token::Sign(SignType::DoubleDot) {
                break;
            }
            self.go();
            let operand = Operand::DoubleDot;
            let right = self.parse_add_expressions();

            left = ASTNode::Expression(ExpressionType::Binary(Box::new(BinaryExpression {
                left: Box::new(left),
                right: Box::new(right),
                operand,
            })))
        }

        left
    }

    fn parse_add_expressions(&mut self) -> ASTNode {
        let mut left = self.parse_multiply_expressions();
        let token = self.curr();

        while token == Token::Operator(OperatorType::Plus)
            || token == Token::Operator(OperatorType::Minus)
        {
            let operator = self.curr();
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
        let mut type_id: Option<String> = None;

        if let Token::Identifier(ident) = self.go() {
            identifier = ident;
        } else {
            panic!(
                "Cannot parse a variable declaration, as an identifier is not passed after `let`."
            );
        }
        
        if self.curr() != Token::Sign(SignType::Colon) {
            if is_immut {
                panic!("Cannot declare an immutable variable without a type.")
            }
        } else {
            self.go();
            
            if let Token::Identifier(r#type) = self.go() {
                type_id = Some(r#type)
            } else {
                panic!("Expected a type name after `:`");
            }
        }

        if self.go() != Token::Operator(OperatorType::Equal) {
            if is_immut {
                panic!("Declaring immutables requires a value.")
            }
        }
        
        let expr = self.parse_expressions();

        ASTNode::VariableDeclaration(is_immut, identifier, type_id.unwrap_or(String::from("1MOSA_UNDEFINED")), Box::new(expr))
    }

    fn parse_multiply_expressions(&mut self) -> ASTNode {
        let mut left = self.parse_equality_expressions();
        let token = self.curr();


        while token == Token::Operator(OperatorType::Multiply)
            || token == Token::Operator(OperatorType::Divide)
        {
            let operator = self.curr();
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


        while token == Token::Sign(SignType::Equality) || token == Token::Sign(SignType::Inequality)
        {
            let operator = self.curr();
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


        while token == Token::Operator(OperatorType::Bigger)
            || token == Token::Operator(OperatorType::Smaller)
            || token == Token::Operator(OperatorType::BiggerEqual)
            || token == Token::Operator(OperatorType::SmallerEqual)
        {
            let operator = self.curr();
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
        // let identifier_token = self.go();
        // let _ = self.go(); // equals sign goes here
        // let value = self.parse_expressions();
        //
        // if let Token::Identifier(identifier) = identifier_token {
        //     ASTNode::Assignment(AssignmentProperty::Variable(identifier), Box::new(value))
        // } else {
        //     unreachable!()
        // }

        let mut left = self.parse_double_dot_expressions();
        let token = self.curr();

        while token == Token::Operator(OperatorType::Equal)
        {
            let operator = self.curr();
            if operator != Token::Operator(OperatorType::Equal) {
                break;
            }
            self.go();
            let right = self.parse_double_dot_expressions();

            left = ASTNode::Assignment(
                Self::get_assignment_property(&left),
                Box::new(right)
            )
        }

        left
    }

    fn parse_self_assign_expression(&mut self) -> ASTNode {
        let _ = self.go(); // the self-assign operator;

        let assignment_thing = self.clone().parse_primary_expressions().unwrap();

        let expr = self.parse_expressions();

        ASTNode::Assignment(Self::get_assignment_property(&assignment_thing), Box::new(expr))


        // let identifier_token = self.curr();
        //
        // if let Token::Identifier(identifier) = identifier_token {
        //     } else {
        //     panic!("Cannot use self-assign operator without an identifier.");
        // }
    }

    fn parse_repeat_expression(&mut self) -> ASTNode {
        self.go();
        let operation = self.parse_expressions();
        if self.curr() == Token::Sign(SignType::Paren(Direction::Close)) {
            self.go();
        }
        let operator = self.curr();


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
        self.go();
        args_map
    }

    fn parse_fn_arg(&mut self) -> (String, String) {
        let identifier_token = self.go();
        // dbg!(identifier_token.clone());
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
        // dbg!("Just removed {}", tk.clone());
        if token != tk {
            panic!("{}", reason)
        }
    }

    fn parse_code_block(&mut self) -> ASTNode {
        let mut nodes: Vec<ASTNode> = Vec::new();

        while self.curr() != Token::Sign(SignType::CurlyBrace(Direction::Close)) && !self.is_end() {
            nodes.push(self.parse_expressions());
        }
        self.go();

        ASTNode::CodeBlock(nodes)
    }

    fn parse_function_call(&mut self) -> ASTNode {
        if let Token::Identifier(identifier) = self.curr() {
            if self.peek() == Token::Sign(SignType::Paren(Direction::Open)) {
                self.go();
                // self.go();
                let arg_list = self.parse_fn_call_arg_list();

                ASTNode::FunctionCall(identifier, arg_list)
            } else {
                self.parse_primary_expressions()
                    .unwrap_or(ASTNode::Expression(ExpressionType::Null))
            }
        } else {
            self.parse_primary_expressions()
                .unwrap_or(ASTNode::Expression(ExpressionType::Null))
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
            dbg!(&tk);
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

        // dbg!(token.clone());

        while token == Token::Sign(SignType::DoubleArrow) {
            let operator = self.curr();
            // dbg!(operator.clone());
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

    fn parse_if_declaration(&mut self) -> ASTNode {
        self.go(); // if

        let condition = Box::new(self.parse_expressions());

        let block = Box::new(self.parse_code_block());

        let mut else_block: Option<Box<ASTNode>> = None;

        // dbg!(self.peek(), self.curr());

        //
        if self.curr() == Token::Keyword(KeywordType::Else) {
            self.go(); // else;
            else_block = Some(Box::new(self.parse_code_block()));
        }

        ASTNode::IfStatement(
            IfStatement {
                condition,
                if_block: block,
                else_block
            }
        )
    }

    fn parse_once_declaration(&mut self) -> ASTNode {
        self.go(); // `once`

        // self.expect_token(Token::Sign(SignType::CurlyBrace(Direction::Open)), "Expecting an opening curly brace.");

        let tk = self.go();
        let mut ifs: Vec<IfStatement> = Vec::new();
        let mut else_block: Option<Box<ASTNode>> = None;

        while self.curr() != Token::Sign(SignType::CurlyBrace(Direction::Close)) && !self.is_end() {
            if let ASTNode::IfStatement(if_st) = self.parse_if_declaration() {
                ifs.push(if_st);
                // self.expect_token(Token::Sign(SignType::Semicolon), "Expected a semicolon");
            }
            // tk = self.go();
            dbg!(&&tk);
        }
        self.go();

        if self.curr() == Token::Keyword(KeywordType::Else) {
            self.go(); // `else`
            else_block = Some(Box::new(self.parse_code_block()));
        }

        ASTNode::OnceStatement(OnceStatement {
            else_block,
            if_statements: ifs
        })
    }

    fn parse_use(&mut self) -> ASTNode {
        self.go(); // `use`
        if self.curr() == Token::Keyword(KeywordType::Native) {
            self.parse_use_native()
        } else {
            ASTNode::Expression(ExpressionType::Null)
        }
    }

    fn parse_use_native(&mut self) -> ASTNode {
        self.go(); // `native`

        if Token::Keyword(KeywordType::Fn) == self.go() {
            if let Token::Identifier(identifier) = self.go() {
                self.expect_token(Token::Sign(SignType::HashSign), &format!("Expected a `#` after `use native {}", &identifier));

                if let Token::String(from) = self.go() {
                    ASTNode::UseNative(UseNative {
                        name: identifier,
                        from,
                    })
                } else {
                    panic!("Expected a string to qualify the path")
                }
            } else {
                panic!("Expected an identifier after `use native`")
            }
        } else {
            unreachable!()
        }
    }

    fn parse_binding_access(&mut self) -> ASTNode {
        self.go(); // `^`

        if let Token::Identifier(identifier) = self.go() {
            ASTNode::BindingAccess(identifier)
        } else {
            panic!("Expected an identifier for binding access")
        }
    }
    
    fn parse_for_expression(&mut self) -> ASTNode {
        self.go(); // `for`
        
        let iterable = self.parse_expressions();
        let block = self.parse_code_block();
        
        ASTNode::ForStatement(
            ForStatement {
                iterable: Box::new(iterable),
                block: Box::new(block),
            }
        )
    }

    fn parse_enum_access(&mut self, entry: &String) -> ASTNode {
        self.go(); // `/>`

        if let Token::Identifier(identifier) = self.go() {
            ASTNode::EnumAccessor(entry.clone(), identifier)
        } else {
            panic!("Expected an identifier after `/>` to access an entry from enum `{}`", entry)
        }
    }

    fn parse_enum_declaration(&mut self) -> ASTNode {
        self.go(); // `enum`

        if let Token::Identifier(identifier) = self.go() {
            if self.go() != Token::Sign(SignType::CurlyBrace(Direction::Open)) {
                panic!("Expected an opening curly brace (`{{`).")
            }

            let entries = self.parse_enum_entries();

            // if self.go() != Token::Sign(SignType::CurlyBrace(Direction::Close)) {
            //     panic!("Expected a closing curly brace (`}}`).")
            // }

            ASTNode::EnumDeclaration(
                identifier,
                entries
            )
        } else {
            panic!("Expected enum identifier.")
        }
    }

    fn parse_enum_entries(&mut self) -> Vec<String> {
        let mut res = vec![];

        if let Token::Identifier(id) = self.go() {
            res.push(id);
        }


        let mut tk = self.go();

        if tk == Token::Sign(SignType::Comma) && self.curr() == Token::Sign(SignType::CurlyBrace(Direction::Close)) {
            tk = self.go();
        }

        while tk == Token::Sign(SignType::Comma) && self.curr() != Token::Sign(SignType::CurlyBrace(Direction::Close)) && !self.is_end() {
            if let Token::Identifier(id) = self.go() {
                res.push(id);
            }
            dbg!(&self.curr());
            tk = self.go();
            if tk == Token::Sign(SignType::Comma) && self.curr() == Token::Sign(SignType::CurlyBrace(Direction::Close)) {
                tk = self.go();
            }
        }

        dbg!(&tk);
        if tk != Token::Sign(SignType::CurlyBrace(Direction::Close)) {
            panic!("Expected a closing curly brace (`}}`).")
        }

        res
    }

    fn parse_layout_declaration(&mut self) -> ASTNode {
        self.go(); // `layout`

        let identifier: String;

        if let Token::Identifier(ident) = self.go() {
            identifier = ident;
        } else {
            panic!("Expected an identifier after `layout`")
        }

        if self.go() != Token::Sign(SignType::CurlyBrace(Direction::Open)) {
            panic!("Expected an opening curly brace (`{{`).")
        }

        let entries = self.parse_layout_entries();

        ASTNode::LayoutDeclaration(LayoutDeclaration {
            name: identifier,
            fields: entries,
        })
    }

    fn parse_layout_entries(&mut self) -> HashMap<String, FieldParserDescription> {
        let mut res = HashMap::new();

        if let Token::Identifier(id) = self.curr() {
            res.insert(id, self.parse_layout_single_entry().unwrap().1);
        }

        let mut tk = self.go();

        if self.curr() == Token::Sign(SignType::CurlyBrace(Direction::Close)) {
            tk = self.go();
        }

        while tk == Token::Sign(SignType::Comma) && self.curr() != Token::Sign(SignType::CurlyBrace(Direction::Close)) && !self.is_end() {
            if let Token::Identifier(id) = self.curr() {
                res.insert(id, self.parse_layout_single_entry().unwrap().1);
            }
            // if self.curr() == Token::Sign(SignType::Comma) {
            //     self.go();
            // }
            tk = self.go();
            if tk == Token::Sign(SignType::Comma) && self.curr() == Token::Sign(SignType::CurlyBrace(Direction::Close)) {
                tk = self.go();
            }
            dbg!(&tk);
        }

        if tk != Token::Sign(SignType::CurlyBrace(Direction::Close)) {
            panic!("Expected a closing curly brace (`}}`).")
        }

        res
    }

    fn parse_layout_single_entry(&mut self) -> Option<(String, FieldParserDescription)> {
        if let Token::Identifier(id) = self.go() {
            if self.go() != Token::Sign(SignType::Colon) {
                panic!("Expected a colon after `{}`", id)
            }

            let type_id: String;

            if let Token::Identifier(r#type) = self.go() {
                type_id = r#type;
            } else {
                panic!("Expected a type identifier after `{}:`", id);
            }

            let maybe_default_type = (self.curr() == Token::Operator(OperatorType::Equal)).then(|| {
                self.go(); // '='

                Box::new(self.parse_expressions())
            });

            Some((id, FieldParserDescription {
                type_id,
                default_value: maybe_default_type
            }))
        } else {
            None
        }
    }

    fn parse_layout_creation(&mut self, name: &String) -> ASTNode {
        self.go(); // {

        let entries = self.parse_layout_creation_entries();
        
        ASTNode::LayoutCreation(
            LayoutCreation {
                name: name.clone(),
                specified_fields: entries,
            }
        )
    }

    fn parse_layout_creation_entries(&mut self) -> HashMap<String, Box<ASTNode>> {
        let mut res = HashMap::new();

        if let Token::Identifier(id) = self.curr() {
            res.insert(id, self.parse_layout_creation_single_entry().unwrap().1);
        }

        let mut tk = self.go();

        if self.curr() == Token::Sign(SignType::CurlyBrace(Direction::Close)) {
            tk = self.go();
        }

        while tk == Token::Sign(SignType::Comma) && self.curr() != Token::Sign(SignType::CurlyBrace(Direction::Close)) && !self.is_end() {
            if let Token::Identifier(id) = self.curr() {
                res.insert(id, self.parse_layout_creation_single_entry().unwrap().1);
            }
            tk = self.go();
            if tk == Token::Sign(SignType::Comma) && self.curr() == Token::Sign(SignType::CurlyBrace(Direction::Close)) {
                tk = self.go();
            }
            dbg!(&tk);
        }

        if tk != Token::Sign(SignType::CurlyBrace(Direction::Close)) {
            panic!("Expected a closing curly brace (`}}`).")
        }

        res
    }

    fn parse_layout_creation_single_entry(&mut self) -> Option<(String, Box<ASTNode>)> {
        if let Token::Identifier(id) = self.go() {
            if self.go() != Token::Operator(OperatorType::Equal) {
                panic!("Expected a colon after `{}`", id)
            }

            // self.go(); // '='

            let exp = Box::new(self.parse_expressions());

            Some((id, exp))
        } else {
            None
        }
    }

    fn get_assignment_property(node: &ASTNode) -> AssignmentProperty {
        if let ASTNode::Identifier(id) = node {
            AssignmentProperty::Variable(id.clone())
        } else if let ASTNode::LayoutFieldAccess(id, prop) = node {
            AssignmentProperty::LayoutField(id.clone(), prop.clone())
        } else {
            panic!("Cannot make the expression into assignment-prop.")
        }
    }

    fn parse_layout_property_access(&mut self, id: &String) -> ASTNode {
        self.go(); // `.`

        if let Token::Identifier(field) = self.go() {
            ASTNode::LayoutFieldAccess(
                id.clone(),
                field
            )
        } else {
            panic!("Expected a field name to access from `{}`.", id)
        }
    }
}
