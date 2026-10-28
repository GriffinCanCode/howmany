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
    jsx: Option<Jsx>,
}

impl JavaScript {
    pub(super) fn new() -> Self {
        Self {
            goal: Goal::Statement,
            delimiters: Vec::new(),
            parentheses: None,
            bodies: Vec::new(),
            restricted_statement: false,
            jsx: None,
        }
    }

    pub(super) fn with_jsx(typescript: bool) -> Self {
        Self {
            jsx: Some(Jsx::new(typescript)),
            ..Self::new()
        }
    }

    pub(super) fn comments_allowed(&self) -> bool {
        self.jsx.as_ref().is_none_or(Jsx::comments_allowed)
    }

    pub(super) fn attribute_string(&self) -> bool {
        self.jsx
            .as_ref()
            .is_some_and(|jsx| matches!(jsx.mode, JsxMode::Tag(tag) if tag.type_arguments == 0))
    }

    /// JSX text and attributes are not JavaScript token streams. Embedded
    /// braces temporarily restore JavaScript, including its regex goal.
    pub(super) fn jsx_step(&mut self, line: &[u8], at: usize) -> Option<usize> {
        let allowed = matches!(self.goal, Goal::Statement | Goal::Expression);
        let step = self.jsx.as_mut()?.step(line, at, allowed);
        match step {
            JsxStep::JavaScript => None,
            JsxStep::ResumeGeneric => {
                self.goal = Goal::Expression;
                None
            }
            JsxStep::Code { end, event } => {
                match event {
                    JsxEvent::None => {}
                    JsxEvent::Value => self.literal(),
                    JsxEvent::EnterExpression => self.tokens(b"{"),
                    JsxEvent::ExitExpression => self.tokens(b"}"),
                }
                Some(end)
            }
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

#[derive(Clone, Copy)]
enum JsxMode {
    JavaScript,
    Text,
    Tag(Tag),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TypeParameter {
    NotAmbiguous,
    NameAfterConst,
    AfterName,
    AfterExtends,
}

#[derive(Clone, Copy)]
struct Tag {
    closing: bool,
    self_closing: bool,
    type_parameter: TypeParameter,
    type_arguments: usize,
}

struct JsxExpression {
    depth: usize,
    return_mode: JsxMode,
}

struct Jsx {
    mode: JsxMode,
    elements: Vec<JsxMode>,
    expressions: Vec<JsxExpression>,
    typescript: bool,
}

enum JsxEvent {
    None,
    Value,
    EnterExpression,
    ExitExpression,
}

enum JsxStep {
    JavaScript,
    ResumeGeneric,
    Code { end: usize, event: JsxEvent },
}

impl Jsx {
    fn new(typescript: bool) -> Self {
        Self {
            mode: JsxMode::JavaScript,
            elements: Vec::new(),
            expressions: Vec::new(),
            typescript,
        }
    }

    fn comments_allowed(&self) -> bool {
        match self.mode {
            JsxMode::JavaScript => true,
            JsxMode::Text => false,
            JsxMode::Tag(tag) => {
                tag.type_parameter != TypeParameter::NotAmbiguous || tag.type_arguments > 0
            }
        }
    }

    fn begin_tag(&mut self, line: &[u8], at: usize) -> Option<JsxStep> {
        let closing = line.get(at + 1) == Some(&b'/');
        let start = at + if closing { 2 } else { 1 };
        let &first = line.get(start)?;
        if first != b'>' && !identifier_start(first) {
            return None;
        }
        let length = line[start..]
            .iter()
            .take_while(|byte| identifier_part(**byte) || matches!(byte, b'-' | b'.' | b':'))
            .count();
        let name = &line[start..start + length];
        let type_parameter = if self.typescript
            && matches!(self.mode, JsxMode::JavaScript)
            && !closing
            && !name.is_empty()
            && name.iter().all(|byte| identifier_part(*byte))
        {
            if name == b"const" {
                TypeParameter::NameAfterConst
            } else {
                TypeParameter::AfterName
            }
        } else {
            TypeParameter::NotAmbiguous
        };
        if !closing {
            self.elements.push(self.mode);
        }
        self.mode = JsxMode::Tag(Tag {
            closing,
            self_closing: false,
            type_parameter,
            type_arguments: 0,
        });
        Some(JsxStep::Code {
            end: start + length,
            event: JsxEvent::Value,
        })
    }

    fn expression(&mut self, at: usize) -> JsxStep {
        self.expressions.push(JsxExpression {
            depth: 0,
            return_mode: self.mode,
        });
        self.mode = JsxMode::JavaScript;
        JsxStep::Code {
            end: at + 1,
            event: JsxEvent::EnterExpression,
        }
    }

    fn finish_tag(&mut self, end: usize, closing: bool) -> JsxStep {
        self.mode = if closing {
            self.elements.pop().unwrap_or(JsxMode::JavaScript)
        } else {
            JsxMode::Text
        };
        JsxStep::Code {
            end,
            event: JsxEvent::Value,
        }
    }

    fn generic(&mut self) -> JsxStep {
        self.mode = self.elements.pop().unwrap_or(JsxMode::JavaScript);
        JsxStep::ResumeGeneric
    }

    fn step(&mut self, line: &[u8], at: usize, expression_allowed: bool) -> JsxStep {
        match self.mode {
            JsxMode::JavaScript => {
                if line[at] == b'<' && expression_allowed {
                    if let Some(step) = self.begin_tag(line, at) {
                        return step;
                    }
                }
                if let Some(expression) = self.expressions.last_mut() {
                    match line[at] {
                        b'{' => expression.depth += 1,
                        b'}' if expression.depth > 0 => expression.depth -= 1,
                        b'}' => {
                            self.mode = self
                                .expressions
                                .pop()
                                .map_or(JsxMode::JavaScript, |expression| expression.return_mode);
                            return JsxStep::Code {
                                end: at + 1,
                                event: JsxEvent::ExitExpression,
                            };
                        }
                        _ => {}
                    }
                }
                JsxStep::JavaScript
            }
            JsxMode::Text => {
                if line[at] == b'{' {
                    return self.expression(at);
                }
                if line[at] == b'<' {
                    if let Some(step) = self.begin_tag(line, at) {
                        return step;
                    }
                }
                let next = memchr::memchr2(b'<', b'{', &line[at + 1..])
                    .map_or(line.len(), |offset| at + offset + 1);
                JsxStep::Code {
                    end: next,
                    event: JsxEvent::None,
                }
            }
            JsxMode::Tag(mut tag) => {
                let byte = line[at];
                if byte.is_ascii_whitespace() {
                    return JsxStep::Code {
                        end: at + 1,
                        event: JsxEvent::None,
                    };
                }
                if tag.type_arguments > 0
                    && (line[at..].starts_with(b"//") || line[at..].starts_with(b"/*"))
                {
                    return JsxStep::JavaScript;
                }
                // TypeScript's TSX grammar distinguishes a type-parameter
                // head by comma/default/constraint tokens, including across
                // physical lines. Ordinary JSX attribute names remain tags.
                if tag.type_parameter != TypeParameter::NotAmbiguous {
                    if line[at..].starts_with(b"//") || line[at..].starts_with(b"/*") {
                        return JsxStep::JavaScript;
                    }
                    let length = line[at..]
                        .iter()
                        .take_while(|byte| identifier_part(**byte))
                        .count();
                    let word = &line[at..at + length];
                    match tag.type_parameter {
                        TypeParameter::NameAfterConst if length > 0 => {
                            tag.type_parameter = TypeParameter::AfterName;
                            self.mode = JsxMode::Tag(tag);
                            return JsxStep::Code {
                                end: at + length,
                                event: JsxEvent::None,
                            };
                        }
                        TypeParameter::AfterName if matches!(byte, b',' | b'=') => {
                            return self.generic()
                        }
                        TypeParameter::AfterName if word == b"extends" => {
                            tag.type_parameter = TypeParameter::AfterExtends;
                            self.mode = JsxMode::Tag(tag);
                            return JsxStep::Code {
                                end: at + length,
                                event: JsxEvent::None,
                            };
                        }
                        TypeParameter::AfterExtends if !matches!(byte, b'=' | b'>' | b'/') => {
                            return self.generic()
                        }
                        _ => tag.type_parameter = TypeParameter::NotAmbiguous,
                    }
                }
                if byte == b'<' {
                    tag.type_arguments += 1;
                } else if byte == b'>' && tag.type_arguments > 0 {
                    tag.type_arguments -= 1;
                } else if byte == b'>' {
                    return self.finish_tag(at + 1, tag.closing || tag.self_closing);
                } else if byte == b'/' && line.get(at + 1) == Some(&b'>') && tag.type_arguments == 0
                {
                    return self.finish_tag(at + 2, true);
                } else if byte == b'/' && tag.type_arguments == 0 {
                    tag.self_closing = true;
                } else if byte == b'{' && tag.type_arguments == 0 {
                    self.mode = JsxMode::Tag(tag);
                    return self.expression(at);
                } else if matches!(byte, b'\'' | b'"') {
                    self.mode = JsxMode::Tag(tag);
                    return JsxStep::JavaScript;
                }
                self.mode = JsxMode::Tag(tag);
                JsxStep::Code {
                    end: at + 1,
                    event: JsxEvent::None,
                }
            }
        }
    }
}
