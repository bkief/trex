pub mod ieee738;

use wasm_bindgen::prelude::*;
use ieee738::{ConductorType, EnvironmentConditions, ConductorState, CONDUCTOR_NAMES};

/// Adds two numbers together in WebAssembly.
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Returns a welcoming greeting from the Rust WASM module.
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! This message is powered by Rust WebAssembly.", name)
}

/// Exposes the list of standard Southwire conductor names.
#[wasm_bindgen]
pub fn get_conductor_list() -> Vec<String> {
    CONDUCTOR_NAMES.iter().map(|s| s.to_string()).collect()
}

/// Details of a conductor for UI display.
#[wasm_bindgen]
pub struct ConductorDetails {
    pub diameter: f64,
    pub heat_capacity: f64,
    pub resistance_low: f64,
    pub resistance_high: f64,
}

/// Fetches details of a specific conductor by name.
#[wasm_bindgen]
pub fn get_conductor_details(name: &str) -> Option<ConductorDetails> {
    let cond_type = ConductorType::from_name(name)?;
    let props = cond_type.properties();
    Some(ConductorDetails {
        diameter: props.D,
        heat_capacity: props.mCp,
        resistance_low: props.R_T_low,
        resistance_high: props.R_T_high,
    })
}

/// Calculates the maximum steady-state current (ampacity) in Amperes
/// using the IEEE 738 standard math.
#[wasm_bindgen]
pub fn calculate_ampacity(
    name: &str,
    t_max: f64,
    t_ambient: f64,
    wind_speed: f64,
    wind_angle_deg: f64,
    elevation: f64,
    solar_radiation: f64,
) -> Result<f64, String> {
    let cond_type = ConductorType::from_name(name)
        .ok_or_else(|| format!("Conductor '{}' not found", name))?;
    
    let state = ConductorState::new(cond_type, t_ambient);
    
    let env = EnvironmentConditions::new(
        t_ambient,
        wind_speed,
        wind_angle_deg,
        elevation,
        solar_radiation,
        90.0, // 90 degrees incidence angle (worst-case peak solar)
    );

    let ampacity = state.calculate_steady_state_ampacity(t_max, &env);
    Ok(ampacity)
}

/// Calculates the steady-state operating temperature in °C
/// for a given load current using the IEEE 738 binary search math.
#[wasm_bindgen]
pub fn calculate_steady_state_temp(
    name: &str,
    current: f64,
    t_ambient: f64,
    wind_speed: f64,
    wind_angle_deg: f64,
    elevation: f64,
    solar_radiation: f64,
) -> Result<f64, String> {
    let cond_type = ConductorType::from_name(name)
        .ok_or_else(|| format!("Conductor '{}' not found", name))?;
    
    let state = ConductorState::new(cond_type, t_ambient);
    
    let env = EnvironmentConditions::new(
        t_ambient,
        wind_speed,
        wind_angle_deg,
        elevation,
        solar_radiation,
        90.0,
    );

    let temp = state.calculate_steady_state_temp(current, &env);
    Ok(temp)
}

/// Simulates transient conductor temperature over a time duration (in minutes)
/// with a load step-change from initial_current to stepped_current at step_time_mins.
/// Returns a list of conductor temperatures sampled at 1-minute intervals.
#[wasm_bindgen]
pub fn simulate_transient_temp(
    name: &str,
    t_ambient: f64,
    wind_speed: f64,
    wind_angle_deg: f64,
    elevation: f64,
    solar_radiation: f64,
    initial_current: f64,
    stepped_current: f64,
    step_time_mins: f64,
    duration_mins: f64,
) -> Result<Vec<f64>, String> {
    let cond_type = ConductorType::from_name(name)
        .ok_or_else(|| format!("Conductor '{}' not found", name))?;
    
    let env = EnvironmentConditions::new(
        t_ambient,
        wind_speed,
        wind_angle_deg,
        elevation,
        solar_radiation,
        90.0,
    );

    // 1. Calculate steady-state starting temperature at the initial current
    let state_temp_init = ConductorState::new(cond_type, t_ambient);
    let initial_steady_temp = state_temp_init.calculate_steady_state_temp(initial_current, &env);

    // 2. Initialize simulation state
    let mut state = ConductorState::new(cond_type, initial_steady_temp);

    let dt = 1.0; // 1 second time steps for stability
    let total_steps = (duration_mins * 60.0) as usize;
    let step_time_secs = step_time_mins * 60.0;
    
    // We will save temperature every minute to avoid sending too much data to JS
    let mut temperatures = Vec::new();
    temperatures.push(state.Tc); // initial temp at minute 0

    for s in 1..=total_steps {
        let t_secs = s as f64;
        let current = if t_secs >= step_time_secs {
            stepped_current
        } else {
            initial_current
        };

        state.tick(current, &env, dt);

        // Save temperature every 60 seconds (1 minute)
        if s % 60 == 0 {
            temperatures.push(state.Tc);
        }
    }

    Ok(temperatures)
}
