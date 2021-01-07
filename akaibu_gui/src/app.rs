use crate::{
    message::Message,
    ui::{
        archive::ArchiveContent, content::Content, resource::ResourceContent,
        scheme::SchemeContent,
    },
    update, Opt,
};
use akaibu::{magic, resource::ResourceMagic};
use iced::{executor, Application, Command};
use std::{fs::File, io::Read};
use structopt::StructOpt;

pub(crate) struct App {
    pub(crate) opt: Opt,
    pub(crate) content: Content,
}

impl Application for App {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let opt = Opt::from_args();

        let mut magic = vec![0; 32];
        File::open(&opt.file)
            .expect("Could not open file")
            .read_exact(&mut magic)
            .expect("Could not read file");
        let archive = magic::Archive::parse(&magic);

        if let magic::Archive::NotRecognized = archive {
            let resource = ResourceMagic::parse_magic(&magic);
            if let ResourceMagic::Unrecognized = resource {
                return (
                Self {
                    opt,
                    content: Content::SchemeView(SchemeContent::new(
                        magic::Archive::get_all_schemes(),
                        "Archive type could not be guessed. Please enter scheme manually:"
                            .to_string(),
                    )),
                },
                Command::none(),
            );
            } else {
                let file_name = opt.file.clone();
                let resource = resource
                    .parse_from_filename(&file_name)
                    .expect("Could not parse resource");
                return (
                    Self {
                        opt,
                        content: Content::ResourceView(ResourceContent::new(
                            resource, file_name,
                        )),
                    },
                    Command::none(),
                );
            }
        }

        let schemes = archive.get_schemes();

        if archive.is_universal() {
            let scheme = schemes.get(0).expect("Expected universal scheme");
            let (archive, dir) =
                scheme.extract(&opt.file).expect("Could not extract");
            (
                Self {
                    opt,
                    content: Content::ArchiveView(Box::new(
                        ArchiveContent::new(archive, dir),
                    )),
                },
                Command::none(),
            )
        } else {
            (
                Self {
                    opt,
                    content: Content::SchemeView(SchemeContent::new(
                        schemes,
                        "Select extract scheme:".to_string(),
                    )),
                },
                Command::none(),
            )
        }
    }
    fn title(&self) -> String {
        format!("Akaibu {}", env!("CARGO_PKG_VERSION"))
    }
    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match update::handle_message(self, message) {
            Ok(command) => command,
            Err(err) => {
                log::error!("{:?}", err);
                Command::perform(async move { err.to_string() }, Message::Error)
            }
        }
    }
    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        self.content.view()
    }
}
