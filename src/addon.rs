use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use semver_parser::parser::Error as SemverErr;
use semver_parser::version;

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
    toc: TOC,
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

        let toc = get_toc(&name, &path)?;

        Ok(Addon {
            name,
            dir: path,
            toc,
        })
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

impl fmt::Display for Addon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.toc.version {
            Some(v) => write!(f, "{} {}", self.name, v),
            None => write!(f, "{}", self.name),
        }
    }
}

fn get_toc(name: &str, dir: &Path) -> Result<TOC> {
    let filename = Path::new(name).with_extension("toc");
    let path = dir.join(filename);
    TOC::parse(&path).map_err(|e| {
        println!("TOC parsing error in: {:?}", path);
        e
    })
}

#[derive(Debug)]
pub(crate) struct TOC {
    version: Option<version::Version>,
    tags: HashMap<TagName, TagValue>,
}

impl TOC {
    fn parse(path: &Path) -> Result<TOC> {
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

        let version = tags.get("Version");
        let version = match version {
            None => None,
            Some(v) => Some(parse_version(v)?),
        };

        Ok(TOC { tags, version })
    }
}

fn parse_version(s: &str) -> Result<version::Version> {
    let version = version::parse(s);
    match version {
        Err(SemverErr::UnexpectedEnd) => {
            let s = format!("{}.0", s);
            parse_version(&s)
        }
        Err(SemverErr::UnexpectedToken(_)) => {
            let s = s.chars().skip(1).collect::<String>();
            parse_version(&s)
        }
        Err(e) => Err(e.into()),
        Ok(v) => Ok(v),
    }
}

type TagName = String;
type TagValue = String;

#[derive(Debug)]
struct Tag(TagName, TagValue);

impl Tag {
    const TAG_MARKER: &'static str = "##";

    pub(crate) fn from_line(line: &str) -> Option<Tag> {
        let line = line.trim().trim_matches('\u{feff}');
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
