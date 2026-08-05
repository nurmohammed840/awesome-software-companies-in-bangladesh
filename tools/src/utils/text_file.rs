use crate::Result;
use std::{
    fs::{self, OpenOptions},
    io::{self, Read},
    path::PathBuf,
};

pub struct TextFile {
    pub text: String,
    pub path: PathBuf,
}

impl TextFile {
    pub fn read(path: PathBuf) -> Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        let mut text = String::new();
        file.read_to_string(&mut text)?;

        Ok(Self { text, path })
    }

    pub fn write(&self, c: impl AsRef<[u8]>) -> io::Result<()> {
        let old = self.text.as_bytes();
        let new = c.as_ref();

        if old == new {
            return Ok(());
        }

        fs::write(&self.path, new)
    }
}
