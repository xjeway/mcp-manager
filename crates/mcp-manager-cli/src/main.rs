mod agent;
mod cli;
mod commands;
mod commit;
mod resolve;
mod session;
mod source;
mod state;

use clap::Parser;
use cli::{Cli, Command};
use session::{Failure, Session};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let session = Session::new(
        cli.yes,
        cli.json,
        agent::detect(|key| std::env::var(key).ok()),
    );

    let result = match &cli.command {
        Command::Add(args) => commands::add::run(&session, args),
        Command::List => commands::list::run(&session),
        Command::Remove(args) => commands::remove::run(&session, args),
        Command::Rollback => commands::rollback::run(&session),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        // A deliberate cancel at a prompt is not a failure.
        Err(Failure::Cancelled) => {
            report("Cancelled, nothing was written.", &session);
            ExitCode::SUCCESS
        }
        Err(Failure::NeedsInput(message)) => {
            report(
                &format!("{message}\nNothing was written. Run it in a terminal to answer interactively, or pass the flags above."),
                &session,
            );
            ExitCode::FAILURE
        }
        Err(Failure::Error(message)) => {
            // The prefix is for the app to recognise the error, not for people.
            let too_new = format!("{}: ", mcp_manager_core::store::TOO_NEW_ERROR);
            report(message.strip_prefix(&too_new).unwrap_or(&message), &session);
            ExitCode::FAILURE
        }
    }
}

fn report(message: &str, session: &Session) {
    if session.json || cliclack::outro_cancel(message).is_err() {
        eprintln!("{message}");
    }
}
