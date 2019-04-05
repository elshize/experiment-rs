// MIT License
//
// Copyright (c) 2019 Michał Siedlaczek
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

extern crate os_pipe;

use super::Verbosity::{Brief, Verbose};
use super::*;
use os_pipe::pipe;
use std::fmt;
use std::process::{Command, ExitStatus};

/// A convenient text representation of a single shell program that provides easy printing and
/// execution.
///
/// ```
/// # use experiment::process::Process;
/// # use std::process::ExitStatus;
/// # use std::process::Command;
/// let process = Process::new("cp", &["/path/to/source", "/path/to/target"]);
/// process.execute().expect("Failed to execute");
/// ```
#[derive(Debug)]
pub struct Process {
    program: String,
    args: Vec<String>,
}

/// A [`Process`](Process.t.html) wrapper implementing `fmt::Display` trait.
/// This indirection is created in order to explicitly set verbosity.
///
/// This object is created with [`Process::display`](struct.Process.html#method.display) method.
pub struct ProcessDisplay<'a> {
    process: &'a Process,
    verbosity: Verbosity,
}

impl Process {
    /// Creates a new [`Process`](Process.t.html).
    ///
    /// # Examples
    /// ```
    /// # use experiment::process::Process;
    /// let arguments = ["a", "b", "c"];
    /// let process = Process::new("program_name", &arguments);
    /// ```
    pub fn new<I, S>(program: &str, args: I) -> Process
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        Process {
            program: String::from(program),
            args: args
                .into_iter()
                .map(|s| String::from(s.as_ref().to_str().expect("Invalid Unicode")))
                .collect(),
        }
    }

    /// Creates a [`ProcessDisplay`](ProcessDisplay.t.html) object with the desired verbosity.
    ///
    /// # Examples
    /// ```
    /// # use experiment::process::Process;
    /// # use experiment::Verbosity::{Verbose, Brief};
    /// let process = Process::new("ls", &["-l", "/path/to/dir"]);
    /// assert_eq!(format!("{}", process.display(Verbose)), "ls -l /path/to/dir".to_string());
    /// assert_eq!(format!("{}", process.display(Brief(2))), "ls -l /path/to/dir".to_string());
    /// assert_eq!(format!("{}", process.display(Brief(1))), "ls -l ...".to_string());
    /// ```
    pub fn display(&self, verbosity: Verbosity) -> ProcessDisplay {
        ProcessDisplay {
            process: &self,
            verbosity,
        }
    }

    /// Generates a [`Command`](https://doc.rust-lang.org/std/process/struct.Command.html) object.
    ///
    /// # Examples
    /// ```
    /// # use experiment::process::Process;
    /// # use std::str::from_utf8;
    /// let process = Process::new("echo", &["Hello,", "World!"]);
    /// let output = process.command().output().expect("Failed to run process");
    /// assert_eq!(from_utf8(&output.stdout).unwrap(), "Hello, World!\n");
    /// ```
    pub fn command(&self) -> Command {
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.args);
        cmd
    }

    /// Executes the command, ignoring the generated output.
    ///
    /// # Examples
    /// ```
    /// # use experiment::process::Process;
    /// let process = Process::new("echo", &["Hello,", "World!"]);
    /// process.execute().expect("Failed to run process");
    ///
    /// let process = Process::new("unknown_process", &Vec::<&str>::new());
    /// assert!(process.execute().is_err());
    /// ```
    pub fn execute(&self) -> std::io::Result<ExitStatus> {
        self.command().status()
    }
}

impl<'a> fmt::Display for ProcessDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_count = match self.verbosity {
            Verbose => self.process.args.len(),
            Brief(max_args) => max_args,
        };
        write!(f, "{}", &self.process.program)?;
        for arg in self.process.args.iter().take(display_count) {
            write!(f, " {}", arg)?;
        }
        if self.verbosity != Verbosity::Verbose && display_count < self.process.args.len() {
            write!(f, " ...")?;
        }
        Ok(())
    }
}

/// A representation of a set of processes interacting through standard input/output.
///
/// # Examples
/// ```
/// # use experiment::pipeline;
/// # use experiment::process::{Process, ProcessPipeline};
/// let pipeline = pipeline!(
///     Process::new("echo", &["-e", "a\\nb\\nc"]),
///     Process::new("grep", &["b"])
/// );
/// assert_eq!(
///     std::str::from_utf8(&pipeline.pipe().output().unwrap().stdout).unwrap(),
///     "b\n"
/// );
/// ```
pub struct ProcessPipeline {
    processes: Vec<Process>,
}

