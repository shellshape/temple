use super::errors::{Error, Result};
use serde::Deserialize;
use std::{fs::File, io::Read, path::Path};

#[derive(Debug)]
pub struct Page {
    pub name: String,
    pub content: String,
    pub config: Option<Config>,
}

impl Page {
    pub fn read(dir: impl AsRef<Path>) -> Result<Self> {
        let mut content = String::new();
        File::open(&dir)?.read_to_string(&mut content)?;

        let content = content.trim();

        let (config, content) = match content.strip_prefix("+++") {
            Some(content) => {
                let Some(end_index) = content.find("+++") else {
                    return Err(Error::UnclosedConfig);
                };
                (
                    Some(Config::parse(&content[..end_index])?),
                    &content[end_index + 3..],
                )
            }
            None => (None, content),
        };

        let name = config
            .as_ref()
            .and_then(|v| v.title.as_ref())
            .map(|v| v.to_string())
            .unwrap_or_else(|| page_name_from_path(dir.as_ref()));

        Ok(Page {
            name,
            content: content.to_string(),
            config,
        })
    }
}

fn page_name_from_path(dir: &Path) -> String {
    let filename = dir.file_name().expect("file name").to_string_lossy();

    let name = match filename.find('_') {
        Some(i) => &filename[i + 1..],
        None => &filename,
    };

    let name = match name.rfind('.') {
        Some(i) => &name[..i],
        None => name,
    };

    name.to_string()
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub title: Option<String>,
    pub path: Option<String>,
    pub output: Option<String>,
    #[serde(default)]
    pub navignore: bool,
}

impl Config {
    pub fn parse(content: impl AsRef<str>) -> Result<Self> {
        Ok(toml::from_str(content.as_ref())?)
    }
}
