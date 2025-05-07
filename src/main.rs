use bar::Bar;
use cosmic::iced::futures::{SinkExt, StreamExt};
use cosmic::iced::{self, Color, Subscription};
use cosmic::iced_runtime::{Appearance, default};
use cosmic::widget::Space;
use iced::window::Id;
use iced::{Element, Task};
use launcher::Launcher;
use window::Window;

mod bar;
mod launcher;
mod window;

pub struct Shell {
    launcher: launcher::Launcher,
    bar: bar::Bar,
}

// Messages are how your logic mutates the app state and GUI
#[derive(Debug, Clone)]
pub enum ShellMessage {
    ShortcutError(String),
    ShortcutActivated(String),
    ShortcutsSetup,
    LauncherMessage(launcher::Message),
    BarMessage(bar::Message),
}

impl Shell {
    pub fn new() -> (Self, Task<ShellMessage>) {
        let (launcher_window, launcher_init_task) = Launcher::new();
        let (bar_window, bar_init_task) = Bar::new();

        (
            Self {
                bar: bar_window,
                launcher: launcher_window,
            },
            Task::batch(vec![
                launcher_init_task.map(|e| ShellMessage::LauncherMessage(e)),
                bar_init_task.map(|e| ShellMessage::BarMessage(e)),
            ]),
        )
    }

    pub fn view(&self, id: Id) -> Element<'_, ShellMessage> {
        if self
            .launcher
            .window
            .is_some_and(|window_id| window_id == id)
        {
            return self
                .launcher
                .view()
                .map(|e| ShellMessage::LauncherMessage(e));
        }
        if id == self.bar.id {
            return self.bar.view().map(|e| ShellMessage::BarMessage(e));
        } else {
            Space::new(0, 0).into()
        }
    }

    pub fn update(&mut self, message: ShellMessage) -> Task<ShellMessage> {
        use ShellMessage::*;
        match message {
            ShortcutActivated(thing) => {
                match thing.as_str() {
                    "ToggleLauncher" => {
                        return Task::done(LauncherMessage(launcher::Message::Open));
                    }
                    _ => println!("Shouldn't happen! Shortcut ID {} is not handled!", thing), // TODO: Enums and hashmaps? We'll see!
                }
                Task::none()
            }
            ShortcutError(error) => {
                dbg!(error);
                Task::none()
            }
            ShortcutsSetup => {
                dbg!("Shortcuts Setup");
                Task::none()
            }
            LauncherMessage(message) => {
                if let launcher::Message::ShellMessage(shell_message) = message {
                    self.update(dbg!(*shell_message.clone()))
                } else {
                    self
                        .launcher
                        .update(message)
                        .map(|e| ShellMessage::LauncherMessage(e))
                }
            }
            BarMessage(message) => {
                if let bar::Message::ShellMessage(shell_message) = message {
                    self.update(dbg!(*shell_message.clone()))
                } else {
                    self
                        .bar
                        .update(message)
                        .map(|e| ShellMessage::BarMessage(e))
                }
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<ShellMessage> {
        iced::Subscription::batch(vec![
            self
                .launcher
                .subscription()
                .map(|message| ShellMessage::LauncherMessage(message)),
            self
                .bar
                .subscription()
                .map(|message| ShellMessage::BarMessage(message)),
                
            Subscription::run(|| {
                iced::stream::channel(10, async |mut output| {
                    let proxy = match ashpd::desktop::global_shortcuts::GlobalShortcuts::new().await
                    {
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
        ])
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
