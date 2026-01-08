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