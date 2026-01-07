use std::fmt::{Display, write};

use tokio::fs::read_dir;

use crate::state::{self, IndexState, ThreadSafeState};

#[derive(Debug)]
pub enum IndexError {
    IoError(std::io::Error)
}
impl Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::IoError(e) => write!(f, "I/O error: {e}"),
            _ => write!(f, "Generic indexing error")
        }
    }
}
impl std::error::Error for IndexError {}
impl From<std::io::Error> for IndexError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

pub async fn index_archive(state: ThreadSafeState) -> Result<(), IndexError> {
    state.lock().await.index_state = IndexState::Regenerating;

    index_cars(state.clone()).await?;
    index_stages(state.clone()).await?;
    index_stage_pieces(state.clone()).await?;
    index_wheels(state.clone()).await?;

    state.lock().await.index_state = IndexState::Healthy;
    Ok(())
}

async fn index_cars(state: ThreadSafeState) -> Result<(), IndexError>  {
    let base_path = format!("{}/cars", state.lock().await.config.filestore);
    let mut dirs = read_dir(base_path).await?;
    
    while let Some(e) = dirs.next_entry().await? {
        if e.metadata().await?.is_dir() {
            let mut files = read_dir(e.path()).await?;
            while let Some(e) = files.next_entry().await? {
                
            }
        }
    };

    todo!()
}

async fn index_stages(state: ThreadSafeState) -> Result<(), IndexError>  {
    todo!()
}

async fn index_stage_pieces(state: ThreadSafeState) -> Result<(), IndexError>  {
    todo!()
}

async fn index_wheels(state: ThreadSafeState) -> Result<(), IndexError> {
    todo!()
}