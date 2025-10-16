use super::errors::{Error, Result};

#[derive(Debug, PartialEq, Eq)]
pub enum Template<'a> {
    Extends {
        name: &'a str,
    },
    Use {
        name: &'a str,
    },
    PageName,
    NavItems,
    CurrentDate {
        format: Option<String>,
    },
    Exec {
        command: &'a str,
        args: Vec<&'a str>,
    },
    PageContent,
}

impl Template<'_> {
    pub fn id(&self) -> &'static str {
        match &self {
            Self::Extends { name: _ } => "extends",
            Self::Use { name: _ } => "use",
            Self::PageName => "pagename",
            Self::NavItems => "navitems",
            Self::CurrentDate { format: _ } => "currentdate",
            Self::Exec {
                command: _,
                args: _,
            } => "exec",
            Self::PageContent => "pagecontent",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TemplateInstance<'a> {
    pub start_pos: usize,
    pub end_pos: usize,
    pub template: Template<'a>,
}

impl TemplateInstance<'_> {
    pub fn insert_between(&self, content: &str, insert: &str) -> String {
        content[..self.start_pos].to_string() + insert + &content[self.end_pos + 1..]
    }

    pub fn remove_between(&self, content: &str) -> String {
        content[..self.start_pos].to_string() + &content[self.end_pos + 1..]
    }
}

pub fn find_next_template(content: &str) -> Result<Option<TemplateInstance>> {
    let Some(start_pos) = content.find("{{") else {
        return Ok(None);
    };

    // TODO: This will cause problems when someone want to use }} in quoted template
    // args, which could be expected to work but will break here. Maybe this could
    // be fixed some time later, but currently this would make stuff too complex
    // for now.
    let Some(end_pos_rel) = content[start_pos..].find("}}") else {
        return Err(Error::UnclosedTemplate);
    };

    let end_pos = start_pos + end_pos_rel + 1;

    let template = parse_template(&content[start_pos + 2..end_pos - 1])?;

    Ok(Some(TemplateInstance {
        start_pos,
        end_pos,
        template,
    }))
}

pub fn find_template<'a>(
    content: &'a str,
    target_id: &'_ str,
) -> Result<Option<TemplateInstance<'a>>> {
    let mut offset = 0;

    while let Some(template) = find_next_template(&content[offset..])? {
        if template.template.id() == target_id {
            return Ok(Some(TemplateInstance {
                start_pos: template.start_pos + offset,
                end_pos: template.end_pos + offset,
                template: template.template,
            }));
        }
        offset += template.end_pos + 1;
    }

    Ok(None)
}

fn parse_template(content: &str) -> Result<Template> {
    let content = content.trim();
    if content.is_empty() {
        return Err(Error::Empty);
    }

    let mut split = vec![];
    let mut active_quote = None;
    let mut start = 0;

    for (i, c) in content.char_indices() {
        if let Some(quote_char) = active_quote {
            if quote_char == c {
                active_quote = None;
                split.push(&content[start..i]);
                start = i + 1;
            }
            continue;
        }

        match c {
            ' ' | '\t' | '\r' | '\n' => {
                if start != i {
                    split.push(&content[start..i]);
                }
                start = i + 1;
            }
            '"' | '\'' => {
                start = i + 1;
                active_quote = Some(c);
            }
            _ => {}
        }
    }

    if active_quote.is_some() {
        return Err(Error::UnclosedQuote);
    }

    let rest = &content[start..];
    if !rest.is_empty() {
        split.push(rest);
    }

    let mut split = split.into_iter();

    match split.next().expect("should not be empty") {
        "extends" => Ok(Template::Extends {
            name: split.next().ok_or(Error::MissingArgument("name"))?,
        }),
        "use" => Ok(Template::Use {
            name: split.next().ok_or(Error::MissingArgument("name"))?,
        }),
        "pagename" => Ok(Template::PageName),
        "navitems" => Ok(Template::NavItems),
        "currentdate" => Ok(Template::CurrentDate {
            format: split.next().map(|v| v.to_owned()),
        }),
        "exec" => {
            let command = split.next().ok_or(Error::MissingArgument("command"))?;
            let args = split.collect();
            Ok(Template::Exec { command, args })
        }
        "pagecontent" => Ok(Template::PageContent),
        name => Err(Error::UnknownTemplate(name.to_string())),
    }
}

#[cfg(test)]
mod test_find_next_template {
    use super::*;

