use std::sync::Arc;

use akaibu::{
    archive::Archive, archive::FileEntry, resource::ResourceMagic,
    resource::ResourceType,
};
use anyhow::Context;

pub async fn get_resource_type(
    archive: Arc<Box<dyn Archive>>,
    entry: FileEntry,
) -> anyhow::Result<ResourceType> {
    let contents = archive.extract(&entry)?;
    let resource_magic = ResourceMagic::parse_magic(&contents);
    let resource = resource_magic
        .get_schemes()
        .get(0)
        .context("Expected universal scheme")?
        .convert_from_bytes(&entry.full_path, contents.to_vec())?;

    Ok(match resource {
        ResourceType::Other => PreviewableResourceMagic::parse(&contents)?,
        _ => resource,
    })
}

#[derive(Debug)]
enum PreviewableResourceMagic {
    PNG,
    JPG,
    BMP,
    ICO,
    RIFF,
    Unrecognized,
}

impl PreviewableResourceMagic {
    pub fn parse_magic(buf: &[u8]) -> Self {
        match buf {
            [137, 80, 78, 71, 13, 10, 26, 10, ..]
            | [135, 80, 78, 71, 13, 10, 26, 10, ..] => Self::PNG,
            [255, 216, 255, ..] => Self::JPG,
            [66, 77, ..] => Self::BMP,
            [0, 0, 1, 0, ..] => Self::ICO,
            [82, 73, 70, 70, ..] => Self::RIFF,
            _ => Self::Unrecognized,
        }
    }
    fn parse(buf: &[u8]) -> anyhow::Result<ResourceType> {
        use self::PreviewableResourceMagic::*;
        Ok(match Self::parse_magic(buf) {
            PNG | JPG | BMP | ICO | RIFF => ResourceType::RgbaImage {
                image: image::load_from_memory(buf)?.to_rgba8(),
            },
            Unrecognized => ResourceType::Other,
        })
    }
}
