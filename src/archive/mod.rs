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