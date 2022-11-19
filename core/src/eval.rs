use std::sync::Arc;

use crate::{
    ast,
    error::{FendError, Interrupt},
    lexer, parser,
    scope::Scope,
    value::Value,
    Span,
};

pub(crate) fn evaluate_to_value<'a, I: Interrupt>(
    input: &'a str,
    scope: Option<Arc<Scope>>,
    attrs: Attrs,
    context: &mut crate::Context,
    int: &I,
) -> Result<Value, FendError> {
    let lex = lexer::lex(input, int);
    let mut tokens = vec![];
    let mut missing_open_parens: i32 = 0;
    for token in lex {
        let token = token?;
        if let lexer::Token::Symbol(lexer::Symbol::CloseParens) = token {
            missing_open_parens += 1;
        }
        tokens.push(token);
    }
    for _ in 0..missing_open_parens {
        tokens.insert(0, lexer::Token::Symbol(lexer::Symbol::OpenParens));
    }
    let parsed = parser::parse_tokens(&tokens)?;
    let result = ast::evaluate(parsed, scope, attrs, context, int)?;
    Ok(result)
}

#[derive(Clone, Copy)]
pub(crate) struct Attrs {
    pub(crate) debug: bool,
    pub(crate) show_approx: bool,
}

fn parse_attrs(mut input: &str) -> (Attrs, &str) {
    let mut attrs = Attrs {
        debug: false,
        show_approx: true,
    };
    while input.starts_with('@') {
        if let Some(remaining) = input.strip_prefix("@debug ") {
            attrs.debug = true;
            input = remaining;
        } else if let Some(remaining) = input.strip_prefix("@noapprox ") {
            attrs.show_approx = false;
            input = remaining;
        }
    }
    (attrs, input)
}

/// This also saves the calculation result in a variable `_` and `ans`
pub(crate) fn evaluate_to_spans<'a, I: Interrupt>(
    input: &'a str,
    scope: Option<Arc<Scope>>,
    context: &mut crate::Context,
    int: &I,
) -> Result<(Vec<Span>, bool), FendError> {
    let (attrs, input) = parse_attrs(input);
    let value = evaluate_to_value(input, scope, attrs, context, int)?;
    context.variables.insert("_".to_string(), value.clone());
    context.variables.insert("ans".to_string(), value.clone());
    Ok((
        if attrs.debug {
            vec![Span::from_string(format!("{:?}", value))]
        } else {
            let mut spans = vec![];
            value.format(0, &mut spans, attrs, context, int)?;
            spans
        },
        value.is_unit(),
    ))
}
