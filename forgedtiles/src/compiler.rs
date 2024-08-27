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

    elements2d: Vec<String>,
    objects3d: Vec<String>,

    curr_parent: Option<usize>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            scanner: Scanner::new("".to_string()),
            parser: Parser::new(),

            elements2d: vec![
                "Texture".to_string(),
                "Vertical".to_string(),
                "Color".to_string(),
                "Noise".to_string(),
                "Bricks".to_string(),
            ],
            objects3d: vec![
                "Voxel".to_string(),
                "sdfCube".to_string(),
                "sdfSphere".to_string(),
            ],

            curr_parent: None,
        }
    }

    pub fn compile(&mut self, code: String) -> Result<FTContext, FTError> {
        let mut context = FTContext::new();

        self.scanner = Scanner::new(code);

        self.curr_parent = None;
        self.add_to_context(&mut context);

        if self.parser.error.is_some() {
            Err(self.parser.error.clone().unwrap())
        } else {
            Ok(context)
        }
    }

    pub fn add_to_context(&mut self, ctx: &mut FTContext) {
        self.advance();

        while !self.matches(TokenType::Eof) {
            let mut consumed = false;

            if self.indent() == 0 {
                if self.parser.current.indent == 0 {
                    self.curr_parent = None;
                }

                if self.parser.current.kind == TokenType::Identifier {
                    let id = self.parser.current.lexeme.as_str();

                    match id {
                        "Shape" => {
                            let mut node = Node::new(NodeRole::Shape);
                            self.advance();
                            self.parse_node_properties(&mut node, ctx);
                            consumed = true;

                            ctx.shapes.push(ctx.nodes.len());
                            ctx.nodes.push(node);
                        }
                        _ => {
                            self.error_at_current("Unknown node.");
                        }
                    }
                }
            }

            if !consumed {
                self.advance();
            }

            if self.parser.error.is_some() {
                break;
            }
        }
    }

    /// Parses the properties for the given object
    fn parse_node_properties(&mut self, node: &mut Node, ctx: &mut FTContext) {
        node.indent = self.parser.current.indent;
        //println!("object on line {}", self.parser.current.line);

        if self.check(TokenType::Star) {
            ctx.output = Some(ctx.nodes.len());
            self.advance();
        }

        loop {
            let property = self.parser.current.lexeme.to_lowercase();
            let indention = self.parser.current.indent;

            if indention < node.indent || self.parser.current.kind == TokenType::Eof {
                self.debug_current(format!("prop break for {}", node.name).as_str());
                break;
            }

            self.consume(TokenType::Identifier, "Expected identifier.");

            if self.check(TokenType::Equal) {
                let value = self.scanner.scanline(1);
                println!(
                    "assignment to {:?}, line {}: {} = {}",
                    node.role, self.parser.current.line, property, value
                );

                if property == "name" {
                    node.name = value;
                } else if let Ok(number) = value.parse::<f32>() {
                    if !node.values.add_string_based(&property, vec![number]) {
                        self.error_at_current(&format!("Unknown property {}", property));
                    }
                } else if value.starts_with('#') {
                    //println!("Color {}", value);
                    let mut chars = value.chars();
                    chars.next();
                    let color = chars.as_str();

                    // use colors_transform::Rgb;

                    // if let Some(rgb) = Rgb::from_hex_str(color).ok() {
                    //     println!("{:?}", rgb);
                    //     value = format!(
                    //         "F4({:.3}, {:.3}, {:.3}, 1.0)",
                    //         rgb.get_red() as F / 255.0,
                    //         rgb.get_green() as F / 255.0,
                    //         rgb.get_blue() as F / 255.0
                    //     );
                    //     println!("{}", value);
                    // }
                }
                //props.push(Property::Property(property, value));
                self.advance();
                // if self.indent() == 0 {
                //     break;
                // }
            }
            // else if self.check(TokenType::LeftParen) {
            //     let mut args = "".to_string();
            //     self.advance();
            //     loop {
            //         if self.check(TokenType::Identifier) {
            //             args += self.parser.current.lexeme.clone().as_str();
            //             self.advance();
            //         } else if self.check(TokenType::RightParen) {
            //             break;
            //         } else if self.check(TokenType::Comma) {
            //             args += ",";
            //             self.advance();
            //         } else {
            //             self.error_at_current("Invalid function arguments");
            //             break;
            //         }
            //     }
            //     let code = self.scanner.scan_indention_block(1, indention);
            //     //println!("function, line {}: {}, {:?}", line, args, code.ok());
            //     if let Some(code) = code.ok() {
            //         props.push(Property::Function(property, args, code));
            //     }
            //     self.advance();
            //     if self.indent() <= node.indent {
            //         break;
            //     }
            // }
            else {
                break;
            }
        }
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
    fn advance_with_whitespace(&mut self) {
        self.parser.previous = self.parser.current.clone();

        loop {
            self.parser.current = self.scanner.scan_token(true);

            if self.parser.current.kind != TokenType::Error {
                break;
            }
        }
    }

    /// Prints the current Token.
    fn debug_current(&mut self, msg: &str) {
        println!("{} {:?}", msg, self.parser.current);
    }

    /// Consume the current token if the type matches
    fn consume(&mut self, kind: TokenType, message: &str) {
        if self.parser.current.kind == kind {
            self.advance();
            return;
        }
        self.error_at_current(message);
    }

    /// Consume the current token if the type matches
    fn _consume_with_whitespace(&mut self, kind: TokenType, message: &str) {
        if self.parser.current.kind == kind {
            self.advance_with_whitespace();
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
    fn indent(&self) -> usize {
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
