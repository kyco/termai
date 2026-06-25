use crate::common::unique_id::generate_uuid_v4;
use crate::session::model::message::Message;
use crate::session::repository::MessageRepository;
use crate::session::{model::session::Session, repository::SessionRepository};
use anyhow::{anyhow, Result};
use chrono::{Duration, NaiveDateTime, Utc};

/// Number of hours a session stays "fresh" before its expiry timestamp. The
/// timestamp doubles as a last-used marker (it is bumped on every save/use).
const SESSION_TTL_HOURS: i64 = 24;

fn fresh_expiry() -> NaiveDateTime {
    Utc::now().naive_utc() + Duration::hours(SESSION_TTL_HOURS)
}

/// Print every saved session (CLI `session list`). Built on [`list_sessions`] so
/// ordering and error handling match the data-returning path used by the chat UI.
pub fn fetch_all_sessions<SR: SessionRepository, MR: MessageRepository>(
    session_repo: &SR,
    message_repository: &MR,
) -> Result<()> {
    let sessions = list_sessions(session_repo, message_repository)?;

    println!("\n");
    for session in sessions {
        println!(
            "session: {}\nis current: {}\nexpires at: {}\nmessage: {}\n{}\n\n",
            session.name,
            session.current,
            session.expires_at,
            session.messages.len(),
            session.id
        );
    }

    Ok(())
}

/// Return all saved sessions (most-recently-used first) with their messages
/// hydrated, so callers can render counts / active markers. This is the
/// data-returning sibling of [`fetch_all_sessions`].
pub fn list_sessions<SR: SessionRepository, MR: MessageRepository>(
    session_repo: &SR,
    message_repository: &MR,
) -> Result<Vec<Session>> {
    let session_entities = session_repo
        .fetch_all_sessions()
        .map_err(|e| anyhow!("Failed to fetch sessions: {:?}", e))?;

    let sessions = session_entities
        .iter()
        .map(|s| {
            let session = Session::from(s);
            session_with_messages(message_repository, &session)
        })
        .collect::<Vec<Session>>();

    Ok(sessions)
}

pub fn get_most_recent_session<SR: SessionRepository, MR: MessageRepository>(
    session_repo: &SR,
    message_repository: &MR,
) -> Result<Session> {
    let session_entities = session_repo
        .fetch_all_sessions()
        .map_err(|e| anyhow!("Failed to fetch sessions: {:?}", e))?;

    if session_entities.is_empty() {
        return Err(anyhow!("No previous sessions found. Start a new session with a name: termai chat --session <name>"));
    }

    // Sort by expires_at (most recent first) and get the first one.
    // expires_at is updated every time the session is used, so it reflects the
    // last usage time.
    let mut sessions = session_entities
        .iter()
        .map(Session::from)
        .collect::<Vec<Session>>();

    sessions.sort_by(|a, b| b.expires_at.cmp(&a.expires_at));

    let most_recent = sessions
        .first()
        .ok_or_else(|| anyhow!("Failed to get most recent session"))?;

    let session = session_with_messages(message_repository, most_recent);
    Ok(session)
}

/// Fetch a session by name, creating it (and marking it current) if it does not
/// exist. This is the `--session <name>` launch path.
pub fn session<SR: SessionRepository, MR: MessageRepository>(
    session_repo: &SR,
    message_repository: &MR,
    name: &str,
) -> Result<Session> {
    let session = match session_repo.fetch_session_by_name(name) {
        Err(_) => {
            let id = generate_uuid_v4().to_string();
            let expires_at = fresh_expiry();

            session_repo
                .remove_current_from_all()
                .map_err(|e| anyhow!("could not clear current session flag: {:?}", e))?;

            session_repo
                .add_session(&id, name, expires_at, true)
                .map_err(|e| anyhow!("could not create a new session: {:?}", e))?;
            println!("New session '{}' expires at {}", name, expires_at);

            let session = session_repo
                .fetch_session_by_name(name)
                .map_err(|e| anyhow!("could not fetch session after create: {:?}", e))?;
            Session::from(&session)
        }
        Ok(session) => Session::from(&session),
    };

    let session = session_with_messages(message_repository, &session);
    Ok(session)
}

