use std::collections::HashMap;
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
pub struct Addon {
    pub name: String,
    dir: PathBuf,
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
        Ok(Addon { name, dir: path })
    }
}

pub fn list_installed(addon_dir: Dir) -> Result<Vec<Addon>> {
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

impl Addon {
    pub(crate) fn get_toc(&self) -> Result<TOC> {
        let filename = Path::new(&self.name).with_extension("toc");
        let path = self.dir.join(filename);
        TOC::parse(path)
    }
}

#[derive(Debug)]
pub(crate) struct TOC {
    interface: u32,
    version: Version,
    tags: HashMap<TagName, TagValue>,
}

type Version = String;

impl TOC {
    fn parse<P: AsRef<Path>>(path: P) -> Result<TOC> {
        use std::io::BufRead;

        let file = fs::File::open(path)?;
        let file = std::io::BufReader::new(file);
        let mut tags = HashMap::new();
        for line in file.lines() {
            let line = line?;
            let tag = Tag::from_line(&line);
            if tag.is_none() {
                break;
            }
            let Tag(tag, value) = tag.unwrap();
            tags.insert(tag, value);
        }

        Ok(TOC {
            tags,
            interface: 0,
            version: String::from(""),
        })
    }
}

type TagName = String;
type TagValue = String;

#[derive(Debug)]
struct Tag(TagName, TagValue);

impl Tag {
    const TAG_MARKER: &'static str = "##";

    pub(crate) fn from_line(line: &str) -> Option<Tag> {
        let line = line.trim();
        if !line.starts_with(Self::TAG_MARKER) {
            return None;
        }
        let line = line.trim_start_matches(Self::TAG_MARKER).trim_start();
        let mut parts = line.splitn(2, ": ");
        let name = parts.next()?.to_owned();
        let value = parts.next()?.to_owned();
        Some(Tag(name, value))
    }
}
