use cosmic::cctk;
use cosmic::cctk::sctk::shell::wlr_layer::Anchor;
use cosmic::iced::alignment::Vertical;
use cosmic::iced::event::Status;
use cosmic::iced::event::wayland::LayerEvent;
use cosmic::iced::futures::{SinkExt, StreamExt};
use cosmic::iced::keyboard::Key;
use cosmic::iced::keyboard::key::Named;
use cosmic::iced::{self, Alignment, Color, Subscription, mouse};
use cosmic::iced_runtime::platform_specific::wayland::layer_surface::{
    IcedOutput, SctkLayerSurfaceSettings,
};
use cosmic::iced_runtime::{Appearance, default};
use cosmic::iced_widget::text_input::Style;
use cosmic::iced_widget::{column, row, text_input};
use cosmic::iced_winit::commands::layer_surface::destroy_layer_surface;
use cosmic::iced_winit::commands::subsurface::KeyboardInteractivity;
use cosmic::widget::Space;
use hyprland::prelude::*;
use hyprland::shared::HyprData;
use iced::border::radius;
use iced::platform_specific::shell::commands::layer_surface::get_layer_surface;
use iced::platform_specific::shell::commands::overlap_notify::overlap_notify;
use iced::widget::{container, text};
use iced::{Border, Element, Event, Length, Padding, Task, Theme, event, keyboard, time, window};

use chrono::Local;

#[derive(Debug)]
pub struct State {
    count: u32,
    active_workspace: i32,
    bar_id: window::Id,
    launcher_window: Option<window::Id>,
    launcher_input: String,
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
    ToggleLauncher,
    PrintThing(String),
    Escape,
    LauncherInput(String),
    LayerFocused(window::Id),
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

