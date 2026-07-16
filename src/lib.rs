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
