use crate::error::AkaibuError;
use tlg_rs::formats::{tlg0::Tlg0, tlg6::Tlg6};

#[derive(Debug)]
pub enum ResourceMagic {
    TLG0,
    TLG5,
    TLG6,
    Unrecognized,
}

impl ResourceMagic {
    pub fn parse_magic(buf: &[u8]) -> Self {
        match buf {
            // TLG0.0\x00sds\x1a
            [84, 76, 71, 48, 46, 48, 0, 115, 100, 115, 26, ..] => Self::TLG0,
            // TLG5.0\x00raw\x1a
            [84, 76, 71, 53, 46, 48, 0, 114, 97, 119, 26, ..] => Self::TLG5,
            // TLG6.0\x00raw\x1a
            [84, 76, 71, 54, 46, 48, 0, 114, 97, 119, 26, ..] => Self::TLG6,
            _ => Self::Unrecognized,
        }
    }
    pub fn parse(&self, buf: Vec<u8>) -> anyhow::Result<ResourceType> {
        match self {
            Self::TLG0 => {
                let image = Tlg0::from_bytes(&buf)?.to_rgba_image()?;
                Ok(ResourceType::Image {
                    width: image.width(),
                    height: image.height(),
                    pixels: image.into_vec(),
                })
            }
            Self::TLG5 => Err(AkaibuError::Unimplemented.into()),
            Self::TLG6 => {
                let image = Tlg6::from_bytes(&buf)?.to_rgba_image()?;
                Ok(ResourceType::Image {
                    width: image.width(),
                    height: image.height(),
                    pixels: image.into_vec(),
                })
            }
            Self::Unrecognized => Err(AkaibuError::Unimplemented.into()),
        }
    }
}

#[derive(Debug)]
pub enum ResourceType {
    Image {
        pixels: Vec<u8>,
        width: u32,
        height: u32,
    },
    Text(String),
}
