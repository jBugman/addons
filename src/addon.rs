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

        let toc_filename = Path::new(&name).with_extension("toc");
        let toc_path = path.join(toc_filename);
        let toc = TOC::parse(&toc_path).map_err(|e| {
            println!("TOC parsing error in: {:?}", toc_path);
            e
        })?;

        Ok(Addon {
            name,
            dir: path,
            toc,
        })
    }
}

impl Addon {
    pub fn description(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("{}", self));
        let notes = self.toc.tags.get("Notes");
        if let Some(notes) = notes {
            lines.push(notes.to_string());
        }
        let author = self.toc.tags.get("Author");
        if let Some(author) = author {
            lines.push(format!("Author: {}", author));
        }
        lines.push(format!("Path: {:?}", self.dir));
        let deps = self.toc.dependencies();
        if !deps.is_empty() {
            lines.push(format!("Dependencies: {}", deps.join(", ")));
        }
        lines.join("\n")
    }
}

fn list_addon_folders(addons_dir: Dir) -> Result<impl Iterator<Item = PathBuf>> {
    let addons_dir = match addons_dir {
        Dir::Custom(path) => path,
        Dir::Default => Path::new(DEFAULT_DIR),
    };
    let dir_contents = fs::read_dir(addons_dir)?;
    let addon_folders = dir_contents
        .filter_map(|r| r.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir());
    Ok(addon_folders)
}

pub fn list_installed(addons_dir: Dir) -> Result<Vec<Addon>> {
    let addon_folders = list_addon_folders(addons_dir)?;
    addon_folders.map(Addon::try_from).collect()
}

pub fn by_name(addons_dir: Dir, name: impl AsRef<str>) -> Result<Addon> {
    let mut addon_folders = list_addon_folders(addons_dir)?;
    let name = name.as_ref().to_lowercase();
    let addon_folder =
        addon_folders.find(|f| f.file_name().unwrap().to_string_lossy().to_lowercase() == name);
    addon_folder
        .ok_or(Error::NotFound)
        .and_then(Addon::try_from)
}

impl fmt::Display for Addon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.toc.version {
            Some(v) => write!(f, "{} {}", self.name, v),
            None => write!(f, "{}", self.name),
        }
    }
}

#[derive(Debug)]
pub(crate) struct TOC {
    version: Option<version::Version>,
    tags: HashMap<TagName, TagValue>,
}

impl TOC {
    fn parse(path: impl AsRef<Path>) -> Result<TOC> {
        use std::io::BufRead;

        let file = fs::File::open(path)?;
        let file = std::io::BufReader::new(file);
        let mut tags = HashMap::new();
        for line in file.lines() {
            let line = line?;
            let tag = Tag::from_line(&line);
            if let Some(tag) = tag {
                let Tag(tag, value) = tag;
                tags.insert(tag, value);
            }
        }

        let version = tags.get("Version");
        let version = match version {
            None => None,
            Some(v) => Some(parse_version(v)?),
        };

        Ok(TOC { tags, version })
    }

    pub fn dependencies(&self) -> Vec<String> {
        let deps = self.tags.get("Dependencies");
        deps.map_or_else(Vec::new, |deps| {
            deps.split(", ").map(|s| s.trim().to_string()).collect()
        })
    }
}

fn parse_version(s: impl AsRef<str>) -> Result<version::Version> {
    let s = s.as_ref();
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

    pub fn from_line(line: impl AsRef<str>) -> Option<Tag> {
        let line = line.as_ref().trim().trim_matches('\u{feff}');
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
