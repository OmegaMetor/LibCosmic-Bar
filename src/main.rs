use cosmic::cctk;
use cosmic::cctk::sctk::shell::wlr_layer::Anchor;
use cosmic::iced::{self, Subscription};
use cosmic::iced_widget::row;
use hyprland::data::Workspaces;
use hyprland::data::*;
use hyprland::prelude::*;
use hyprland::shared::HyprData;
use iced::border::radius;
use iced::platform_specific::shell::commands::layer_surface::get_layer_surface;
use iced::platform_specific::shell::commands::overlap_notify::overlap_notify;
use iced::widget::{container, text};
use iced::{Border, Element, Event, Length, Padding, Task, Theme, event, time, window};

use chrono::Local;

/// The application model type.  See [the cosmic::iced book](https://book.cosmic::iced.rs/) for details.
#[derive(Debug)]
pub struct State {
    count: u32,
    active_workspace: i32,
}

/// Root struct of application
pub struct Shell {
    state: State,
}

/// Messages are how your logic mutates the app state and GUI
#[derive(Debug, Clone)]
pub enum ShellMessage {
    TimeTick(iced::time::Instant),
    ButtonPressed,
    OpenBlueman,
    HyprlandEvent(hyprland::event_listener::Event),
    HyprlandError,
    SetWorkspace(i32),
}

impl Shell {
    pub fn new() -> (Self, Task<ShellMessage>) {
        let id = window::Id::unique();

        let layer_shell_task = get_layer_surface(
            iced::platform_specific::runtime::wayland::layer_surface::SctkLayerSurfaceSettings {
                id,
                size: Some((Some(0), Some(30))),
                pointer_interactivity: true,
                keyboard_interactivity: cctk::sctk::shell::wlr_layer::KeyboardInteractivity::None,
                layer: cctk::sctk::shell::wlr_layer::Layer::Top,
                anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
                exclusive_zone: 20,
                ..Default::default()
            },
        );

        (
            Self {
                state: State {
                    count: 0,
                    active_workspace: hyprland::data::Workspace::get_active()
                        .expect("Failed to get hyprland workspace")
                        .id,
                },
            },
            Task::batch(vec![layer_shell_task, overlap_notify(id, true)]),
        )
    }

    /// Entry-point from `cosmic::iced`` into app to construct UI
    pub fn view(&self, _id: window::Id) -> Element<'_, ShellMessage> {
        container(
            row![
                iced::widget::button(text(self.state.count)).on_press(ShellMessage::ButtonPressed),
                iced::widget::row({
                    let mut workspaces = hyprland::data::Workspaces::get().unwrap().to_vec();

                    workspaces.sort_by_key(|workspace| workspace.id);
                    workspaces.into_iter().map(|workspace| {
                        iced::widget::button(text(workspace.id))
                            .on_press_maybe(if workspace.id != self.state.active_workspace {
                                Some(ShellMessage::SetWorkspace(workspace.id))
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
            .spacing(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([0, 10])),
        )
        .style(|theme: &Theme| container::Style {
            border: Border {
                radius: radius(10),
                ..Default::default()
            },
            background: Some(theme.extended_palette().background.weak.color.into()),
            ..Default::default()
        })
        .padding(Padding::from([0, 0]))
        .into()
    }

    /// Entry-point from `cosmic::iced` to handle user and system events
    pub fn update(&mut self, message: ShellMessage) -> Task<ShellMessage> {
        match message {
            ShellMessage::TimeTick(_) => Task::none(),
            ShellMessage::ButtonPressed => {
                self.state.count += 1;
                Task::none()
            }
            ShellMessage::OpenBlueman => {
                let _ = std::process::Command::new("hyprctl")
                    .arg("dispatch exec blueman-manager")
                    .spawn();
                Task::none()
            }
            ShellMessage::HyprlandError => Task::none(),
            ShellMessage::HyprlandEvent(event) => match event {
                hyprland::event_listener::Event::WorkspaceChanged(data) => {
                    self.state.active_workspace = data.id;
                    Task::none()
                }
                _ => {
                    dbg!(event);
                    Task::none()
                }
            },
            ShellMessage::SetWorkspace(id) => {
                use hyprland::dispatch;
                use hyprland::dispatch::Dispatch;
                use hyprland::dispatch::DispatchType;
                let _ = dispatch!(Workspace, dispatch::WorkspaceIdentifierWithSpecial::Id(id));
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<ShellMessage> {
        iced::Subscription::batch([
            time::every(std::time::Duration::from_millis(100)).map(ShellMessage::TimeTick),
            event::listen_with(|event, _status, _| match event {
                Event::PlatformSpecific(event::PlatformSpecific::Wayland(
                    event::wayland::Event::Layer(e, ..),
                )) => {
                    dbg!(e);
                    None
                }
                _ => None,
            }),
            Subscription::run(|| hyprland::event_listener::EventStream::new()).map(|hyprevent| {
                match hyprevent {
                    Ok(result) => ShellMessage::HyprlandEvent(result),
                    Err(_) => ShellMessage::HyprlandError,
                }
            }),
        ])
    }
}

fn main() -> iced::Result {
    let iced_settings = iced::settings::Settings {
        id: Some("Testing".to_string()),
        fonts: vec![],
        antialiasing: true,
        exit_on_close_request: true,
        is_daemon: false,
        ..Default::default()
    };

    // A function that returns the app struct
    let app_factory = || Shell::new();

    iced::daemon("Testing", Shell::update, Shell::view)
        .settings(iced_settings)
        .subscription(Shell::subscription)
        .run_with(app_factory)
}
