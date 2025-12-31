// $$\                               $$\             $$\
// $$ |                              $$ |            $$ |
// $$ |       $$$$$$\  $$$$$$\$$$$\  $$$$$$$\   $$$$$$$ | $$$$$$\
// $$ |       \____$$\ $$  _$$  _$$\ $$  __$$\ $$  __$$ | \____$$\
// $$ |       $$$$$$$ |$$ / $$ / $$ |$$ |  $$ |$$ /  $$ | $$$$$$$ |
// $$ |      $$  __$$ |$$ | $$ | $$ |$$ |  $$ |$$ |  $$ |$$  __$$ |
// $$$$$$$$\ \$$$$$$$ |$$ | $$ | $$ |$$$$$$$  |\$$$$$$$ |\$$$$$$$ |
// \________| \_______|\__| \__| \__|\_______/  \_______| \_______|

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

// 类型别名：环境表
pub type Environment = HashMap<String, Rc<Expression>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Variable(char),
    Lambda(char, Rc<Expression>),
    Applied(Rc<Expression>, Rc<Expression>),
    // [新增] 宏引用：存储宏的名字，比如 "Plus", "One"
    Constant(String),
}

impl Expression {
    pub fn var(c: char) -> Rc<Self> { Rc::new(Expression::Variable(c)) }
    pub fn lambda(param: char, body: Rc<Expression>) -> Rc<Self> { Rc::new(Expression::Lambda(param, body)) }
    pub fn apply(func: Rc<Expression>, arg: Rc<Expression>) -> Rc<Self> { Rc::new(Expression::Applied(func, arg)) }
    pub fn constant(name: impl Into<String>) -> Rc<Self> { Rc::new(Expression::Constant(name.into())) }
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
                    Expression::Variable(_) | Expression::Applied(_, _) | Expression::Constant(_) => write!(f, "{}", func)?,
                    Expression::Lambda(_, _) => write!(f, "({})", func)?,
                }
                match arg.as_ref() {
                    Expression::Variable(_) | Expression::Constant(_) => write!(f, "{}", arg),
                    _ => write!(f, "({})", arg),
                }
            }
            // [新增] 显示宏名字
            Expression::Constant(name) => write!(f, "[{}]", name),
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
        // [新增] 宏相等性：名字相同即相同
        (Expression::Constant(n1), Expression::Constant(n2)) => n1 == n2,
        _ => false,
    }
}

pub fn a_equal(expr1: &Expression, expr2: &Expression) -> bool {
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
        // [新增] 宏没有自由变量（它是闭合的或者全局的）
        Expression::Constant(_) => {}
    }
}

pub fn get_free_variables(expr: &Expression) -> HashSet<char> {
    let mut free = HashSet::new(); free_variables(expr, &HashSet::new(), &mut free); free
}

fn generate_fresh_var(existing: &HashSet<char>) -> char {
    for c in 'a'..='z' { if !existing.contains(&c) { return c; } }
    'z'
}

