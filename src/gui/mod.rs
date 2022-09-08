use iced::alignment::{Horizontal, Vertical};
use iced::{
    button, container, text_input, window::Settings as Window, Alignment, Application, Background,
    Button, Color, Column, Command, Container, Element, Length, Padding, Point, Rectangle,
    Renderer, Row, Settings, Size, Text, TextInput,
};
use iced_native::Subscription;
use std::path::PathBuf;

mod cleaner;

pub struct RutabagaApplication {
    path_folder: PathBuf,
    path_folder_button_state: ButtonState,
    path_folder_input_state: text_input::State,

    start_button_state: ButtonState,
    stop_button_state: ButtonState,

    current_state: RutabagaState,
    progress: Progress,

    path_folder_for_process: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct ButtonState {
    state: button::State,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    PathInputChanged(String),
    SelectFolder,
    SelectedFolder(Option<PathBuf>),
    Clear(()),
    ProcessStart,
    ProcessCancel,
    Process(cleaner::Progress),
}

#[derive(Debug, Clone)]
enum RutabagaState {
    SelectFolder,
    Processed,
    Finished,
    Errored,
}

#[derive(Debug, Clone, Default)]
struct Progress {
    renamed: u64,
    cleared: u64,
    total: u64,
}

impl RutabagaApplication {
    pub fn start() -> iced::Result {
        let settings: Settings<()> = Settings {
            window: Window {
                size: (560, 210),
                resizable: false,
                decorations: true,
                // icon: Some(application_icon()),
                ..iced::window::Settings::default()
            },
            default_font: Some(include_bytes!("../../resources/fonts/Roboto-Regular.ttf")),
            default_text_size: 16,
            antialiasing: true,
            ..iced::Settings::default()
        };

        Self::run(settings)
    }

    fn change_enabled(&mut self) {
        if self.path_folder.as_os_str().is_empty() {
            self.path_folder_button_state.enabled = true;
            self.stop_button_state.enabled = false;
            self.start_button_state.enabled = false;
        } else {
            self.path_folder_button_state.enabled = true;
            self.stop_button_state.enabled = true;
            self.start_button_state.enabled = true;
        }

        match self.current_state {
            RutabagaState::SelectFolder => {}
            RutabagaState::Processed => {
                self.path_folder_button_state.enabled = false;
                self.stop_button_state.enabled = true;
                self.start_button_state.enabled = false;
            }
            RutabagaState::Finished => {
                self.path_folder_button_state.enabled = true;
                self.stop_button_state.enabled = true;
                self.start_button_state.enabled = true;
            }
            RutabagaState::Errored => {
                self.path_folder_button_state.enabled = true;
                self.stop_button_state.enabled = true;
                self.start_button_state.enabled = true;
            }
        }
    }
}

impl Application for RutabagaApplication {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                path_folder: Default::default(),
                path_folder_button_state: Default::default(),
                path_folder_input_state: Default::default(),
                start_button_state: Default::default(),
                stop_button_state: Default::default(),
                current_state: RutabagaState::SelectFolder,
                progress: Default::default(),
                path_folder_for_process: None,
            },
            Command::perform(async {}, Message::Clear),
        )
    }

    fn title(&self) -> String {
        "Rutabaga".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::PathInputChanged(val) => self.path_folder = PathBuf::from(val),
            Message::Clear(_) => {
                self.path_folder = Default::default();
                self.current_state = RutabagaState::SelectFolder;
                self.path_folder_for_process = None;
                self.change_enabled();
            }
            Message::SelectFolder => {
                return Command::perform(crate::core::select_folder(), Message::SelectedFolder)
            }
            Message::SelectedFolder(path) => {
                match path {
                    None => {}
                    Some(path) => self.path_folder = path,
                }
                self.change_enabled();
            }
            Message::ProcessStart => self.path_folder_for_process = Some(self.path_folder.clone()),
            Message::ProcessCancel => return Command::perform(async {}, Message::Clear),
            Message::Process(progress) => match progress {
                cleaner::Progress::Started => {
                    self.current_state = RutabagaState::Processed;
                    self.change_enabled();
                }
                cleaner::Progress::Advanced { .. } => {}
                cleaner::Progress::Finished => {
                    self.path_folder_for_process = None;
                    self.current_state = RutabagaState::Finished;
                    self.change_enabled();
                }
                cleaner::Progress::Errored => {
                    self.path_folder_for_process = None;
                    self.path_folder_for_process = None;
                    self.current_state = RutabagaState::Finished;
                }
            },
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        Column::new()
            .spacing(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([18, 24])
            .push(folder_path(
                self.path_folder.to_str().unwrap_or_default(),
                &mut self.path_folder_button_state,
                &mut self.path_folder_input_state,
            ))
            .push(Row::new().height(Length::Fill))
            .push(
                Row::new()
                    .spacing(16)
                    .align_items(Alignment::Center)
                    .push(state_indicator(&self.current_state))
                    .push(start_stop_button(
                        &mut self.start_button_state,
                        &mut self.stop_button_state,
                    )),
            )
            .push(line())
            .push(progress(
                self.progress.renamed,
                self.progress.cleared,
                self.progress.total,
            ))
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        match &self.path_folder_for_process {
            None => Subscription::none(),
            Some(path) => cleaner::clear_folder(path.clone()).map(Message::Process),
        }
    }
}

