use crate::global::ComplexDataType;
use crate::global::ReferenceType::Function;
use crate::global::{DataType, PrimitiveDataType};
use crate::interpreter::scope::{FunctionData, ScopeLayoutDeclaration};
use crate::lexer::structs::{Direction, KeywordType, OperatorType, SignType, Token, TokenValue};
use crate::lexer::tokenize;
use crate::modules::ModuleExport;
use crate::modules::{Module, ModuleStorage};
use crate::mosa_fs;
use crate::parser::structs::OnceStatement;
use crate::parser::structs::UseNative;
use crate::parser::structs::{
    ASTNode, AssignmentProperty, BinaryExpression, ExpressionType, IfStatement, LayoutCreation,
    Operand,
};
use crate::parser::structs::{FieldParserDescription, LayoutDeclaration};
use crate::parser::structs::{ForStatement, ParserFunctionData};
use crate::{err, logging};
use indexmap::IndexMap;
use std::collections::{HashMap, VecDeque};
use std::process::exit;
use std::sync::{Arc, RwLock};

pub mod structs;

#[derive(Clone)]
pub struct Parser {
    tokens: VecDeque<Token>,
    last: Token,
    module: Arc<Module>,
    module_storage: Arc<ModuleStorage>,
    root: String,
    relative_root: String,
    should_end: bool,
}

impl Parser {
    pub fn file_name(&self) -> String {
        self.module.name()
    }

    pub fn set_end(&mut self) {
        self.should_end = true
    }

    pub fn new(
        source: String,
        module: Module,
        module_storage: Arc<ModuleStorage>,
        root: String,
        path: String,
    ) -> Self {
        let tokens = tokenize(module.name(), source);

        dbg!(&tokens);

        Self {
            tokens,
            last: Parser::end_t(),
            module: Arc::new(module),
            module_storage,
            root,
            relative_root: path,
            should_end: false,
        }
    }

    fn is_end(&self) -> bool {
        if self.tokens.is_empty() {
            true
        } else {
            self.tokens[0].value == TokenValue::End
        }
    }

    fn end(&self) -> Token {
        Token {
            value: TokenValue::End,
            line: 0,
            column: 0,
            file_name: self.file_name(),
        }
    }

    fn end_t() -> Token {
        Token {
            value: TokenValue::End,
            line: 0,
            column: 0,
            file_name: "FILE_NAME".to_string(),
        }
    }

    fn curr(&self) -> Token {
        if self.tokens.is_empty() {
            self.end()
        } else {
            self.tokens[0].clone()
        }
    }

    fn go(&mut self) -> Token {
        self.last = self.tokens.pop_front().unwrap_or(self.end()).clone();
        self.last.clone()
    }

    fn last(&self) -> Token {
        self.last.clone()
    }

    fn peek(&mut self) -> Token {
        self.tokens.get(1).cloned().unwrap_or(self.end())
    }

    pub fn gen_ast(&mut self) -> ASTNode {
        let mut body: Vec<ASTNode> = vec![];

        while !self.is_end() {
            let parse = self.parse_expressions();
            if let ASTNode::InternalMulti(mut nodes) = parse {
                body.append(&mut nodes);
            } else {
                body.push(parse);
            }

            if self.should_end {
                // exit(100);
                self.go();
            }
        }

        self.module.set_ast(body.clone());
        self.module_storage.push((*self.module).clone());

        ASTNode::Program(body)
    }

