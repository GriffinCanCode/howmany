//! JavaScript's solidus has two lexical goals: a regular-expression literal
//! starts an expression, while division follows an expression. Keep that goal
//! with the delimiter roles instead of guessing from slash-delimited text.
//!
//! Regex bodies follow ECMAScript 12.9.5: escaped characters and character
//! classes protect a slash; quotes inside a regex have no string semantics.

#[derive(Clone, Copy, PartialEq, Eq)]
enum Goal {
    Statement,
    Expression,
    Division,
    Property,
}

#[derive(Clone, Copy)]
enum Parentheses {
    Expression,
    Control,
    Parameters { expression: bool },
}

#[derive(Clone, Copy)]
enum Body {
    Statement,
    Function { expression: bool },
    Arrow,
    Class { expression: bool },
}

#[derive(Clone, Copy)]
enum Delimiter {
    Parentheses(Parentheses),
    Array,
    Object,
    Body(Body),
}

struct PendingBody {
    role: Body,
    depth: usize,
}

/// Token context shared across physical lines. Comments do not alter it.
pub(super) struct JavaScript {
    goal: Goal,
    delimiters: Vec<Delimiter>,
    parentheses: Option<Parentheses>,
    bodies: Vec<PendingBody>,
    restricted_statement: bool,
}

impl JavaScript {
    pub(super) fn new() -> Self {
        Self {
            goal: Goal::Statement,
            delimiters: Vec::new(),
            parentheses: None,
            bodies: Vec::new(),
            restricted_statement: false,
        }
    }

    /// A completed string, template or regex is an operand, never an operator.
    pub(super) fn literal(&mut self) {
        self.goal = Goal::Division;
        if self
            .bodies
            .last()
            .is_some_and(|body| matches!(body.role, Body::Arrow))
        {
            self.bodies.pop();
        }
    }

    pub(super) fn line_terminator(&mut self) {
        if self.restricted_statement {
            self.goal = Goal::Statement;
            self.restricted_statement = false;
        }
    }

    fn identifier(&mut self, word: &[u8]) {
        if self.goal == Goal::Property {
            self.goal = Goal::Division;
            return;
        }
        match word {
            b"break" | b"continue" if self.goal == Goal::Statement => {
                self.restricted_statement = true;
            }
            b"if" | b"while" | b"for" | b"with" | b"switch" | b"catch" => {
                self.parentheses = Some(Parentheses::Control);
                self.goal = Goal::Expression;
            }
            b"function" => {
                self.parentheses = Some(Parentheses::Parameters {
                    expression: self.goal != Goal::Statement,
                });
                self.goal = Goal::Expression;
            }
            b"class" => {
                self.bodies.push(PendingBody {
                    role: Body::Class {
                        expression: self.goal != Goal::Statement,
                    },
                    depth: self.delimiters.len(),
                });
                self.goal = Goal::Expression;
            }
            b"return" | b"throw" | b"case" | b"delete" | b"void" | b"typeof" | b"new" | b"in"
            | b"instanceof" | b"yield" | b"await" | b"const" | b"let" | b"var" | b"extends"
            | b"as" | b"satisfies" => self.goal = Goal::Expression,
            b"else" | b"do" | b"try" | b"finally" => self.goal = Goal::Statement,
            // Declaration modifiers preserve whether a declaration or an
            // expression is being introduced, e.g. `export default function`.
            b"export" | b"default" | b"async" | b"declare" | b"abstract" => {}
            _ => self.goal = Goal::Division,
        }
    }

