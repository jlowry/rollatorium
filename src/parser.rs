use crate::{
    Result,
    ast::{
        Annotation, BinaryOperator, DiceSize, Node, Selector, SelectorKind, SetOperation,
        SetOperator, UnaryOperator,
    },
    error::RollatoriumError,
    lexer::Lexer,
    token::Token,
};

// ---------- Parser ----------
pub(crate) struct Parser<'a> {
    lexer: Lexer<'a>,
    cur_token: Token<'a>,
    input: &'a str,
    selector_depth: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Result<Self> {
        let mut lexer = Lexer::new(input);
        let first = lexer.next_token()?;
        Ok(Parser {
            lexer,
            cur_token: first,
            input,
            selector_depth: 0,
        })
    }

    fn eat(&mut self, expected: Token<'a>) -> Result<()> {
        if std::mem::discriminant(&self.cur_token) == std::mem::discriminant(&expected) {
            self.cur_token = self.lexer.next_token()?;
            Ok(())
        } else {
            Err(RollatoriumError::Parser(format!(
                "Expected {:?}, got {:?} in '{}'",
                expected, self.cur_token, self.input
            )))
        }
    }

    pub fn parse(&mut self) -> Result<Node<&'a str>> {
        let expr = self.parse_comparison()?;
        if self.cur_token != Token::Eof {
            return Err(RollatoriumError::Parser(format!(
                "Unexpected trailing input: {:?}",
                self.cur_token
            )));
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Node<&'a str>> {
        let mut node = self.parse_additive()?;
        loop {
            let operator = match self.cur_token {
                Token::EqualEqual => Some(BinaryOperator::Equal),
                Token::NotEqual => Some(BinaryOperator::NotEqual),
                Token::Greater => Some(BinaryOperator::Greater),
                Token::GreaterEqual => Some(BinaryOperator::GreaterEqual),
                Token::Less => Some(BinaryOperator::Less),
                Token::LessEqual => Some(BinaryOperator::LessEqual),
                _ => None,
            };

            let Some(operator) = operator else { break };
            let token = self.cur_token.clone();
            self.eat(token)?;
            let right = self.parse_additive()?;
            node = Node::Binary {
                operator,
                left: Box::new(node),
                right: Box::new(right),
            };
        }
        Ok(node)
    }

    fn parse_additive(&mut self) -> Result<Node<&'a str>> {
        let mut node = self.parse_multiplicative()?;
        loop {
            let operator = match self.cur_token {
                Token::Plus => Some(BinaryOperator::Add),
                Token::Minus => Some(BinaryOperator::Subtract),
                _ => None,
            };

            let Some(operator) = operator else { break };
            let token = self.cur_token.clone();
            self.eat(token)?;
            let right = self.parse_multiplicative()?;
            node = Node::Binary {
                operator,
                left: Box::new(node),
                right: Box::new(right),
            };
        }
        Ok(node)
    }

    fn parse_multiplicative(&mut self) -> Result<Node<&'a str>> {
        let mut node = self.parse_unary()?;
        loop {
            let operator = match self.cur_token {
                Token::Star => Some(BinaryOperator::Multiply),
                Token::Slash => Some(BinaryOperator::Divide),
                Token::DoubleSlash => Some(BinaryOperator::IntDivide),
                Token::Percent => Some(BinaryOperator::Modulo),
                _ => None,
            };

            let Some(operator) = operator else { break };
            let token = self.cur_token.clone();
            self.eat(token)?;
            let right = self.parse_unary()?;
            node = Node::Binary {
                operator,
                left: Box::new(node),
                right: Box::new(right),
            };
        }
        Ok(node)
    }

    fn parse_unary(&mut self) -> Result<Node<&'a str>> {
        match self.cur_token {
            Token::Plus => {
                self.eat(Token::Plus)?;
                Ok(Node::Unary {
                    operator: UnaryOperator::Plus,
                    operand: Box::new(self.parse_unary()?),
                })
            }
            Token::Minus => {
                self.eat(Token::Minus)?;
                Ok(Node::Unary {
                    operator: UnaryOperator::Minus,
                    operand: Box::new(self.parse_unary()?),
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Node<&'a str>> {
        let node = self.parse_atom()?;
        let node = self.parse_modifiers(node)?;
        self.parse_annotations(node)
    }

    fn parse_atom(&mut self) -> Result<Node<&'a str>> {
        match &self.cur_token {
            Token::Number(value) => {
                let literal = Node::Literal(*value);
                self.eat(Token::Number(*value))?;
                if matches!(self.cur_token, Token::Dice | Token::DicePercent) {
                    self.parse_dice_literal(Some(literal))
                } else {
                    Ok(literal)
                }
            }
            Token::Dice | Token::DicePercent => self.parse_dice_literal(None),
            Token::LParen => self.parse_parenthesized_or_set(),
            Token::AnnotationStart => Err(RollatoriumError::Parser(
                "Unexpected annotation start; annotations must follow an expression".into(),
            )),
            token => Err(RollatoriumError::Parser(format!(
                "Unexpected token {:?} in '{}'",
                token, self.input
            ))),
        }
    }

    fn parse_parenthesized_or_set(&mut self) -> Result<Node<&'a str>> {
        self.eat(Token::LParen)?;
        if self.cur_token == Token::RParen {
            self.eat(Token::RParen)?;
            return Ok(Node::Set {
                elements: Vec::new(),
                operations: Vec::new(),
            });
        }

        let first = self.parse_comparison()?;
        let mut elements = vec![first];
        let mut is_set = false;

        while self.cur_token == Token::Comma {
            is_set = true;
            self.eat(Token::Comma)?;
            if self.cur_token == Token::RParen {
                break;
            }
            elements.push(self.parse_comparison()?);
        }

        self.eat(Token::RParen)?;

        let set_ops_follow = matches!(
            self.cur_token,
            Token::Keep
                | Token::Drop
                | Token::Reroll
                | Token::RerollOnce
                | Token::RerollAdd
                | Token::Explode
                | Token::Min
                | Token::Max
        );

        let first_is_dice = matches!(
            elements.first(),
            Some(Node::Dice { .. }) | Some(Node::DiceWithOps { .. })
        );

        if is_set || (set_ops_follow && !first_is_dice) {
            Ok(Node::Set {
                elements,
                operations: Vec::new(),
            })
        } else {
            Ok(elements
                .into_iter()
                .next()
                .expect("at least one element present"))
        }
    }

    fn parse_dice_literal(&mut self, quantity: Option<Node<&'a str>>) -> Result<Node<&'a str>> {
        match self.cur_token.clone() {
            Token::Dice => {
                self.eat(Token::Dice)?;
                let faces = match &self.cur_token {
                    Token::Number(value) => {
                        let value = *value;
                        self.eat(Token::Number(value))?;
                        Node::Literal(value)
                    }
                    token => {
                        return Err(RollatoriumError::Parser(format!(
                            "Expected die size after 'd', found {:?} in '{}'",
                            token, self.input
                        )));
                    }
                };
                Ok(Node::Dice {
                    num: quantity.map(Box::new),
                    size: DiceSize::Value(Box::new(faces)),
                })
            }
            Token::DicePercent => {
                self.eat(Token::DicePercent)?;
                Ok(Node::Dice {
                    num: quantity.map(Box::new),
                    size: DiceSize::Percent,
                })
            }
            _ => Err(RollatoriumError::Parser(format!(
                "Invalid dice expression in '{}'",
                self.input
            ))),
        }
    }

    fn parse_modifiers(&mut self, node: Node<&'a str>) -> Result<Node<&'a str>> {
        if self.selector_depth > 0 {
            return Ok(node);
        }

        let mut operations = Vec::new();
        loop {
            let (operator, symbol) = match self.cur_token {
                Token::Keep => {
                    self.eat(Token::Keep)?;
                    (SetOperator::Keep, "k")
                }
                Token::Drop => {
                    self.eat(Token::Drop)?;
                    (SetOperator::Drop, "p")
                }
                Token::Reroll => {
                    self.eat(Token::Reroll)?;
                    (SetOperator::Reroll, "rr")
                }
                Token::RerollOnce => {
                    self.eat(Token::RerollOnce)?;
                    (SetOperator::RerollOnce, "ro")
                }
                Token::RerollAdd => {
                    self.eat(Token::RerollAdd)?;
                    (SetOperator::RerollAdd, "ra")
                }
                Token::Explode => {
                    self.eat(Token::Explode)?;
                    (SetOperator::Explode, "!")
                }
                Token::Min => {
                    self.eat(Token::Min)?;
                    (SetOperator::Minimum, "mi")
                }
                Token::Max => {
                    self.eat(Token::Max)?;
                    (SetOperator::Maximum, "ma")
                }
                _ => break,
            };

            let selectors = self.parse_selector_list(symbol, operator)?;
            operations.push(SetOperation {
                operator,
                selectors,
            });
        }

        if operations.is_empty() {
            return Ok(node);
        }

        match node {
            Node::Dice { num, size } => Ok(Node::DiceWithOps {
                dice: Box::new(Node::Dice { num, size }),
                operations,
            }),
            Node::DiceWithOps {
                dice,
                operations: mut existing,
            } => {
                existing.extend(operations);
                Ok(Node::DiceWithOps {
                    dice,
                    operations: existing,
                })
            }
            Node::Set {
                elements,
                operations: mut existing,
            } => {
                existing.extend(operations);
                Ok(Node::Set {
                    elements,
                    operations: existing,
                })
            }
            other => Err(RollatoriumError::Parser(format!(
                "Set operations can only be applied to dice or sets, not {:?}",
                other
            ))),
        }
    }

    fn parse_selector_list(
        &mut self,
        symbol: &str,
        operator: SetOperator,
    ) -> Result<Vec<Selector<&'a str>>> {
        if !self.is_selector_start(&self.cur_token) {
            return Err(RollatoriumError::Parser(format!(
                "Expected selector after '{}' in '{}'",
                symbol, self.input
            )));
        }

        let mut selectors = Vec::new();
        while self.is_selector_start(&self.cur_token) {
            selectors.push(self.parse_selector()?);
        }

        if selectors.is_empty() {
            return Err(RollatoriumError::Parser(format!(
                "Operator '{:?}' must be followed by at least one selector",
                operator
            )));
        }

        Ok(selectors)
    }

    fn parse_selector(&mut self) -> Result<Selector<&'a str>> {
        let (kind, prefix) = match self.cur_token {
            Token::SelectorHigh => {
                self.eat(Token::SelectorHigh)?;
                (SelectorKind::Highest, "h")
            }
            Token::SelectorLow => {
                self.eat(Token::SelectorLow)?;
                (SelectorKind::Lowest, "l")
            }
            Token::Greater => {
                self.eat(Token::Greater)?;
                (SelectorKind::GreaterThan, ">")
            }
            Token::GreaterEqual => {
                self.eat(Token::GreaterEqual)?;
                (SelectorKind::GreaterThanOrEqual, ">=")
            }
            Token::Less => {
                self.eat(Token::Less)?;
                (SelectorKind::LessThan, "<")
            }
            Token::LessEqual => {
                self.eat(Token::LessEqual)?;
                (SelectorKind::LessThanOrEqual, "<=")
            }
            Token::EqualEqual => {
                self.eat(Token::EqualEqual)?;
                (SelectorKind::EqualTo, "==")
            }
            Token::NotEqual => {
                self.eat(Token::NotEqual)?;
                (SelectorKind::NotEqual, "!=")
            }
            _ => (SelectorKind::Literal, "literal"),
        };

        if !self.selector_value_starts(&self.cur_token) {
            let label = if kind == SelectorKind::Literal {
                "selector"
            } else {
                prefix
            };
            return Err(RollatoriumError::Parser(format!(
                "Expected selector target after '{}' in '{}'",
                label, self.input
            )));
        }

        let target = self.with_selector_context(|parser| parser.parse_selector_value_inner())?;
        Ok(Selector {
            kind,
            target: Box::new(target),
        })
    }

    fn is_selector_start(&self, token: &Token<'a>) -> bool {
        matches!(
            token,
            Token::SelectorHigh
                | Token::SelectorLow
                | Token::Greater
                | Token::GreaterEqual
                | Token::Less
                | Token::LessEqual
                | Token::EqualEqual
                | Token::NotEqual
                | Token::Plus
                | Token::Minus
                | Token::Number(_)
                | Token::LParen
                | Token::Dice
                | Token::DicePercent
        )
    }

    fn selector_value_starts(&self, token: &Token<'a>) -> bool {
        matches!(
            token,
            Token::Plus
                | Token::Minus
                | Token::Number(_)
                | Token::LParen
                | Token::Dice
                | Token::DicePercent
        )
    }

    fn with_selector_context<F>(&mut self, f: F) -> Result<Node<&'a str>>
    where
        F: FnOnce(&mut Self) -> Result<Node<&'a str>>,
    {
        self.selector_depth += 1;
        let result = f(self);
        self.selector_depth -= 1;
        result
    }

    fn parse_selector_value_inner(&mut self) -> Result<Node<&'a str>> {
        match &self.cur_token {
            Token::Plus => {
                self.eat(Token::Plus)?;
                Ok(Node::Unary {
                    operator: UnaryOperator::Plus,
                    operand: Box::new(self.parse_selector_value_inner()?),
                })
            }
            Token::Minus => {
                self.eat(Token::Minus)?;
                Ok(Node::Unary {
                    operator: UnaryOperator::Minus,
                    operand: Box::new(self.parse_selector_value_inner()?),
                })
            }
            Token::Number(value) => {
                let literal = Node::Literal(*value);
                self.eat(Token::Number(*value))?;
                if matches!(self.cur_token, Token::Dice | Token::DicePercent) {
                    self.parse_dice_literal(Some(literal))
                } else {
                    Ok(literal)
                }
            }
            Token::Dice | Token::DicePercent => self.parse_dice_literal(None),
            Token::LParen => {
                self.eat(Token::LParen)?;
                let expr = if self.cur_token == Token::RParen {
                    return Err(RollatoriumError::Parser(
                        "Empty parentheses are not valid selector targets".into(),
                    ));
                } else {
                    self.parse_comparison()?
                };
                self.eat(Token::RParen)?;
                Ok(expr)
            }
            token => Err(RollatoriumError::Parser(format!(
                "Invalid selector target starting with {:?} in '{}'",
                token, self.input
            ))),
        }
    }

    fn parse_annotations(&mut self, node: Node<&'a str>) -> Result<Node<&'a str>> {
        if self.selector_depth > 0 {
            return Ok(node);
        }

        let base = node;
        let mut annotations = Vec::new();

        while let Token::AnnotationStart = self.cur_token {
            self.eat(Token::AnnotationStart)?;
            // The tag borrows directly from the input (`&'a str`, Copy) — no allocation.
            let tag = match &self.cur_token {
                Token::AnnotationText(value) => *value,
                token => {
                    return Err(RollatoriumError::Parser(format!(
                        "Expected annotation text, found {:?} in '{}'",
                        token, self.input
                    )));
                }
            };
            self.eat(Token::AnnotationText(tag))?;

            if let Token::AnnotationEnd = self.cur_token {
                self.eat(Token::AnnotationEnd)?;
            } else {
                return Err(RollatoriumError::Parser(
                    "Unterminated annotation; expected closing ']'".into(),
                ));
            }

            annotations.push(Annotation { tag });
        }

        if annotations.is_empty() {
            return Ok(base);
        }

        match base {
            Node::Annotated {
                expr,
                annotations: mut existing,
            } => {
                existing.extend(annotations);
                Ok(Node::Annotated {
                    expr,
                    annotations: existing,
                })
            }
            _ => Ok(Node::Annotated {
                expr: Box::new(base),
                annotations,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::ast::{
        Annotation, DiceSize, Node, Selector, SelectorKind, SetOperation, SetOperator,
        UnaryOperator,
    };

    fn parse(input: &str) -> Node<&str> {
        let mut parser = Parser::new(input).expect("lexer to succeed");
        parser.parse().expect("parser to succeed")
    }

    #[test]
    fn parses_basic_dice() {
        let node = parse("4d6");
        assert_eq!(
            node,
            Node::Dice {
                num: Some(Box::new(Node::Literal(4.0))),
                size: DiceSize::Value(Box::new(Node::Literal(6.0))),
            }
        );
    }

    #[test]
    fn parses_percent_dice() {
        let node = parse("d%");
        assert_eq!(
            node,
            Node::Dice {
                num: None,
                size: DiceSize::Percent,
            }
        );
    }

    #[test]
    fn parses_set_literal() {
        let node = parse("(1, 2)");
        assert_eq!(
            node,
            Node::Set {
                elements: vec![Node::Literal(1.0), Node::Literal(2.0)],
                operations: Vec::new(),
            }
        );
    }

    #[test]
    fn parses_grouping_without_set() {
        let node = parse("(1 + 2)");
        assert_eq!(
            node,
            Node::Binary {
                operator: crate::ast::BinaryOperator::Add,
                left: Box::new(Node::Literal(1.0)),
                right: Box::new(Node::Literal(2.0)),
            }
        );
    }

    #[test]
    fn parses_dice_with_operations() {
        let node = parse("4d6kh3");
        assert_eq!(
            node,
            Node::DiceWithOps {
                dice: Box::new(Node::Dice {
                    num: Some(Box::new(Node::Literal(4.0))),
                    size: DiceSize::Value(Box::new(Node::Literal(6.0))),
                }),
                operations: vec![SetOperation {
                    operator: SetOperator::Keep,
                    selectors: vec![Selector {
                        kind: SelectorKind::Highest,
                        target: Box::new(Node::Literal(3.0)),
                    }],
                }],
            }
        );
    }

    #[test]
    fn parses_annotations() {
        let node = parse("3d6 [fire]");
        assert_eq!(
            node,
            Node::Annotated {
                expr: Box::new(Node::Dice {
                    num: Some(Box::new(Node::Literal(3.0))),
                    size: DiceSize::Value(Box::new(Node::Literal(6.0))),
                }),
                annotations: vec![Annotation { tag: "fire" }],
            }
        );
    }

    #[test]
    fn parses_unary_in_selector() {
        let node = parse("d6k-1");
        assert_eq!(
            node,
            Node::DiceWithOps {
                dice: Box::new(Node::Dice {
                    num: None,
                    size: DiceSize::Value(Box::new(Node::Literal(6.0))),
                }),
                operations: vec![SetOperation {
                    operator: SetOperator::Keep,
                    selectors: vec![Selector {
                        kind: SelectorKind::Literal,
                        target: Box::new(Node::Unary {
                            operator: UnaryOperator::Minus,
                            operand: Box::new(Node::Literal(1.0)),
                        }),
                    }],
                }],
            }
        );
    }
}
