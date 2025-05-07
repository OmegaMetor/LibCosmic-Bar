use std::path::{Path, PathBuf};

use cosmic::cctk;
use cosmic::cctk::sctk::shell::wlr_layer::Anchor;
use cosmic::iced::event::Status;
use cosmic::iced::keyboard::Key;
use cosmic::iced::keyboard::key::Named;
use cosmic::iced::{self, Alignment, Color, Event, event, keyboard, mouse};
use cosmic::iced_runtime::platform_specific::wayland::layer_surface::{
    IcedOutput, SctkLayerSurfaceSettings,
};
use cosmic::iced_widget::text_input::Style;
use cosmic::iced_widget::{column, row, text_input};
use cosmic::iced_winit::commands::layer_surface::{destroy_layer_surface, get_layer_surface};
use cosmic::iced_winit::commands::subsurface::KeyboardInteractivity;
use cosmic::widget::Space;
use hyprland::dispatch;
use hyprland::dispatch::Dispatch;
use hyprland::dispatch::DispatchType;
use iced::border::radius;
use iced::widget::{container, text};
use iced::{Border, Element, Length, Task, Theme, window};
use walkdir::WalkDir;
use xdg_desktop_entries::{ApplicationDesktopEntry, DesktopEntryType};

use rust_fuzzy_search::fuzzy_compare;

use crate::ShellMessage;
use crate::window_trait::Window;

#[derive(Debug)]
pub struct Launcher {
    pub window: Option<window::Id>,
    pub input: String,
    pub results: Vec<ApplicationDesktopEntry>, // TODO: Change this to be a generic action, so i can do math and stuff too
    pub apps: Vec<ApplicationDesktopEntry>,
    pub selected_item: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    Input(String),
    Submit,
    Open,
    Close,
    SelectionUp,
    SelectionDown,
    ShellMessage(Box<ShellMessage>),
}

impl Window for Launcher {
    type Message = Message;

    fn new() -> (Self, Task<Self::Message>)
    {
        (
            Self {
                window: None,
                input: "".to_string(),
                results: Vec::new(),
                apps: Vec::new(),
                selected_item: 0,
            },
            Task::none(),
        )
    }

