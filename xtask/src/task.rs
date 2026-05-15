use std::env;
use std::ffi::OsString;

use crate::audit;
use crate::command;
use crate::dev_env;
use crate::{Error, Result};

const CHECK: &[&str] = &["check", "--workspace"];
const CHECK_NIGHTLY: &[&str] = &["+nightly", "check", "--workspace"];
const CLIPPY: &[&str] = &[
    "clippy",
    "--workspace",
    "--all-targets",
    "--",
    "-D",
    "warnings",
];
const FMT: &[&str] = &["fmt", "--all"];
const FMT_CHECK: &[&str] = &["fmt", "--all", "--", "--check"];
const TEST: &[&str] = &["test", "--workspace"];
const NEXTEST: &[&str] = &["nextest", "run", "--workspace"];
const INSTALL_HOOKS: &[&str] = &["install"];
const PRE_COMMIT: &[&str] = &["run", "pre-commit"];
const PRE_PUSH: &[&str] = &["run", "pre-push"];
const GITLEAKS_DETECT: &[&str] = &["detect", "--no-banner"];

const TASKS: &[Task] = &[
    Task::new(
        "setup",
        "verify required dev tools and install/sync hooks",
        TaskKind::Builtin(Builtin::Setup),
    ),
    Task::new(
        "doctor",
        "verify required dev tools and hook files",
        TaskKind::Builtin(Builtin::Doctor),
    ),
    Task::new("check", "cargo check --workspace", TaskKind::Cargo(CHECK)),
    Task::new(
        "check-cranelift",
        "RUSTFLAGS=-Zcodegen-backend=cranelift cargo +nightly check --workspace",
        TaskKind::CargoWithEnv {
            args: CHECK_NIGHTLY,
            key: "RUSTFLAGS",
            value: "-Zcodegen-backend=cranelift",
        },
    ),
    Task::new(
        "clippy",
        "cargo clippy --workspace --all-targets -- -D warnings",
        TaskKind::Cargo(CLIPPY),
    ),
    Task::new("fmt", "cargo fmt --all", TaskKind::Cargo(FMT)),
    Task::new(
        "fmt-check",
        "cargo fmt --all -- --check",
        TaskKind::Cargo(FMT_CHECK),
    ),
    Task::new("test", "cargo test --workspace", TaskKind::Cargo(TEST)),
    Task::new(
        "nextest",
        "cargo nextest run --workspace",
        TaskKind::Cargo(NEXTEST),
    ),
    Task::new(
        "ci",
        "fmt-check, check, clippy, nextest",
        TaskKind::Sequence(&["fmt-check", "check", "clippy", "nextest"]),
    ),
    Task::new(
        "verify",
        "ci plus cargo test --workspace doctest/compatibility pass",
        TaskKind::Sequence(&["ci", "test"]),
    ),
    Task::new(
        "hooks",
        "lefthook install",
        TaskKind::Command("lefthook", INSTALL_HOOKS),
    ),
    Task::new(
        "install-hooks",
        "lefthook install",
        TaskKind::Command("lefthook", INSTALL_HOOKS),
    ),
    Task::new(
        "pre-commit",
        "lefthook run pre-commit",
        TaskKind::Command("lefthook", PRE_COMMIT),
    ),
    Task::new(
        "pre-push",
        "lefthook run pre-push",
        TaskKind::Command("lefthook", PRE_PUSH),
    ),
    Task::new(
        "refresh-docs",
        "scripts/refresh-docs.sh",
        TaskKind::Script("scripts/refresh-docs.sh"),
    ),
    Task::new(
        "review-docs",
        "scripts/review-changes.sh",
        TaskKind::Script("scripts/review-changes.sh"),
    ),
    Task::new(
        "check-deps",
        "scripts/check-dependency-updates.sh",
        TaskKind::Script("scripts/check-dependency-updates.sh"),
    ),
    Task::new(
        "audit-docs",
        "validate authored docs frontmatter, links, and upstream_refs",
        TaskKind::Builtin(Builtin::AuditDocs),
    ),
    Task::new(
        "secrets",
        "gitleaks detect --no-banner",
        TaskKind::Command("gitleaks", GITLEAKS_DETECT),
    ),
];

#[derive(Clone, Copy)]
struct Task {
    name: &'static str,
    description: &'static str,
    kind: TaskKind,
}

impl Task {
    const fn new(name: &'static str, description: &'static str, kind: TaskKind) -> Self {
        Self {
            name,
            description,
            kind,
        }
    }
}

#[derive(Clone, Copy)]
enum TaskKind {
    Builtin(Builtin),
    Cargo(&'static [&'static str]),
    CargoWithEnv {
        args: &'static [&'static str],
        key: &'static str,
        value: &'static str,
    },
    Command(&'static str, &'static [&'static str]),
    Script(&'static str),
    Sequence(&'static [&'static str]),
}

#[derive(Clone, Copy)]
enum Builtin {
    AuditDocs,
    Doctor,
    Setup,
}

pub fn run() -> Result<()> {
    let mut args = env::args_os();
    let _binary = args.next();

    let Some(task_name) = args.next() else {
        print_usage();
        return Ok(());
    };

    let task_name = task_name.to_string_lossy();
    if matches!(task_name.as_ref(), "-h" | "--help" | "help") {
        print_usage();
        return Ok(());
    }

    let Some(task) = task_by_name(&task_name) else {
        return Err(Error::Usage(format!("unknown xtask: {task_name}")));
    };

    run_task(task, args.collect())
}

fn task_by_name(name: &str) -> Option<&'static Task> {
    TASKS.iter().find(|task| task.name == name)
}

fn run_task(task: &Task, extra: Vec<OsString>) -> Result<()> {
    match task.kind {
        TaskKind::Builtin(Builtin::AuditDocs) => {
            reject_extra(task.name, &extra)?;
            audit::run()
        }
        TaskKind::Builtin(Builtin::Doctor) => {
            reject_extra(task.name, &extra)?;
            dev_env::doctor()
        }
        TaskKind::Builtin(Builtin::Setup) => {
            reject_extra(task.name, &extra)?;
            dev_env::setup()
        }
        TaskKind::Cargo(args) => command::cargo(args, extra),
        TaskKind::CargoWithEnv { args, key, value } => {
            command::run_with_env("cargo", args, key, value, extra)
        }
        TaskKind::Command(program, args) => command::run(program, args, extra),
        TaskKind::Script(path) => command::run(path, std::iter::empty::<&str>(), extra),
        TaskKind::Sequence(tasks) => {
            reject_extra(task.name, &extra)?;
            for task_name in tasks {
                let task = task_by_name(task_name)
                    .ok_or_else(|| Error::Usage(format!("unknown sequence task: {task_name}")))?;
                run_task(task, Vec::new())?;
            }
            Ok(())
        }
    }
}

fn reject_extra(task_name: &str, extra: &[OsString]) -> Result<()> {
    if extra.is_empty() {
        Ok(())
    } else {
        Err(Error::Usage(format!(
            "xtask {task_name} does not accept extra arguments"
        )))
    }
}

pub fn print_usage() {
    println!("AgentCast repository tasks\n");
    println!("Usage:");
    println!("  cargo xtask <task> [task args...]\n");
    println!("Tasks:");
    for task in TASKS {
        println!("  {:<16} {}", task.name, task.description);
    }
}