/// Load an EXISTING saved session by name, erroring if it does not exist (unlike
/// [`session`], which auto-creates). Marks the session current and bumps its
/// last-used timestamp. Used by the in-chat `/load` command.
pub fn load_session<SR: SessionRepository, MR: MessageRepository>(
    session_repo: &SR,
    message_repository: &MR,
    name: &str,
) -> Result<Session> {
    let entity = session_repo
        .fetch_session_by_name(name)
        .map_err(|_| anyhow!("Session '{}' not found", name))?;

    session_repo
        .remove_current_from_all()
        .map_err(|e| anyhow!("could not clear current session flag: {:?}", e))?;

    let mut session = Session::from(&entity);
    session.current = true;
    session.expires_at = fresh_expiry();

    session_repo
        .update_session(&session.id, &session.name, session.expires_at, true)
        .map_err(|e| anyhow!("could not mark session current: {:?}", e))?;

    Ok(session_with_messages(message_repository, &session))
}

/// Promote an in-memory (temporary) session to a persisted one under `name`:
/// insert the session row, flip it to non-temporary/current, and flush all of
/// its messages. This is what makes `/save` (and exit auto-save) actually
/// persist a chat that was started without `--session`.
pub fn promote_session<SR: SessionRepository, MR: MessageRepository>(
    session_repo: &SR,
    message_repository: &MR,
    session: &mut Session,
    name: &str,
) -> Result<()> {
    session.name = name.to_string();
    session.temporary = false;
    session.current = true;
    session.expires_at = fresh_expiry();

    session_repo
        .remove_current_from_all()
        .map_err(|e| anyhow!("could not clear current session flag: {:?}", e))?;

    session_repo
        .add_session(&session.id, &session.name, session.expires_at, true)
        .map_err(|e| anyhow!("could not persist session '{}': {:?}", name, e))?;

    flush_new_messages(message_repository, session)?;

    Ok(())
}

/// Rename an already-persisted session, guarding against name collisions and
/// preserving its last-used timestamp (a rename is not a use).
pub fn rename_session<SR: SessionRepository>(
    session_repo: &SR,
    session: &mut Session,
    new_name: &str,
) -> Result<()> {
    if let Ok(existing) = session_repo.fetch_session_by_name(new_name) {
        if existing.id != session.id {
            return Err(anyhow!(
                "A different session named '{}' already exists",
                new_name
            ));
        }
    }

    session.name = new_name.to_string();
    session_repo
        .update_session(
            &session.id,
            &session.name,
            session.expires_at,
            session.current,
        )
        .map_err(|e| anyhow!("could not rename session: {:?}", e))?;

    Ok(())
}

/// Whether a session with this exact name already exists.
pub fn session_exists<SR: SessionRepository>(session_repo: &SR, name: &str) -> bool {
    session_repo.fetch_session_by_name(name).is_ok()
}

/// Incrementally persist a non-temporary session: flush any new messages and
/// bump the last-used timestamp. No-op for temporary sessions (they must be
/// promoted first). Writes generated message ids back into `session` so the
/// same message is never inserted twice.
pub fn session_add_messages<SR: SessionRepository, MR: MessageRepository>(
    session_repo: &SR,
    message_repository: &MR,
    session: &mut Session,
) -> Result<()> {
    if session.temporary {
        return Ok(());
    }

    flush_new_messages(message_repository, session)?;

    let expires_at = fresh_expiry();
    session.expires_at = expires_at;
    session_repo
        .update_session(&session.id, &session.name, expires_at, session.current)
        .map_err(|e| anyhow!("could not update session: {:?}", e))?;

    Ok(())
}

/// Insert every message that has not been persisted yet (id == "") and write the
/// generated id back into the in-memory message, so a subsequent save does not
/// re-insert it (the root cause of the historic message-duplication bug).
fn flush_new_messages<MR: MessageRepository>(
    message_repository: &MR,
    session: &mut Session,
) -> Result<()> {
    let session_id = session.id.clone();
    for message in session.messages.iter_mut() {
        if !message.id.is_empty() {
            continue;
        }
        let new_id = generate_uuid_v4().to_string();
        let persisted = message.copy_with_id(new_id.clone());
        message_repository
            .add_message_to_session(&persisted.to_entity(&session_id))
            .map_err(|e| anyhow!("could not add message to session: {:?}", e))?;
        message.id = new_id;
    }
    Ok(())
}

