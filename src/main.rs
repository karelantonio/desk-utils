use core::default::Default;

use env_logger::Env;
use iced::{
    Element,
    widget::{column, text_input},
};

use crate::discover::{DesktopEntry, desktop_apps};

mod discover;
mod fuzzy;

fn main() -> iced::Result {
    env_logger::init_from_env(
        Env::new()
            .filter_or("TYLA_LOG", "warn")
            .write_style("TYLA_LOG_STYLE"),
    );
    iced::run(App::update, App::view)
}

#[derive(Debug, Clone)]
enum Msg {
    SearchTextChanged(String),
}

#[derive(Debug)]
struct App {
    search_text: String,
    entries: Vec<DesktopEntry>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            search_text: Default::default(),
            entries: desktop_apps(),
        }
    }
}

impl App {
    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::SearchTextChanged(txt) => self.search_text = txt,
        }
    }

    fn view(&self) -> Element<'_, Msg> {
        column![
            text_input("Search term here...", &self.search_text).on_input(Msg::SearchTextChanged),
            "Here goes the results"
        ]
        .into()
    }
}
