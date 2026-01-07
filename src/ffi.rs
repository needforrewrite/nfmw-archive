use libc::{c_char, c_int};

#[repr(C)]
pub struct SimulateTimeTrialArgs {
    /// Pointer to UTF-8 encoded stage name, null-terminated
    pub stage_name: *const c_char,
    
    /// Pointer to array of CarInfoUnmanaged
    pub cars: *const CarInfoUnmanaged,
    /// Number of cars
    pub car_count: c_int,
    
    /// Pointer to time trial data
    pub time_trial_data: *const c_char,
    /// Length of time trial data
    pub time_trial_data_length: c_int,
}

#[repr(C)]
pub struct CarInfoUnmanaged {
    /// Pointer to UTF-8 encoded car name, null-terminated
    pub car_name: *const c_char,
    pub start_x: c_int,
    pub start_z: c_int,
}

unsafe extern "system" {
    /// Simulates a time trial to completion with a limit of 100M ticks.
    /// Returns the number of elapsed ticks, or -1 on timeout.
    pub fn nfmw_simulate_tt(args: *const SimulateTimeTrialArgs) -> i32;
}