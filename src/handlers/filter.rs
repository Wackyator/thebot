use sqlx::{Pool, Postgres};
use teloxide::{payloads::SendMessageSetters, requests::Requester, types::Message, utils::html};

use anyhow::anyhow;

use crate::{
    types::db::Note,
    utils::{
        self,
        db::{self, insert_note},
        perms,
    },
};

pub async fn get_note(
    bot: &crate::types::TBot,
    message: &Message,
    from_command: bool,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    let note_id: String;

    if !from_command {
        note_id = message
            .text()
            .ok_or(anyhow!("Unable to access message text"))?
            .replace("#", "");
    } else {
        let (_, text) = utils::extract_user_and_text(bot, message, pool).await;
        if text.is_none() {
            bot.send_message(message.chat.id, "You need to give me a note name!")
                .reply_to_message_id(message.id)
                .await?;

            return Ok(());
        }
        note_id = text.ok_or(anyhow!("Unable to access message text"))?;
    }

    let note = db::get_note(message.chat.id.0, note_id, pool).await?;

    bot.send_message(message.chat.id, note.note_content)
        .reply_to_message_id(message.id)
        .await?;

    Ok(())
}

pub async fn save_note(
    bot: &crate::types::TBot,
    message: &Message,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    tokio::try_join!(
        perms::require_user_admin(bot, message), // user requires admin permissions
    )?;

    let (_, text) = utils::extract_user_and_text(bot, message, pool).await;
    if text.is_none() {
        bot.send_message(message.chat.id, "You need to give the note a name!")
            .reply_to_message_id(message.id)
            .await?;
        return Ok(());
    }

    let content = text
        .as_ref()
        .ok_or(anyhow!("Unable to access message text"))?
        .split_once(' ');

    if content.is_none() {
        bot.send_message(message.chat.id, "You need to give the note some content!")
            .reply_to_message_id(message.id)
            .await?;
        return Ok(());
    }

    let (note_id, note_content) = content.ok_or(anyhow!("Unable to destructure content"))?;

    let chat_id = message.chat.id.0;
    let note = Note {
        chat_id,
        note_id: note_id.to_owned(),
        note_content: note_content.to_owned(),
    };

    match insert_note(&note, pool).await {
        Ok(_) => {
            bot.send_message(
                message.chat.id,
                format!("Saved note {}.", html::code_inline(note_id)),
            )
            .reply_to_message_id(message.id)
            .await?;
        }
        Err(e) => {
            bot.send_message(message.chat.id, e.to_string())
                .reply_to_message_id(message.id)
                .await?;
        }
    }

    Ok(())
}

pub async fn delete_note(
    bot: &crate::types::TBot,
    message: &Message,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    let (_, text) = utils::extract_user_and_text(bot, message, pool).await;
    if text.is_none() {
        bot.send_message(message.chat.id, "You need to give me a note name!")
            .reply_to_message_id(message.id)
            .await?;

        return Ok(());
    }

    let note_id = text.ok_or(anyhow!("Unable to access message text"))?;
    let _ = db::delete_note(message.chat.id.0, note_id.as_str(), pool).await?;

    bot.send_message(
        message.chat.id,
        format!("Successfully deleted {}!", html::code_inline(&note_id)),
    )
    .reply_to_message_id(message.id)
    .await?;

    Ok(())
}

pub async fn get_all_notes(
    bot: &crate::types::TBot,
    message: &Message,
    pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    let chat_id = message.chat.id.0;
    let (notes, chat) = tokio::try_join!(
        db::get_all_notes(chat_id, pool),
        db::get_chat(chat_id, pool),
    )?;

    let fmt_notes = notes
        .iter()
        .map(|n| format!("- {}", html::code_inline(n.note_id.as_str())))
        .fold(String::new(), |acc, ref v| acc + v + "\n");

    bot.send_message(
        message.chat.id,
        format!(
            "Following are all the notes in {}:\n{fmt_notes}",
            chat.chat_name.unwrap_or("current chat".to_string())
        ),
    )
    .reply_to_message_id(message.id)
    .await?;

    Ok(())
}
