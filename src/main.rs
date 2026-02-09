use env_logger::Env;
use iced::{
    Element,
    widget::{column, text_input},
};

mod discover;

fn main() -> iced::Result {
    env_logger::init_from_env(
        Env::new()
            .filter_or("TYLA_LOG", "info")
            .write_style("TYLA_LOG_STYLE"),
    );
    discover::desktop_apps();
    iced::run(App::update, App::view)
}

#[derive(Debug, Clone)]
enum Msg {
    SearchTextChanged(String),
}

#[derive(Default, Debug)]
struct App {
    search_text: String,
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