    /// Consume source tokens between literals and comments. Token boundaries,
    /// rather than substring matches, determine reserved-word grammar roles.
    pub(super) fn tokens(&mut self, source: &[u8]) {
        let mut at = 0;
        while at < source.len() {
            let byte = source[at];
            if byte.is_ascii_whitespace() {
                at += 1;
                continue;
            }
            if byte != b'{'
                && self
                    .bodies
                    .last()
                    .is_some_and(|body| matches!(body.role, Body::Arrow))
            {
                self.bodies.pop();
            }
            if identifier_start(byte) {
                let end = at
                    + source[at..]
                        .iter()
                        .take_while(|byte| identifier_part(**byte))
                        .count();
                self.identifier(&source[at..end]);
                at = end;
                continue;
            }
            if byte.is_ascii_digit() {
                at += 1;
                while source
                    .get(at)
                    .is_some_and(|byte| identifier_part(*byte) || *byte == b'.')
                {
                    at += 1;
                }
                self.goal = Goal::Division;
                continue;
            }
            match byte {
                b'(' => {
                    let role = self.parentheses.take().unwrap_or(Parentheses::Expression);
                    self.delimiters.push(Delimiter::Parentheses(role));
                    self.goal = Goal::Expression;
                }
                b')' => match self.delimiters.pop() {
                    Some(Delimiter::Parentheses(Parentheses::Control)) => {
                        self.goal = Goal::Statement;
                    }
                    Some(Delimiter::Parentheses(Parentheses::Parameters { expression })) => {
                        self.bodies.push(PendingBody {
                            role: Body::Function { expression },
                            depth: self.delimiters.len(),
                        });
                        self.goal = Goal::Statement;
                    }
                    _ => self.goal = Goal::Division,
                },
                b'[' => {
                    self.delimiters.push(Delimiter::Array);
                    self.goal = Goal::Expression;
                }
                b']' => {
                    self.delimiters.pop();
                    self.goal = Goal::Division;
                }
                b'{' => {
                    let pending = if self
                        .bodies
                        .last()
                        .is_some_and(|body| body.depth == self.delimiters.len())
                    {
                        self.bodies.pop().map(|body| body.role)
                    } else {
                        None
                    };
                    let delimiter = pending.map_or_else(
                        || {
                            if self.goal == Goal::Statement {
                                Delimiter::Body(Body::Statement)
                            } else {
                                Delimiter::Object
                            }
                        },
                        Delimiter::Body,
                    );
                    self.goal = if matches!(delimiter, Delimiter::Object) {
                        Goal::Expression
                    } else {
                        Goal::Statement
                    };
                    self.delimiters.push(delimiter);
                }
                b'}' => {
                    self.goal = match self.delimiters.pop() {
                        Some(Delimiter::Body(Body::Statement))
                        | Some(Delimiter::Body(Body::Function { expression: false }))
                        | Some(Delimiter::Body(Body::Class { expression: false })) => {
                            Goal::Statement
                        }
                        _ => Goal::Division,
                    };
                }
                b';' => {
                    let depth = self.delimiters.len();
                    self.bodies.retain(|body| body.depth < depth);
                    self.goal = Goal::Statement;
                }
                b'.' => {
                    if source.get(at..at + 3) == Some(b"...") {
                        at += 2;
                        self.goal = Goal::Expression;
                    } else {
                        self.goal = Goal::Property;
                    }
                }
                b'=' if source.get(at + 1) == Some(&b'>') => {
                    at += 1;
                    self.bodies.push(PendingBody {
                        role: Body::Arrow,
                        depth: self.delimiters.len(),
                    });
                    self.goal = Goal::Expression;
                }
                b'+' | b'-' if source.get(at + 1) == Some(&byte) => {
                    at += 1;
                    // Prefix updates still expect their operand; postfix
                    // updates have already completed one.
                    if self.goal != Goal::Division {
                        self.goal = Goal::Expression;
                    }
                }
                _ => self.goal = Goal::Expression,
            }
            at += 1;
        }
    }

    /// Consume a slash that is neither `//` nor `/*`. Regex bodies cannot span
    /// physical lines; an unfinished editor buffer therefore cannot poison the
    /// quote state of subsequent lines.
    pub(super) fn slash(&mut self, line: &[u8], at: usize) -> usize {
        if matches!(self.goal, Goal::Division | Goal::Property) {
            self.goal = Goal::Expression;
            return at
                + if line.get(at + 1) == Some(&b'=') {
                    2
                } else {
                    1
                };
        }
        let mut cursor = at + 1;
        let mut in_class = false;
        while cursor < line.len() {
            match line[cursor] {
                b'\\' => cursor = (cursor + 2).min(line.len()),
                b'[' => {
                    in_class = true;
                    cursor += 1;
                }
                b']' => {
                    in_class = false;
                    cursor += 1;
                }
                b'/' if !in_class => {
                    cursor += 1;
                    while line.get(cursor).is_some_and(|byte| identifier_part(*byte)) {
                        cursor += 1;
                    }
                    self.literal();
                    return cursor;
                }
                _ => cursor += 1,
            }
        }
        self.literal();
        line.len()
    }
}

fn identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') || !byte.is_ascii()
}

fn identifier_part(byte: u8) -> bool {
    identifier_start(byte) || byte.is_ascii_digit()
}
