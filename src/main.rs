use crate::discover::{DesktopEntry, desktop_apps};
use core::{cmp::Ordering, default::Default};
use env_logger::Env;
use iced::{
    Color, Element, Length, Padding, Task, Theme,
    border::radius,
    color,
    widget::{Id, column, container, operation, scrollable, text, text_input},
};

mod discover;
mod fuzzy;

fn main() -> iced::Result {
    env_logger::init_from_env(
        Env::new()
            .filter_or("TYLA_LOG", "warn")
            .write_style("TYLA_LOG_STYLE"),
    );
    iced::application(App::new, App::update, App::view)
        .title("Tiny Launcher")
        .theme(App::theme)
        .run()
}

#[derive(Debug, Clone)]
enum Msg {
    SearchTextChanged(String),
}

#[derive(Debug)]
struct App {
    search_text: String,
    entries: Vec<DesktopEntry>,
    strings: Vec<Vec<char>>,
    results: Vec<(f64, usize)>,
    search_box_id: Id,
}

impl Default for App {
    fn default() -> Self {
        let entr = desktop_apps();
        let mut strings = Vec::new();
        for e in &entr {
            strings.push(e.name.chars().collect());
            strings.push(e.cmd.chars().collect());
        }
        Self {
            search_text: Default::default(),
            entries: entr,
            strings,
            results: Vec::new(),
            search_box_id: Id::new("search_box"),
        }
    }
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        let slf = Self::default();
        let id = slf.search_box_id.clone();
        (slf, operation::focus(id))
    }

    fn theme(&self) -> Theme {
        Theme::Dracula
    }

    fn do_fuzzy_search(&mut self, txt: &str) {
        // Perform a search if are different
        // There are not a lot of entries, so the O(n*maxlen*k) runtime is negligible
        let mapped: Vec<&[char]> = self.strings.iter().map(|v| v.as_slice()).collect();
        let term: Vec<char> = txt.chars().collect();

        let mut res: Vec<(f64, usize)> = (0..self.entries.len()).map(|v| (1f64, v)).collect();

        for val in fuzzy::search_in_chars(&term, &mapped)
            .iter()
            .enumerate()
            .map(|(i, &v)| (v, i / 2))
        {
            if let Some(Ordering::Greater) = res[val.1].0.partial_cmp(&val.0) {
                res[val.1].0 = val.0;
            }
        }
        res.sort_by(|f1, f2| match f1.partial_cmp(f2) {
            Some(val) => val,
            Option::None => Ordering::Equal,
        });
        self.results = res;
    }

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::SearchTextChanged(txt) => {
                self.results.clear();
                if txt.len() > 0 {
                    self.do_fuzzy_search(&txt);
                }
                self.search_text = txt;
            }
        }
    }

    fn search_box(&self) -> impl Into<Element<'_, Msg>> {
        container(
            text_input("Search term here...", &self.search_text)
                .on_input(Msg::SearchTextChanged)
                .width(Length::Fill)
                .style(|thm, st| {
                    let defstyle = text_input::default(thm, st);
                    text_input::Style {
                        border: defstyle.border.width(0),
                        background: iced::Background::Color(Color::TRANSPARENT),
                        ..defstyle
                    }
                })
                .id(self.search_box_id.clone()),
        )
        .padding(Padding {
            top: 16.0,
            right: 16.0,
            bottom: 16.0,
            left: 16.0,
        })
        .style(|theme| {
            let palette = theme.extended_palette();

            container::Style {
                background: Some(palette.background.weak.color.into()),
                text_color: Some(palette.background.weak.text),
                border: iced::Border {
                    color: color!(0),
                    width: 0.0,
                    radius: radius(28),
                },
                ..Default::default()
            }
        })
    }

    fn result_item(&self, (perc, idx): &(f64, usize)) -> Element<'_, Msg> {
        text(format!(
            "{:.0} - {}",
            (1.0 - perc) * 100.0,
            self.entries[*idx].name
        ))
        .into()
    }

    fn view(&self) -> Element<'_, Msg> {
        column![
            container(self.search_box()).padding(8),
            scrollable(column![].extend(self.results.iter().map(|arg| self.result_item(arg))))
                .width(Length::Fill)
        ]
        .into()
    }
}