    fn parse_expressions(&mut self) -> ASTNode {
        dbg!(&&&self.curr());
        match self.curr().value {
            TokenValue::Keyword(keyword) => match keyword {
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
                }
                KeywordType::For => self.parse_for_expression(),
                KeywordType::Enum => self.parse_enum_declaration(),
                KeywordType::Typeof => {
                    self.go(); // `typeof`
                    let v = self.parse_start_expr();

                    ASTNode::Typeof(Box::new(v))
                }
                KeywordType::Layout => self.parse_layout_declaration(),
                KeywordType::Mix => self.parse_mix(None),
                KeywordType::Exp => self.parse_exp(),
                _ => ASTNode::Expression(ExpressionType::Null),
            },
            TokenValue::Identifier(_) => self.parse_start_expr(),
            TokenValue::Boolean(value) => {
                self.go();
                ASTNode::Boolean(value)
            }
            TokenValue::Operator(operator_type) => {
                if operator_type == OperatorType::SelfAssign {
                    self.parse_self_assign_expression()
                } else {
                    self.parse_add_expressions()
                }
            }
            TokenValue::Sign(sign_type) => {
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

        if let TokenValue::Sign(sign_type) = &token.value {
            match sign_type {
                SignType::Paren(direction) => {
                    return if *direction == Direction::Open {
                        self.go();
                        Some(self.parse_expressions())
                        // dbg!("Removed", self.go());
                    } else {
                        self.go();
                        None
                    };
                }
                SignType::DoubleColon => {
                    self.go();
                    return match self.parse_fn_lower("MOSA_INTERNAL_LAMBDA".to_string()) {
                        ASTNode::FunctionDeclaration(_n, a, b, r) => Some(ASTNode::Lambda(a, b, r)),
                        _ => unreachable!(),
                    };
                }
                _ => {}
            }
        }

        if let TokenValue::String(v) = token.clone().value {
            self.go();
            return Some(ASTNode::String(v));
        } else if let TokenValue::Number(v) = token.clone().value {
            self.go();
            return Some(ASTNode::Number(v));
        } else if let TokenValue::Identifier(v) = token.clone().value {
            self.go();
            if self.curr().value == TokenValue::Sign(SignType::Arrow) {
                return Some(self.parse_complex_type_access(&v));
            } else if self.curr().value == TokenValue::Sign(SignType::CurlyBrace(Direction::Open)) {
                return Some(self.parse_layout_creation(&v));
            } else if self.curr().value == TokenValue::Sign(SignType::Dot) {
                return Some(self.parse_layout_property_access(&v));
            }
            return Some(ASTNode::Identifier(v));
        } else if let TokenValue::Sign(sign_type) = token.clone().value {
            if let SignType::Semicolon = sign_type {
                self.go();
                return Some(ASTNode::Expression(ExpressionType::Null));
            } else if sign_type == SignType::Caret {
                return Some(self.parse_binding_access());
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

        while token.value == TokenValue::Sign(SignType::DoubleDot) {
            let operator = self.curr();
            if operator.value != TokenValue::Sign(SignType::DoubleDot) {
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

        while token.value == TokenValue::Operator(OperatorType::Plus)
            || token.value == TokenValue::Operator(OperatorType::Minus)
        {
            let operator = self.curr();
            if operator.value != TokenValue::Operator(OperatorType::Plus)
                && operator.value != TokenValue::Operator(OperatorType::Minus)
            {
                break;
            }
            self.go();
            let operand = if operator.value == TokenValue::Operator(OperatorType::Plus) {
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
        let is_immut = let_or_const.value == TokenValue::Keyword(KeywordType::Immut);
        if is_immut {
            self.expect_token(
                TokenValue::Keyword(KeywordType::Let),
                "After immut should follow the let keyword",
            );
        }

        let identifier: String;
        let tk = self.go();

        if let TokenValue::Identifier(ident) = tk.value {
            identifier = ident;
        } else {
            logging::err!(
                &tk.file_name,
                tk.line,
                tk.column,
                "Cannot parse a variable declaration, as an identifier is not passed after `let`.",
            );
            self.set_end();
            return ASTNode::InternalStop(tk.line, tk.file_name);
        }

        let mut data_type = DataType::InternalInfer;

        if self.curr().value != TokenValue::Sign(SignType::Colon) {
            if is_immut {
                let tk = self.curr();
                logging::err!(
                    &self.curr().file_name,
                    self.curr().line,
                    self.curr().column,
                    "Cannot declare an immutable without a type."
                );
                self.set_end();
                return ASTNode::InternalStop(self.curr().line, self.curr().file_name);
            }
        } else {
            self.go();

            data_type = self.parse_data_type();
        }

        if self.go().value != TokenValue::Operator(OperatorType::Equal) {
            if is_immut {
                let tok = self.curr();
                logging::err!(
                    &tok.file_name,
                    tok.line,
                    tok.column,
                    "Declaring immutables requires a value.",
                );
                self.set_end();
                return ASTNode::InternalStop(tok.line, tok.file_name);
            }
        }

        let expr = self.parse_expressions();

        ASTNode::VariableDeclaration(is_immut, identifier, data_type, Box::new(expr))
    }

    fn parse_multiply_expressions(&mut self) -> ASTNode {
        let mut left = self.parse_equality_expressions();
        let token = self.curr();

        while token.value == TokenValue::Operator(OperatorType::Multiply)
            || token.value == TokenValue::Operator(OperatorType::Divide)
        {
            let operator = self.curr();
            if operator.value != TokenValue::Operator(OperatorType::Multiply)
                && operator.value != TokenValue::Operator(OperatorType::Divide)
            {
                break;
            }
            self.go();
            let operand = if operator.value == TokenValue::Operator(OperatorType::Multiply) {
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

        while token.value == TokenValue::Sign(SignType::Equality)
            || token.value == TokenValue::Sign(SignType::Inequality)
        {
            let operator = self.curr();
            if operator.value != TokenValue::Sign(SignType::Equality)
                && operator.value != TokenValue::Sign(SignType::Inequality)
            {
                break;
            }
            self.go();
            let operand = if operator.value == TokenValue::Sign(SignType::Equality) {
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

        while token.value == TokenValue::Operator(OperatorType::Bigger)
            || token.value == TokenValue::Operator(OperatorType::Smaller)
            || token.value == TokenValue::Operator(OperatorType::BiggerEqual)
            || token.value == TokenValue::Operator(OperatorType::SmallerEqual)
        {
            let operator = self.curr();
            if !(operator.value == TokenValue::Operator(OperatorType::Bigger)
                || operator.value == TokenValue::Operator(OperatorType::Smaller)
                || operator.value == TokenValue::Operator(OperatorType::BiggerEqual)
                || operator.value == TokenValue::Operator(OperatorType::SmallerEqual))
            {
                break;
            }
            self.go();
            let operand = match operator.value {
                TokenValue::Operator(t) => match t {
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

        while token.value == TokenValue::Operator(OperatorType::Equal) {
            let operator = self.curr();
            if operator.value != TokenValue::Operator(OperatorType::Equal) {
                break;
            }
            self.go();
            let right = self.parse_double_dot_expressions();

            left = ASTNode::Assignment(Self::get_assignment_property(&left), Box::new(right))
        }

        left
    }

    fn parse_self_assign_expression(&mut self) -> ASTNode {
        let _ = self.go(); // the self-assign operator;

        let assignment_thing = self.clone().parse_primary_expressions().unwrap();

        let expr = self.parse_expressions();

        ASTNode::Assignment(
            Self::get_assignment_property(&assignment_thing),
            Box::new(expr),
        )
    }

    fn parse_repeat_expression(&mut self) -> ASTNode {
        self.go();
        let operation = self.parse_expressions();
        if self.curr().value == TokenValue::Sign(SignType::Paren(Direction::Close)) {
            self.go();
        }
        let operator = self.curr();

        if operator.value == TokenValue::Operator(OperatorType::Repeat) {
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
        dbg!(&&&&&&&&&&&&identifier_token);
        if let TokenValue::Identifier(identifier) = identifier_token.value {
            self.go(); // identifier
            self.parse_fn_lower(identifier)
        } else {
            err!(
                self.curr().file_name,
                self.curr().line,
                self.curr().column,
                "Expecting an identifier after the `fn` keyword."
            );
            self.set_end();
            exit(100);
        }
    }

    fn parse_fn_lower(&mut self, identifier: String) -> ASTNode {
        if self.curr().value == TokenValue::Sign(SignType::Paren(Direction::Open)) {
            let args_list = self.parse_fn_args_list();

            self.expect_token(
                TokenValue::Sign(SignType::Arrow),
                "Expected an arrow (->) after the arguments.",
            );

            let data_type =
                if self.curr().value == TokenValue::Sign(SignType::CurlyBrace(Direction::Open)) {
                    DataType::Primitive(PrimitiveDataType::Null)
                } else {
                    self.parse_data_type()
                };

            self.expect_token(
                TokenValue::Sign(SignType::CurlyBrace(Direction::Open)),
                "Expected a code block.",
            );

            let body = self.parse_code_block();

            ASTNode::FunctionDeclaration(identifier, args_list, Box::new(body), data_type)
        } else {
            err!(
                self.curr().file_name,
                self.curr().line,
                self.curr().column,
                "Expected an opening paren."
            );
            self.set_end();
            ASTNode::InternalStop(self.curr().line, self.curr().file_name)
        }
    }

    fn parse_fn_args_list(&mut self) -> IndexMap<String, DataType> {
        self.go(); // paren
        let mut args_map: IndexMap<String, DataType> = IndexMap::new();
        while self.curr().value != TokenValue::Sign(SignType::Paren(Direction::Close)) {
            let (arg_name, data_type) = self.parse_fn_arg();
            if self.curr().value == TokenValue::Sign(SignType::Comma) {
                self.go();
            }
            args_map.insert(arg_name, data_type);
        }
        self.go();
        args_map
    }

    fn parse_fn_arg(&mut self) -> (String, DataType) {
        let identifier_token = self.go();
        // dbg!(identifier_token.clone());
        if let TokenValue::Identifier(identifier) = identifier_token.value {
            if self.curr().value == TokenValue::Sign(SignType::Colon) {
                self.go(); // colon

                let data_type = self.parse_data_type();

                (identifier, data_type)
            } else {
                err!(
                    self.curr().file_name,
                    self.curr().line,
                    self.curr().column,
                    "Expecting a colon after the argument name."
                );
                self.set_end();
                ("UNKNOWN".to_string(), DataType::InternalInfer)
            }
        } else {
            err!(
                self.curr().file_name,
                self.curr().line,
                self.curr().column,
                "Expecting an identifier in the arguments list."
            );
            self.set_end();
            ("UNKNOWN".to_string(), DataType::InternalInfer)
        }
    }

    fn expect_token(&mut self, token: TokenValue, reason: &str) {
        let tk = self.go();
        // dbg!("Just removed {}", tk.clone());
        if token != tk.value {
            err!(tk.file_name, tk.line, tk.column, "{}", reason)
        }
    }

    fn parse_code_block(&mut self) -> ASTNode {
        let mut nodes: Vec<ASTNode> = Vec::new();

        while self.curr().value != TokenValue::Sign(SignType::CurlyBrace(Direction::Close))
            && !self.is_end()
        {
            nodes.push(self.parse_expressions());
        }
        self.go();

        ASTNode::CodeBlock(nodes)
    }

    fn parse_function_call(&mut self) -> ASTNode {
        let left = self
            .parse_primary_expressions()
            .unwrap_or(ASTNode::Expression(ExpressionType::Null));

        if self.curr().value == TokenValue::Sign(SignType::Paren(Direction::Open)) {
            // self.go();
            // self.go();

            let arg_list = self.parse_fn_call_arg_list();

            ASTNode::FunctionCall(Box::new(left), arg_list)
        } else {
            left
        }
    }

    fn parse_fn_call_arg_list(&mut self) -> Vec<ASTNode> {
        self.expect_token(
            TokenValue::Sign(SignType::Paren(Direction::Open)),
            "Expected an opening paren.",
        );

        if self.curr().value == TokenValue::Sign(SignType::Paren(Direction::Close)) {
            self.go();
            return Vec::new();
        }

        let mut list: Vec<ASTNode> = vec![self.parse_expressions()];
        let mut tk = self.go();

        while tk.value == TokenValue::Sign(SignType::Comma) && !self.is_end() {
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

        let arg_type = self.parse_data_type();

        self.expect_token(
            TokenValue::Sign(SignType::Arrow),
            "Expected an arrow, then a data-type",
        );

        let data_type = self.parse_data_type();

        let mut args = IndexMap::new();

        args.insert(String::from("it"), arg_type);

        self.expect_token(
            TokenValue::Sign(SignType::CurlyBrace(Direction::Open)),
            "Expected a code block.",
        );

        let body = self.parse_code_block();

        ASTNode::FunctionDeclaration(identifier, args, Box::new(body), data_type)
    }

    fn parse_double_arrow_call(&mut self) -> ASTNode {
        let mut left = self.parse_function_call();
        let token = self.curr();

        // dbg!(token.clone());

        while token.value == TokenValue::Sign(SignType::DoubleArrow) {
            let operator = self.curr();
            // dbg!(operator.clone());
            if operator.value != TokenValue::Sign(SignType::DoubleArrow) {
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
                left = ASTNode::FunctionCall(Box::new(ASTNode::Identifier(id)), vec![left.clone()])
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
        if self.curr().value == TokenValue::Keyword(KeywordType::Else) {
            self.go(); // else;
            else_block = Some(Box::new(self.parse_code_block()));
        }

        ASTNode::IfStatement(IfStatement {
            condition,
            if_block: block,
            else_block,
        })
    }

    fn parse_once_declaration(&mut self) -> ASTNode {
        self.go(); // `once`

        // self.expect_token(Token::Sign(SignType::CurlyBrace(Direction::Open)), "Expecting an opening curly brace.");

        let tk = self.go();
        let mut ifs: Vec<IfStatement> = Vec::new();
        let mut else_block: Option<Box<ASTNode>> = None;

        while self.curr().value != TokenValue::Sign(SignType::CurlyBrace(Direction::Close))
            && !self.is_end()
        {
            if let ASTNode::IfStatement(if_st) = self.parse_if_declaration() {
                ifs.push(if_st);
                // self.expect_token(Token::Sign(SignType::Semicolon), "Expected a semicolon");
            }
            // tk = self.go();
            dbg!(&&tk);
        }
        self.go();

        if self.curr().value == TokenValue::Keyword(KeywordType::Else) {
            self.go(); // `else`
            else_block = Some(Box::new(self.parse_code_block()));
        }

        ASTNode::OnceStatement(OnceStatement {
            else_block,
            if_statements: ifs,
        })
    }

    fn parse_use(&mut self) -> ASTNode {
        self.go(); // `use`
        if self.curr().value == TokenValue::Keyword(KeywordType::Native) {
            self.parse_use_native()
        } else {
            self.parse_use_module()
        }
    }

    fn parse_use_module(&mut self) -> ASTNode {
        let mut path = String::new();
        let tk = self.go();

        if let TokenValue::Identifier(i) = tk.value {
            path.push_str(&i);
        } else {
            err!(
                tk.file_name.clone(),
                tk.line,
                tk.column,
                "Expected an identifier while parsing a module."
            );
            return ASTNode::InternalStop(tk.line, tk.file_name);
        };

        while self.curr().value == TokenValue::Sign(SignType::Colon) {
            self.go();
            if let TokenValue::Identifier(i) = self.curr().value {
                path.push(':');
                path.push_str(&i);
                self.go();
            }
        }

        self.expect_token(
            TokenValue::Sign(SignType::TildeArrow),
            "Expected a tilde-arrow (~>) to define the import symbol.",
        );

        let module = Module::new(path.clone());

        let src =
            mosa_fs::read_from_path(path.clone(), self.root.clone(), self.relative_root.clone());

        let mut parser = Parser::new(
            src,
            module,
            self.module_storage.clone(),
            self.root.clone(),
            mosa_fs::relative_from(path.clone()),
        );
        parser.gen_ast();

        if let TokenValue::Identifier(symbol) = self.go().value {
            ASTNode::UseModule(path, symbol)
        } else {
            err!(ft self.curr(), "Expected an identifier to define the imported symbol.");
            self.set_end();
            ASTNode::InternalStop(self.curr().line, self.curr().file_name)
        }
    }

    fn parse_use_native(&mut self) -> ASTNode {
        self.go(); // `native`

        if TokenValue::Keyword(KeywordType::Fn) == self.go().value {
            if let TokenValue::Identifier(identifier) = self.go().value {
                self.expect_token(
                    TokenValue::Sign(SignType::HashSign),
                    &format!("Expected a `#` after `use native {}", &identifier),
                );

                if let TokenValue::String(from) = self.go().value {
                    ASTNode::UseNative(UseNative {
                        name: identifier,
                        from,
                    })
                } else {
                    err!(ft self.last(), "Expected a string to qualify the path.");
                    self.set_end();
                    ASTNode::InternalStop(self.last().line, self.last().file_name)
                }
            } else {
                err!(ft self.last(), "Expected an identifier after `use native`.");
                self.set_end();
                ASTNode::InternalStop(self.last().line, self.last().file_name)
            }
        } else {
            unreachable!()
        }
    }

    fn parse_binding_access(&mut self) -> ASTNode {
        self.go(); // `^`

        if let TokenValue::Identifier(identifier) = self.go().value {
            ASTNode::BindingAccess(identifier)
        } else {
            err!(ft self.last(), "Expected an identifier to access a binding.");
            self.set_end();
            ASTNode::InternalStop(self.last().line, self.last().file_name)
        }
    }

    fn parse_for_expression(&mut self) -> ASTNode {
        self.go(); // `for`

        let iterable = self.parse_expressions();
        let block = self.parse_code_block();

        ASTNode::ForStatement(ForStatement {
            iterable: Box::new(iterable),
            block: Box::new(block),
        })
    }

    fn parse_complex_type_access(&mut self, entry: &String) -> ASTNode {
        self.go(); // `->`

        if let TokenValue::Identifier(identifier) = self.go().value {
            ASTNode::ComplexTypeAccessor(entry.clone(), identifier)
        } else {
            err!(ft self.last(), "Expected an identifier after `->` to access from an enum or a layout `{}`", entry);
            self.set_end();
            ASTNode::InternalStop(self.last().line, self.last().file_name)
        }
    }

    fn parse_enum_declaration(&mut self) -> ASTNode {
        self.go(); // `enum`

        if let TokenValue::Identifier(identifier) = self.go().value {
            if self.go().value != TokenValue::Sign(SignType::CurlyBrace(Direction::Open)) {
                err!(ft self.last(), "Expected an opening curly brace (`{{`).");
                self.set_end();
                return ASTNode::InternalStop(self.last().line, self.last().file_name);
            }

            let entries = self.parse_enum_entries();

            ASTNode::EnumDeclaration(identifier, entries)
        } else {
            err!(ft self.last(), "Expected enum identifier.");
            self.set_end();
            ASTNode::InternalStop(self.last().line, self.last().file_name)
        }
    }

    fn parse_enum_entries(&mut self) -> Vec<String> {
        let mut res = vec![];

        if let TokenValue::Identifier(id) = self.go().value {
            res.push(id);
        }

        let mut tk = self.go();

        if tk.value == TokenValue::Sign(SignType::Comma)
            && self.curr().value == TokenValue::Sign(SignType::CurlyBrace(Direction::Close))
        {
            tk = self.go();
        }

        while tk.value == TokenValue::Sign(SignType::Comma)
            && self.curr().value != TokenValue::Sign(SignType::CurlyBrace(Direction::Close))
            && !self.is_end()
        {
            if let TokenValue::Identifier(id) = self.go().value {
                res.push(id);
            }
            dbg!(&self.curr());
            tk = self.go();
            if tk.value == TokenValue::Sign(SignType::Comma)
                && self.curr().value == TokenValue::Sign(SignType::CurlyBrace(Direction::Close))
            {
                tk = self.go();
            }
        }

        dbg!(&tk);
        if tk.value != TokenValue::Sign(SignType::CurlyBrace(Direction::Close)) {
            err!(ft tk, "Expected a closing curly brace (`}}`).");
        }

        res
    }

    fn parse_layout_declaration(&mut self) -> ASTNode {
        self.go(); // `layout`

        let identifier: String;

        if let TokenValue::Identifier(ident) = self.go().value {
            identifier = ident;
        } else {
            err!(ft self.last(), "Expected an identifier after `layout`");
            self.set_end();
            return ASTNode::InternalStop(self.last().line, self.last().file_name);
        }

        if self.go().value != TokenValue::Sign(SignType::CurlyBrace(Direction::Open)) {
            err!(ft self.last(), "Expected an opening curly brace (`{{`).");
            self.set_end();
            return ASTNode::InternalStop(self.last().line, self.last().file_name);
        }

        let entries = self.parse_layout_entries();

        let tr = self.try_parse_internal_mix(identifier.clone());

        if let Some(mix) = tr {
            ASTNode::InternalMulti(vec![
                ASTNode::LayoutDeclaration(LayoutDeclaration {
                    name: identifier,
                    fields: entries,
                }),
                mix,
            ])
        } else {
            ASTNode::LayoutDeclaration(LayoutDeclaration {
                name: identifier,
                fields: entries,
            })
        }
    }

    fn try_parse_internal_mix(&mut self, internal_ident: String) -> Option<ASTNode> {
        if self.curr().value == TokenValue::Keyword(KeywordType::Mix)
            && self.peek().value == TokenValue::Sign(SignType::At)
        {
            Some(self.parse_mix(Some(internal_ident)))
        } else {
            None
        }
    }

    fn parse_layout_entries(&mut self) -> HashMap<String, FieldParserDescription> {
        let mut res = HashMap::new();

        if let TokenValue::Identifier(id) = self.curr().value {
            res.insert(id, self.parse_layout_single_entry().unwrap().1);
        }

        let mut tk = self.go();

        if self.curr().value == TokenValue::Sign(SignType::CurlyBrace(Direction::Close)) {
            tk = self.go();
        }

        while tk.value == TokenValue::Sign(SignType::Comma)
            && self.curr().value != TokenValue::Sign(SignType::CurlyBrace(Direction::Close))
            && !self.is_end()
        {
            if let TokenValue::Identifier(id) = self.curr().value {
                res.insert(id, self.parse_layout_single_entry().unwrap().1);
            }
            // if self.curr() == Token::Sign(SignType::Comma) {
            //     self.go();
            // }
            tk = self.go();
            if tk.value == TokenValue::Sign(SignType::Comma)
                && self.curr().value == TokenValue::Sign(SignType::CurlyBrace(Direction::Close))
            {
                tk = self.go();
            }
            dbg!(&tk);
        }

        if tk.value != TokenValue::Sign(SignType::CurlyBrace(Direction::Close)) {
            err!(ft tk, "Expected a closing curly brace (`}}`).");
            self.set_end();
        }

        res
    }

    fn parse_layout_single_entry(&mut self) -> Option<(String, FieldParserDescription)> {
        if let TokenValue::Identifier(id) = self.go().value {
            if self.go().value != TokenValue::Sign(SignType::Colon) {
                err!(ft self.last(), "Expected a colon after `{}`", id);
                self.set_end();
                return None;
            }

            let type_id: String;

            if let TokenValue::Identifier(r#type) = self.go().value {
                type_id = r#type;
            } else {
                err!(ft self.last(), "Expected a type identifier after `{}:`", id);
                self.set_end();
                return None;
            }

            let maybe_default_type =
                (self.curr().value == TokenValue::Operator(OperatorType::Equal)).then(|| {
                    self.go(); // '='

                    Box::new(self.parse_expressions())
                });

            Some((
                id,
                FieldParserDescription {
                    type_id,
                    default_value: maybe_default_type,
                },
            ))
        } else {
            None
        }
    }

    fn parse_layout_creation(&mut self, name: &String) -> ASTNode {
        self.go(); // {

        let entries = self.parse_layout_creation_entries();

        ASTNode::LayoutCreation(LayoutCreation {
            name: name.clone(),
            specified_fields: entries,
        })
    }

    fn parse_layout_creation_entries(&mut self) -> HashMap<String, Box<ASTNode>> {
        let mut res = HashMap::new();

        if let TokenValue::Identifier(id) = self.curr().value {
            res.insert(id, self.parse_layout_creation_single_entry().unwrap().1);
        }

        let mut tk = self.go();

        if self.curr().value == TokenValue::Sign(SignType::CurlyBrace(Direction::Close)) {
            tk = self.go();
        }

        while tk.value == TokenValue::Sign(SignType::Comma)
            && self.curr().value != TokenValue::Sign(SignType::CurlyBrace(Direction::Close))
            && !self.is_end()
        {
            if let TokenValue::Identifier(id) = self.curr().value {
                res.insert(id, self.parse_layout_creation_single_entry().unwrap().1);
            }
            tk = self.go();
            if tk.value == TokenValue::Sign(SignType::Comma)
                && self.curr().value == TokenValue::Sign(SignType::CurlyBrace(Direction::Close))
            {
                tk = self.go();
            }
            dbg!(&tk);
        }

        if tk.value != TokenValue::Sign(SignType::CurlyBrace(Direction::Close)) {
            err!(ft tk, "Expected a closing curly brace (`}}`).")
        }

        res
    }

    fn parse_layout_creation_single_entry(&mut self) -> Option<(String, Box<ASTNode>)> {
        if let TokenValue::Identifier(id) = self.go().value {
            if self.go().value != TokenValue::Operator(OperatorType::Equal) {
                err!(ft self.last(), "Expected an equals sign after `{}`", id);
                self.set_end();
                return None;
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
            err!(
                "NO TRACE",
                0,
                0,
                "Cannot make the expression into assignment-prop."
            );
            AssignmentProperty::Variable("INTERNAL".to_string())
        }
    }

    fn parse_layout_property_access(&mut self, id: &String) -> ASTNode {
        self.go(); // `.`

        if let TokenValue::Identifier(field) = self.go().value {
            ASTNode::LayoutFieldAccess(id.clone(), field)
        } else {
            err!(ft self.last(), "Expected a field name to access from `{}`.", id);
            self.set_end();
            ASTNode::InternalStop(self.last().line, self.last().file_name)
        }
    }

    fn parse_data_type(&mut self) -> DataType {
        if let TokenValue::Identifier(ident) = self.go().value {
            if ident == "nul" {
                let inner = self.parse_data_type();

                DataType::Primitive(PrimitiveDataType::Nullable(Box::new(inner)))
            } else {
                DataType::from_str(ident)
            }
        } else {
            err!(ft self.last(), "Expected an identifier or `nul` for a data type.");
            self.set_end();
            DataType::Complex(ComplexDataType::Indefinite)
        }
    }

    fn parse_mix(&mut self, internal_ident: Option<String>) -> ASTNode {
        self.go(); // `mix`

        let identifier;
        if let Some(i) = internal_ident {
            identifier = i;
            self.go(); // @
        } else {
            if let TokenValue::Identifier(id) = self.go().value {
                identifier = id;
            } else {
                err!(ft self.last(), "Expected an identifier marking the layout name.");
                self.set_end();
                return ASTNode::InternalStop(self.last().line, self.last().file_name);
            }
        }

        if self.go().value != TokenValue::Sign(SignType::CurlyBrace(Direction::Open)) {
            err!(ft self.last(), "Expected an opening curly braces.");
            self.set_end();
            return ASTNode::InternalStop(self.last().line, self.last().file_name);
        }

        let mut functions: Vec<ParserFunctionData> = Vec::new();

        while self.curr().value != TokenValue::Sign(SignType::CurlyBrace(Direction::Close)) {
            functions.push(self.parse_mix_function(Some(&identifier)));
        }

        if self.go().value != TokenValue::Sign(SignType::CurlyBrace(Direction::Close)) {
            err!(ft self.last(), "Expected a closing curly braces.");
            self.set_end();
            return ASTNode::InternalStop(self.last().line, self.last().file_name);
        }

        ASTNode::MixStatement(identifier, functions)
    }

    fn parse_mix_function(&mut self, identifier: Option<&String>) -> ParserFunctionData {
        dbg!(&&&&&&self.curr());

        let is_tied = self.curr().value == TokenValue::Keyword(KeywordType::Tied);

        if is_tied {
            dbg!(self.go());
        }

        let mut parse_fn = self.parse_fn_declaration();

        if let ASTNode::FunctionDeclaration(ref name, ref args, ref body, ref return_type) =
            parse_fn
        {
            if is_tied {
                let mut arg = IndexMap::new();
                arg.insert(
                    "self".to_string(),
                    DataType::Complex(ComplexDataType::LayoutOrEnum(identifier.cloned().unwrap())),
                );
                arg.extend(args.clone().into_iter());
                dbg!(&arg);
                parse_fn = ASTNode::FunctionDeclaration(
                    name.clone(),
                    arg,
                    body.clone(),
                    return_type.clone(),
                );
            }
        }

        if let ASTNode::FunctionDeclaration(name, args, body, return_type) = parse_fn {
            if let ASTNode::CodeBlock(code) = *body {
                ParserFunctionData {
                    name,
                    args,
                    body: code,
                    return_type,
                    tied: is_tied,
                }
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }

    fn parse_exp(&mut self) -> ASTNode {
        self.go();

        if self.curr().value == TokenValue::Keyword(KeywordType::Fn) {
            let fun = self.parse_fn_declaration();

            let fd =
                if let ASTNode::FunctionDeclaration(name, args, body, return_type) = fun.clone() {
                    ParserFunctionData {
                        name,
                        args,
                        body: if let ASTNode::CodeBlock(b) = *body {
                            b
                        } else {
                            unreachable!()
                        },
                        return_type,
                        tied: false,
                    }
                } else {
                    unreachable!()
                };

            self.module.push_unmodulated_fn(fd.name.clone(), fd);

            fun
        } else if self.curr().value == TokenValue::Keyword(KeywordType::Layout) {
            let layout = self.parse_layout_declaration();

            if let ASTNode::LayoutDeclaration(ld) = layout.clone() {
                self.module.push_unmodulated_layout(ld.name.clone(), ld);

                layout
            } else if let ASTNode::InternalMulti(ldv) = layout.clone() {
                let ld = if let ASTNode::LayoutDeclaration(v) = ldv[0].clone() {
                    v
                } else {
                    unreachable!()
                };

                // let sld = ScopeLayoutDeclaration {
                //     name: ld.name,
                //     fields: ld.fields,
                //     mixed: Arc::new(RwLock::new(HashMap::new()))
                // };

                self.module.push_unmodulated_layout(ld.name.clone(), ld);

                layout
            } else {
                unreachable!()
            }
        } else if self.curr().value == TokenValue::Keyword(KeywordType::Enum) {
            unreachable!()
        } else {
            err!(ft self.curr(), "Now, exports are only supported for functions.");
            self.set_end();
            ASTNode::InternalStop(self.curr().line, self.curr().file_name)
        }
    }
}
