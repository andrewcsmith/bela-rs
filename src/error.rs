use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    Init,
    Start,
    Stop,
    Cleanup,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Error: {:?}.", self)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::Init => "Bela_initAudio error",
            Error::Start => "Bela_startAudio error",
            Error::Stop => "Bela_stopAudio error",
            Error::Cleanup => "Bela_cleanupAudio error",
        }
    }
}
