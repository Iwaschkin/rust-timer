//! Review numbers, counted by the tool.
//!
//! These are the numbers a reviewer asks for at a slice boundary. They are
//! reported, never enforced: the guard and the check decide what fails. An
//! agent asked to count its own gates gets them wrong, and in the direction
//! that flatters the work, so the counting belongs here.
//!
//! Gates that need project knowledge, such as which vocabulary belongs to which
//! module, are not here. The plan's ownership table states those as greps,
//! because only the plan knows the tokens.
//!
//! Each number is a lexical count of one source line at a time, not a parse. A
//! declaration split across lines or produced by a macro is not counted, and a
//! repeated name says only that two declarations share it: whether they duplicate
//! one another, and whether differently named types do, is the reviewer's call.

use crate::{Result, files};
use std::{
    collections::BTreeMap,
    fmt, fs,
    path::{Path, PathBuf},
};

/// The numbers a slice review reports.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Gates {
    /// Imports that bring in a whole module by name.
    pub glob_imports: usize,
    /// Items declared `pub`, excluding those narrowed to a crate or parent.
    pub public_items: usize,
    /// Items declared `pub(crate)` or `pub(super)`.
    pub crate_items: usize,
    /// Total size of the files under the evidence directory.
    pub evidence_bytes: u64,
    /// How many files sit under the evidence directory.
    pub evidence_files: usize,
    /// Columns in the longest line of Rust source.
    pub longest_line: usize,
    /// Where that line is, as a path and line number.
    pub longest_line_at: String,
    /// Struct, enum, trait and union names declared more than once, in order.
    pub repeated_shapes: Vec<String>,
    /// How many test functions were found.
    pub test_functions: usize,
    /// Test names used more than once, in order.
    pub repeated_test_names: Vec<String>,
    /// How many Rust files were counted.
    pub source_files: usize,
}

impl fmt::Display for Gates {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "glob imports: {}", self.glob_imports)?;
        writeln!(
            formatter,
            "pub items: {} against pub(crate) items: {}",
            self.public_items, self.crate_items
        )?;
        writeln!(
            formatter,
            "evidence: {} bytes in {} files",
            self.evidence_bytes, self.evidence_files
        )?;
        let at = if self.longest_line_at.is_empty() {
            "no Rust source"
        } else {
            &self.longest_line_at
        };
        writeln!(formatter, "longest line: {} at {at}", self.longest_line)?;
        writeln!(
            formatter,
            "shape names declared more than once: {}{}",
            self.repeated_shapes.len(),
            if self.repeated_shapes.is_empty() {
                String::new()
            } else {
                format!(" ({})", self.repeated_shapes.join(", "))
            }
        )?;
        writeln!(
            formatter,
            "test functions: {}, names used more than once: {}{}",
            self.test_functions,
            self.repeated_test_names.len(),
            if self.repeated_test_names.is_empty() {
                String::new()
            } else {
                format!(" ({})", self.repeated_test_names.join(", "))
            }
        )?;
        write!(
            formatter,
            "counted over {} Rust files. Provider vocabulary outside its module, \
             and addresses on the application struct, are the plan's own greps.",
            self.source_files
        )
    }
}

/// Count the review numbers for the project rooted at `root`, over the files git
/// lists.
///
/// # Errors
/// Returns an error when git cannot list the project or a listed file cannot be read.
pub fn gates(root: &Path) -> Result<Gates> {
    let listed = files::list(root)?;
    // This project's own Rust sources, not the tooling workspace's.
    let sources: Vec<&PathBuf> = listed
        .iter()
        .filter(|relative| {
            relative.extension().is_some_and(|value| value == "rs")
                && !relative.starts_with("xtask")
        })
        .collect();
    let mut gates = Gates {
        source_files: sources.len(),
        ..Gates::default()
    };
    let mut shapes: BTreeMap<String, usize> = BTreeMap::new();
    let mut tests: BTreeMap<String, usize> = BTreeMap::new();
    for source in sources {
        let text = fs::read_to_string(root.join(source))?;
        let relative = source.display().to_string();
        let mut announced = false;
        for (number, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            if is_test_attribute(trimmed) {
                announced = true;
            } else if let Some(name) = function_name(trimmed) {
                if announced {
                    let seen = tests.entry(name).or_default();
                    *seen = seen.saturating_add(1);
                    gates.test_functions = gates.test_functions.saturating_add(1);
                }
                announced = false;
            }
            if is_glob_import(trimmed) {
                gates.glob_imports = gates.glob_imports.saturating_add(1);
            }
            match visibility(trimmed) {
                Some(Visibility::Public) => {
                    gates.public_items = gates.public_items.saturating_add(1);
                }
                Some(Visibility::Crate) => {
                    gates.crate_items = gates.crate_items.saturating_add(1);
                }
                None => {}
            }
            if let Some(name) = shape_name(trimmed) {
                let seen = shapes.entry(name).or_default();
                *seen = seen.saturating_add(1);
            }
            let columns = line.chars().count();
            if columns > gates.longest_line {
                gates.longest_line = columns;
                gates.longest_line_at = format!("{relative}:{}", number.saturating_add(1));
            }
        }
    }
    gates.repeated_shapes = shapes
        .into_iter()
        .filter_map(|(name, seen)| (seen > 1).then_some(name))
        .collect();
    gates.repeated_test_names = tests
        .into_iter()
        .filter_map(|(name, seen)| (seen > 1).then_some(name))
        .collect();
    for evidence in listed
        .iter()
        .filter(|relative| relative.starts_with("evidence"))
    {
        gates.evidence_files = gates.evidence_files.saturating_add(1);
        gates.evidence_bytes = gates
            .evidence_bytes
            .saturating_add(fs::metadata(root.join(evidence))?.len());
    }
    Ok(gates)
}