    fn view(self: &Self) -> Element<'_, Self::Message> {
        container(
            row![
                Space::with_width(Length::FillPortion(1)),
                column(vec![
                    Space::with_height(Length::FillPortion(1)).into(),
                    text_input("", &self.input)
                        .on_input(Message::Input)
                        .on_submit_maybe({
                            if self.results.len() != 0 {
                                Some(Message::Submit)
                            } else {
                                None
                            }
                        })
                        .id("launcher")
                        .size(30)
                        .style(|theme: &Theme, _status| Style {
                            background: theme.extended_palette().background.weak.color.into(),
                            border: Border {
                                color: theme.extended_palette().background.base.color.into(),
                                width: 0.0,
                                radius: radius(20)
                            },
                            icon: theme.extended_palette().primary.base.color.into(),
                            placeholder: theme.extended_palette().primary.weak.color.into(),
                            value: Color::BLACK,
                            selection: theme.extended_palette().secondary.base.color.into(),
                        })
                        .padding(15)
                        .into(),
                    iced::widget::column({
                        let mut items = vec![];
                        let show_count = self.results.len();
                        if show_count > 0 {
                            items.push(
                                container(
                                    iced::widget::column({
                                        let mut items = vec![];
                                        for i in 0..show_count {
                                            items.push({
                                                let mut item = container(
                                                    text(&self.results[i].name)
                                                        .size(20)
                                                        .wrapping(text::Wrapping::WordOrGlyph),
                                                )
                                                .center_y(Length::Fill)
                                                .padding(10)
                                                .width(Length::Fill)
                                                .style(|theme: &Theme| container::Style {
                                                    border: Border {
                                                        radius: radius(20),
                                                        width: 0.0,
                                                        ..Default::default()
                                                    },
                                                    background: Some(
                                                        theme
                                                            .extended_palette()
                                                            .background
                                                            .weak
                                                            .color
                                                            .into(),
                                                    ),
                                                    ..Default::default()
                                                });
                                                if i == self.selected_item {
                                                    item = item.style(|theme: &Theme| {
                                                        container::Style {
                                                            border: Border {
                                                                radius: radius(20),
                                                                width: 0.0,
                                                                ..Default::default()
                                                            },
                                                            background: Some(
                                                                theme
                                                                    .extended_palette()
                                                                    .primary
                                                                    .base
                                                                    .color
                                                                    .into(),
                                                            ),
                                                            ..Default::default()
                                                        }
                                                    });
                                                }
                                                item.into()
                                            })
                                        }
                                        items
                                    })
                                    .spacing(8),
                                )
                                .padding(4)
                                .style(|theme: &Theme| container::Style {
                                    background: Some(
                                        theme.extended_palette().background.strong.color.into(),
                                    ),
                                    border: Border {
                                        radius: radius(20),
                                        width: 0.0,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })
                                .clip(true)
                                .height(Length::FillPortion(show_count as u16))
                                .into(),
                            )
                        }

                        if show_count < 5 {
                            items.push(
                                Space::with_height(Length::FillPortion(5 - show_count as u16))
                                    .into(),
                            )
                        }
                        items
                    })
                    .width(Length::FillPortion(1))
                    .height(Length::FillPortion(1))
                    .into(),
                    Space::with_height(Length::FillPortion(1)).into()
                ])
                .spacing(5)
                .width(Length::FillPortion(1))
                .align_x(Alignment::Center),
                Space::with_width(Length::FillPortion(1))
            ]
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(
                theme
                    .extended_palette()
                    .background
                    .strong
                    .color
                    .scale_alpha(0.25)
                    .into(),
            ),
            ..Default::default()
        })
        .into()
    }

    fn update(self: &mut Self, message: Self::Message) -> Task<Self::Message> {
        use Message::*;
        return match message {
            Input(input) => {
                self.input = input;

                let mut searched_apps: Vec<(&ApplicationDesktopEntry, f32)> = self
                    .apps
                    .iter()
                    .filter_map(|app| {
                        let similarity = fuzzy_compare(
                            app.name.to_lowercase().as_str(),
                            self.input.to_lowercase().as_str(),
                        );
                        if similarity == 0.0 {
                            return None;
                        }

                        Some((app, similarity))
                    })
                    .collect();

                searched_apps.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                self.results.clear();
                if self.input.len() != 0 {
                    let count = std::cmp::min(searched_apps.len(), 5);
                    self.selected_item = std::cmp::min(self.selected_item, count);

                    for i in 0..count {
                        self.results.push(searched_apps[i].0.clone())
                    }
                }

                Task::none()
            }
            Submit => {
                if let Some(command) = &self.results[self.selected_item].exec {
                    let re = regex::Regex::new("%(?<code>.)").unwrap();

                    let replaced_command = re.replace_all(command, |captures: &regex::Captures| {
                        let ch = &captures["code"];
                        match ch {
                            "f" => "",  // TODO: Implement somehow?
                            "F" => "",  // TODO: Implement somehow?
                            "u" => "",  // TODO: Implement somehow?
                            "U" => "",  // TODO: Implement somehow?
                            "i" => "",  // TODO: Implement somehow?
                            "c" => "",  // TODO: Implement somehow?
                            "k" => "",  // TODO: Implement somehow?
                            "%" => "%", // Replace %% with %.
                            _ => "",
                        }
                    });
                    let _ = dispatch!(Exec, &replaced_command);
                    Task::done(Close)
                } else {
                    Task::none()
                }
            }
            Open => {
                if self.window.is_some() {
                    return Task::done(Close);
                }

                let id = window::Id::unique();

                let layer_shell_task = get_layer_surface(SctkLayerSurfaceSettings {
                    id,
                    layer: cctk::sctk::shell::wlr_layer::Layer::Top,
                    output: IcedOutput::Active,
                    keyboard_interactivity: KeyboardInteractivity::Exclusive,
                    pointer_interactivity: true,
                    anchor: Anchor::all(),
                    exclusive_zone: -1,
                    ..Default::default()
                });
                self.window = Some(id);
                self.input = "".to_string();
                self.results.clear();
                self.apps.clear();
                self.selected_item = 0;

                // Fill launcher appps vec
                // hm

                self.apps.extend(
                    std::iter::once(match std::env::var("XDG_DATA_HOME") {
                        Ok(dir) => PathBuf::from(dir),
                        Err(_) => {
                            let home_dir = std::env::var("HOME").unwrap();
                            Path::new(&home_dir).join(".local").join("share")
                        }
                    })
                    .chain(
                        std::env::var("XDG_DATA_DIRS")
                            .unwrap_or("/usr/local/share/:/usr/share/".into())
                            .split(':')
                            .map(|path| PathBuf::from(path)),
                    )
                    .map(|path| path.join("applications"))
                    .filter(|path| path.exists())
                    .map(|path| {
                        WalkDir::new(path)
                            .follow_links(true)
                            .into_iter()
                            .filter_map(|entry| entry.ok())
                            .filter(|entry| {
                                entry.file_type().is_file()
                                    && entry.path().extension().is_some_and(|e| e == "desktop")
                            })
                    })
                    .flatten()
                    .map(|entry| xdg_desktop_entries::parse_desktop_entry(entry.path()))
                    .filter_map(|value| match value {
                        Ok(entry) => {
                            if let DesktopEntryType::Application(app_entry) = entry {
                                if app_entry.no_display.is_some_and(|b| b) {
                                    return None;
                                }
                                if app_entry.hidden.is_some_and(|b| b) {
                                    return None;
                                }
                                // TODO: Handle OnlyShowIn, NotShowIn, TryExec
                                Some(app_entry)
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    }),
                );
                layer_shell_task
            }
            Close => {
                if let Some(id) = self.window {
                    self.window = None;
                    return destroy_layer_surface(id);
                }
                Task::none()
            }
            SelectionUp => {
                if self.window.is_none() || self.results.len() == 0 {
                    return Task::none();
                }

                self.selected_item = (self.selected_item as i32 - 1)
                .clamp(0, self.results.len() as i32 - 1)
                as usize;
                
                Task::none()
            }
            SelectionDown => {
                if self.window.is_none() || self.results.len() == 0 {
                    return Task::none();
                }
                
                self.selected_item = (self.selected_item + 1).clamp(0, self.results.len() - 1);

                Task::none()
            }

            ShellMessage(_) => Task::none(),
        };
    }

    fn subscription(self: &Self) -> iced::Subscription<Self::Message> {
        event::listen_with(|event, status, _id: window::Id| match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => match key {
                Key::Named(Named::Escape) => Some(Message::Close),
                Key::Named(Named::ArrowUp) => Some(Message::SelectionUp),
                Key::Named(Named::ArrowDown) => Some(Message::SelectionDown),
                _ => None,
            },
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if status == Status::Ignored =>
            {
                Some(Message::Close)
            }
            _ => None,
        })
    }
}
