use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Error, Result};

const REQUIRED_FRONTMATTER_KEYS: &[&str] = &[
    "title",
    "doc_type",
    "status",
    "owner",
    "audience",
    "scope",
    "source_of_truth",
    "upstream_refs",
    "related",
    "last_reviewed",
    "last_modified",
    "modified_on_branch",
    "modified_at_version",
    "modified_at_commit",
    "review_basis",
];

pub fn run() -> Result<()> {
    let root = env::current_dir().map_err(Error::Io)?;
    let docs = root.join("docs");
    let mut errors = Vec::new();
    let mut checked = 0usize;

    for path in markdown_files(&docs)? {
        if path
            .components()
            .any(|component| component.as_os_str() == "references")
        {
            continue;
        }

        checked += 1;
        let content = fs::read_to_string(&path).map_err(Error::Io)?;
        let relative = relative_path(&root, &path);
        check_doc(&root, &path, &relative, &content, &mut errors);
    }

    if errors.is_empty() {
        println!("audit-docs: checked {checked} authored markdown files");
        Ok(())
    } else {
        Err(Error::Audit(errors))
    }
}

fn check_doc(root: &Path, path: &Path, relative: &str, content: &str, errors: &mut Vec<String>) {
    let Some(frontmatter) = frontmatter(content) else {
        errors.push(format!("{relative}: missing YAML frontmatter"));
        return;
    };

    for key in REQUIRED_FRONTMATTER_KEYS {
        if !has_frontmatter_key(frontmatter, key) {
            errors.push(format!("{relative}: missing frontmatter key `{key}`"));
        }
    }

    if frontmatter.contains("modified_at_commit: \"unborn\"")
        || frontmatter.contains("modified_at_commit: unborn")
    {
        errors.push(format!("{relative}: modified_at_commit is still unborn"));
    }

    check_path_list(root, relative, frontmatter, "upstream_refs", errors);
    check_path_list(root, relative, frontmatter, "related", errors);
    check_markdown_links(root, path, relative, content, errors);
}

fn markdown_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_markdown_files(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_markdown_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(Error::Io)? {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(&path, files)?;
        } else if path.extension().and_then(OsStr::to_str) == Some("md") {
            files.push(path);
        }
    }
    Ok(())
}

fn frontmatter(content: &str) -> Option<&str> {
    let content = content.strip_prefix("---\n")?;
    let end = content.find("\n---\n")?;
    Some(&content[..end])
}

fn has_frontmatter_key(frontmatter: &str, key: &str) -> bool {
    frontmatter
        .lines()
        .any(|line| line.starts_with(key) && line[key.len()..].starts_with(':'))
}

fn check_path_list(
    root: &Path,
    relative: &str,
    frontmatter: &str,
    key: &str,
    errors: &mut Vec<String>,
) {
    for value in frontmatter_list_values(frontmatter, key) {
        if !root.join(&value).exists() {
            errors.push(format!(
                "{relative}: `{key}` target does not exist: {value}"
            ));
        }
    }
}

fn frontmatter_list_values(frontmatter: &str, key: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut in_list = false;

    for line in frontmatter.lines() {
        if line.starts_with(key) && line[key.len()..].starts_with(':') {
            in_list = true;
            let rest = line[key.len() + 1..].trim();
            if rest.starts_with('[') && rest.ends_with(']') {
                values.extend(inline_list_values(rest));
                in_list = false;
            }
            continue;
        }

        if in_list {
            if let Some(value) = line.trim_start().strip_prefix("- ") {
                values.push(unquote(value.trim()).to_owned());
            } else if !line.starts_with(' ') && !line.starts_with('\t') {
                in_list = false;
            }
        }
    }

    values
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect()
}

fn inline_list_values(value: &str) -> Vec<String> {
    let inner = value.trim_start_matches('[').trim_end_matches(']');
    inner
        .split(',')
        .map(|part| unquote(part.trim()).to_owned())
        .filter(|part| !part.is_empty())
        .collect()
}

fn check_markdown_links(
    root: &Path,
    path: &Path,
    relative: &str,
    content: &str,
    errors: &mut Vec<String>,
) {
    for (line_number, line) in content.lines().enumerate() {
        let mut rest = line;
        while let Some(open) = rest.find("](") {
            rest = &rest[open + 2..];
            let Some(close) = rest.find(')') else {
                break;
            };

            let target = &rest[..close];
            rest = &rest[close + 1..];
            if should_skip_link(target) {
                continue;
            }

            let target_path = target
                .split('#')
                .next()
                .unwrap_or_default()
                .split('?')
                .next()
                .unwrap_or_default();
            if target_path.is_empty() {
                continue;
            }

            let resolved = if target_path.starts_with('/') {
                root.join(target_path.trim_start_matches('/'))
            } else {
                path.parent().unwrap_or(root).join(target_path)
            };

            if !resolved.exists() {
                errors.push(format!(
                    "{relative}:{}: markdown link target does not exist: {target}",
                    line_number + 1
                ));
            }
        }
    }
}

fn should_skip_link(target: &str) -> bool {
    target.starts_with('#')
        || target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
        || target.starts_with("tel:")
}

fn unquote(value: &str) -> &str {
    value.trim_matches('"').trim_matches('\'')
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
