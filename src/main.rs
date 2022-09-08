#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::gui::RutabagaApplication;

mod core;
mod gui;

#[tokio::main]
async fn main() -> iced::Result {
    RutabagaApplication::start()
}
