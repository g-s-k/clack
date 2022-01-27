use crate::GainPluginMainThread;
use clack_extensions::{
    gui::attached::implementation::PluginAttachedGui, gui::attached::window::AttachableWindow,
    gui::UiSize,
};
use clack_plugin::plugin::PluginError;

use std::ffi::CStr;

use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use iced_baseview::*;

pub type GuiWindow = WindowHandle<Message>;

impl PluginAttachedGui for GainPluginMainThread {
    fn attach(
        &mut self,
        parent: AttachableWindow,
        display_name: Option<&CStr>,
    ) -> Result<(), PluginError> {
        let title = display_name
            .map(|t| t.to_string_lossy())
            .unwrap_or_else(|| "Some default title I dunno".into());

        let settings = Settings {
            window: WindowOpenOptions {
                title: title.into_owned(),
                size: Size::new(300.0, 300.0),
                scale: WindowScalePolicy::SystemScaleFactor,
            },
            flags: (),
        };

        let window = IcedWindow::<MyProgram>::open_parented(&parent, settings);

        self.open_window = Some(window);

        Ok(())
    }
}

impl clack_extensions::gui::implementation::PluginGui for GainPluginMainThread {
    fn create(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn destroy(&mut self) {
        if let Some(mut window) = self.open_window.take() {
            window.close_window()
        }
    }

    fn get_size(&mut self) -> Result<UiSize, PluginError> {
        Ok(UiSize {
            width: 300,
            height: 300,
        })
    }

    fn can_resize(&mut self) -> bool {
        false
    }

    fn set_size(&mut self, _size: UiSize) -> bool {
        false
    }

    fn show(&mut self) {}

    fn hide(&mut self) {}
}

#[derive(Debug, Clone)]
pub enum Message {
    SliderChanged(u32),
}

struct MyProgram {
    slider_state: slider::State,
    slider_value: u32,
    slider_value_str: String,
}

impl Application for MyProgram {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                slider_state: slider::State::new(),
                slider_value: 0,
                slider_value_str: String::from("0"),
            },
            Command::none(),
        )
    }

    fn update(
        &mut self,
        _window: &mut WindowQueue,
        message: Self::Message,
    ) -> Command<Self::Message> {
        match message {
            Message::SliderChanged(value) => {
                self.slider_value = value;
                self.slider_value_str = format!("{}", value);
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let slider_widget = Slider::new(
            &mut self.slider_state,
            0..=1000,
            self.slider_value,
            Message::SliderChanged,
        );

        let content = Column::new()
            .width(Length::Fill)
            .align_items(Align::Center)
            .padding(20)
            .spacing(20)
            .push(Text::new("Slide me!"))
            .push(slider_widget)
            .push(Text::new(self.slider_value_str.as_str()));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
