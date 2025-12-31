// ==========================================
// 1. Lambda 求解器核心 (你的代码封装在此)
// ==========================================

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Variable(char),
    Lambda(char, Rc<Expression>),
    Applied(Rc<Expression>, Rc<Expression>),
}

impl Expression {
    pub fn var(c: char) -> Rc<Self> { Rc::new(Expression::Variable(c)) }
    pub fn lambda(param: char, body: Rc<Expression>) -> Rc<Self> { Rc::new(Expression::Lambda(param, body)) }
    pub fn apply(func: Rc<Expression>, arg: Rc<Expression>) -> Rc<Self> { Rc::new(Expression::Applied(func, arg)) }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Variable(c) => write!(f, "{}", c),
            Expression::Lambda(param, body) => {
                let mut params = vec![*param];
                let mut current = body.as_ref();
                while let Expression::Lambda(p, b) = current {
                    params.push(*p);
                    current = b.as_ref();
                }
                write!(f, "λ{}.{}", params.iter().collect::<String>(), current)
            }
            Expression::Applied(func, arg) => {
                match func.as_ref() {
                    Expression::Variable(_) | Expression::Applied(_, _) => write!(f, "{}", func)?,
                    Expression::Lambda(_, _) => write!(f, "({})", func)?,
                }
                match arg.as_ref() {
                    Expression::Variable(_) => write!(f, "{}", arg),
                    _ => write!(f, "({})", arg),
                }.expect("TODO: panic message");
                Ok(())
            }
        }
    }
}

pub fn alpha_equal(expr1: &Expression, expr2: &Expression, mapping: &mut HashMap<char, char>) -> bool {
    match (expr1, expr2) {
        (Expression::Variable(v1), Expression::Variable(v2)) => {
            if let Some(mapped) = mapping.get(v1) { *mapped == *v2 } else { mapping.insert(*v1, *v2); true }
        }
        (Expression::Lambda(p1, b1), Expression::Lambda(p2, b2)) => {
            let mut new_mapping = mapping.clone();
            new_mapping.insert(*p1, *p2);
            alpha_equal(b1.as_ref(), b2.as_ref(), &mut new_mapping)
        }
        (Expression::Applied(f1, a1), Expression::Applied(f2, a2)) => {
            alpha_equal(f1.as_ref(), f2.as_ref(), mapping) && alpha_equal(a1.as_ref(), a2.as_ref(), mapping)
        }
        _ => false,
    }
}

pub fn α_equal(expr1: &Expression, expr2: &Expression) -> bool {
    alpha_equal(expr1, expr2, &mut HashMap::new())
}

fn free_variables(expr: &Expression, bound: &HashSet<char>, free: &mut HashSet<char>) {
    match expr {
        Expression::Variable(v) => { if !bound.contains(v) { free.insert(*v); } }
        Expression::Lambda(param, body) => {
            let mut new_bound = bound.clone(); new_bound.insert(*param);
            free_variables(body.as_ref(), &new_bound, free);
        }
        Expression::Applied(func, arg) => {
            free_variables(func.as_ref(), bound, free);
            free_variables(arg.as_ref(), bound, free);
        }
    }
}

pub fn get_free_variables(expr: &Expression) -> HashSet<char> {
    let mut free = HashSet::new(); free_variables(expr, &HashSet::new(), &mut free); free
}

fn generate_fresh_var(existing: &HashSet<char>) -> char {
    for c in 'a'..='z' { if !existing.contains(&c) { return c; } }
    'z'
}

fn α_convert_inner(expr: &Expression, old_param: char, new_param: char, bound: &HashSet<char>) -> Rc<Expression> {
    match expr {
        Expression::Variable(v) => if *v == old_param { Expression::var(new_param) } else { Expression::var(*v) },
        Expression::Lambda(param, body) => {
            if *param == old_param { Expression::lambda(new_param, α_convert_inner(body.as_ref(), old_param, new_param, bound)) }
            else {
                let mut new_bound = bound.clone(); new_bound.insert(*param);
                Expression::lambda(*param, α_convert_inner(body.as_ref(), old_param, new_param, &new_bound))
            }
        }
        Expression::Applied(func, arg) => Expression::apply(
            α_convert_inner(func.as_ref(), old_param, new_param, bound),
            α_convert_inner(arg.as_ref(), old_param, new_param, bound),
        )
    }
}

