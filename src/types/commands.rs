use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum UserCommands {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "start the bot.")]
    Start,
    #[command(description = "save a note.")]
    Save,
    #[command(description = "retrieve a note.")]
    Get,
    #[command(description = "delete a note.")]
    Delete,
    #[command(description = "get all notes in chat.")]
    Notes,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These are sudo commands:")]
pub enum SudoCommands {
    #[command(description = "sudo help.")]
    SHelp,
}
