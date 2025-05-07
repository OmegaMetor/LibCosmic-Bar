use std::path::{Path, PathBuf};

use cosmic::cctk;
use cosmic::cctk::sctk::shell::wlr_layer::Anchor;
use cosmic::iced::alignment::Vertical;
use cosmic::iced::event::Status;
use cosmic::iced::event::wayland::LayerEvent;
use cosmic::iced::futures::{SinkExt, StreamExt};
use cosmic::iced::keyboard::Key;
use cosmic::iced::keyboard::key::Named;
use cosmic::iced::{self, Alignment, Color, Subscription, mouse};
use cosmic::iced_core::SmolStr;
use cosmic::iced_runtime::platform_specific::wayland::layer_surface::{
    IcedOutput, SctkLayerSurfaceSettings,
};
use cosmic::iced_runtime::{Appearance, default};
use cosmic::iced_widget::text_input::Style;
use cosmic::iced_widget::{column, row, text_input};
use cosmic::iced_winit::commands::layer_surface::destroy_layer_surface;
use cosmic::iced_winit::commands::subsurface::KeyboardInteractivity;
use cosmic::widget::Space;
use hyprland::dispatch::DispatchType;
use hyprland::dispatch::{Dispatch, WorkspaceIdentifierWithSpecial};
use hyprland::shared::HyprData;
use hyprland::{dispatch, prelude::*};
use iced::border::radius;
use iced::platform_specific::shell::commands::layer_surface::get_layer_surface;
use iced::platform_specific::shell::commands::overlap_notify::overlap_notify;
use iced::widget::{container, text};
use iced::{Border, Element, Event, Length, Padding, Task, Theme, event, keyboard, time, window};
use xdg_desktop_entries::{ApplicationDesktopEntry, DesktopEntryType};

use chrono::Local;
use rust_fuzzy_search::fuzzy_compare;
use walkdir::WalkDir;

trait Window {
    type Message;

    fn new() -> (Self, Task<Self::Message>)
    where
        Self: Sized;
    fn view(self: &Self) -> Element<'_, Self::Message>;
    fn update(self: &Self, message: Self::Message) -> Task<Self::Message>;
    fn subscription(self: &Self) -> iced::Subscription<Self::Message>;
}

#[derive(Debug)]
pub struct LauncherWindow {
    launcher_window: Option<window::Id>,
    launcher_input: String,
    launcher_results: Vec<ApplicationDesktopEntry>, // TODO: Change this to be a generic action, so i can do math and stuff too
    launcher_apps: Vec<ApplicationDesktopEntry>,
    launcher_selection: usize,
}

#[derive(Debug, Clone)]
pub enum LauncherMessage {
    LauncherInput(String),
    LauncherSubmit,
}

impl Window for LauncherWindow {
    type Message = LauncherMessage;