pub fn substitute(target: &Expression, var: char, replacement: &Expression) -> Rc<Expression> {
    let free_vars_repl = get_free_variables(replacement);
    fn substitute_inner(expr: &Expression, var: char, replacement: &Expression, bound: &HashSet<char>, free_vars_repl: &HashSet<char>) -> Rc<Expression> {
        match expr {
            Expression::Variable(v) => if *v == var { Rc::from(replacement.clone()) } else { Expression::var(*v) },
            Expression::Lambda(param, body) => {
                if *param == var { Expression::lambda(*param, body.clone()) }
                else if free_vars_repl.contains(param) {
                    let mut all_vars = bound.clone(); all_vars.insert(*param);
                    free_vars_repl.iter().for_each(|&v| { all_vars.insert(v); });
                    let fresh = generate_fresh_var(&all_vars);
                    let mut new_bound = bound.clone(); new_bound.insert(fresh);
                    let new_body = α_convert_inner(body.as_ref(), *param, fresh, &new_bound);
                    Expression::lambda(fresh, substitute_inner(new_body.as_ref(), var, replacement, &new_bound, free_vars_repl))
                } else {
                    let mut new_bound = bound.clone(); new_bound.insert(*param);
                    Expression::lambda(*param, substitute_inner(body.as_ref(), var, replacement, &new_bound, free_vars_repl))
                }
            }
            Expression::Applied(func, arg) => Expression::apply(
                substitute_inner(func.as_ref(), var, replacement, bound, free_vars_repl),
                substitute_inner(arg.as_ref(), var, replacement, bound, free_vars_repl),
            )
        }
    }
    substitute_inner(target, var, replacement, &HashSet::new(), &free_vars_repl)
}

pub fn beta_reduce(expr: &Expression) -> Rc<Expression> {
    match expr {
        Expression::Variable(_) => Rc::new(expr.clone()),
        Expression::Lambda(param, body) => Expression::lambda(*param, beta_reduce(body.as_ref())),
        Expression::Applied(func, arg) => {
            match func.as_ref() {
                Expression::Lambda(param, body) => {
                    // 修正：Call-by-name 策略，参数先不归约，直接代入
                    // 如果想做 Call-by-value，这里需要先 reduced_arg = beta_reduce(arg)
                    substitute(body.as_ref(), *param, arg)
                }
                _ => {
                    let reduced_func = beta_reduce(func.as_ref());
                    let reduced_arg = beta_reduce(arg.as_ref()); // 递归归约子项
                    if let Expression::Lambda(param, body) = reduced_func.as_ref() {
                        substitute(body.as_ref(), *param, &reduced_arg)
                    } else {
                        Expression::apply(reduced_func, reduced_arg)
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEndOfInput,
    InvalidCharacter(char),
    InvalidLambdaSyntax,
    MismatchedParentheses,
}

// 简单的 Display 实现以便 UI 显示错误
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn parse(input: &str) -> Result<Rc<Expression>, ParseError> {
    // UI 修正：先过滤掉所有空白字符
    let clean_input: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let mut chars = clean_input.chars().peekable();
    parse_expression(&mut chars, false)
}

fn parse_expression<I>(chars: &mut std::iter::Peekable<I>, in_lambda: bool) -> Result<Rc<Expression>, ParseError>
where I: Iterator<Item = char> {
    let mut expr = parse_atom(chars, in_lambda)?;
    while let Some(&c) = chars.peek() {
        if c == ')' || (in_lambda && c == '.') { break; }
        let next_expr = parse_atom(chars, in_lambda)?;
        expr = Expression::apply(expr, next_expr);
    }
    Ok(expr)
}

fn parse_atom<I>(chars: &mut std::iter::Peekable<I>, in_lambda: bool) -> Result<Rc<Expression>, ParseError>
where I: Iterator<Item = char> {
    match chars.peek() {
        Some(&c) if c.is_ascii_lowercase() => { chars.next(); Ok(Expression::var(c)) }
        Some(&c) if c == '\\' || c == 'λ' => {
            chars.next();
            let mut params = Vec::new();
            while let Some(&c) = chars.peek() {
                if c == '.' { break; }
                if c.is_ascii_lowercase() { params.push(c); chars.next(); }
                else { return Err(ParseError::InvalidLambdaSyntax); }
            }
            if params.is_empty() { return Err(ParseError::InvalidLambdaSyntax); }
            if chars.next() != Some('.') { return Err(ParseError::InvalidLambdaSyntax); }
            let body = parse_expression(chars, true)?;
            let mut expr = body;
            for param in params.into_iter().rev() { expr = Expression::lambda(param, expr); }
            Ok(expr)
        }
        Some(&'(') => {
            chars.next();
            let expr = parse_expression(chars, in_lambda)?;
            match chars.next() { Some(')') => Ok(expr), _ => Err(ParseError::MismatchedParentheses) }
        }
        Some(&c) => Err(ParseError::InvalidCharacter(c)),
        None => Err(ParseError::UnexpectedEndOfInput),
    }
}
