use core::{cmp::Ordering, default::Default};

use env_logger::Env;
use iced::{
    Element,
    widget::{column, keyed::Column, pick_list, scrollable, text, text_input},
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
    strings: Vec<Vec<char>>,
    results: Vec<(f64, usize)>,
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
        }
    }
}

impl App {
    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::SearchTextChanged(txt) => {
                self.results.clear();
                if txt.len() > 0 {
                    // Perform a search if are different
                    // There are not a lot of entries, so the O(n*maxlen*k) runtime is negligible
                    let mapped: Vec<&[char]> = self.strings.iter().map(|v| v.as_slice()).collect();
                    let term: Vec<char> = txt.chars().collect();
                    let mut res: Vec<(f64, usize)> = fuzzy::search_in_chars(&term, &mapped)
                        .iter()
                        .enumerate()
                        .map(|(i, &v)| (v, i))
                        .collect();
                    // Now sort by the float
                    res.sort_by(|a, b| match a.partial_cmp(b) {
                        Some(val) => val,
                        Option::None => Ordering::Equal,
                    });
                    self.results.extend(&res);
                }
                self.search_text = txt;
            }
        }
    }

    fn result_item(&self, (perc, idx): &(f64, usize)) -> Element<'_, Msg> {
        text(format!(
            "{:.0} - {}",
            (1.0 - perc) * 100.0,
            self.entries[idx / 2].name
        ))
        .into()
    }

    fn view(&self) -> Element<'_, Msg> {
        column![
            text_input("Search term here...", &self.search_text).on_input(Msg::SearchTextChanged),
            scrollable(column![].extend(self.results.iter().map(|arg| self.result_item(arg))))
        ]
        .into()
    }
}