    #[test]
    fn general() {
        assert!(matches!(
            find_next_template("some content {{extends foo}} more content"),
            Ok(Some(TemplateInstance {
                start_pos: 13,
                end_pos: 27,
                template: Template::Extends { name: "foo" }
            }))
        ));

        assert!(matches!(
            find_next_template("some content {{ extends foo }} more content"),
            Ok(Some(TemplateInstance {
                start_pos: 13,
                end_pos: 29,
                template: Template::Extends { name: "foo" }
            }))
        ));

        assert!(matches!(
            find_next_template("some content {{ extends 'foo bar' }} more content"),
            Ok(Some(TemplateInstance {
                start_pos: 13,
                end_pos: 35,
                template: Template::Extends { name: "foo bar" }
            }))
        ));

        assert!(matches!(
            find_next_template("some content more content"),
            Ok(None)
        ));

        assert!(matches!(
            find_next_template("some content {{ extends 'foo bar' more content"),
            Err(Error::UnclosedTemplate)
        ));

        assert!(matches!(
            find_next_template("some content {{}} more content"),
            Err(Error::Empty)
        ));

        assert!(matches!(
            find_next_template("some content {{  }} more content"),
            Err(Error::Empty)
        ));
    }
}

#[cfg(test)]
mod test_find_template {
    use super::*;

    #[test]
    fn general() {
        assert!(matches!(
            find_template("a {{ pagename }} b {{ pagecontent }} c", "pagecontent"),
            Ok(Some(TemplateInstance {
                start_pos: 19,
                end_pos: 35,
                template: Template::PageContent
            }))
        ));
    }
}

#[cfg(test)]
mod test_template_parse {
    use super::*;

    #[test]
    fn general() {
        assert!(matches!(
            parse_template("extends foo"),
            Ok(Template::Extends { name: "foo" })
        ));
        assert!(matches!(
            parse_template(" \textends   foo "),
            Ok(Template::Extends { name: "foo" })
        ));
        assert!(matches!(
            parse_template(r#"extends "foo bar""#),
            Ok(Template::Extends { name: "foo bar" })
        ));
        assert!(matches!(
            parse_template(r#""extends"  'foo "bar"'"#),
            Ok(Template::Extends {
                name: r#"foo "bar""#
            })
        ));
    }

    #[test]
    fn general_negative() {
        assert!(matches!(parse_template(""), Err(Error::Empty)));
        assert!(matches!(parse_template(" \t\n "), Err(Error::Empty)));
        assert!(matches!(
            parse_template("extends 'foo bar"),
            Err(Error::UnclosedQuote)
        ));
        assert!(matches!(
            parse_template("extends"),
            Err(Error::MissingArgument("name"))
        ));
        assert!(matches!(
            parse_template("thisdoesnotexist"),
            Err(Error::UnknownTemplate(v)) if &v == "thisdoesnotexist"
        ));
    }

    #[test]
    fn extends() {
        assert!(matches!(
            parse_template("extends foo"),
            Ok(Template::Extends { name: "foo" })
        ));

        assert!(matches!(
            parse_template("extends"),
            Err(Error::MissingArgument("name"))
        ));
    }

    #[test]
    fn r#use() {
        assert!(matches!(
            parse_template("use foo"),
            Ok(Template::Use { name: "foo" })
        ));

        assert!(matches!(
            parse_template("use"),
            Err(Error::MissingArgument("name"))
        ));
    }

    #[test]
    fn pagename() {
        assert!(matches!(parse_template("pagename"), Ok(Template::PageName)));
    }

    #[test]
    fn navitems() {
        assert!(matches!(parse_template("navitems"), Ok(Template::NavItems)));
    }

    #[test]
    fn currentdate() {
        assert!(matches!(
            parse_template("currentdate"),
            Ok(Template::CurrentDate { format: None })
        ));

        assert!(matches!(
            parse_template("currentdate 'some format'"),
            Ok(Template::CurrentDate { format: Some(f) }) if &f == "some format"
        ));
    }

    #[test]
    fn exec() {
        let res = parse_template("exec ls");
        match res.unwrap() {
            Template::Exec { command, args } => {
                assert_eq!(command, "ls");
                assert!(args.is_empty());
            }
            _ => panic!("invliad template"),
        }

        let res = parse_template(r#"exec do "some stuff"   'with "quotes"' yeah"#);
        match res.unwrap() {
            Template::Exec { command, args } => {
                assert_eq!(command, "do");
                assert_eq!(args, vec!["some stuff", r#"with "quotes""#, "yeah"]);
            }
            _ => panic!("invliad template"),
        }

        assert!(matches!(
            parse_template(" exec  "),
            Err(Error::MissingArgument("command"))
        ));
    }
}
