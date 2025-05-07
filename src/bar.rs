use cosmic::{iced::Subscription, Task};

use crate::{window_trait::Window, ShellMessage};

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
pub struct Bar {}

#[derive(Debug, Clone)]
pub enum Message {
    ShellMessage(Box<ShellMessage>),
    B
}

impl Window for Bar {
    type Message = Message;

    fn new() -> (Self, cosmic::Task<Self::Message>) {
        todo!()
    }

    fn view(self: &Self) -> cosmic::iced::Element<'_, Self::Message> {
        todo!()
    }

    fn update(self: &mut Self, message: Self::Message) -> cosmic::Task<Self::Message> {
        match message {
            _ => Task::none()
        }
    }

    fn subscription(self: &Self) -> cosmic::iced::Subscription<Self::Message> {
        Subscription::none()
    }
}
