use crate::scheme::{self, Scheme};

#[derive(Debug)]
pub enum Archive {
    ACV1,
    CPZ7,
    GXP,
    PF8,
    NotRecognized,
}

impl Archive {
    /// Parse first few bytes of file to detect archive type
    pub fn parse(buf: &[u8]) -> Self {
        match buf {
            // ACV1
            [0x41, 0x43, 0x56, 0x31, ..] => Self::ACV1,
            // CPZ7
            [0x43, 0x50, 0x5A, 0x37, ..] => Self::CPZ7,
            // GXP
            [0x47, 0x58, 0x50, 0x00, ..] => Self::GXP,
            // pf8
            [0x70, 0x66, 0x38, ..] => Self::PF8,
            _ => Self::NotRecognized,
        }
    }
    /// Is archive extraction scheme not game dependent
    pub fn is_universal(&self) -> bool {
        match self {
            Self::ACV1 => false,
            Self::CPZ7 => false,
            Self::GXP => true,
            Self::PF8 => true,
            Self::NotRecognized => false,
        }
    }
    /// Get list of all schemes for given archive type
    pub fn get_schemes(&self) -> Vec<Box<dyn Scheme>> {
        match self {
            Self::ACV1 => scheme::acv1::Acv1Scheme::get_schemes(),
            Self::CPZ7 => scheme::cpz7::Cpz7Scheme::get_schemes(),
            Self::GXP => scheme::gxp::GxpScheme::get_schemes(),
            Self::PF8 => scheme::pf8::Pf8Scheme::get_schemes(),
            Self::NotRecognized => vec![],
        }
    }
}