use std::collections::HashMap;

/// A parsed segment of a format template.
#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    /// Literal text — rendered as-is.
    Literal(String),
    /// Variable reference — resolved from the variable map.
    Variable(String),
    /// Conditional — rendered only if the variable is non-empty.
    /// Syntax: ${?varname|text with ${var} refs}
    Conditional {
        var: String,
        inner: Vec<Segment>,
    },
}

/// A pre-parsed format template for efficient reuse.
#[derive(Debug, Clone)]
pub struct FormatTemplate {
    pub segments: Vec<Segment>,
}

impl FormatTemplate {
    /// Parse a format string into a template.
    pub fn parse(template: &str) -> Self {
        Self {
            segments: parse_segments(template),
        }
    }

    /// Resolve the template with the given variables.
    pub fn render(&self, vars: &HashMap<&str, &str>) -> String {
        let mut result = String::new();
        render_segments(&self.segments, vars, &mut result);
        result
    }
}

/// Convenience: parse and render in one call.
pub fn format_string(template: &str, vars: &HashMap<&str, &str>) -> String {
    FormatTemplate::parse(template).render(vars)
}

/// Build a variable map from key-value pairs.
#[macro_export]
macro_rules! fmt_vars {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(map.insert($key, $val);)*
        map
    }};
}

fn render_segments(segments: &[Segment], vars: &HashMap<&str, &str>, out: &mut String) {
    for seg in segments {
        match seg {
            Segment::Literal(s) => out.push_str(s),
            Segment::Variable(name) => {
                if let Some(val) = vars.get(name.as_str()) {
                    out.push_str(val);
                }
            }
            Segment::Conditional { var, inner } => {
                let val = vars.get(var.as_str()).copied().unwrap_or("");
                if !val.is_empty() {
                    render_segments(inner, vars, out);
                }
            }
        }
    }
}

fn parse_segments(input: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut chars = input.char_indices().peekable();
    let mut literal = String::new();

    while let Some(&(i, ch)) = chars.peek() {
        if ch == '$' && input[i..].starts_with("${") {
            // Flush literal
            if !literal.is_empty() {
                segments.push(Segment::Literal(std::mem::take(&mut literal)));
            }

            // Skip "${"
            chars.next();
            chars.next();

            // Check for conditional: ${?var|...}
            if input[i..].starts_with("${?") {
                // Already consumed "${", need to skip "?"
                chars.next(); // skip '?'
                let var_and_rest = consume_until_balanced(&mut chars, input);
                if let Some(pipe_pos) = var_and_rest.find('|') {
                    let var = var_and_rest[..pipe_pos].to_string();
                    let inner_str = &var_and_rest[pipe_pos + 1..];
                    let inner = parse_segments(inner_str);
                    segments.push(Segment::Conditional { var, inner });
                }
            } else {
                // Regular variable: ${name}
                let name = consume_until_close(&mut chars);
                segments.push(Segment::Variable(name));
            }
        } else {
            literal.push(ch);
            chars.next();
        }
    }

    if !literal.is_empty() {
        segments.push(Segment::Literal(literal));
    }

    segments
}

/// Consume characters until '}', handling nested '${...}'.
fn consume_until_balanced(
    chars: &mut std::iter::Peekable<std::str::CharIndices>,
    _input: &str,
) -> String {
    let mut result = String::new();
    let mut depth = 1;

    while let Some(&(_, ch)) = chars.peek() {
        chars.next();
        if ch == '}' {
            depth -= 1;
            if depth == 0 {
                break;
            }
            result.push(ch);
        } else if ch == '{' {
            depth += 1;
            result.push(ch);
        } else {
            result.push(ch);
        }
    }

    result
}

/// Consume characters until '}' for a simple ${name} variable.
fn consume_until_close(
    chars: &mut std::iter::Peekable<std::str::CharIndices>,
) -> String {
    let mut name = String::new();
    while let Some(&(_, ch)) = chars.peek() {
        chars.next();
        if ch == '}' {
            break;
        }
        name.push(ch);
    }
    name
}

