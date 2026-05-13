use std::ffi::{OsStr, OsString};
use std::process::Command;

use crate::{Error, Result};

pub fn cargo<I>(args: &[&str], extra: I) -> Result<()>
where
    I: IntoIterator<Item = OsString>,
{
    run("cargo", args, extra)
}

pub fn run<A, I>(program: &str, args: A, extra: I) -> Result<()>
where
    A: IntoIterator,
    A::Item: AsRef<OsStr>,
    I: IntoIterator<Item = OsString>,
{
    let status = Command::new(program)
        .args(args)
        .args(extra)
        .status()
        .map_err(Error::Io)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::Failed {
            program: program.to_owned(),
            code: status.code(),
        })
    }
}

pub fn run_with_env<A, I>(program: &str, args: A, key: &str, value: &str, extra: I) -> Result<()>
where
    A: IntoIterator,
    A::Item: AsRef<OsStr>,
    I: IntoIterator<Item = OsString>,
{
    let status = Command::new(program)
        .env(key, value)
        .args(args)
        .args(extra)
        .status()
        .map_err(Error::Io)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::Failed {
            program: program.to_owned(),
            code: status.code(),
        })
    }
}
