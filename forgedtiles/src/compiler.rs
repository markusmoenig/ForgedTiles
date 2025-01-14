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
                                "Repeat" => {
                                    node = Some(Node::new(NodeRole::Pattern, NodeSubRole::Repeat));
                                }
                                "Offset" => {
                                    node = Some(Node::new(NodeRole::Pattern, NodeSubRole::Offset));
                                }
                                "Stack" => {
                                    node = Some(Node::new(NodeRole::Pattern, NodeSubRole::Stack));
                                }
                                "Group" => {
                                    node = Some(Node::new(NodeRole::Pattern, NodeSubRole::Group));
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
                                "Front" => {
                                    node = Some(Node::new(NodeRole::Face, NodeSubRole::Front));
                                }
                                "Right" => {
                                    node = Some(Node::new(NodeRole::Face, NodeSubRole::Right));
                                }
                                "Back" => {
                                    node = Some(Node::new(NodeRole::Face, NodeSubRole::Back));
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
                    "Material" => {
                        if self.check(TokenType::Star) {
                            ctx.output = Some(ctx.nodes.len());
                            self.advance();
                        }
                        self.consume(TokenType::Less, "Expected '<'.");
                        if let Some(shape) = self.consume(
                            TokenType::Identifier,
                            "Expected a valid material after 'Material'.",
                        ) {
                            match shape.as_str() {
                                "BSDF" => {
                                    node = Some(Node::new(NodeRole::Material, NodeSubRole::BSDF));
                                }
                                _ => {
                                    self.error_at_current(&format!("Unknown material '{}'.", shape))
                                }
                            }
                        }
                    }
                    "Meta" => {
                        self.consume(TokenType::Less, "Expected '<'.");
                        if let Some(meta) = self
                            .consume(TokenType::Identifier, "Expected a valid face after 'Face'.")
                        {
                            match meta.as_str() {
                                "Material" => {
                                    node =
                                        Some(Node::new(NodeRole::Meta, NodeSubRole::MetaMaterial));
                                }
                                "Delete" => {
                                    node = Some(Node::new(NodeRole::Meta, NodeSubRole::MetaDelete));
                                }
                                _ => self.error_at_current(&format!(
                                    "Unknown meta directive '{}'.",
                                    meta
                                )),
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
                                ctx.shapes.push(ctx.nodes.len() as u8);
                            }
                            NodeRole::Pattern => {
                                ctx.patterns.push(ctx.nodes.len() as u8);
                            }
                            NodeRole::Face => {
                                ctx.faces.push(ctx.nodes.len() as u8);
                            }
                            NodeRole::Material => {
                                ctx.materials.push(ctx.nodes.len() as u8);
                            }
                            _ => {}
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
                // Values
                self.consume(TokenType::Equal, "Expected '=' after property name.");

                if let Some(role) = FTExpressionRole::from_string(&property) {
                    let mut expr_str = String::new();

                    loop {
                        if self.check(TokenType::Semicolon)
                            || self.check(TokenType::Comma)
                            || self.check(TokenType::Eof)
                        {
                            break;
                        } else {
                            expr_str += &self.parser.current.lexeme;
                            self.advance();
                        }
                    }

                    //println!("{:?} {}", role, expr_str);
                    // Add the expression
                    node.expressions.add(role, &expr_str);
                } else if self.check(TokenType::Number)
                    && FTExpressionRole::from_string(&property).is_none()
                {
                    if let Ok(number) = self.parser.current.lexeme.parse::<f32>() {
                        //println!("{} = {}", property, number);
                        if !node.values.add_string_based(&property, vec![number]) {
                            self.error_at_current(&format!("Unknown property {}", property));
                        }
                    }
                    self.advance();
                } else if self.check(TokenType::Identifier)
                    || self.check(TokenType::LeftBracket)
                    || self.check(TokenType::HexColor)
                    || self.check(TokenType::String)
                {
                    let mut has_bracket = false;
                    if self.check(TokenType::LeftBracket) {
                        has_bracket = true;
                        self.advance();
                    }

                    let map_value = self.parser.current.lexeme.clone();
                    if property == "material" {
                        self.advance();

                        if map_value.to_lowercase() == "none" {
                            continue;
                        } else if let Some(value) = ctx.variables.get(&map_value) {
                            node.material = Some(*value as u8);
                        } else {
                            self.error_at_current(&format!("Unknown variable ('{}').", map_value));
                        }
                    } else if property == "texture" {
                        self.advance();

                        if map_value.to_lowercase() == "none" {
                            continue;
                        } else {
                            node.map
                                .insert("texture".to_string(), vec![map_value.replace("\"", "")]);
                        }
                    } else if property == "color" {
                        if self.check(TokenType::HexColor) {
                            let mut color = map_value.clone();
                            color.remove(0);
                            if let Some(color) = self.hex_to_rgb_normalized(&color) {
                                node.values.add(FTValueRole::Color, color);
                            } else {
                                self.error_at_current(&format!("Invalid hex color {}", map_value));
                            }
                        }
                        self.advance();
                    } else if property == "content" || property == "cutout" {
                        if !has_bracket {
                            self.error_at_current("Expected '[' at beginning of content list.");
                            return;
                        }

                        self.advance();
                        if node.role == NodeRole::Meta {
                            // For meta nodes read the list of seeds / hashes
                            if let Ok(first) = map_value.parse::<i32>() {
                                node.links = self.read_number_list_as_i32_list(first);
                                if node.sub_role == NodeSubRole::MetaDelete {
                                    ctx.meta_delete.append(&mut node.links);
                                }
                            }
                        } else if map_value != "]" {
                            if property == "cutout" {
                                let cutout = self.read_string_list_as_ref_list(map_value, ctx);
                                if !cutout.is_empty() {
                                    node.values.add(FTValueRole::Cutout, vec![cutout[0] as f32]);
                                }
                            } else {
                                node.links = self.read_string_list_as_ref_list(map_value, ctx);
                            }
                        }
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

    /// Read a comma separated list of integers and take their references as link list.
    pub fn read_number_list_as_i32_list(&mut self, first: i32) -> Vec<i32> {
        let mut list: Vec<i32> = vec![first];

        loop {
            if self.check(TokenType::Comma) {
                self.advance();
            } else if self.check(TokenType::Eof) {
                break;
            }

            if self.check(TokenType::Number) {
                if let Ok(v) = self.current().lexeme.parse::<i32>() {
                    list.push(v);
                }
                self.advance();
            } else if self.check(TokenType::RightBracket) {
                self.advance();
                break;
            } else {
                self.error_at_current(&format!(
                    "Expected ']' at end of list, got '{}'.",
                    self.parser.current.lexeme
                ));
                break;
            }
        }

        list
    }

    /// Read a comma separated list of strings and take their references as link list.
    pub fn read_string_list_as_ref_list(&mut self, first: String, ctx: &FTContext) -> Vec<i32> {
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
            } else if self.check(TokenType::RightBracket) {
                self.advance();
                break;
            } else {
                self.error_at_current(&format!(
                    "Expected ']' at end of list, got '{}'.",
                    self.parser.current.lexeme
                ));
                break;
            }
        }

        let mut values: Vec<i32> = vec![];

        for i in list {
            if let Some(value) = ctx.variables.get(&i) {
                values.push(*value as i32);
            } else {
                self.error_at_current(&format!("Unknown variable ('{}').", i));
            }
        }

        values
    }

    /// Read a hex color.
    fn hex_to_rgb_normalized(&self, hex: &str) -> Option<Vec<f32>> {
        // Ensure the string is exactly 6 characters long
        if hex.len() != 6 {
            return None;
        }

        // Try to parse the red, green, and blue components
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        // Normalize the values to the range [0.0, 1.0]
        Some(vec![r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0])
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
