use std::io;

pub mod index;
pub mod parse;

/// Field info that may be relevant to all archive entries.
pub struct GenericArchiveFileInfo {
    pub name: String,
    pub author: Option<String>,
    /// (Folder, Filename)
    /// Always defined for stages; never for anything else.
    pub soundtrack: Option<(String, String)>,
    pub description: Option<String>,
    pub tags: Vec<String>
}

#[derive(serde::Deserialize, PartialEq, Eq)]
pub enum ArchiveItemType {
    Car = 0,
    Stage = 1,
    StagePiece = 2,
    Wheel = 3
}
impl ToString for ArchiveItemType {
    fn to_string(&self) -> String {
        match self {
            ArchiveItemType::Car => "car".to_owned(),
            ArchiveItemType::Stage => "stage".to_owned(),
            ArchiveItemType::StagePiece => "stage_piece".to_owned(),
            ArchiveItemType::Wheel => "wheel".to_owned()
        }
    }
}
impl ArchiveItemType {
    pub fn dir_name(&self) -> String {
        format!("{}s", self.to_string())
    }
}
impl TryFrom<&str> for ArchiveItemType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "car" => Ok(ArchiveItemType::Car),
            "stage" => Ok(ArchiveItemType::Stage),
            "stage_piece" => Ok(ArchiveItemType::StagePiece),
            "wheel" => Ok(ArchiveItemType::Wheel),
            _ => Err(())
        }
    }
}

pub fn ensure_default_dirs_exist(path: &str) -> io::Result<()>
{
    const TYPES: [ArchiveItemType; 4] = [
        ArchiveItemType::Car,
        ArchiveItemType::Stage,
        ArchiveItemType::StagePiece,
        ArchiveItemType::Wheel
    ];

    for t in TYPES {
        let name = t.dir_name();
        let fullpath = format!("{path}/{name}");

        if !std::fs::exists(&fullpath)? {
            std::fs::create_dir(&fullpath)?;
        }
    }

    Ok(())
}