fn session_with_messages<MR: MessageRepository>(
    message_repository: &MR,
    session: &Session,
) -> Session {
    let messages = message_repository
        .fetch_messages_for_session(&session.id)
        .unwrap_or_default()
        .iter()
        .map(Message::from)
        .collect::<Vec<Message>>();
    session.copy_with_messages(messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::common::model::role::Role;
    use crate::repository::db::SqliteRepository;
    use tempfile::TempDir;

    fn temp_repo() -> (TempDir, SqliteRepository) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let repo = SqliteRepository::new(path.to_str().unwrap()).unwrap();
        (dir, repo)
    }

    #[test]
    fn promote_persists_temporary_session_and_messages() {
        let (_dir, repo) = temp_repo();
        let mut session = Session::new_temporary();
        session.add_raw_message("hello".to_string(), Role::User);
        session.add_raw_message("hi there".to_string(), Role::Assistant);

        promote_session(&repo, &repo, &mut session, "my-chat").unwrap();

        assert!(!session.temporary);
        assert!(session_exists(&repo, "my-chat"));

        // Reload from the DB and confirm both messages survived, in order.
        let reloaded = load_session(&repo, &repo, "my-chat").unwrap();
        assert_eq!(reloaded.messages.len(), 2);
        assert_eq!(reloaded.messages[0].content, "hello");
        assert_eq!(reloaded.messages[0].role, Role::User);
        assert_eq!(reloaded.messages[1].content, "hi there");
    }

    #[test]
    fn repeated_saves_do_not_duplicate_messages() {
        let (_dir, repo) = temp_repo();
        let mut session = Session::new_temporary();
        session.add_raw_message("turn one".to_string(), Role::User);

        // First save promotes; subsequent saves are incremental.
        promote_session(&repo, &repo, &mut session, "dedup").unwrap();
        session_add_messages(&repo, &repo, &mut session).unwrap();
        session_add_messages(&repo, &repo, &mut session).unwrap();

        // Add a new turn and save again.
        session.add_raw_message("turn two".to_string(), Role::Assistant);
        session_add_messages(&repo, &repo, &mut session).unwrap();
        session_add_messages(&repo, &repo, &mut session).unwrap();

        let reloaded = load_session(&repo, &repo, "dedup").unwrap();
        assert_eq!(
            reloaded.messages.len(),
            2,
            "messages must not be duplicated across saves"
        );
    }

    #[test]
    fn load_session_errors_for_unknown_name() {
        let (_dir, repo) = temp_repo();
        let err = load_session(&repo, &repo, "does-not-exist").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn rename_session_rejects_collisions() {
        let (_dir, repo) = temp_repo();

        let mut a = Session::new_temporary();
        a.add_raw_message("a".to_string(), Role::User);
        promote_session(&repo, &repo, &mut a, "alpha").unwrap();

        let mut b = Session::new_temporary();
        b.add_raw_message("b".to_string(), Role::User);
        promote_session(&repo, &repo, &mut b, "beta").unwrap();

        // Renaming beta -> alpha must fail (collision with a different session).
        let err = rename_session(&repo, &mut b, "alpha").unwrap_err();
        assert!(err.to_string().contains("already exists"));

        // Renaming to a free name succeeds.
        rename_session(&repo, &mut b, "gamma").unwrap();
        assert!(session_exists(&repo, "gamma"));
        assert!(!session_exists(&repo, "beta"));
    }

    #[test]
    fn list_sessions_returns_counts() {
        let (_dir, repo) = temp_repo();
        let mut s = Session::new_temporary();
        s.add_raw_message("only message".to_string(), Role::User);
        promote_session(&repo, &repo, &mut s, "listed").unwrap();

        let sessions = list_sessions(&repo, &repo).unwrap();
        let found = sessions.iter().find(|x| x.name == "listed").unwrap();
        assert_eq!(found.messages.len(), 1);
    }
}
