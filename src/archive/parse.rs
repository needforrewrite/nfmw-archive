use std::fmt::Display;

use crate::archive::GenericArchiveFileInfo;

#[derive(Debug)]
pub enum ArchiveFileParseError {
    /// (Field Name, The Character)
    InvalidCharactersInField(String, String),
    /// (Field Name, Max Length)
    FieldValueTooLong(String, i32),
    /// (Reason)
    InvalidMusicPath(String),
    /// (Field Name) - not all fields are required
    MissingField(String),
    Other(String),
}
impl Display for ArchiveFileParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other(s) => write!(f, "Generic parse error: {s}"),
            _ => write!(f, "Generic parse error"),
        }
    }
}
impl std::error::Error for ArchiveFileParseError {}

/// (Attribute, attribute value)
/// The line must be lowercased entering this function
pub fn parse_line(line: String) -> Result<Option<(String, Vec<String>)>, ArchiveFileParseError> {
    // Not all lines follow the formatting this file looks for, and that's intended.
    // This function only errors if it finds a line that *should* follow this format,
    // but isn't.

    // First, determine if this is even a line that we want to parse. The format is
    // attribute(param1,param2,...)

    if !('a'..='z').any(|s| line.starts_with(s)) {
        return Ok(None);
    } else if !line.contains(')') || !line.contains('(') {
        return Ok(None);
    }

    let close_bracket_idx = line.chars().position(|c| c == ')').unwrap();
    let open_bracket_idx = line.chars().position(|c| c == '(').unwrap();

    if close_bracket_idx < open_bracket_idx {
        return Ok(None);
    } else if line.chars().filter(|c| c == &'(' || c == &')').count() > 2 {
        return Ok(None);
    }

    let attr = line[0..open_bracket_idx].trim().to_owned();

    let parts_raw = line[open_bracket_idx + 1..close_bracket_idx].to_owned();
    let parts = parts_raw
        .split(",")
        .map(|x| x.to_owned())
        .collect::<Vec<_>>();

    Ok(Some((attr, parts)))
}

pub fn parse_file(content: String, expect_soundtrack: bool) -> Result<GenericArchiveFileInfo, ArchiveFileParseError> {
    let lines = content.split("\n").map(|x| x.to_ascii_lowercase());
    let mut name: Option<String> = None;
    let mut author: Option<String> = None;
    let mut description: Option<String> = None;
    let mut soundtrack: Option<(String, String)> = None;
    let mut tags: Vec<String> = Vec::new();

    for line in lines {
        let parsed = parse_line(line)?;
        if let Some(p) = parsed {
            if p.0 == "name" {
                name = Some(p.1.join(","));
            } else if p.0 == "author" || p.0 == "carmaker" || p.0 == "stagemaker" {
                author = Some(p.1.join(","))
            } else if p.0 == "tag" {
                tags.push(p.1.join(","));
            } else if p.0 == "desc" || p.0 == "description" {
                description = Some(p.1.join(","));
            } else if p.0 == "soundtrack" && expect_soundtrack == true {
                if p.1.len() != 2 {
                    return Err(ArchiveFileParseError::InvalidMusicPath("Must have one folder and one filename.".to_owned()))
                }
                soundtrack = Some((p.1[0].clone(), p.1[1].clone()));
            }
        }
    }

    if name.is_none() {
        return Err(ArchiveFileParseError::MissingField("Name".to_string()));
    }

    Ok(GenericArchiveFileInfo {
        name: name.unwrap(),
        author,
        soundtrack,
        description,
        tags,
    })
}