enum Visibility {
    Public,
    Crate,
}

/// Items this project declares, by the visibility written in front of them.
fn visibility(line: &str) -> Option<Visibility> {
    let rest = line.strip_prefix("pub")?;
    let (narrowed, rest) = match rest.strip_prefix('(') {
        Some(inside) => {
            let (scope, after) = inside.split_once(')')?;
            (matches!(scope.trim(), "crate" | "super"), after)
        }
        None => (false, rest),
    };
    let rest = after_qualifiers(rest.strip_prefix(' ')?);
    let keyword = rest.split_whitespace().next()?;
    if !matches!(
        keyword,
        "fn" | "struct" | "enum" | "trait" | "mod" | "const" | "static" | "type" | "union"
    ) {
        return None;
    }
    Some(if narrowed {
        Visibility::Crate
    } else {
        Visibility::Public
    })
}

/// An attribute that makes the next function a test, whichever runtime supplies
/// it and whatever arguments it takes: `#[test]`, `#[tokio::test]`,
/// `#[tokio::test(flavor = "current_thread")]`, `#[async_std::test]` and their kin.
fn is_test_attribute(line: &str) -> bool {
    line.strip_prefix("#[")
        .and_then(|rest| rest.strip_suffix(']'))
        .is_some_and(|inside| {
            let path = inside.split_once('(').map_or(inside, |(path, _)| path);
            path == "test" || path.ends_with("::test")
        })
}

/// The line after any visibility written in front of it.
fn without_visibility(line: &str) -> Option<&str> {
    match line.strip_prefix("pub") {
        Some(after) => match after.strip_prefix('(') {
            Some(inside) => inside.split_once(')')?.1.strip_prefix(' '),
            None => after.strip_prefix(' '),
        },
        None => Some(line),
    }
}

/// What follows the function qualifiers `async`, `const`, `unsafe` and `extern "abi"`.
///
/// `const` also opens an item of its own, so it is a qualifier only in front of
/// `fn` or another qualifier.
fn after_qualifiers(mut rest: &str) -> &str {
    while let Some((word, tail)) = rest.split_once(' ') {
        let tail = tail.trim_start();
        let next = tail.split_whitespace().next().unwrap_or("");
        rest = match word {
            "async" | "unsafe" => tail,
            "const" if matches!(next, "fn" | "async" | "unsafe" | "extern") => tail,
            "extern" => tail
                .strip_prefix('"')
                .and_then(|abi| abi.split_once('"'))
                .map_or(tail, |(_, after)| after.trim_start()),
            _ => return rest,
        };
    }
    rest
}

/// The name a function is declared under, whatever its visibility and whether
/// or not it is asynchronous.
fn function_name(line: &str) -> Option<String> {
    let name: String = after_qualifiers(without_visibility(line)?)
        .strip_prefix("fn ")?
        .chars()
        .take_while(|letter| letter.is_alphanumeric() || *letter == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}

/// A `use` that brings in every name of a module, alone or inside a group.
fn is_glob_import(line: &str) -> bool {
    without_visibility(line)
        .and_then(|rest| rest.strip_prefix("use "))
        .is_some_and(|tail| tail.contains("::*"))
}

/// The name a struct, enum, trait or union is declared under, wherever the
/// declaration sits and whatever its visibility.
fn shape_name(line: &str) -> Option<String> {
    let mut words = without_visibility(line)?.split_whitespace();
    let keyword = words.next()?;
    if !matches!(keyword, "struct" | "enum" | "trait" | "union") {
        return None;
    }
    let name: String = words
        .next()?
        .chars()
        .take_while(|letter| letter.is_alphanumeric() || *letter == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}
