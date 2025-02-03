use crate::common::unique_id::generate_uuid_v4;
use crate::session::model::message::Message;
use crate::session::repository::MessageRepository;
use crate::session::{model::session::Session, repository::SessionRepository};
use anyhow::Result;
use chrono::{Duration, NaiveDateTime, Utc};

pub fn fetch_all_sessions<SR: SessionRepository, MR: MessageRepository>(
    session_repo: &SR,
    message_repository: &MR,
) -> Result<()> {
    let session_entities = session_repo.fetch_all_sessions().unwrap_or_else(|_| vec![]);
    let sessions = session_entities
        .iter()
        .map(|s| Session::from(s))
        .collect::<Vec<Session>>();

    println!("\n");
    for session in sessions {
        let session = session_with_messages(message_repository, &session);
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

pub fn session<SR: SessionRepository, MR: MessageRepository>(
    session_repo: &SR,
    message_repository: &MR,
    name: &str,
) -> Result<Session> {
    let session = match session_repo.fetch_session_by_name(name) {
        Err(_) => {
            let id = generate_uuid_v4().to_string();
            let now = Utc::now().naive_utc();
            let expires_at: NaiveDateTime = now + Duration::hours(24);

            match session_repo.remove_current_from_all() {
                Ok(_) => {}
                Err(err) => panic!(
                    "could not remove current from previous sessions: {:#?}",
                    err
                ),
            }

            match session_repo.add_session(&id, name, expires_at, true) {
                Ok(_) => println!("New session '{}' expires at {}", name, expires_at),
                Err(err) => panic!("Could not create a new session: {:#?}", err),
            }
            let session = session_repo
                .fetch_session_by_name(name)
                .expect("could not fetch session");
            Session::from(&session)
        }
        Ok(session) => Session::from(&session),
    };

    let session = session_with_messages(message_repository, &session);
    Ok(session)
}

pub fn session_add_messages<SR: SessionRepository, MR: MessageRepository>(
    session_repo: &SR,
    message_repository: &MR,
    session: &Session,
) -> Result<()> {
    if !session.temporary {
        let new_messages = session
            .messages
            .iter()
            .filter(|message| message.id == "")
            .collect::<Vec<&Message>>();
        for message in new_messages {
            let message_with_id = message.copy_with_id(generate_uuid_v4().to_string());
            message_repository
                .add_message_to_session(&message_with_id.to_entity(&session.id))
                .expect("could not add new message to session");
        }
        let now = Utc::now().naive_utc();
        let expires_at: NaiveDateTime = now + Duration::hours(24);
        session_repo
            .update_session(&session.id, &session.name, expires_at, session.current)
            .expect("could not update session");
    }
    Ok(())
}

fn session_with_messages<MR: MessageRepository>(
    message_repository: &MR,
    session: &Session,
) -> Session {
    let messages = message_repository
        .fetch_messages_for_session(&session.id)
        .unwrap_or(Vec::new())
        .iter()
        .map(|m| Message::from(m))
        .collect::<Vec<Message>>();
    let session = session.copy_with_messages(messages);
    session
}
