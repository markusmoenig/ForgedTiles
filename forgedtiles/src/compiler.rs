use crate::prelude::*;

use crate::scanner::TokenType;

#[derive(Clone, Debug)]
pub struct FTError {
    pub description: String,
    pub line: u32,
}

impl FTError {
    pub fn new(description: String, line: u32) -> Self {
        Self { description, line }
    }
}

struct Parser {
    current: Token,
    previous: Token,

    error: Option<FTError>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            current: Token::synthetic("".to_owned()),
            previous: Token::synthetic("".to_owned()),
            error: None,
        }
    }
}

pub struct Compiler {
    scanner: Scanner,

    parser: Parser,

    curr_parent: Option<usize>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            scanner: Scanner::new("".to_string()),
            parser: Parser::new(),

            curr_parent: None,
        }
    }

    /// Compile the given code.
    pub fn compile(&mut self, code: String) -> Result<FTContext, FTError> {
        let mut context = FTContext::new();

        self.scanner = Scanner::new(code);

        self.curr_parent = None;
        self.parse(&mut context);

        if self.parser.error.is_some() {
            Err(self.parser.error.clone().unwrap())
        } else {
            Ok(context)
        }
    }

    /// Parse the code and add the content to the context.
    pub fn parse(&mut self, ctx: &mut FTContext) {
        self.advance();

        while !self.matches(TokenType::Eof) {
            if self.current().kind == TokenType::Let {
                self.declaration(ctx);
            } else {
                self.error_at_current(&format!(
                    "Unknown instruction '{}'.",
                    self.parser.current.lexeme
                ));
            }

            if self.has_error() {
                break;
            }
        }
    }

    /// Declaration (let)
    fn declaration(&mut self, ctx: &mut FTContext) {
        //println!("declaration");

        self.advance();
        if let Some(target) =
            self.consume(TokenType::Identifier, "Expected an identifier after 'let'.")
        {
            self.consume(TokenType::Equal, "Expected '='.");

            if let Some(node_type) =
                self.consume(TokenType::Identifier, "Expected an identifier after 'let'.")
            {
                let mut node: Option<Node> = None;

                match node_type.as_str() {
                    "Shape" => {
                        if self.check(TokenType::Star) {
                            ctx.output = Some(ctx.nodes.len());
                            self.advance();
                        }

                        self.consume(TokenType::Less, "Expected '<'.");
                        if let Some(shape) = self.consume(
                            TokenType::Identifier,
                            "Expected a valid shape after 'Shape'.",
                        ) {
                            match shape.as_str() {
                                "Box" => {
                                    node = Some(Node::new(NodeRole::Shape, NodeSubRole::Box));
                                }
                                "Disc" => {
                                    node = Some(Node::new(NodeRole::Shape, NodeSubRole::Disc));
                                }
                                _ => self.error_at_current(&format!("Unknown shape '{}'.", shape)),
                            }
                        }
                    }
                    "Pattern" => {
                        if self.check(TokenType::Star) {
                            ctx.output = Some(ctx.nodes.len());
                            self.advance();
                        }
                        self.consume(TokenType::Less, "Expected '<'.");
                        if let Some(shape) = self.consume(
                            TokenType::Identifier,
                            "Expected a valid pattern after 'Pattern'.",
                        ) {
                            match shape.as_str() {
                                "Bricks" => {
                                    node = Some(Node::new(NodeRole::Pattern, NodeSubRole::Bricks));
                                }
                                "Tiles" => {
                                    node = Some(Node::new(NodeRole::Pattern, NodeSubRole::Tiles));
                                }
                                _ => {
                                    self.error_at_current(&format!("Unknown pattern '{}'.", shape))
                                }
                            }
                        }
                    }
                    "Face" => {
                        if self.check(TokenType::Star) {
                            ctx.output = Some(ctx.nodes.len());
                            self.advance();
                        }
                        self.consume(TokenType::Less, "Expected '<'.");
                        if let Some(face) = self
                            .consume(TokenType::Identifier, "Expected a valid face after 'Face'.")
                        {
                            match face.as_str() {
                                "Floor" => {
                                    node = Some(Node::new(NodeRole::Face, NodeSubRole::Floor));
                                }
                                "Left" => {
                                    node = Some(Node::new(NodeRole::Face, NodeSubRole::Left));
                                }
                                "Top" => {
                                    node = Some(Node::new(NodeRole::Face, NodeSubRole::Top));
                                }
                                "Right" => {
                                    node = Some(Node::new(NodeRole::Face, NodeSubRole::Right));
                                }
                                "Bottom" => {
                                    node = Some(Node::new(NodeRole::Face, NodeSubRole::Bottom));
                                }
                                "MiddleX" => {
                                    node = Some(Node::new(NodeRole::Face, NodeSubRole::MiddleX));
                                }
                                "MiddleY" => {
                                    node = Some(Node::new(NodeRole::Face, NodeSubRole::MiddleY));
                                }
                                _ => self.error_at_current(&format!("Unknown face '{}'.", face)),
                            }
                        }
                    }
                    _ => {
                        self.error_at_current(&format!("Unknown type '{}'.", node_type));
                    }
                }
                self.consume(TokenType::Greater, "Expected '>'.");

                if !self.has_error() {
                    // Add the new node to the context.
                    if let Some(node) = &mut node {
                        node.name = target.clone();
                        ctx.variables.insert(target, ctx.nodes.len());

                        self.parse_node_properties(node, ctx);
                        match &node.role {
                            NodeRole::Shape => {
                                ctx.shapes.push(ctx.nodes.len());
                            }
                            NodeRole::Pattern => {
                                ctx.patterns.push(ctx.nodes.len());
                            }
                            NodeRole::Face => {
                                ctx.patterns.push(ctx.nodes.len());
                            }
                        }
                        ctx.nodes.push(node.clone());
                    }
                }
            }
        }
    }

    /// Parses the properties for the given node.
    fn parse_node_properties(&mut self, node: &mut Node, ctx: &mut FTContext) {
        if self.check(TokenType::Colon) {
            self.advance();
        }

        loop {
            if self.check(TokenType::Semicolon) {
                self.advance();
                break;
            } else if self.check(TokenType::Comma) {
                self.advance();
            } else if self.check(TokenType::Eof) {
                break;
            } else if let Some(property) = self.consume(
                TokenType::Identifier,
                &format!(
                    "Expected property identifier, got '{}'. Missing ';' after declaration ?.",
                    self.parser.current.lexeme
                ),
            ) {
                self.consume(TokenType::Equal, "Expected '=' after property name.");
                if self.check(TokenType::Number) {
                    if let Ok(number) = self.parser.current.lexeme.parse::<f32>() {
                        println!("{} = {}", property, number);
                        if !node.values.add_string_based(&property, vec![number]) {
                            self.error_at_current(&format!("Unknown property {}", property));
                        }
                    }
                    self.advance();
                } else if self.check(TokenType::Identifier) {
                    let map_value = self.parser.current.lexeme.clone();
                    if property == "content" {
                        self.advance();

                        node.links = self.read_string_list_as_ref_list(map_value, ctx);
                        println!("{} = {:?}", property, node.links);
                    } else {
                        node.map.insert(property, vec![map_value]);
                        self.advance();
                    }
                } else {
                    self.error_at_current(&format!(
                        "Unknown property value, got '{}'.",
                        self.parser.current.lexeme
                    ));
                    break;
                }
            } else {
                self.error_at_current(&format!(
                    "Unknown property ('{}'). Missing ';' after declaration ?.",
                    self.parser.current.lexeme
                ));
                break;
            }
        }
    }

    /// Read a comma separated list of strings and take their references as link list.
    pub fn read_string_list_as_ref_list(&mut self, first: String, ctx: &FTContext) -> Vec<u16> {
        let mut list: Vec<String> = vec![first];
        loop {
            if self.check(TokenType::Comma) {
                self.advance();
            } else if self.check(TokenType::Eof) {
                break;
            }

            if self.check(TokenType::Identifier) {
                list.push(self.current().lexeme.clone());
                self.advance();
            } else {
                break;
            }
        }

        let mut values: Vec<u16> = vec![];

        for i in list {
            if let Some(value) = ctx.variables.get(&i) {
                values.push(*value as u16);
            } else {
                self.error_at_current(&format!("Unknown variable ('{}').", i));
            }
        }

        values
    }

    /// Advance one token
    fn advance(&mut self) {
        self.parser.previous = self.parser.current.clone();

        loop {
            self.parser.current = self.scanner.scan_token(false);

            if self.parser.current.kind != TokenType::Error {
                break;
            }
        }
    }

    /// Advance one token and allow whitespace
    fn _advance_with_whitespace(&mut self) {
        self.parser.previous = self.parser.current.clone();

        loop {
            self.parser.current = self.scanner.scan_token(true);

            if self.parser.current.kind != TokenType::Error {
                break;
            }
        }
    }

    /// Return a reference to the current token.
    fn current(&self) -> &Token {
        &self.parser.current
    }

    /// Prints the current Token.
    fn _debug_current(&mut self, msg: &str) {
        println!("{} {:?}", msg, self.parser.current);
    }

    /// Consume the current token if the type matches and return it's lexeme.
    fn consume(&mut self, kind: TokenType, message: &str) -> Option<String> {
        if self.parser.current.kind == kind {
            let lex = self.parser.current.lexeme.clone();
            self.advance();
            Some(lex)
        } else {
            self.error_at_current(message);
            None
        }
    }

    /// Consume the current token if the type matches
    fn _consume_with_whitespace(&mut self, kind: TokenType, message: &str) {
        if self.parser.current.kind == kind {
            self._advance_with_whitespace();
            return;
        }
        self.error_at_current(message);
    }

    /// If the current token matches advance, else do not.
    fn matches(&mut self, kind: TokenType) -> bool {
        if !self.check(kind) {
            false
        } else {
            self.advance();
            true
        }
    }

    /// Check if the current token matches.
    fn check(&self, kind: TokenType) -> bool {
        self.parser.current.kind == kind
    }

    /// get_The indent level of the current token.
    fn _indent(&self) -> usize {
        self.parser.current.indent
    }

    /// Error at the current token
    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.parser.current.clone(), message)
    }

    /// Error at the previous token
    fn _error(&mut self, message: &str) {
        self.error_at(self.parser.previous.clone(), message)
    }

    /// Returns true if we had an error during parsing.
    fn has_error(&self) -> bool {
        self.parser.error.is_some()
    }

    /// Error at the given token
    fn error_at(&mut self, _token: Token, message: &str) {
        if self.parser.error.is_some() {
            return;
        }
        self.parser.error = Some(FTError::new(
            message.to_string(),
            self.parser.current.line as u32,
        ));
    }
}
