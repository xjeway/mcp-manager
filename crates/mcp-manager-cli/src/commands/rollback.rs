use crate::session::{Failure, Outcome, Session};
use crate::state;
use mcp_manager_core::{storage, store};
use std::path::Path;

pub fn run(session: &Session) -> Outcome<()> {
    session.intro(concat!(env!("CARGO_BIN_NAME"), " rollback"))?;
    let mut remembered = state::load(&session.ctx);
    let (Some(previous), Some(applied)) = (
        remembered.last_previous_config.clone(),
        remembered.last_applied_config.clone(),
    ) else {
        session.outro("Nothing to roll back.")?;
        return Ok(());
    };

    session.info(format!(
        "Restores the server list and {} client file{} from before the last change.",
        remembered.last_backups.len(),
        if remembered.last_backups.len() == 1 {
            ""
        } else {
            "s"
        }
    ))?;
    if !session.interactive && !session.yes {
        return Err(Failure::NeedsInput(
            "Pass --yes to roll back without a prompt.".to_string(),
        ));
    }
    session.confirm("Roll back?")?;

    let backups = remembered.last_backups.clone();
    store::update(&session.ctx, |config| {
        // Undoing on top of a later edit (from the app or another command)
        // would silently discard it, in servers.yaml and in client files.
        if serde_json::to_value(&*config).ok() != serde_json::to_value(&applied).ok() {
            return Err(
                "The server list changed after the last change made here (in MCP Manager or \
                 another command), so rolling back would discard that edit. Nothing was changed."
                    .to_string(),
            );
        }
        let edited = remembered
            .last_written_files
            .iter()
            .filter(|file| {
                store::fingerprint(Path::new(&file.path)).ok().as_deref()
                    != Some(file.fingerprint.as_str())
            })
            .map(|file| file.path.clone())
            .collect::<Vec<_>>();
        if !edited.is_empty() {
            return Err(format!(
                "These client files were edited after the last change made here, so rolling \
                 back would discard those edits: {}. Nothing was changed.",
                edited.join(", ")
            ));
        }
        // Client files are restored only once servers.yaml is known to be
        // ours, and before it is replaced, so a failure leaves both as they were.
        storage::rollback(backups)?;
        *config = previous;
        Ok(())
    })?;

    remembered.clear_rollback();
    state::save(&session.ctx, &remembered)?;
    session.outro("Rolled back. Client files created by that change were left in place.")?;
    Ok(())
}
