use std::path::Path;
use std::process::Command;

use crate::{Error, Result};

const REQUIRED_TOOLS: &[ToolCheck] = &[
    ToolCheck::new("cargo", &["--version"]),
    ToolCheck::new("rustc", &["--version"]),
    ToolCheck::new("cargo", &["nextest", "--version"]),
    ToolCheck::new("lefthook", &["version"]),
    ToolCheck::new("gitleaks", &["version"]),
    ToolCheck::new("taplo", &["--version"]),
    ToolCheck::new("just", &["--version"]),
];

const OPTIONAL_TOOLS: &[ToolCheck] = &[ToolCheck::new("cargo", &["+nightly", "--version"])];

#[derive(Clone, Copy)]
struct ToolCheck {
    program: &'static str,
    args: &'static [&'static str],
}

impl ToolCheck {
    const fn new(program: &'static str, args: &'static [&'static str]) -> Self {
        Self { program, args }
    }

    fn label(&self) -> String {
        if self.args.is_empty() {
            self.program.to_owned()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        }
    }
}

pub fn doctor() -> Result<()> {
    check_dev_environment(false)
}

pub fn setup() -> Result<()> {
    check_dev_environment(true)
}

fn check_dev_environment(install_hooks: bool) -> Result<()> {
    println!("AgentCast dev environment\n");

    let mut issues = Vec::new();

    println!("Required tools:");
    for tool in REQUIRED_TOOLS {
        check_tool(*tool, true, &mut issues);
    }

    println!("\nOptional tools:");
    for tool in OPTIONAL_TOOLS {
        check_tool(*tool, false, &mut issues);
    }

    if install_hooks {
        println!("\nGit hooks:");
        install_lefthook()?;
    }

    check_hooks(&mut issues);

    if issues.is_empty() {
        println!("\nDev environment is ready.");
        Ok(())
    } else {
        Err(Error::DevEnv(issues))
    }
}

fn check_tool(tool: ToolCheck, required: bool, issues: &mut Vec<String>) {
    match Command::new(tool.program).args(tool.args).output() {
        Ok(output) if output.status.success() => {
            let version =
                first_output_line(&output.stdout).or_else(|| first_output_line(&output.stderr));
            match version {
                Some(version) => println!("  ok   {:<28} {version}", tool.label()),
                None => println!("  ok   {}", tool.label()),
            }
        }
        Ok(output) => {
            let status = output.status.code().map_or_else(
                || "terminated by signal".to_owned(),
                |code| format!("exited with status {code}"),
            );
            if required {
                issues.push(format!(
                    "required tool `{}` is installed but {status}",
                    tool.label()
                ));
                println!("  fail {} ({status})", tool.label());
            } else {
                println!("  skip {} ({status}; optional)", tool.label());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if required {
                issues.push(format!(
                    "required tool `{}` was not found on PATH",
                    tool.label()
                ));
                println!("  fail {} (not found)", tool.label());
            } else {
                println!("  skip {} (not found; optional)", tool.label());
            }
        }
        Err(error) => {
            if required {
                issues.push(format!("could not run `{}`: {error}", tool.label()));
                println!("  fail {} ({error})", tool.label());
            } else {
                println!("  skip {} ({error}; optional)", tool.label());
            }
        }
    }
}

fn first_output_line(bytes: &[u8]) -> Option<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

fn install_lefthook() -> Result<()> {
    let status = Command::new("lefthook")
        .arg("install")
        .status()
        .map_err(Error::Io)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::Failed {
            program: "lefthook".to_owned(),
            code: status.code(),
        })
    }
}

fn check_hooks(issues: &mut Vec<String>) {
    println!("\nHook files:");
    for hook in [".git/hooks/pre-commit", ".git/hooks/pre-push"] {
        if Path::new(hook).is_file() {
            println!("  ok   {hook}");
        } else {
            issues.push(format!(
                "missing hook file `{hook}`; run `cargo xtask setup`"
            ));
            println!("  fail {hook} (missing)");
        }
    }
}
