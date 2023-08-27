

use anyhow::{anyhow};
use teloxide::{
    prelude::*,
    types::{ChatMember, ChatMemberStatus},
};

pub async fn require_user_admin(bot: &crate::types::TBot, message: &Message) -> anyhow::Result<()> {
    let user_id = match message.from() {
        Some(user) => user.id,
        None => {
            return Err(anyhow!("User not found"));
        }
    };

    match is_user_admin(bot, message, user_id).await {
        Ok(_) => Ok(()),
        Err(_) => {
            bot.send_message(message.chat.id, "You need to be an admin for this to work!")
                .reply_to_message_id(message.id)
                .await?;
            Err(anyhow!("User is not admin"))
        }
    }
}

pub async fn is_user_admin(
    bot: &crate::types::TBot,
    message: &Message,
    user_id: UserId,
) -> anyhow::Result<()> {
    if message.chat.is_private() {
        return Ok(());
    }

    let chat_member: ChatMember = bot.get_chat_member(message.chat.id, user_id).await?;

    match chat_member.status() {
        ChatMemberStatus::Administrator => Ok(()),
        ChatMemberStatus::Owner => Ok(()),
        _ => Err(anyhow!("User is not admin")),
    }
}
