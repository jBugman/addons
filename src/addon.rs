use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

const DEFAULT_DIR: &str = "/Applications/World of Warcraft/_retail_/Interface/Addons";

#[allow(dead_code)]
pub enum Dir<'a> {
    Custom(&'a Path),
    Default,
}

#[derive(Debug)]
pub(crate) struct Addon {
    pub name: String,
    path: PathBuf,
}

impl TryFrom<PathBuf> for Addon {
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        let name = path
            .file_name()
            .ok_or(Error::New("not a folder"))?
            .to_str()
            .ok_or(Error::New("not a valid utf8"))?
            .to_owned();
        Ok(Addon { name, path })
    }
}

pub(crate) fn list_installed(addon_dir: Dir) -> Result<Vec<Addon>> {
    let addon_dir = match addon_dir {
        Dir::Custom(path) => path,
        Dir::Default => Path::new(DEFAULT_DIR),
    };
    let dir_contents = fs::read_dir(addon_dir)?;
    dir_contents
        .filter_map(|r| r.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .map(Addon::try_from)
        .collect()
}
