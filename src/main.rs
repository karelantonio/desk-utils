use crate::discover::{DesktopEntry, desktop_apps};
use core::{cmp::Ordering, default::Default, include_bytes};
use env_logger::Env;
use iced::{
    Color, Element, Font, Length, Padding, Task, Theme,
    border::radius,
    color, font,
    widget::{
        Id, column, container,
        operation::{self, focus},
        row, scrollable, space, text, text_input,
    },
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
    FontLoaded,
    SearchTextChanged(String),
    CouldNotLoadFont(font::Error),
}

#[derive(Debug)]
struct App {
    search_text: String,
    entries: Vec<DesktopEntry>,
    strings: Vec<Vec<char>>,
    results: Vec<(f64, usize)>,
    search_box_id: Id,
}

const SEARCH: char = '\u{E65F}';
const PLAY: char = '\u{E6B9}';
const RIGHT_ARROW: char = '\u{E89B}';
const DOLLAR: char = '\u{E769}';
const RELOAD: char = '\u{E8A4}';

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
        (
            slf,
            font::load(include_bytes!("./icons.ttf"))
                .then(|res| {
                    Task::done(match res {
                        Ok(()) => Msg::FontLoaded,
                        Err(err) => Msg::CouldNotLoadFont(err),
                    })
                })
                .chain(focus(id)),
        )
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
            Msg::FontLoaded => {
                // Nothing to do
                log::debug!("Font file loaded successfully");
            }
            Msg::CouldNotLoadFont(_err) => {
                log::error!("Could not load font for icons");
            }
        }
    }

    fn search_box(&self) -> impl Into<Element<'_, Msg>> {
        container(row![
            text(SEARCH)
                .font(Font::with_name("Material-Design-Icons"))
                .size(24.0.dp()),
            space().width(16.0.dp()),
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
        ])
        .padding(Padding {
            top: 13.0.dp(),
            right: 13.0.dp(),
            bottom: 13.0.dp(),
            left: 13.0.dp(),
        })
        .style(|theme| {
            let palette = theme.extended_palette();

            container::Style {
                background: Some(palette.background.weak.color.into()),
                text_color: Some(palette.background.weak.text),
                border: iced::Border {
                    color: color!(0),
                    width: 0.0,
                    radius: radius(28.0.dp()),
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

trait DpExt {
    fn dp(&self) -> f32;
}

impl DpExt for f32 {
    fn dp(&self) -> f32 {
        *self
    }
}
