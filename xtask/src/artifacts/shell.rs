//! Literal command recognition, with quoted data kept separate from executable strings.

use std::{iter::Peekable, str::Chars};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Dialect {
    Posix,
    PowerShell,
    Batch,
}

impl Dialect {
    pub(super) fn for_extension(extension: &str) -> Self {
        match extension {
            "ps1" => Self::PowerShell,
            "cmd" | "bat" => Self::Batch,
            _ => Self::Posix,
        }
    }

    fn command_option(self, word: &str) -> bool {
        match self {
            Self::Posix => word.starts_with('-') && !word.starts_with("--") && word.contains('c'),
            Self::PowerShell => matches!(word, "-c" | "-command" | "-commandwithargs"),
            Self::Batch => matches!(word, "/c" | "/k"),
        }
    }

    fn escapes(self, ch: char, quote: Option<char>, next: Option<&char>) -> bool {
        match self {
            Self::Posix => {
                ch == '\\'
                    && quote != Some('\'')
                    && (quote.is_none()
                        || next.is_some_and(|ch| matches!(ch, '$' | '`' | '"' | '\\')))
            }
            Self::PowerShell => ch == '`' && quote != Some('\''),
            Self::Batch => ch == '^' && quote.is_none(),
        }
    }
}

pub(super) fn runs_python(line: &str, dialect: Dialect) -> bool {
    let lower = line.to_ascii_lowercase();
    let code = lower.strip_prefix("#!").unwrap_or(&lower);
    scan(&mut code.chars().peekable(), dialect, None)
}

/// Read one line or literal substitution. This does not evaluate variables, functions,
/// aliases, multiline strings or dynamically constructed commands.
fn scan(chars: &mut Peekable<Chars<'_>>, dialect: Dialect, end: Option<char>) -> bool {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut quote = None;
    while let Some(ch) = chars.next() {
        if dialect.escapes(ch, quote, chars.peek()) {
            if let Some(escaped) = chars.next() {
                word.push(escaped);
                started = true;
            }
            continue;
        }
        if quote == Some(ch) {
            if dialect == Dialect::PowerShell && ch == '\'' && chars.peek() == Some(&'\'') {
                chars.next();
                word.push(ch);
            } else {
                quote = None;
            }
            continue;
        }
        if quote.is_none() && (ch == '"' || (ch == '\'' && dialect != Dialect::Batch)) {
            quote = Some(ch);
            started = true;
            continue;
        }
        if quote.is_none() && end == Some(ch) {
            break;
        }
        let substitution = quote != Some('\'')
            && dialect != Dialect::Batch
            && ch == '$'
            && chars.peek() == Some(&'(');
        let backticks = quote != Some('\'') && dialect == Dialect::Posix && ch == '`';
        if substitution || backticks || (quote.is_none() && ch == '(') {
            if substitution {
                chars.next();
            }
            let closing = if backticks { '`' } else { ')' };
            if scan(chars, dialect, Some(closing)) {
                return true;
            }
            // The result is data; its eventual expansion into a command is review's.
            word.push_str("${expression}");
            started = true;
            continue;
        }
        if quote.is_some() {
            word.push(ch);
            continue;
        }
        if ch == '#' && !started && dialect != Dialect::Batch {
            break;
        }
        let separator = matches!(ch, '|' | '&' | ')')
            || (dialect != Dialect::Batch && matches!(ch, ';' | '{' | '}'));
        if ch.is_whitespace() || separator {
            if started {
                words.push(std::mem::take(&mut word));
                started = false;
            }
            if separator {
                if command_runs_python(&words) {
                    return true;
                }
                words.clear();
            }
        } else {
            word.push(ch);
            started = true;
        }
    }
    if started {
        words.push(word);
    }
    command_runs_python(&words)
}

fn command_runs_python(words: &[String]) -> bool {
    let mut words = words.iter().map(|word| word.trim_matches(['[', ']', ',']));
    let mut wrapped = false;
    let mut shell: Option<Dialect> = None;
    let name = loop {
        let Some(word) = words.next() else {
            return false;
        };
        if let Some(dialect) = shell
            && dialect.command_option(word)
        {
            return words.next().is_some_and(|code| runs_python(code, dialect));
        }
        let name = word
            .trim_start_matches(['@', '+', '-'])
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(word)
            .trim_end_matches(".exe");
        let option = wrapped && word.starts_with('-');
        let assignment = word.contains('=') && !word.starts_with('=');
        if option || assignment || matches!(name, "if" | "then" | "else" | "elif" | "do" | "!") {
            continue;
        }
        shell = match name {
            "sh" | "bash" | "zsh" => Some(Dialect::Posix),
            "pwsh" | "powershell" => Some(Dialect::PowerShell),
            "cmd" => Some(Dialect::Batch),
            _ => None,
        };
        if shell.is_some()
            || matches!(
                name,
                "env" | "sudo" | "exec" | "command" | "time" | "nohup" | "xargs" | "call"
            )
        {
            wrapped = true;
            continue;
        }
        break name;
    };
    if matches!(name, "uv" | "pip" | "pip3" | "pipx" | "poetry" | "pdm") {
        return words.next().is_some_and(|subcommand| {
            matches!(
                subcommand,
                "run" | "sync" | "install" | "exec" | "lock" | "add" | "export"
            )
        });
    }
    matches!(
        name,
        "python" | "python3" | "py" | "ruff" | "pytest" | "pyright" | "mypy" | "black"
    ) || name.starts_with("python3.")
}