fn a_convert_inner(expr: &Expression, old_param: char, new_param: char, bound: &HashSet<char>) -> Rc<Expression> {
    match expr {
        Expression::Variable(v) => if *v == old_param { Expression::var(new_param) } else { Expression::var(*v) },
        Expression::Lambda(param, body) => {
            if *param == old_param { Expression::lambda(new_param, a_convert_inner(body.as_ref(), old_param, new_param, bound)) }
            else {
                let mut new_bound = bound.clone(); new_bound.insert(*param);
                Expression::lambda(*param, a_convert_inner(body.as_ref(), old_param, new_param, &new_bound))
            }
        }
        Expression::Applied(func, arg) => Expression::apply(
            a_convert_inner(func.as_ref(), old_param, new_param, bound),
            a_convert_inner(arg.as_ref(), old_param, new_param, bound),
        ),
        // [新增] 宏不受 α 转换影响
        Expression::Constant(_) => Rc::new(expr.clone()),
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
                    let new_body = a_convert_inner(body.as_ref(), *param, fresh, &new_bound);
                    Expression::lambda(fresh, substitute_inner(new_body.as_ref(), var, replacement, &new_bound, free_vars_repl))
                } else {
                    let mut new_bound = bound.clone(); new_bound.insert(*param);
                    Expression::lambda(*param, substitute_inner(body.as_ref(), var, replacement, &new_bound, free_vars_repl))
                }
            }
            Expression::Applied(func, arg) => Expression::apply(
                substitute_inner(func.as_ref(), var, replacement, bound, free_vars_repl),
                substitute_inner(arg.as_ref(), var, replacement, bound, free_vars_repl),
            ),
            // [新增] 宏不参与替换（除非先被展开，否则视作原子）
            Expression::Constant(_) => Rc::new(expr.clone()),
        }
    }
    substitute_inner(target, var, replacement, &HashSet::new(), &free_vars_repl)
}

