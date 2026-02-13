use sqlx::types::Uuid;
use tokio::fs::write;
use crate::{ffi::SimulateTimeTrialResult, state::ThreadSafeState};

pub fn get_tt_file_path(root: &str, tt_id: Uuid) -> String {
    format!("{}/tt/{}.timetrial", root, tt_id.hyphenated())
}

pub async fn write_tt_file(state: &ThreadSafeState, tt_id: Uuid, data: &[u8]) -> Result<(), std::io::Error> {
    let path = get_tt_file_path(&state.lock().await.config.filestore, tt_id);
    write(path, data).await
}

pub fn validate_upload_tt_file(file_bytes: &[u8]) -> Result<SimulateTimeTrialResult, String> {
    if file_bytes.len() > 10 * 1024 * 1024 {
        return Err("File size exceeds 10 MB limit".to_string());
    }

    let sim_result = unsafe { crate::ffi::nfmw_simulate_tt(file_bytes.as_ptr() as *const _) };
    if sim_result.has_error {
        return Err(format!("TT simulation failed: {}", String::from_utf8_lossy(&sim_result.exception.message)));
    }

    if sim_result.elapsed_ticks <= 0 {
        return Err("Simulated TT has non-positive tick count, invalid TT data".to_string());
    }

    if sim_result.expected_ticks <= 0 {
        return Err("Simulated TT has non-positive expected tick count, invalid TT data".to_string());
    }

    if sim_result.elapsed_ticks != sim_result.expected_ticks {
        return Err("The simulation did not succeed; either the car or stage is corrupted, or you are using an old version.".to_string());
    }

    Ok(sim_result)
}

pub fn get_tt_version(file_bytes: &[u8]) -> Result<i32, String> {
    let info = unsafe { crate::ffi::nfmw_get_tt_info(file_bytes.as_ptr() as *const _) };
    if info.has_error {
        return Err(format!("TT info fetch failed: {}", String::from_utf8_lossy(&info.exception.message)));
    }
    Ok(info.replay_version)
}