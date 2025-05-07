use cosmic::iced::{self, Element, Task};

pub trait Window {
    type Message;

    fn new() -> (Self, Task<Self::Message>)
    where
        Self: Sized;
    fn view(self: &Self) -> Element<'_, Self::Message>;
    fn update(self: &mut Self, message: Self::Message) -> Task<Self::Message>;
    fn subscription(self: &Self) -> iced::Subscription<Self::Message>;
}