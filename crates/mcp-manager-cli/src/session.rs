//! How this run talks to the user: whether it may prompt, how it reports, and
//! how failures map to exit codes.

use crate::agent::Agent;
use crate::resolve::Decision;
use mcp_manager_core::platform::PlatformContext;
use std::fmt::Display;
use std::io::{self, IsTerminal};

#[derive(Debug)]
pub enum Failure {
    /// The user backed out of a prompt; nothing was written.
    Cancelled,
    /// A step needed an answer but prompting is not possible. The message
    /// names the flag that answers it.
    NeedsInput(String),
    Error(String),
}

impl From<String> for Failure {
    fn from(message: String) -> Self {
        Failure::Error(message)
    }
}

impl From<io::Error> for Failure {
    fn from(error: io::Error) -> Self {
        if error.kind() == io::ErrorKind::Interrupted {
            Failure::Cancelled
        } else {
            Failure::Error(error.to_string())
        }
    }
}

pub type Outcome<T> = Result<T, Failure>;

pub struct Session {
    pub ctx: PlatformContext,
    /// Prompts may be shown: stdin is a terminal and neither `--yes`,
    /// `--json` nor a detected agent ruled them out.
    pub interactive: bool,
    /// Take defaults and skip the final confirmation.
    pub yes: bool,
    /// Print one JSON document on stdout and nothing decorative.
    pub json: bool,
    pub agent: Option<Agent>,
}

impl Session {
    pub fn new(yes: bool, json: bool, agent: Option<Agent>) -> Self {
        let yes = yes || agent.is_some();
        Session {
            ctx: PlatformContext::current(),
            interactive: !yes && !json && io::stdin().is_terminal(),
            yes,
            json,
            agent,
        }
    }

    pub fn intro(&self, title: &str) -> Outcome<()> {
        if self.json {
            return Ok(());
        }
        cliclack::intro(title)?;
        if let Some(agent) = &self.agent {
            cliclack::log::info(format!(
                "Running inside {} — answering prompts with defaults",
                agent.name
            ))?;
        }
        Ok(())
    }

    pub fn info(&self, message: impl Display) -> Outcome<()> {
        if !self.json {
            cliclack::log::info(message)?;
        }
        Ok(())
    }

    pub fn warn(&self, message: impl Display) -> Outcome<()> {
        if self.json {
            eprintln!("warning: {message}");
        } else {
            cliclack::log::warning(message)?;
        }
        Ok(())
    }

    pub fn note(&self, title: &str, body: &str) -> Outcome<()> {
        if !self.json {
            cliclack::note(title, body)?;
        }
        Ok(())
    }

    pub fn outro(&self, message: impl Display) -> Outcome<()> {
        if !self.json {
            cliclack::outro(message)?;
        }
        Ok(())
    }

    /// Settles a multi-choice step: asks when interactive, otherwise takes the
    /// preselected answer, or fails naming `flag` when there is none.
    pub fn choose_many<T: Clone + Eq>(
        &self,
        decision: Decision<T>,
        prompt: &str,
        label: impl Fn(&T) -> (String, String),
        flag: &str,
    ) -> Outcome<Vec<T>> {
        match decision {
            Decision::Use(chosen, note) => {
                if let Some(note) = note {
                    self.info(note)?;
                }
                Ok(chosen)
            }
            Decision::Ask {
                options,
                preselected,
            } if self.interactive => {
                let mut question = cliclack::multiselect(prompt)
                    .initial_values(preselected)
                    .required(true)
                    .max_rows(12);
                if options.len() > 8 {
                    question = question.filter_mode();
                }
                for option in options {
                    let (text, hint) = label(&option);
                    question = question.item(option, text, hint);
                }
                Ok(question.interact()?)
            }
            Decision::Ask { preselected, .. } if !preselected.is_empty() => Ok(preselected),
            Decision::Ask { .. } => Err(Failure::NeedsInput(format!(
                "{prompt}: pass {flag} to answer without a prompt."
            ))),
        }
    }

    pub fn choose_one<T: Clone + Eq>(
        &self,
        decision: Decision<T>,
        prompt: &str,
        label: impl Fn(&T) -> (String, String),
        default: T,
    ) -> Outcome<T> {
        match decision {
            Decision::Use(chosen, note) => {
                if let Some(note) = note {
                    self.info(note)?;
                }
                Ok(chosen.into_iter().next().unwrap_or(default))
            }
            Decision::Ask {
                options,
                preselected,
            } if self.interactive => {
                let mut question = cliclack::select(prompt);
                if let Some(first) = preselected.into_iter().next() {
                    question = question.initial_value(first);
                }
                for option in options {
                    let (text, hint) = label(&option);
                    question = question.item(option, text, hint);
                }
                Ok(question.interact()?)
            }
            Decision::Ask { .. } => Ok(default),
        }
    }

    /// The last question before writing; `--yes` and non-interactive runs skip it.
    pub fn confirm(&self, prompt: &str) -> Outcome<()> {
        if !self.interactive {
            return Ok(());
        }
        if cliclack::confirm(prompt).initial_value(true).interact()? {
            Ok(())
        } else {
            Err(Failure::Cancelled)
        }
    }

    pub fn secret(&self, prompt: &str) -> Outcome<String> {
        Ok(cliclack::password(prompt)
            .mask('•')
            .allow_empty()
            .interact()?)
    }
}
