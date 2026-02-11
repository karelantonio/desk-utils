use crate::discover::{DesktopEntry, desktop_apps};
use core::{cmp::Ordering, default::Default, include_bytes};
use env_logger::Env;
use iced::{
    Background, Color, Element, Font, Length, Padding, Subscription, Task, Theme,
    border::radius,
    color, font, keyboard,
    widget::{
        Id, column, container,
        operation::{self, focus},
        row, scrollable, space, text, text_input,
    },
};
use std::process::Command;

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
        .subscription(App::subscription)
        .run()
}

#[derive(Debug, Clone)]
enum Msg {
    FontLoaded,
    SearchTextChanged(String),
    CouldNotLoadFont(font::Error),
    EnterPressed,
    EscapePressed,
    UpPressed,
    DownPressed,
    CtrlEnterPressed,
}

#[derive(Debug)]
struct App {
    search_text: String,
    entries: Vec<DesktopEntry>,
    strings: Vec<Vec<char>>,
    results: Vec<(f64, usize)>,
    search_box_id: Id,
    selected_idx: usize,
    exec_error: Option<String>,
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
            selected_idx: 0,
            exec_error: None,
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

    fn subscription(&self) -> Subscription<Msg> {
        keyboard::listen().filter_map(|k: keyboard::Event| match k {
            keyboard::Event::KeyReleased {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                modified_key: _,
                physical_key: _,
                location: _,
                modifiers: _,
            } => Some(Msg::EscapePressed),
            keyboard::Event::KeyReleased {
                key: keyboard::Key::Named(keyboard::key::Named::ArrowUp),
                modified_key: _,
                physical_key: _,
                location: _,
                modifiers: _,
            } => Some(Msg::UpPressed),
            keyboard::Event::KeyReleased {
                key: keyboard::Key::Named(keyboard::key::Named::ArrowDown),
                modified_key: _,
                physical_key: _,
                location: _,
                modifiers: _,
            } => Some(Msg::DownPressed),
            keyboard::Event::KeyReleased {
                key: keyboard::Key::Named(keyboard::key::Named::Enter),
                modified_key: _,
                physical_key: _,
                location: _,
                modifiers,
            } => Some(if modifiers.control() {
                Msg::CtrlEnterPressed
            } else {
                Msg::EnterPressed
            }),
            _ => None,
        })
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

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::SearchTextChanged(txt) => {
                self.exec_error = None;
                self.results.clear();
                self.selected_idx = 0;
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
            Msg::EnterPressed => {
                // If term is empty then return
                if self.search_text.len() == 0 {
                    // Quit
                    return iced::exit();
                }
            }
            Msg::EscapePressed => {
                return iced::exit();
            }
            Msg::UpPressed => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
            }
            Msg::DownPressed => {
                if self.selected_idx + 1 < self.results.len() {
                    self.selected_idx += 1;
                }
            }
            Msg::CtrlEnterPressed => {
                if self.search_text.len() > 0 {
                    if let Err(err) = Command::new(self.search_text.clone()).spawn() {
                        log::error!("Could not spawn child: {err}");
                        self.exec_error = Some(format!("Could not spawn child: {err}"));
                    } else {
                        return iced::exit();
                    }
                }
            }
        }

        Task::none()
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

    fn result_item(&self, ridx: usize, (_perc, idx): &(f64, usize)) -> Element<'_, Msg> {
        let elem = &self.entries[*idx];
        let is_selected = self.selected_idx == ridx;
        let cont = container(text(format!("{idx:03} - {}", elem.name)))
            .padding(8)
            .width(Length::Fill);
        let cont = if is_selected {
            cont.style(|_thm| container::Style {
                background: Some(Background::Color(color!(0x101418))),
                ..Default::default()
            })
        } else {
            cont
        };

        cont.into()
    }

    fn error_message(&self) -> Element<'_, Msg> {
        let colors = self.theme().extended_palette().warning;
        if let Some(ref txt) = self.exec_error {
            container(
                container(text(txt).color(colors.base.text))
                    .style(move |thm| {
                        let colors = colors.clone();
                        container::Style {
                            background: Some(Background::Color(colors.base.color.clone())),
                            border: iced::Border {
                                color: colors.base.color,
                                width: 2.0,
                                radius: radius(8),
                            },
                            ..Default::default()
                        }
                    })
                    .padding(14)
                    .width(Length::Fill),
            )
            .padding(8)
            .into()
        } else {
            space().into()
        }
    }

    fn view(&self) -> Element<'_, Msg> {
        column![
            container(self.search_box()).padding(8),
            scrollable(
                column![].push(self.error_message()).extend(
                    self.results
                        .iter()
                        .enumerate()
                        .map(|(i, arg)| self.result_item(i, arg))
                )
            )
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
