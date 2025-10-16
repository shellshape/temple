pub mod errors;
pub mod page;
pub mod parsing;

use chrono::Local;
use errors::{Error, Result};
use page::Page;
use parsing::{find_next_template, find_template, Template};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;

pub struct Builder {
    public_dir: PathBuf,
    pages_dir: PathBuf,
    templates_dir: PathBuf,

    target_dir: PathBuf,
}

impl Builder {
    pub fn new(source_dir: impl Into<PathBuf>, target_dir: impl Into<PathBuf>) -> Builder {
        let source_dir = source_dir.into();
        Builder {
            public_dir: source_dir.join("public"),
            pages_dir: source_dir.join("pages"),
            templates_dir: source_dir.join("templates"),
            target_dir: target_dir.into(),
        }
    }

    pub fn build(&self) -> Result<()> {
        if self.target_dir.exists() {
            fs::remove_dir_all(&self.target_dir)?;
        }

        dircpy::copy_dir(&self.public_dir, self.target_dir.join("public"))?;

        let mut entries =
            fs::read_dir(&self.pages_dir)?.collect::<std::result::Result<Vec<_>, _>>()?;
        entries.sort_by_key(|e| e.file_name());

        let pages = entries
            .iter()
            .map(|entry| Page::read(entry.path()))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        for page in &pages {
            log::debug!("Processing page '{}' ...", page.name);

            let parsedcontent = self.apply_template(&page.content, page, &pages)?;

            let outpath = page
                .config
                .as_ref()
                .and_then(|v| v.output.as_ref())
                .map(|v| self.target_dir.join(v))
                .unwrap_or_else(|| self.target_dir.join(&page.name).join("index.html"));

            let out_dir = outpath.parent().expect("parent dir");
            if !out_dir.exists() {
                fs::create_dir_all(out_dir)?;
            }

            let mut f = File::create(outpath)?;
            write!(f, "{parsedcontent}")?;
        }

        Ok(())
    }

    fn apply_template(&self, content: &str, page: &Page, pages: &[Page]) -> Result<String> {
        let mut content = content.trim().to_string();

        while let Some(t) = find_next_template(&content)? {
            content = match t.template {
                Template::Extends { name } => {
                    let mut content = t.remove_between(&content);
                    let template_contents = self.get_template_content(name)?;
                    let pagecontent_tpl = find_template(template_contents.trim(), "pagecontent")?
                        .ok_or(Error::ExtendWithNoPageContent)?;
                    content = pagecontent_tpl.insert_between(&template_contents, &content);
                    self.apply_template(&content, page, pages)?
                }
                Template::Use { name } => {
                    let template_contents =
                        self.apply_template(&self.get_template_content(name)?, page, pages)?;
                    t.insert_between(&content, &template_contents)
                }
                Template::PageName => t.insert_between(&content, &page.name),
                Template::NavItems => {
                    let mut navitems = Vec::with_capacity(pages.len());
                    for p in pages {
                        if p.config.as_ref().is_some_and(|c| c.navignore) {
                            continue;
                        }
                        let path = p
                            .config
                            .as_ref()
                            .and_then(|c| c.path.clone())
                            .unwrap_or_else(|| format!("/{}", p.name));
                        let active = if p.name == page.name { r#" class="active""# } else { "" };
                        let name = &p.name;
                        navitems.push(format!(r#"<a href="{path}"{active}>{name}</a>"#));
                    }
                    t.insert_between(&content, &navitems.join("\n"))
                }
                Template::CurrentDate { ref format } => {
                    let date = Local::now()
                        .format(
                            format
                                .as_ref()
                                .map(|v| v.as_str())
                                .unwrap_or_else(|| "%Y-%m-%d %H:%M:%S"),
                        )
                        .to_string();
                    t.insert_between(&content, &date)
                }
                Template::Exec { command, ref args } => {
                    let res = Command::new(command).args(args).output()?;
                    if !res.status.success() {
                        return Err(Error::ExecCommandFailed(
                            res.status,
                            String::from_utf8_lossy(&res.stderr).to_string(),
                        ));
                    }
                    t.insert_between(&content, &String::from_utf8_lossy(&res.stdout))
                }
                Template::PageContent => return Err(Error::ToplevelPageContent),
            };
        }

        Ok(content)
    }

    fn get_template_content(&self, name: &str) -> Result<String> {
        let mut content = String::new();
        File::open(self.templates_dir.join(format!("{name}.html")))?
            .read_to_string(&mut content)?;
        Ok(content.to_string())
    }
}
