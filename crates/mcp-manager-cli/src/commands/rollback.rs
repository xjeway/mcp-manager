use crate::session::{Failure, Outcome, Session};
use crate::state;
use mcp_manager_core::{storage, store};

pub fn run(session: &Session) -> Outcome<()> {
    session.intro(concat!(env!("CARGO_BIN_NAME"), " rollback"))?;
    let mut remembered = state::load(&session.ctx);
    let Some(previous) = remembered.last_previous_config.clone() else {
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

    storage::rollback(remembered.last_backups.clone())?;
    store::update(&session.ctx, |config| {
        *config = previous;
        Ok(())
    })?;

    remembered.last_backups.clear();
    remembered.last_previous_config = None;
    state::save(&session.ctx, &remembered)?;
    session.outro("Rolled back. Client files created by that change were left in place.")?;
    Ok(())
}
