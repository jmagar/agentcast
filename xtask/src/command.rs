use std::env;
use std::ffi::{OsStr, OsString};
use std::process::Command;

use crate::{Error, Result};

pub fn cargo<I>(args: &[&str], extra: I) -> Result<()>
where
    I: IntoIterator<Item = OsString>,
{
    run_os(cargo_program(), args, extra)
}

pub fn run<A, I>(program: &str, args: A, extra: I) -> Result<()>
where
    A: IntoIterator,
    A::Item: AsRef<OsStr>,
    I: IntoIterator<Item = OsString>,
{
    run_os(OsString::from(program), args, extra)
}

fn run_os<A, I>(program: OsString, args: A, extra: I) -> Result<()>
where
    A: IntoIterator,
    A::Item: AsRef<OsStr>,
    I: IntoIterator<Item = OsString>,
{
    let status = Command::new(&program)
        .args(args)
        .args(extra)
        .status()
        .map_err(Error::Io)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::Failed {
            program: program.to_string_lossy().into_owned(),
            code: status.code(),
        })
    }
}

fn cargo_program() -> OsString {
    env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"))
}
