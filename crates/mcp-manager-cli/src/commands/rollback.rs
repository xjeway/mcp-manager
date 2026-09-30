use crate::session::{Failure, Outcome, Session};
use mcp_manager_core::history;

pub fn run(session: &Session) -> Outcome<()> {
    session.intro(concat!(env!("CARGO_BIN_NAME"), " rollback"))?;
    let Some(change) = history::latest(&session.ctx) else {
        session.outro("Nothing to roll back.")?;
        return Ok(());
    };

    let files = change.file_count();
    session.info(format!(
        "Restores the server list and {files} client file{} from before the last change.",
        if files == 1 { "" } else { "s" },
    ))?;
    if !session.interactive && !session.yes {
        return Err(Failure::NeedsInput(
            "Pass --yes to roll back without a prompt.".to_string(),
        ));
    }
    session.confirm("Roll back?")?;

    let report = history::rollback_last(&session.ctx)?;
    let mut message = "Rolled back.".to_string();
    if report.removed > 0 {
        message.push_str(&format!(
            " Removed {} client file{} that change created.",
            report.removed,
            if report.removed == 1 { "" } else { "s" }
        ));
    }
    if report.remaining > 0 {
        message.push_str(&format!(
            " {} earlier change{} can still be undone.",
            report.remaining,
            if report.remaining == 1 { "" } else { "s" }
        ));
    }
    session.outro(message)?;
    Ok(())
}
