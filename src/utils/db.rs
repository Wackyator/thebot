use sqlx::{Pool, Postgres};
use teloxide::types::{ChatKind, Message};
use tokio;

use crate::{
    types::{
        db::{Chat, Note, User},
        TBot,
    },
    POOL,
};

pub async fn save_details(bot: &TBot, message: &Message) -> anyhow::Result<()> {
    // opportunistically save user/chat details to db
    tokio::try_join!(
        save_user_handler(bot, message, &*POOL),
        save_chat_handler(bot, message, &*POOL)
    )?;

    Ok(())
}

pub async fn save_user_handler(
    _bot: &crate::types::TBot,
    message: &Message,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    if let Some(user) = message.from() {
        let username = user.username.as_ref().map(|s| s.to_lowercase());
        insert_user(
            &User {
                user_id: user.id.0 as i64,
                user_name: username,
                full_name: user.full_name(),
            },
            pool,
        )
        .await?;
    }

    Ok(())
}

pub async fn save_chat_handler(
    _bot: &crate::types::TBot,
    message: &Message,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    if message.chat.is_chat() {
        let chat = &message.chat;

        match &chat.kind {
            ChatKind::Public(pu) => insert_chat(chat.id.0, pu.title.clone(), pool).await?,
            ChatKind::Private(pri) => insert_chat(chat.id.0, pri.username.clone(), pool).await?,
        }
    }

    Ok(())
}

pub async fn insert_user(user: &User, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT into users (user_id, user_name, full_name) VALUES ($1, $2, $3) 
        ON CONFLICT (user_id) DO 
        UPDATE SET (user_name, full_name) = (excluded.user_name, excluded.full_name)
        WHERE (users.user_name, users.full_name) IS DISTINCT FROM (excluded.user_name, excluded.full_name)
        "#,
        user.user_id,
        user.user_name,
        user.full_name,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_chat(
    chat_id: i64,
    chat_name: Option<String>,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT into chats (chat_id, chat_name) VALUES ($1, $2)
        ON CONFLICT (chat_id) DO
        UPDATE SET chat_name = excluded.chat_name
        WHERE (chats.chat_name) IS DISTINCT FROM (excluded.chat_name)
        "#,
        chat_id,
        chat_name
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_note(note: &Note, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT into notes (chat_id, note_id, note_content) VALUES ($1, $2, $3)
        ON CONFLICT (chat_id, note_id) DO
        UPDATE SET note_content = excluded.note_content
        WHERE (notes.note_content) IS DISTINCT FROM (excluded.note_content)
    "#,
        note.chat_id,
        note.note_id,
        note.note_content
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_note(
    chat_id: i64,
    note_id: String,
    pool: &Pool<Postgres>,
) -> anyhow::Result<Note> {
    let note = sqlx::query_as!(
        Note,
        "SELECT * FROM notes WHERE chat_id = $1 AND note_id = $2",
        chat_id,
        note_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(note)
}

pub async fn get_user(
    user_id: Option<i64>,
    user_name: Option<String>,
    pool: &Pool<Postgres>,
) -> anyhow::Result<User> {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE user_id = $1 OR user_name = $2",
        user_id,
        user_name,
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_chat(chat_id: i64, pool: &Pool<Postgres>) -> anyhow::Result<Chat> {
    let chat = sqlx::query_as!(Chat, "SELECT * FROM chats WHERE chat_id = $1", chat_id)
        .fetch_one(pool)
        .await?;

    Ok(chat)
}

pub async fn delete_note(chat_id: i64, note_id: &str, pool: &Pool<Postgres>) -> anyhow::Result<()> {
    sqlx::query!(
        "DELETE FROM notes WHERE chat_id = $1 AND note_id = $2",
        chat_id,
        note_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_all_notes(chat_id: i64, pool: &Pool<Postgres>) -> anyhow::Result<Vec<Note>> {
    Ok(
        sqlx::query_as!(Note, "SELECT * FROM notes WHERE chat_id = $1", chat_id)
            .fetch_all(pool)
            .await?,
    )
}
