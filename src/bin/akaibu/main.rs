use akaibu::{
    error::AkaibuError,
    magic::Archive,
    resource::{ResourceMagic, ResourceType},
    scheme::Scheme,
};
use anyhow::Context;
use image::ImageBuffer;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::io::{Read, Write};
use std::{fs::File, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Files to process
    #[structopt(required = true, name = "ARCHIVES", parse(from_os_str))]
    files: Vec<PathBuf>,

    /// Directory to output extracted files
    #[structopt(
        short = "o",
        long = "output",
        parse(from_os_str),
        default_value = "ext/"
    )]
    output_dir: PathBuf,

    /// Convert resource files to commonly used formats
    #[structopt(short, long)]
    convert: bool,
}

fn main() {
    env_logger::init();
    let opt = Opt::from_args();

    match run(&opt) {
        Ok(_) => (),
        Err(err) => log::error!("Error while extracting: {}", err),
    }
}

fn run(opt: &Opt) -> anyhow::Result<()> {
    opt.files
        .iter()
        .filter(|file| file.is_file())
        .try_for_each(|file| {
            let mut magic = vec![0; 32];
            File::open(&file)?.read_exact(&mut magic)?;

            let archive_magic = Archive::parse(&magic);
            if let Archive::NotRecognized = archive_magic {
                if opt.convert {
                    let resource_magic = ResourceMagic::parse_magic(&magic);
                    let mut contents = Vec::with_capacity(1 << 20);
                    File::open(&file)?.read_to_end(&mut contents)?;
                    return write_resource(
                        resource_magic.parse(contents)?,
                        file,
                    );
                } else {
                    return Err(AkaibuError::UnrecognizedFormat(
                        file.clone(),
                        magic,
                    )
                    .into());
                }
            }

            log::debug!("Archive: {:?}", archive_magic);
            let schemes = archive_magic.get_schemes();
            let scheme = if archive_magic.is_universal() {
                schemes.get(0).context("Scheme list is empty")?
            } else {
                schemes
                    .get(prompt_for_game(&schemes, &file))
                    .context("Could no get scheme from scheme list")?
            };
            log::debug!("Scheme {:?}", scheme);

            let a = scheme.extract(&file)?;
            let progress_bar = init_progressbar(
                &format!("Extracting: {:?}", file),
                a.get_files().len() as u64,
            );

            a.get_files()
                .par_iter()
                .progress_with(progress_bar)
                .try_for_each(|f| {
                    let buf = a.extract(f.file_name)?;
                    let mut output_file_name = PathBuf::from(&opt.output_dir);
                    output_file_name.push(&f.file_name);
                    std::fs::create_dir_all(
                        &output_file_name
                            .parent()
                            .context("Could not get parent directory")?,
                    )?;
                    log::debug!(
                        "Extracting resource: {:?} {:X?}",
                        output_file_name,
                        f
                    );
                    File::create(output_file_name)?.write_all(&buf)?;
                    Ok(())
                })
        })
}

fn prompt_for_game(schemes: &[Box<dyn Scheme>], file_name: &PathBuf) -> usize {
    use colored::*;
    use read_input::prelude::*;

    let msg = schemes
        .iter()
        .enumerate()
        .map(|s| format!(" {}: {}\n", s.0, s.1.get_name()))
        .fold(
            format!("{:?}\nSelect game by typing number:\n", file_name),
            |mut v, s| {
                v += &s;
                v
            },
        );
    input::<usize>()
        .repeat_msg(msg)
        .err("Invalid input value".red())
        .inside_err(
            0..schemes.len(),
            format!("Please input value from 0 to {}", schemes.len() - 1).red(),
        )
        .get()
}

fn init_progressbar(prefix: &str, size: u64) -> ProgressBar {
    let progress_bar = ProgressBar::new(size).with_style(
        ProgressStyle::default_bar().template(
            " {spinner} {prefix} {wide_bar:} {pos:>6}/{len:6} ETA:[{eta}]",
        ),
    );
    progress_bar.set_prefix(prefix);
    progress_bar
}

fn write_resource(
    resource: ResourceType,
    file_name: &PathBuf,
) -> anyhow::Result<()> {
    match resource {
        ResourceType::Image {
            pixels,
            width,
            height,
        } => {
            let mut new_file_name = file_name.clone();
            new_file_name.set_extension("png");
            let image: ImageBuffer<image::Rgba<u8>, Vec<u8>> =
                ImageBuffer::from_raw(width, height, pixels)
                    .context("Could not create image")?;
            image.save(new_file_name)?;
            Ok(())
        }
        ResourceType::Text(s) => {
            let mut new_file_name = file_name.clone();
            new_file_name.set_extension("png");
            File::create(new_file_name)?.write_all(s.as_bytes())?;
            Ok(())
        }
    }
}
