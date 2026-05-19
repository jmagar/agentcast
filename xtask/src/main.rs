mod audit;
mod audit_deps;
mod command;
mod dev_env;
mod task;

use std::process::ExitCode;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Usage(String),
    Audit(Vec<String>),
    DependencyAudit(Vec<String>),
    DevEnv(Vec<String>),
    Io(std::io::Error),
    Failed { program: String, code: Option<i32> },
}

fn main() -> ExitCode {
    match task::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(Error::Usage(message)) => {
            eprintln!("{message}");
            eprintln!();
            task::print_usage();
            ExitCode::from(2)
        }
        Err(Error::Audit(errors)) => {
            eprintln!("xtask audit-docs failed with {} issue(s):", errors.len());
            for error in errors {
                eprintln!("- {error}");
            }
            ExitCode::FAILURE
        }
        Err(Error::DependencyAudit(errors)) => {
            eprintln!("xtask audit-deps failed with {} issue(s):", errors.len());
            for error in errors {
                eprintln!("- {error}");
            }
            ExitCode::FAILURE
        }
        Err(Error::DevEnv(errors)) => {
            eprintln!(
                "xtask dev environment check failed with {} issue(s):",
                errors.len()
            );
            for error in errors {
                eprintln!("- {error}");
            }
            ExitCode::FAILURE
        }
        Err(Error::Io(error)) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
        Err(Error::Failed { program, code }) => {
            match code {
                Some(code) => eprintln!("xtask: `{program}` exited with status {code}"),
                None => eprintln!("xtask: `{program}` terminated by signal"),
            }
            ExitCode::FAILURE
        }
    }
}
