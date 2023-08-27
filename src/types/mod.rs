use teloxide::{
    adaptors::{DefaultParseMode, Throttle},
    prelude::*,
    types::UserId,
};

pub mod commands;
pub mod db;

pub type TBot = DefaultParseMode<Throttle<Bot>>;

#[derive(Clone)]
pub struct ConfigParameters {
    pub sudo: Vec<UserId>,
}
