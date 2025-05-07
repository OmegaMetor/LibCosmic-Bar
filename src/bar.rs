use chrono::Local;
use cosmic::cctk;
use cosmic::cctk::sctk::shell::wlr_layer::Anchor;
use cosmic::iced::alignment::Vertical;
use cosmic::iced::{self, Border, Length, Padding, Subscription, Task, Theme, time, window};
use cosmic::iced_runtime::platform_specific::wayland::layer_surface::IcedOutput;
use cosmic::iced_widget::row;
use hyprland::dispatch::DispatchType;
use hyprland::dispatch::{Dispatch, WorkspaceIdentifierWithSpecial};
use hyprland::shared::HyprData;
use hyprland::{dispatch, prelude::*};
use iced::border::radius;
use iced::platform_specific::shell::commands::layer_surface::get_layer_surface;
use iced::widget::{container, text};

use crate::{ShellMessage, window::Window};

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

#[derive(Debug)]
pub struct Bar {
    pub count: u32,
    pub active_workspace: i32,
    pub id: window::Id,
}

#[derive(Debug, Clone)]
pub enum Message {
    ShellMessage(Box<ShellMessage>),
    TimeTick(iced::time::Instant),
    ButtonPressed,
    OpenBlueman,
    HyprlandEvent(hyprland::event_listener::Event),
    HyprlandError,
    SetWorkspace(WorkspaceIdentifier),
}

impl Window for Bar {
    type Message = Message;

    fn new() -> (Self, cosmic::Task<Self::Message>) {
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
                count: 0,
                active_workspace: hyprland::data::Workspace::get_active()
                    .expect("Failed to get hyprland workspace")
                    .id,
                id: id,
            },
            layer_shell_task,
        )
    }

    fn view(self: &Self) -> cosmic::iced::Element<'_, Self::Message> {
        container(
            row![
                text(self.count),
                iced::widget::row({
                    let mut workspaces = hyprland::data::Workspaces::get().unwrap().to_vec();

                    workspaces.sort_by_key(|workspace| workspace.id);
                    workspaces.into_iter().map(|workspace| {
                        let name = workspace.name;
                        iced::widget::button(text(name.clone()))
                            .on_press_maybe(if workspace.id != self.active_workspace {
                                Some(Message::SetWorkspace(WorkspaceIdentifier::Name(name)))
                            } else {
                                None
                            })
                            .into()
                    })
                }),
                text("Hello, World! I'm a bad status bar!")
                    .width(Length::Fill)
                    .center(),
                iced::widget::button("").on_press(Message::OpenBlueman),
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
    }

    fn update(self: &mut Self, message: Self::Message) -> cosmic::Task<Self::Message> {
        use Message::*;
        match message {
            TimeTick(_) => Task::none(),
            ButtonPressed => {
                self.count += 1;
                Task::none()
            }
            OpenBlueman => {
                let _ = dispatch!(Exec, "blueman-manager");
                Task::none()
            }
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
                Task::none()
            }
            HyprlandError => Task::none(),
            HyprlandEvent(event) => match event {
                hyprland::event_listener::Event::WorkspaceChanged(data) => {
                    self.active_workspace = data.id;
                    Task::none()
                }
                _ => Task::none(),
            },
            ShellMessage(_) => Task::none(),
        }
    }

    fn subscription(self: &Self) -> cosmic::iced::Subscription<Self::Message> {
        Subscription::batch([
            time::every(std::time::Duration::from_millis(100)).map(Message::TimeTick),
            Subscription::run(|| hyprland::event_listener::EventStream::new()).map(|hyprevent| {
                match hyprevent {
                    Ok(result) => Message::HyprlandEvent(result),
                    Err(_) => Message::HyprlandError,
                }
            }),
        ])
    }
}
