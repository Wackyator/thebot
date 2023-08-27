
use sqlx::{Pool, Postgres};
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{ChatId, Message, MessageEntity, MessageEntityKind},
};

pub mod db;
pub mod perms;

pub async fn extract_user_and_text(
    bot: &crate::types::TBot,
    message: &Message,
    pool: &Pool<Postgres>,
) -> (Option<u64>, Option<String>) {
    if let Some(msg_text) = message.text() {
        // split into command and args
        let split_text: Vec<_> = msg_text.splitn(2, char::is_whitespace).collect();

        // only command exists, return ID from reply
        if split_text.len() < 2 {
            return id_from_reply(bot, message);
        }

        // parse second part of split
        let text_to_parse = split_text[1];
        // split into vec of strings
        let args: Vec<_> = text_to_parse.split_whitespace().collect();

        let mut user_id: Option<u64> = None; // user id to return
        let mut text: Option<String> = Some(split_text[1].to_owned()); // text to return
        let mut ent: Option<&MessageEntity> = None; // mentioned entity in message

        // if entities exist in message
        if let Some(entities) = message.entities() {
            // filter out only text mention entities
            let filtered_entities = entities
                .iter()
                .filter(|ent| matches!(ent.kind, MessageEntityKind::TextMention { user: _ }));

            // use first entity for extracting user
            if !filtered_entities.count() == 0 {
                ent = Some(&entities[0]);
            }

            // if entity offset matches (command end/text start) then all is well
            if !entities.is_empty() && ent.is_some() {
                if ent.unwrap().offset
                    == msg_text.len() - text_to_parse.len()
                {
                    ent = Some(&entities[0]);
                    user_id = match &ent.unwrap().kind {
                        MessageEntityKind::TextMention { user } => Some(user.id.0),
                        _ => None,
                    };

                    text = Some(
                        msg_text[ent.unwrap().offset + ent.unwrap().length..].to_owned(),
                    );
                }
            } 
            // args exist and first arg is a @ mention
            else if !args.is_empty() && args[0].starts_with('@') {
                let user_name = args[0];
                let res =
                    db::get_user(None, Some(user_name.to_string().replace('@', "")), pool).await;

                if res.is_ok() {
                    user_id = Some(res.unwrap().user_id as u64);
                    let split: Vec<_> = msg_text.splitn(3, char::is_whitespace).collect();
                    if split.len() >= 3 {
                        text = Some(split[2].to_owned());
                    }
                } else {
                    bot.send_message(
                        message.chat.id,
                        "Could not find a user by this name; are you sure I've seen them before?",
                    )
                    .reply_to_message_id(message.id)
                    .await
                    .ok();
                    return (None, None);
                }
            }
            // check if first argument is a user ID
            else if !args.is_empty() {
                if let Ok(id) = args[0].parse::<u64>() {
                    user_id = Some(id);
                    let res: Vec<_> = msg_text.splitn(3, char::is_whitespace).collect();
                    if res.len() >= 3 {
                        text = Some(res[2].to_owned());
                    }
                }
            } 
            // check if command is a reply to message
            else if message.reply_to_message().is_some() {
                (user_id, text) = id_from_reply(bot, message);
            } else {
                // nothing satisfied, bail
                return (None, None);
            }

            // check if bot has interacted with this user before
            if let Some(id) = user_id {
                match bot.get_chat(ChatId(id as i64)).await {
                    Ok(_) => {}
                    Err(_) => {
                        // haven't seen this user, bail
                        bot.send_message(
                            message.chat.id, 
                            "I don't seem to have interacted with this user before - please forward a message from them to give me control!",
                        ).reply_to_message_id(message.id).await.ok();
                        return (None, None);
                    }
                }
            }
        }

    // return user ID and extracted text
    return (user_id, text);
    }
    
    (None, None)
}

pub fn id_from_reply(
    _bot: &crate::types::TBot,
    message: &Message,
) -> (Option<u64>, Option<String>) {
    // check for reply
    let prev_message = message.reply_to_message();
    if prev_message.is_none() {
        return (None, None);
    }

    // if can get user from replied-to message
    if let Some(user) = prev_message.unwrap().from() {
        // if quoted message has some text
        if let Some(msg_text) = prev_message.unwrap().text() {
            // split into args
            let res: Vec<_> = msg_text.splitn(2, char::is_whitespace).collect();

            // no args, return only user ID
            if res.len() < 2 {
                return (Some(user.id.0), Some("".to_owned()));
            }

            // return user ID and text
            return (Some(user.id.0), Some(res[1].to_owned()));
        }
    }

    // nothing found, bail
    (None, None)
}