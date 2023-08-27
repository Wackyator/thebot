mod handlers;
mod types;
mod utils;

use std::vec;

use dotenv::dotenv;
use lazy_static::lazy_static;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use teloxide::{
    adaptors::throttle::Limits, prelude::*, types::ParseMode, utils::command::BotCommands,
};

use types::{commands::*, ConfigParameters, TBot};

use crate::{handlers::filter, utils::db::save_details};

lazy_static! {
    static ref DATABASE_URL: String = std::env::var("DATABASE_URL").expect("Expected database url");
    static ref POOL: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(2)
        .connect_lazy(&DATABASE_URL)
        .unwrap();
    static ref BOT_ID: i64 = std::env::var("BOT_ID")
        .expect("BOT_ID is required")
        .parse()
        .expect("Expected BOT_ID to be numeric");
}

#[tokio::main]
async fn main() {
    dotenv().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(-1);
    });

    pretty_env_logger::init();
    log::info!("Starting the bot...");

    let bot = Bot::from_env()
        .throttle(Limits::default())
        .parse_mode(ParseMode::Html);

    let params = ConfigParameters {
        sudo: vec![UserId(850322305)],
    };

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<UserCommands>()
                .endpoint(user_cmd_handler),
        )
        .branch(
            dptree::filter(|cfg: ConfigParameters, msg: Message| {
                msg.from()
                    .map(|u| cfg.sudo.contains(&u.id))
                    .unwrap_or_default()
            })
            .filter_command::<SudoCommands>()
            .endpoint(sudo_cmd_handler),
        )
        .branch(
            dptree::filter(|| true).endpoint(|bot: TBot, msg: Message| async move {
                save_details(&bot, &msg).await?;

                // check if update contains any text
                let text = msg.text();
                if text.is_none() {
                    return Ok(());
                }

                // handle note
                let unwrapped_text = text.unwrap();
                if unwrapped_text.starts_with('#')
                    && unwrapped_text.split_whitespace().count() < 2
                    && unwrapped_text != "#"
                {
                    filter::get_note(&bot, &msg, false, &*POOL).await?;
                }
                Ok(())
            }),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![params])
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: {upd:?}");
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    log::info!("Exiting the bot");
}

async fn user_cmd_handler(bot: TBot, message: Message, cmd: UserCommands) -> anyhow::Result<()> {
    save_details(&bot, &message).await?;
    match cmd {
        UserCommands::Help => {
            bot.send_message(message.chat.id, UserCommands::descriptions().to_string())
                .await?;
        }
        UserCommands::Start => {
            bot.send_message(message.chat.id, "start_message").await?;
        }
        UserCommands::Save => {
            filter::save_note(&bot, &message, &*POOL).await?;
        }
        UserCommands::Get => {
            filter::get_note(&bot, &message, true, &*POOL).await?;
        }
        UserCommands::Delete => {
            filter::delete_note(&bot, &message, &*POOL).await?;
            // bot.send_message(message.chat.id, "deleted note!")
            //     .reply_to_message_id(message.id)
            //     .await?;
        }
        UserCommands::Notes => {
            filter::get_all_notes(&bot, &message, &*POOL).await?;
            // bot.send_message(message.chat.id, "Following are all the notes in this chat!")
            //     .reply_to_message_id(message.id)
            //     .await?;
        }
    };

    Ok(())
}

async fn sudo_cmd_handler(bot: TBot, msg: Message, cmd: SudoCommands) -> anyhow::Result<()> {
    save_details(&bot, &msg).await?;
    match cmd {
        SudoCommands::SHelp => {
            bot.send_message(msg.chat.id, "shelp command message")
                .await?;
        }
    };

    Ok(())
}
