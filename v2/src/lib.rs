//! Native translation of jira2md 3.0.0 `to_jira` (Apache-2.0, Fokke Zandbergen).
//! Modified in 2026: Rust implementation, bounded work and output. See NOTICE.md.
use fancy_regex::{Captures, Regex, RegexBuilder};
use std::fmt;
use std::sync::OnceLock;

pub const MAX_INPUT_BYTES: usize = 16 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 128 * 1024;
pub const MAX_MATCHES: usize = 32_768;
pub const BACKTRACK_LIMIT: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertError {
    InputLimit,
    OutputLimit,
    WorkLimit,
    SourceTableError,
}
impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InputLimit => "Selected text exceeds the 16384-byte conversion limit",
            Self::OutputLimit => "Converted text exceeds the 131072-byte conversion limit",
            Self::WorkLimit => "Conversion exceeded the bounded regex work limit",
            Self::SourceTableError => "The selected table triggers a jira2md 3.0.0 table error",
        })
    }
}
impl std::error::Error for ConvertError {}

// ECMAScript (non-/u) whitespace, dot and multiline assertions. Every delimiter
// and backreference in to_jira is ASCII: UTF-8 scalar matching cannot split a
// surrogate pair at a capture boundary. Lone UTF-16 surrogates are not Rust str.
const WS: &str = r"[\x09-\x0d\x20\u{00a0}\u{1680}\u{2000}-\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\u{feff}]";
fn js_regex(pattern: &str) -> Regex {
    let mut translated = String::new();
    let mut chars = pattern.chars();
    let mut in_class = false;
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next().expect("static pattern escape") {
                's' => translated.push_str(WS),
                'S' => translated.push_str(&WS.replacen('[', "[^", 1)),
                'd' => translated.push_str("[0-9]"),
                x => {
                    translated.push('\\');
                    translated.push(x);
                }
            },
            '[' => {
                in_class = true;
                translated.push(c);
            }
            ']' => {
                in_class = false;
                translated.push(c);
            }
            '.' if !in_class => translated.push_str(r"[^\r\n\u{2028}\u{2029}]"),
            '^' if !in_class => translated.push_str(r"(?:\A|(?<=[\r\n\u{2028}\u{2029}]))"),
            '$' if !in_class => translated.push_str(r"(?=[\r\n\u{2028}\u{2029}]|\z)"),
            _ => translated.push(c),
        }
    }
    RegexBuilder::new(&translated)
        .backtrack_limit(BACKTRACK_LIMIT)
        .build()
        .expect("static jira2md pattern")
}
const PATTERNS: &[&str] = &[
    r"^\n((?:\|.*?)+\|)[ \t]*\n((?:\|\s*?-{3,}\s*?)+\|)[ \t]*\n((?:(?:\|.*?)+\|[ \t]*\n)*)$",
    r"([*_]+)(\S.*?)\1",
    r"^([#]+)(.*?)$",
    r"^(.*?)\n([=-]+)$",
    r"^([ \t]*)\d+\.\s+",
    r"^([ \t]*)\*\s+",
    r"<(del|ins|sup|sub)>(.*?)</\1>",
    r"(\s+)~~(.*?)~~(\s+)",
    r"```(.+\n)?((?:.|\n)*?)```",
    r"`([^`]+)`",
    r"!\[[^\]]*\]\(([^)]+)\)",
    r"\[([^\]]+)\]\(([^)]+)\)",
    r"<([^>]+)>",
    r"^>",
    r"[^|]+(?=\|)",
    r"^\|(.*)[ \t]*\|",
];
fn patterns() -> &'static [Regex] {
    static RE: OnceLock<Vec<Regex>> = OnceLock::new();
    RE.get_or_init(|| PATTERNS.iter().map(|p| js_regex(p)).collect())
}
fn group<'a>(c: &Captures<'a>, i: usize) -> &'a str {
    c.get(i).map_or("", |m| m.as_str())
}
fn whitespace(c: char) -> bool {
    matches!(c, '\u{9}'..='\u{d}' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}
fn append(out: &mut String, s: &str) -> Result<(), ConvertError> {
    if out
        .len()
        .checked_add(s.len())
        .filter(|&n| n <= MAX_OUTPUT_BYTES)
        .is_none()
    {
        return Err(ConvertError::OutputLimit);
    }
    out.push_str(s);
    Ok(())
}
fn replace<F>(
    text: &str,
    regex: &Regex,
    first: bool,
    remaining: &mut usize,
    mut replacement: F,
) -> Result<String, ConvertError>
where
    F: FnMut(&Captures<'_>, &mut usize) -> Result<String, ConvertError>,
{
    let mut out = String::new();
    let mut end = 0;
    for c in regex.captures_iter(text) {
        let c = c.map_err(|_| ConvertError::WorkLimit)?;
        *remaining = remaining.checked_sub(1).ok_or(ConvertError::WorkLimit)?;
        let m = c.get(0).expect("whole match");
        append(&mut out, &text[end..m.start()])?;
        append(&mut out, &replacement(&c, remaining)?)?;
        end = m.end();
        if first {
            break;
        }
    }
    append(&mut out, &text[end..])?;
    Ok(out)
}
fn cells(line: &str, remaining: &mut usize) -> Result<Vec<String>, ConvertError> {
    let mut result = Vec::new();
    for c in patterns()[14].captures_iter(line) {
        *remaining = remaining.checked_sub(1).ok_or(ConvertError::WorkLimit)?;
        result.push(group(&c.map_err(|_| ConvertError::WorkLimit)?, 0).to_string());
    }
    if result.is_empty() {
        return Err(ConvertError::SourceTableError);
    }
    Ok(result)
}

/// Convert selected text, preserving jira2md's ordered replacements (not CommonMark).
/// Errors contain no input. All conversion completes before any host OS effect.
pub fn to_jira(input: &str) -> Result<String, ConvertError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(ConvertError::InputLimit);
    }
    let mut text = input.to_owned();
    let mut remaining = MAX_MATCHES;
    for (rule, regex) in patterns()[..14].iter().enumerate() {
        text = replace(&text, regex, false, &mut remaining, |c, budget| {
            let a = group(c, 1);
            let b = group(c, 2);
            Ok(match rule {
                0 => {
                    let headers = cells(a, budget)?;
                    let separators = cells(b, budget)?;
                    let rows = group(c, 3);
                    if headers.len() != separators.len() {
                        group(c, 0).to_string()
                    } else if rows.split('\n').count() == 2 && headers.len() == 1 {
                        let body = replace(rows, &patterns()[15], true, budget, |c, _| {
                            Ok(group(c, 1).to_string())
                        })?;
                        format!(
                            "{{panel:title={}}}\n{}\n{{panel}}\n",
                            headers[0].trim_matches(whitespace),
                            body.trim_matches(whitespace)
                        )
                    } else {
                        format!("||{}||\n{rows}", headers.join("||"))
                    }
                }
                1 => match a.len() {
                    1 => format!("_{b}_"),
                    2 => format!("*{b}*"),
                    3 => format!("_*{b}*_"),
                    _ => format!("{a}{b}{a}"),
                },
                2 => format!("h{}.{b}", a.len()),
                3 => format!("h{}. {a}", if b.starts_with('=') { 1 } else { 2 }),
                4 => format!("{} ", "#".repeat(a.len() / 3 + 1)),
                5 => format!("{} ", "*".repeat(a.len() / 2 + 1)),
                6 => {
                    let delimiter = match a {
                        "del" => '-',
                        "ins" => '+',
                        "sup" => '^',
                        "sub" => '~',
                        _ => unreachable!(),
                    };
                    format!("{delimiter}{b}{delimiter}")
                }
                7 => format!("{a}-{b}-{}", group(c, 3)),
                8 => {
                    if c.get(1).is_some() {
                        format!("{{code:{}}}\n{b}{{code}}", a.replace('\n', ""))
                    } else {
                        format!("{{code}}{b}{{code}}")
                    }
                }
                9 => format!("{{{{{a}}}}}"),
                10 => format!("!{a}!"),
                11 => format!("[{a}|{b}]"),
                12 => format!("[{a}]"),
                13 => "bq.".into(),
                _ => unreachable!(),
            })
        })?;
    }
    Ok(text)
}