impl ProcessPipeline {
    /// Creates a process pipeline. Typically, it is better to use [`pipeline`](../macro.pipeline.html) macro.
    pub fn new(processes: Vec<Process>) -> ProcessPipeline {
        ProcessPipeline { processes }
    }

    /// Creates a [`PipelineDisplay`](PipelineDisplay.t.html) object with the desired verbosity.
    ///
    /// # Examples
    /// ```
    /// # use experiment::pipeline;
    /// # use experiment::process::{Process, ProcessPipeline};
    /// # use experiment::Verbosity::{Verbose, Brief};
    /// let pipeline = pipeline!(
    ///     Process::new("echo", &["-e", "a\\nb\\nc"]),
    ///     Process::new("grep", &["b"])
    /// );
    /// assert_eq!(
    ///     format!("{}", pipeline.display(Verbose)),
    ///     "echo -e a\\nb\\nc\n\t| grep b".to_string()
    /// );
    /// assert_eq!(
    ///     format!("{}", pipeline.display(Brief(2))),
    ///     "echo -e a\\nb\\nc\n\t| grep b".to_string()
    /// );
    /// assert_eq!(
    ///     format!("{}", pipeline.display(Brief(1))),
    ///     "echo -e ...\n\t| grep b".to_string()
    /// );
    /// ```
    pub fn display(&self, verbosity: Verbosity) -> PipelineDisplay {
        PipelineDisplay {
            pipeline: &self,
            verbosity,
        }
    }

    /// Generates a pipeline of
    /// [`Command`](https://doc.rust-lang.org/std/process/struct.Command.html)s and returns the last
    /// one.
    ///
    /// # Examples
    /// ```
    /// # use experiment::pipeline;
    /// # use experiment::process::{Process, ProcessPipeline};
    /// let pipeline = pipeline!(
    ///     Process::new("echo", &["-e", "a\\nb\\nc"]),
    ///     Process::new("grep", &["b"])
    /// );
    /// assert_eq!(
    ///     std::str::from_utf8(&pipeline.pipe().output().unwrap().stdout).unwrap(),
    ///     "b\n"
    /// );
    /// ```
    pub fn pipe(&self) -> Command {
        assert!(self.processes.len() > 1);
        let mut cmds = self
            .processes
            .iter()
            .map(|ref p| {
                let mut cmd = Command::new(&p.program);
                cmd.args(&p.args);
                cmd
            })
            .collect::<Vec<_>>();
        for window in (0..cmds.len()).collect::<Vec<_>>().windows(2) {
            match *window {
                [first, second] => {
                    let (reader, writer) = pipe().expect("Failed opening a pipe");
                    cmds[first].stdout(writer);
                    cmds[second].stdin(reader);
                    cmds[first].spawn().expect("Failed to spawn");
                }
                _ => panic!("Programming error"),
            }
        }
        cmds.pop().expect("No last element")
    }

    /// Executes the entire pipeline disregarding the output.
    pub fn execute(&self) -> std::io::Result<ExitStatus> {
        self.pipe().status()
    }
}

/// A [`ProcessPipeline`](ProcessPipeline.t.html) wrapper implementing `fmt::Display` trait.
/// This indirection is created in order to explicitly set verbosity.
///
/// This object is created with
/// [`ProcessPipeline::display`](struct.ProcessPipeline.html#method.display) method.
pub struct PipelineDisplay<'a> {
    pipeline: &'a ProcessPipeline,
    verbosity: Verbosity,
}

impl<'a> fmt::Display for PipelineDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.pipeline.processes.is_empty() {
            write!(f, "{}", self.pipeline.processes[0].display(self.verbosity))?;
            for cmd in &self.pipeline.processes[1..] {
                write!(f, "\n\t| {}", cmd.display(self.verbosity))?;
            }
        }
        Ok(())
    }
}

/// Creates a [`ProcessPipeline`](ProcessPipeline.t.html) from provided processes.
///
/// # Examples
/// ```
/// # use experiment::pipeline;
/// # use experiment::process::{Process, ProcessPipeline};
/// let pipeline = pipeline!(
///     Process::new("echo", &["-e", "a\\nb\\nc"]),
///     Process::new("grep", &["b"])
/// );
/// ```
#[macro_export]
macro_rules! pipeline {
    ($($cmd:expr), *) => {{
        let mut vec: Vec<Process> = Vec::new();
        $(vec.push($cmd);)*
        ProcessPipeline::new(vec)
    }};
}