// [修改] 增加环境表参数
pub fn beta_reduce(expr: &Expression, env: &Environment) -> Rc<Expression> {
    match expr {
        Expression::Variable(_) => Rc::new(expr.clone()),
        // [新增] 宏展开逻辑：惰性展开
        // 如果在环境表中找到了宏定义，就把它展开（Clone一份），作为这一步的归约结果
        Expression::Constant(name) => {
            if let Some(definition) = env.get(name) {
                definition.clone()
            } else {
                // 如果没找到定义（可能是自由宏），保持原样
                Rc::new(expr.clone())
            }
        },
        Expression::Lambda(param, body) => Expression::lambda(*param, beta_reduce(body.as_ref(), env)),
        Expression::Applied(func, arg) => {
            match func.as_ref() {
                Expression::Lambda(param, body) => {
                    // β 归约
                    substitute(body.as_ref(), *param, arg)
                }
                Expression::Constant(name) => {
                    // 特殊情况：如果函数位置是个宏，先展开它
                    if let Some(definition) = env.get(name) {
                        // 展开后应用参数，构成一个新的 Applied
                        // 这里不立即 reduce，留给下一帧，这样用户能看到展开的过程
                        Expression::apply(definition.clone(), arg.clone())
                    } else {
                        // 没找到定义，尝试归约参数
                        Expression::apply(func.clone(), beta_reduce(arg.as_ref(), env))
                    }
                }
                _ => {
                    let reduced_func = beta_reduce(func.as_ref(), env);
                    let reduced_arg = beta_reduce(arg.as_ref(), env);
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
    InvalidMacroDefinition, // 新增
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ==========================================
// 新增：注释预处理器
// ==========================================
fn remove_comments(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        // 检测起始符号 '-'
        if c == '-' {
            //以此窥探下一个字符
            if let Some(&next) = chars.peek() {
                if next == '-' {
                    // 匹配到 '--'，说明是注释开始
                    chars.next(); // 消耗掉第二个 '-'

                    // 进一步检查是否是多行注释 '--/'
                    if let Some(&third) = chars.peek() {
                        if third == '/' {
                            // === 多行注释模式: --/ ... / ===
                            chars.next(); // 消耗掉 '/'

                            // 吞掉后续字符，直到遇到 '/'
                            loop {
                                match chars.next() {
                                    Some('/') => break, // 结束符号
                                    Some(_) => continue, // 注释内容，忽略
                                    None => break, // 文件结束
                                }
                            }
                            continue; // 完成处理，跳过后续追加逻辑
                        }
                    }

                    // === 单行注释模式: -- ... \n ===
                    // 吞掉后续字符，直到遇到换行符
                    loop {
                        match chars.next() {
                            Some('\n') => break, // 换行符，单行注释结束
                            Some(_) => continue, // 注释内容，忽略
                            None => break, // 文件结束
                        }
                    }
                    continue; // 完成处理
                }
            }
        }

        // 非注释内容，保留
        output.push(c);
    }
    output
}

// ==========================================
// 修改后的解析入口
// ==========================================

// 返回 (环境表, 主表达式)
pub fn parse_program(input: &str) -> Result<(Environment, Rc<Expression>), ParseError> {
    // 1. [新增] 预处理：移除注释
    // 必须在移除空白之前进行，因为单行注释依赖 '\n' 来判断结束
    let input_no_comments = remove_comments(input);

    // 2. [原有逻辑] 清理空白
    // 这一步会把剩余代码中的空格、换行全部去掉，变成紧凑格式
    let clean_input: String = input_no_comments.chars().filter(|c| !c.is_whitespace()).collect();

    if clean_input.is_empty() {
        return Err(ParseError::UnexpectedEndOfInput);
    }

    let mut env = Environment::new();

    // 3. 按分号分割
    let mut parts: Vec<&str> = clean_input.split(';').filter(|s| !s.is_empty()).collect();

    if parts.is_empty() {
        return Err(ParseError::UnexpectedEndOfInput);
    }

    // 4. 取出最后一个作为主表达式
    let main_expr_str = parts.pop().unwrap();

    // 5. 解析前面的定义部分
    for def_str in parts {
        let split_def: Vec<&str> = def_str.split(':').collect();
        if split_def.len() != 2 {
            return Err(ParseError::InvalidMacroDefinition);
        }

        let name_part = split_def[0];
        let body_part = split_def[1];

        if !name_part.starts_with('[') || !name_part.ends_with(']') {
            return Err(ParseError::InvalidMacroDefinition);
        }
        let name = &name_part[1..name_part.len() - 1];
        if name.is_empty() {
            return Err(ParseError::InvalidMacroDefinition);
        }

        let body_expr = parse_one_expression(body_part)?;
        env.insert(name.to_string(), body_expr);
    }

    // 6. 解析主表达式
    let main_expr = parse_one_expression(main_expr_str)?;

    Ok((env, main_expr))
}

// parse 接口
fn parse_one_expression(input: &str) -> Result<Rc<Expression>, ParseError> {
    let mut chars = input.chars().peekable();
    parse_expression(&mut chars, false)
}

// 测试单行
pub fn parse(input: &str) -> Result<Rc<Expression>, ParseError> {
    parse_one_expression(&input.chars().filter(|c| !c.is_whitespace()).collect::<String>())
}


fn parse_expression<I>(chars: &mut std::iter::Peekable<I>, in_lambda: bool) -> Result<Rc<Expression>, ParseError>
where I: Iterator<Item = char> {
    let mut expr = parse_atom(chars, in_lambda)?;
    while let Some(&c) = chars.peek() {
        if c == ')' || (in_lambda && c == '.') || c == ';' { break; } // 遇到 ; 也要停（防御性）
        let next_expr = parse_atom(chars, in_lambda)?;
        expr = Expression::apply(expr, next_expr);
    }
    Ok(expr)
}

fn parse_atom<I>(chars: &mut std::iter::Peekable<I>, in_lambda: bool) -> Result<Rc<Expression>, ParseError>
where I: Iterator<Item = char> {
    match chars.peek() {
        Some(&c) if c.is_ascii_lowercase() => { chars.next(); Ok(Expression::var(c)) }
        // [新增] 遇到 [ 解析宏
        Some(&'[') => {
            chars.next(); // 吃掉 [
            let mut name = String::new();
            while let Some(&c) = chars.peek() {
                if c == ']' { break; }
                // 允许宏名包含字母数字等，这里简单处理不校验字符合法性，只要不是 ]
                name.push(c);
                chars.next();
            }
            if name.is_empty() { return Err(ParseError::InvalidLambdaSyntax); }
            if chars.next() != Some(']') { return Err(ParseError::InvalidLambdaSyntax); } // 吃掉 ]
            Ok(Expression::constant(name))
        }
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