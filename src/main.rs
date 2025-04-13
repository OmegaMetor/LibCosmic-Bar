use cosmic::cctk;
use cosmic::cctk::sctk::shell::wlr_layer::Anchor;
use cosmic::iced::alignment::Vertical;
use cosmic::iced::futures::{SinkExt, StreamExt};
use cosmic::iced::{self, Subscription};
use cosmic::iced_widget::row;
use hyprland::prelude::*;
use hyprland::shared::HyprData;
use iced::border::radius;
use iced::platform_specific::shell::commands::layer_surface::get_layer_surface;
use iced::platform_specific::shell::commands::overlap_notify::overlap_notify;
use iced::widget::{container, text};
use iced::{Border, Element, Event, Length, Padding, Task, Theme, event, time, window};

use chrono::Local;

#[derive(Debug)]
pub struct State {
    count: u32,
    active_workspace: i32,
}

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
    ShortcutError(String),
    ShortcutActivated(String),
    ShortcutsSetup,
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
                layer: cctk::sctk::shell::wlr_layer::Layer::Top,
                anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
                exclusive_zone: exclusive_zone,
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

    pub fn view(&self, _id: window::Id) -> Element<'_, ShellMessage> {
        container(
            row![
                text(self.state.count),
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
            .align_y(Vertical::Center)
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
                _ => Task::none(),
            },
            ShellMessage::SetWorkspace(id) => {
                use hyprland::dispatch;
                use hyprland::dispatch::Dispatch;
                use hyprland::dispatch::DispatchType;
                let _ = dispatch!(Workspace, dispatch::WorkspaceIdentifierWithSpecial::Id(id));
                Task::none()
            }
            ShellMessage::ShortcutActivated(thing) => {
                match thing.as_str() {
                    "Hello" => {println!("Shortcut activated"); self.state.count += 1;},
                    _ => println!("Shouldn't happen! Shortcut ID {} is not handled!", thing) // TODO: Enums and hashmaps? We'll see!
                }
                Task::none()
            }
            ShellMessage::ShortcutError(error) => {
                dbg!(error);
                Task::none()
            }
            ShellMessage::ShortcutsSetup => {
                dbg!("Shortcuts Setup");
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<ShellMessage> {
        iced::Subscription::batch({
            let subscriptions = vec![
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
                            "Hello",
                            "This does a thing",
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
                                let _ = output.send(ShellMessage::ShortcutActivated(event.shortcut_id().to_string())).await;
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
        .run_with(Shell::new)
}