        (
            Self {
                state: State {
                    count: 0,
                    active_workspace: hyprland::data::Workspace::get_active()
                        .expect("Failed to get hyprland workspace")
                        .id,
                    bar_id: id,
                    launcher_window: None,
                    launcher_input: "".to_string(),
                },
            },
            Task::batch(vec![layer_shell_task, overlap_notify(id, true)]),
        )
    }

    pub fn view(&self, id: window::Id) -> Element<'_, ShellMessage> {
        if id == self.state.bar_id {
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
                    radius: radius(0),
                    ..Default::default()
                },
                background: Some(theme.extended_palette().background.weak.color.into()),
                ..Default::default()
            })
            .padding(Padding::from([0, 0]))
            .into()
        } else {
            row![
                Space::with_width(Length::FillPortion(1)),
                column(vec![
                    Space::with_height(Length::FillPortion(1)).into(),
                    text_input("", &self.state.launcher_input)
                        .on_input(ShellMessage::LauncherInput)
                        .id("launcher")
                        .size(30)
                        .style(|theme: &Theme, _status| Style {
                            background: theme.extended_palette().background.weak.color.into(),
                            border: Border {
                                color: theme.extended_palette().background.base.color.into(),
                                width: 0.0,
                                radius: radius(15)
                            },
                            icon: theme.extended_palette().primary.base.color.into(),
                            placeholder: theme.extended_palette().primary.weak.color.into(),
                            value: Color::BLACK,
                            selection: theme.extended_palette().secondary.base.color.into(),
                        })
                        .padding(15)
                        .into(),
                    if self.state.launcher_input.len() != 0 {
                        container(
                            iced::widget::column({
                                let mut items = vec![];
                                let show_count = std::cmp::min(self.state.launcher_input.len(), 5);
                                if show_count > 0 {
                                    items.push(
                                        container(
                                            iced::widget::column({
                                                let mut items = vec![];
                                                for i in 0..show_count {
                                                    let c = container(text!("Hello {i}").size(20))
                                                        .center_y(Length::Fill)
                                                        .padding(10)
                                                        .width(Length::Fill)
                                                        .style(|theme: &Theme| container::Style {
                                                            border: Border {
                                                                radius: radius(20),
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
                                                    items.push(c.into())
                                                }
                                                items
                                            })
                                            .spacing(5),
                                        )
                                        .style(|theme: &Theme| container::Style {
                                            background: Some(
                                                theme
                                                    .extended_palette()
                                                    .background
                                                    .strong
                                                    .color
                                                    .into(),
                                            ),
                                            border: Border {
                                                radius: radius(20),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })
                                        .height(Length::FillPortion(show_count as u16))
                                        .into(),
                                    )
                                }

                                if show_count < 5 {
                                    items.push(
                                        Space::with_height(Length::FillPortion(
                                            5 - show_count as u16,
                                        ))
                                        .into(),
                                    )
                                }
                                items
                            })
                            .width(Length::Fill),
                        )
                        .height(Length::FillPortion(1))
                        .width(Length::FillPortion(1))
                        .into()
                    } else {
                        Space::with_height(Length::FillPortion(1)).into()
                    },
                    Space::with_height(Length::FillPortion(1)).into()
                ])
                .spacing(5)
                .width(Length::FillPortion(1))
                .align_x(Alignment::Center),
                Space::with_width(Length::FillPortion(1))
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }
    }

    pub fn update(&mut self, message: ShellMessage) -> Task<ShellMessage> {
        match message {
            ShellMessage::PrintThing(thing) => {
                println!("{}", thing);
                Task::none()
            }
            ShellMessage::TimeTick(_) => Task::none(),
            ShellMessage::ButtonPressed => {
                self.state.count += 1;
                Task::none()
            }
            ShellMessage::OpenBlueman => {
                let _ = std::process::Command::new("hyprctl")
                    .arg("dispatch exec blueman-manager")
                    .spawn();
                return if let Some(_) = self.state.launcher_window {
                    Task::done(ShellMessage::ToggleLauncher)
                } else {
                    Task::none()
                };
            }
            ShellMessage::HyprlandError => Task::none(),
            ShellMessage::HyprlandEvent(event) => match event {
                hyprland::event_listener::Event::WorkspaceChanged(data) => {
                    self.state.active_workspace = data.id;
                    return if let Some(_) = self.state.launcher_window {
                        Task::done(ShellMessage::ToggleLauncher)
                    } else {
                        Task::none()
                    };
                }
                _ => Task::none(),
            },
            ShellMessage::SetWorkspace(id) => {
                use hyprland::dispatch;
                use hyprland::dispatch::Dispatch;
                use hyprland::dispatch::DispatchType;
                let _ = dispatch!(Workspace, dispatch::WorkspaceIdentifierWithSpecial::Id(id));
                return if let Some(_) = self.state.launcher_window {
                    Task::done(ShellMessage::ToggleLauncher)
                } else {
                    Task::none()
                };
            }
            ShellMessage::ShortcutActivated(thing) => {
                match thing.as_str() {
                    "ToggleLauncher" => {
                        self.state.count += 1;
                        // Toggle popup

                        return Task::done(ShellMessage::ToggleLauncher);
                    }
                    _ => println!("Shouldn't happen! Shortcut ID {} is not handled!", thing), // TODO: Enums and hashmaps? We'll see!
                }
                Task::none()
            }
            ShellMessage::ToggleLauncher => {
                match self.state.launcher_window {
                    Some(id) => {
                        // TODO: Cleanup and stuff
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
                            ..Default::default()
                        });

                        self.state.launcher_window = Some(id);
                        self.state.launcher_input = "".to_string();
                        layer_shell_task
                    }
                }
            }
            ShellMessage::ShortcutError(error) => {
                dbg!(error);
                Task::none()
            }
            ShellMessage::ShortcutsSetup => {
                dbg!("Shortcuts Setup");
                Task::none()
            }
            ShellMessage::Escape => match self.state.launcher_window {
                Some(_) => Task::done(ShellMessage::ToggleLauncher),
                None => Task::none(),
            },
            ShellMessage::LauncherInput(input) => {
                self.state.launcher_input = input;
                Task::none()
            }
            ShellMessage::LayerFocused(layer_id) => match self.state.launcher_window {
                Some(id) if id == layer_id => text_input::focus("launcher"),
                _ => Task::none(),
            },
        }
    }

    pub fn subscription(&self) -> iced::Subscription<ShellMessage> {
        iced::Subscription::batch({
            let subscriptions = vec![
                time::every(std::time::Duration::from_millis(100)).map(ShellMessage::TimeTick),
                event::listen_with(|event, status, _| match event {
                    Event::PlatformSpecific(event::PlatformSpecific::Wayland(
                        event::wayland::Event::Layer(LayerEvent::Focused, _, id),
                    )) => Some(ShellMessage::LayerFocused(id)),
                    Event::Keyboard(keyboard::Event::KeyReleased {
                        key: Key::Named(Named::Escape),
                        ..
                    }) => Some(ShellMessage::Escape),
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                        if status == Status::Ignored =>
                    {
                        Some(ShellMessage::Escape)
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
        .run_with(Shell::new)
}