    fn new() -> (Self, Task<Self::Message>)
    where
        Self: Sized,
    {
        (
            Self {
                launcher_window: None,
                launcher_input: "".to_string(),
                launcher_results: Vec::new(),
                launcher_apps: Vec::new(),
                launcher_selection: 0,
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
                    text_input("", &self.launcher_input)
                        .on_input(LauncherMessage::LauncherInput)
                        .on_submit_maybe({
                            if self.launcher_results.len() != 0 {
                                Some(LauncherMessage::LauncherSubmit)
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
                        let show_count = self.launcher_results.len();
                        if show_count > 0 {
                            items.push(
                                container(
                                    iced::widget::column({
                                        let mut items = vec![];
                                        for i in 0..show_count {
                                            items.push({
                                                let mut item = container(
                                                    text(&self.launcher_results[i].name)
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
                                                if i == self.launcher_selection {
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

    fn update(self: &Self, message: Self::Message) -> Task<Self::Message> {
        return match message {
            LauncherMessage::LauncherInput(input) => {
                self.launcher_input = input;

                let mut searched_apps: Vec<(&ApplicationDesktopEntry, f32)> = self
                    .launcher_apps
                    .iter()
                    .filter_map(|app| {
                        let similarity = fuzzy_compare(
                            app.name.to_lowercase().as_str(),
                            self.launcher_input.to_lowercase().as_str(),
                        );
                        if similarity == 0.0 {
                            return None;
                        }

                        Some((app, similarity))
                    })
                    .collect();

                searched_apps.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                self.launcher_results.clear();
                if self.state.launcher_input.len() != 0 {
                    let count = std::cmp::min(searched_apps.len(), 5);
                    self.state.launcher_selection =
                        std::cmp::min(self.state.launcher_selection, count);

                    for i in 0..count {
                        self.state.launcher_results.push(searched_apps[i].0.clone())
                    }
                }

                Task::none()
            }
            LauncherMessage::LauncherSubmit => {
                dbg!(&self.state.launcher_results[self.state.launcher_selection]);
                if let Some(command) =
                    &self.state.launcher_results[self.state.launcher_selection].exec
                {
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
                    Task::done(ToggleLauncher)
                } else {
                    Task::none()
                }
            }
        };
    }

    fn subscription(self: &Self) -> iced::Subscription<Self::Message> {
        Subscription::none()
    }
}

#[derive(Debug)]
pub struct State {
    count: u32,
    active_workspace: i32,
    bar_id: window::Id,
    launcher_window: Option<window::Id>,
    launcher_window_t: LauncherWindow
}

pub struct Shell {
    state: State,
}

// This enum is for identifying workspaces that also includes the special workspace
// Because hyprland-rs' doesn't work because of lifetime stuff
#[derive(Debug, Clone)]
pub enum WorkspaceIdentifier {
    // The workspace Id
    Id(i32),
    // The workspace relative to the current workspace
    Relative(i32),
    // The workspace on the monitor relative to the current workspace
    RelativeMonitor(i32),
    // The workspace on the monitor relative to the current workspace, including empty workspaces
    RelativeMonitorIncludingEmpty(i32),
    // The open workspace relative to the current workspace
    RelativeOpen(i32),
    // The previous Workspace
    Previous,
    // The first available empty workspace
    Empty,
    // The name of the workspace
    Name(String),
    // The special workspace
    Special(Option<String>),
}

// Messages are how your logic mutates the app state and GUI
#[derive(Debug, Clone)]
pub enum ShellMessage {
    TimeTick(iced::time::Instant),
    ButtonPressed,
    OpenBlueman,
    HyprlandEvent(hyprland::event_listener::Event),
    HyprlandError,
    SetWorkspace(WorkspaceIdentifier),
    ShortcutError(String),
    ShortcutActivated(String),
    ShortcutsSetup,
    ToggleLauncher,
    Escape,
    ArrowKey(Key<SmolStr>),
    LayerFocused(window::Id),
    LauncherMessage(LauncherMessage),
}

impl Shell {
    pub fn new() -> (Self, Task<ShellMessage>) {
        let id = window::Id::unique();

        let bar_size = Some((None, Some(30)));
        let exclusive_zone = 30;

        let layer_shell_task = get_layer_surface(
            iced::platform_specific::runtime::wayland::layer_surface::SctkLayerSurfaceSettings {
                id,
                size: bar_size,
                layer: cctk::sctk::shell::wlr_layer::Layer::Bottom,
                anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
                exclusive_zone: exclusive_zone,
                output: IcedOutput::All,
                ..Default::default()
            },
        );

        let (launcher_window, task) = LauncherWindow::new();

        (
            Self {
                state: State {
                    count: 0,
                    active_workspace: hyprland::data::Workspace::get_active()
                        .expect("Failed to get hyprland workspace")
                        .id,
                    bar_id: id,
                    launcher_window: None,
                    launcher_window_t: launcher_window
                },
            },
            Task::batch(vec![layer_shell_task, overlap_notify(id, true), task.map(|e| ShellMessage::LauncherMessage(e))]),
        )
    }

    pub fn view(&self, id: window::Id) -> Element<'_, ShellMessage> {

        if self.state.launcher_window_t.launcher_window.is_some_and(|window_id| window_id == id) {
            return self.state.launcher_window_t.view().map(|e| ShellMessage::LauncherMessage(e));
        }

        if id == self.state.bar_id {
            container(
                row![
                    text(self.state.count),
                    iced::widget::row({
                        let mut workspaces = hyprland::data::Workspaces::get().unwrap().to_vec();

                        workspaces.sort_by_key(|workspace| workspace.id);
                        workspaces.into_iter().map(|workspace| {
                            let name = workspace.name;
                            iced::widget::button(text(name.clone()))
                                .on_press_maybe(if workspace.id != self.state.active_workspace {
                                    Some(ShellMessage::SetWorkspace(WorkspaceIdentifier::Name(
                                        name,
                                    )))
                                } else {
                                    None
                                })
                                .into()
                        })
                    }),
                    text("Hello, World! I'm a bad status bar!")
                        .width(Length::Fill)
                        .center(),
                    iced::widget::button("").on_press(ShellMessage::OpenBlueman),
                    text(format!(
                        "{}",
                        Local::now().format("%A, %B %e, %Y  %H:%M:%S")
                    )),
                ]
                .align_y(Vertical::Center)
                .spacing(10)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(Padding::from([0, 10])),
            )
            .style(|theme: &Theme| container::Style {
                border: Border {
                    radius: radius(0),
                    ..Default::default()
                },
                background: Some(theme.extended_palette().background.weak.color.into()),
                ..Default::default()
            })
            .padding(Padding::from([0, 0]))
            .into()
        } else {
            self.view_launcher()
        }
    }

    fn view_launcher(&self) -> Element<'_, ShellMessage> {
        
    }

    pub fn update(&mut self, message: ShellMessage) -> Task<ShellMessage> {
        use ShellMessage::*;
        match message {
            TimeTick(_) => Task::none(),
            ButtonPressed => {
                self.state.count += 1;
                Task::none()
            }
            OpenBlueman => {
                let _ = dispatch!(Exec, "blueman-manager");
                return if let Some(_) = self.state.launcher_window {
                    Task::done(ToggleLauncher)
                } else {
                    Task::none()
                };
            }
            HyprlandError => Task::none(),
            HyprlandEvent(event) => match event {
                hyprland::event_listener::Event::WorkspaceChanged(data) => {
                    self.state.active_workspace = data.id;
                    return if let Some(_) = self.state.launcher_window {
                        Task::done(ToggleLauncher)
                    } else {
                        Task::none()
                    };
                }
                _ => Task::none(),
            },
            SetWorkspace(workspace_identifier) => {
                use hyprland::dispatch;
                use hyprland::dispatch::Dispatch;
                use hyprland::dispatch::DispatchType;
                let _ = dispatch!(Workspace, {
                    match &workspace_identifier {
                        WorkspaceIdentifier::Id(id) => WorkspaceIdentifierWithSpecial::Id(*id),
                        WorkspaceIdentifier::Name(name) => {
                            WorkspaceIdentifierWithSpecial::Name(name.as_str())
                        }
                        _ => WorkspaceIdentifierWithSpecial::Empty,
                    }
                });
                return if let Some(_) = self.state.launcher_window {
                    Task::done(ToggleLauncher)
                } else {
                    Task::none()
                };
            }
            ShortcutActivated(thing) => {
                match thing.as_str() {
                    "ToggleLauncher" => {
                        self.state.count += 1;
                        // Toggle popup

                        return Task::done(ToggleLauncher);
                    }
                    _ => println!("Shouldn't happen! Shortcut ID {} is not handled!", thing), // TODO: Enums and hashmaps? We'll see!
                }
                Task::none()
            }
            ToggleLauncher => {
                match self.state.launcher_window {
                    Some(id) => {
                        self.state.launcher_window = None;
                        return destroy_layer_surface(id);
                    }
                    None => {
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
                        self.state.launcher_window = Some(id);
                        self.state.launcher_input = "".to_string();
                        self.state.launcher_results.clear();
                        self.state.launcher_apps.clear();
                        self.state.launcher_selection = 0;

                        // Fill launcher appps vec
                        // hm

                        self.state.launcher_apps.extend(
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
                                            && entry
                                                .path()
                                                .extension()
                                                .is_some_and(|e| e == "desktop")
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
                }
            }
            ShortcutError(error) => {
                dbg!(error);
                Task::none()
            }
            ShortcutsSetup => {
                dbg!("Shortcuts Setup");
                Task::none()
            }
            Escape => match self.state.launcher_window {
                Some(_) => Task::done(ToggleLauncher),
                None => Task::none(),
            },
            LayerFocused(layer_id) => match self.state.launcher_window {
                Some(id) if id == layer_id => text_input::focus("launcher"),
                _ => Task::none(),
            },
            ArrowKey(key) => {
                if self.state.launcher_window.is_none() || self.state.launcher_results.len() == 0 {
                    return Task::none();
                }

                self.state.launcher_selection = (self.state.launcher_selection as i32
                    + match key {
                        Key::Named(Named::ArrowUp) => -1,
                        Key::Named(Named::ArrowDown) => 1,
                        _ => 0,
                    })
                .clamp(0, self.state.launcher_results.len() as i32 - 1)
                    as usize;

                Task::none()
            }
            LauncherMessage(message) => {
                self.state.launcher_window_t.update(message).map(|e| ShellMessage::LauncherMessage(e))
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<ShellMessage> {
        iced::Subscription::batch({
            let subscriptions = vec![
                self.state.launcher_window_t.subscription().map(|message| ShellMessage::LauncherMessage(message)),
                event::listen_with(|event, status, _| match event {
                    Event::PlatformSpecific(event::PlatformSpecific::Wayland(
                        event::wayland::Event::Layer(LayerEvent::Focused, _, id),
                    )) => Some(ShellMessage::LayerFocused(id)),
                    Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => match key {
                        Key::Named(Named::Escape) => Some(ShellMessage::Escape),
                        Key::Named(Named::ArrowUp) | Key::Named(Named::ArrowDown) => {
                            Some(ShellMessage::ArrowKey(key))
                        }
                        _ => None,
                    },
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                        if status == Status::Ignored =>
                    {
                        Some(ShellMessage::Escape)
                    }
                    _ => None,
                }),
                time::every(std::time::Duration::from_millis(100)).map(ShellMessage::TimeTick),
                Subscription::run(|| hyprland::event_listener::EventStream::new()).map(
                    |hyprevent| match hyprevent {
                        Ok(result) => ShellMessage::HyprlandEvent(result),
                        Err(_) => ShellMessage::HyprlandError,
                    },
                ),
                Subscription::run(|| {
                    iced::stream::channel(10, async |mut output| {
                        let proxy =
                            match ashpd::desktop::global_shortcuts::GlobalShortcuts::new().await {
                                Ok(proxy) => proxy,
                                Err(error) => {
                                    dbg!(error);
                                    return;
                                }
                            };

                        let session = match proxy.create_session().await {
                            Ok(session) => session,
                            Err(error) => {
                                dbg!(error);
                                return;
                            }
                        };

                        let shortcuts = vec![ashpd::desktop::global_shortcuts::NewShortcut::new(
                            "ToggleLauncher",
                            "Toggles the Application Launcher menu",
                        )];

                        let _ = proxy.bind_shortcuts(&session, &shortcuts, None).await;

                        let mut activated_stream = match proxy.receive_activated().await {
                            Ok(stream) => stream,
                            Err(error) => {
                                dbg!(error);
                                return;
                            }
                        };

                        loop {
                            if let Some(event) = activated_stream.next().await {
                                let _ = output
                                    .send(ShellMessage::ShortcutActivated(
                                        event.shortcut_id().to_string(),
                                    ))
                                    .await;
                            };
                        }
                    })
                }),
            ];

            subscriptions
        })
    }
}

fn main() -> iced::Result {
    iced::daemon("Testing", Shell::update, Shell::view)
        .subscription(Shell::subscription)
        .style(|_state, theme| Appearance {
            background_color: Color::TRANSPARENT,
            ..default(theme)
        })
        .antialiasing(true)
        .run_with(Shell::new)
}