/// Resolve a regex-based format string where ${0} is full match, ${1} ${2} are capture groups.
pub fn format_regex_captures(template: &str, captures: &regex::Captures) -> String {
    let mut result = String::new();
    let mut chars = template.char_indices().peekable();
    let mut literal = String::new();

    while let Some(&(i, ch)) = chars.peek() {
        if ch == '$' && template[i..].starts_with("${") {
            if !literal.is_empty() {
                result.push_str(&literal);
                literal.clear();
            }
            chars.next();
            chars.next();
            let name = consume_until_close(&mut chars);
            if let Ok(idx) = name.parse::<usize>() {
                if let Some(m) = captures.get(idx) {
                    result.push_str(m.as_str());
                }
            }
        } else {
            literal.push(ch);
            chars.next();
        }
    }
    if !literal.is_empty() {
        result.push_str(&literal);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_variable() {
        let vars = fmt_vars!("nick" => "alice", "text" => "hello");
        assert_eq!(format_string("<${nick}> ${text}", &vars), "<alice> hello");
    }

    #[test]
    fn no_variables() {
        let vars: HashMap<&str, &str> = HashMap::new();
        assert_eq!(format_string("plain text", &vars), "plain text");
    }

    #[test]
    fn missing_variable() {
        let vars = fmt_vars!("nick" => "alice");
        assert_eq!(format_string("<${nick}> ${text}", &vars), "<alice> ");
    }

    #[test]
    fn conditional_present() {
        let vars = fmt_vars!("nick" => "alice", "message" => "goodbye");
        assert_eq!(
            format_string("${nick} left${?message| (${message})}", &vars),
            "alice left (goodbye)"
        );
    }

    #[test]
    fn conditional_absent() {
        let vars = fmt_vars!("nick" => "alice", "message" => "");
        assert_eq!(
            format_string("${nick} left${?message| (${message})}", &vars),
            "alice left"
        );
    }

    #[test]
    fn conditional_missing_key() {
        let vars = fmt_vars!("nick" => "alice");
        assert_eq!(
            format_string("${nick} left${?message| (${message})}", &vars),
            "alice left"
        );
    }

    #[test]
    fn multiple_variables() {
        let vars = fmt_vars!(
            "time" => "12:00",
            "nick" => "bob",
            "channel" => "#rust",
            "userhost" => "bob@host.com"
        );
        assert_eq!(
            format_string("[${time}] --> ${nick} (${userhost}) has joined ${channel}", &vars),
            "[12:00] --> bob (bob@host.com) has joined #rust"
        );
    }

    #[test]
    fn format_template_reuse() {
        let tmpl = FormatTemplate::parse("[${time}] <${nick}> ${text}");
        let vars1 = fmt_vars!("time" => "12:00", "nick" => "alice", "text" => "hi");
        let vars2 = fmt_vars!("time" => "12:01", "nick" => "bob", "text" => "hey");
        assert_eq!(tmpl.render(&vars1), "[12:00] <alice> hi");
        assert_eq!(tmpl.render(&vars2), "[12:01] <bob> hey");
    }

    #[test]
    fn empty_template() {
        let vars: HashMap<&str, &str> = HashMap::new();
        assert_eq!(format_string("", &vars), "");
    }

    #[test]
    fn adjacent_variables() {
        let vars = fmt_vars!("a" => "x", "b" => "y");
        assert_eq!(format_string("${a}${b}", &vars), "xy");
    }

    #[test]
    fn dollar_without_brace() {
        let vars: HashMap<&str, &str> = HashMap::new();
        assert_eq!(format_string("cost is $5", &vars), "cost is $5");
    }

    #[test]
    fn regex_captures_format() {
        let re = regex::Regex::new(r"Client connecting: (\S+) \((\S+)@(\S+)\)").unwrap();
        let text = "Client connecting: alice (alice@host.com)";
        let caps = re.captures(text).unwrap();
        assert_eq!(
            format_regex_captures("[connect] ${1} (${2}@${3})", &caps),
            "[connect] alice (alice@host.com)"
        );
    }

    #[test]
    fn regex_captures_full_match() {
        let re = regex::Regex::new(r"KILL message: (.+)").unwrap();
        let text = "KILL message: foo was killed by bar";
        let caps = re.captures(text).unwrap();
        assert_eq!(
            format_regex_captures("[kill] ${0}", &caps),
            "[kill] KILL message: foo was killed by bar"
        );
    }
}