fn button<'a>(
    state: &'a mut button::State,
    label: &'a str,
    message: Message,
    enabled: bool,
) -> iced_native::widget::Button<'a, Message, Renderer> {
    let button = Button::new(
        state,
        Text::new(label)
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center),
    );

    match enabled {
        true => button.on_press(message),
        false => button,
    }
}

fn folder_path<'a>(
    path: &'a str,
    path_folder_button_state: &'a mut ButtonState,
    path_folder_input_state: &'a mut text_input::State,
) -> Row<'a, Message> {
    Row::new()
        .spacing(16)
        .push(
            TextInput::new(path_folder_input_state, "", path, Message::PathInputChanged)
                .width(Length::Fill)
                .padding(Padding::from([4, 8, 4, 8])),
        )
        .push(button(
            &mut path_folder_button_state.state,
            "Select folder",
            Message::SelectFolder,
            path_folder_button_state.enabled,
        ))
        .align_items(Alignment::Center)
}

fn state_indicator<'a>(state: &RutabagaState) -> iced_native::widget::text::Text<Renderer> {
    let (text, color) = match state {
        RutabagaState::SelectFolder => ("Please select a folder", Color::from_rgb8(38, 38, 38)),
        RutabagaState::Processed => ("In process...", Color::from_rgb8(229, 178, 72)),
        RutabagaState::Finished => ("Completed", Color::from_rgb8(93, 202, 107)),
        RutabagaState::Errored => ("Error", Color::from_rgb8(227, 72, 72)),
    };

    Text::new(text)
        .horizontal_alignment(Horizontal::Left)
        .vertical_alignment(Vertical::Center)
        .width(Length::Fill)
        .color(color)
}

fn start_stop_button<'a>(
    start_button_state: &'a mut ButtonState,
    stop_button_state: &'a mut ButtonState,
) -> Row<'a, Message> {
    Row::new()
        .spacing(8)
        .push(button(
            &mut stop_button_state.state,
            "Cancel",
            Message::ProcessCancel,
            stop_button_state.enabled,
        ))
        .push(button(
            &mut start_button_state.state,
            "Start",
            Message::ProcessStart,
            start_button_state.enabled,
        ))
}

fn progress<'a>(renamed: u64, cleared: u64, total: u64) -> Element<'a, Message> {
    Row::new()
        .spacing(8)
        .push(
            Text::new(format!("Renamed {renamed}/{total}"))
                .horizontal_alignment(Horizontal::Left)
                .vertical_alignment(Vertical::Center)
                .width(Length::Fill),
        )
        .push(
            Text::new(format!("Cleared {cleared}/{total}"))
                .horizontal_alignment(Horizontal::Right)
                .vertical_alignment(Vertical::Center)
                .width(Length::Fill),
        )
        .into()
}

fn line<'a>() -> Element<'a, Message> {
    struct LineStyle;
    impl container::StyleSheet for LineStyle {
        fn style(&self) -> container::Style {
            container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgb8(38, 38, 38))),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Default::default(),
            }
        }
    }

    Container::new(Row::new())
        .style(LineStyle {})
        .height(Length::Units(1))
        .width(Length::Fill)
        .into()
}
