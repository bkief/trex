pub mod conductors;
pub mod ieee738;
pub mod sag;

use wasm_bindgen::prelude::*;
use ieee738::{ConductorType, EnvironmentConditions, ConductorState, CONDUCTOR_NAMES};

/// Adds two numbers together in WebAssembly.
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Exposes the list of standard Southwire conductor names.
#[wasm_bindgen]
pub fn get_conductor_list() -> Vec<String> {
    CONDUCTOR_NAMES.iter().map(|s| s.to_string()).collect()
}

/// Calculates solar heat radiation flux (Q_se in W/m^2) using IEEE 738 Table 4 (Clear) 
/// or Table 5 (Industrial) equations, corrected for elevation (m) and solar altitude (degrees).
#[wasm_bindgen]
pub fn calculate_solar_radiation(
    atmosphere: &str,
    solar_altitude_deg: f64,
    elevation: f64,
) -> f64 {
    let atmosphere_enum = match atmosphere.to_lowercase().as_str() {
        "industrial" => ieee738::AtmosphereType::Industrial,
        _ => ieee738::AtmosphereType::Clear,
    };
    
    let h_c = solar_altitude_deg.clamp(0.0, 90.0);
    
    let mut q_s = match atmosphere_enum {
        ieee738::AtmosphereType::Clear => {
            -42.2391 + 63.8044 * h_c - 1.9220 * h_c.powi(2) + 3.46921e-2 * h_c.powi(3) 
            - 3.61118e-4 * h_c.powi(4) + 1.94318e-6 * h_c.powi(5) - 4.07608e-9 * h_c.powi(6)
        },
        ieee738::AtmosphereType::Industrial => {
            53.1821 + 14.2110 * h_c + 6.6138e-1 * h_c.powi(2) - 3.1658e-2 * h_c.powi(3) 
            + 5.4654e-4 * h_c.powi(4) - 4.3446e-6 * h_c.powi(5) + 1.3236e-8 * h_c.powi(6)
        }
    };
    if q_s < 0.0 { q_s = 0.0; }
    
    let k_solar = 1.0 + 1.148e-4 * elevation - 1.108e-8 * elevation.powi(2);
    k_solar * q_s
}

/// Calculates astronomical solar heat radiation flux (Q_se in W/m^2) using IEEE 738 equations
/// for a given latitude (deg), day_of_year (1..365), hour_of_day (0..24), elevation (m), and atmosphere type.
#[wasm_bindgen]
pub fn calculate_astronomical_solar(
    latitude: f64,
    day_of_year: u32,
    hour_of_day: f64,
    elevation: f64,
    atmosphere: &str,
) -> f64 {
    let atmosphere_enum = match atmosphere.to_lowercase().as_str() {
        "industrial" => ieee738::AtmosphereType::Industrial,
        _ => ieee738::AtmosphereType::Clear,
    };
    ieee738::calculate_astronomical_solar_flux(latitude, day_of_year, hour_of_day, elevation, atmosphere_enum)
}

/// Calculates 24-hour diurnal solar irradiance curve Q_se (W/m^2) for hours 0 through 23 of today.
#[wasm_bindgen]
pub fn calculate_daily_solar_curve(
    latitude: f64,
    day_of_year: u32,
    elevation: f64,
    atmosphere: &str,
) -> Vec<f64> {
    let atmosphere_enum = match atmosphere.to_lowercase().as_str() {
        "industrial" => ieee738::AtmosphereType::Industrial,
        _ => ieee738::AtmosphereType::Clear,
    };
    let mut curve = Vec::with_capacity(24);
    for hour in 0..24 {
        let hour_f = hour as f64 + 0.5; // sample mid-hour e.g. 12:30 for hour 12
        let q_se = ieee738::calculate_astronomical_solar_flux(latitude, day_of_year, hour_f, elevation, atmosphere_enum);
        curve.push(q_se);
    }
    curve
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
    emissivity: f64,
    absorptivity: f64,
) -> Result<f64, String> {
    let cond_type = ConductorType::from_name(name)
        .ok_or_else(|| format!("Conductor '{}' not found", name))?;
    
    let mut state = ConductorState::new(cond_type, t_ambient);
    state.properties.epsilon = emissivity;
    state.properties.alpha = absorptivity;
    
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
    emissivity: f64,
    absorptivity: f64,
) -> Result<f64, String> {
    let cond_type = ConductorType::from_name(name)
        .ok_or_else(|| format!("Conductor '{}' not found", name))?;
    
    let mut state = ConductorState::new(cond_type, t_ambient);
    state.properties.epsilon = emissivity;
    state.properties.alpha = absorptivity;
    
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
    emissivity: f64,
    absorptivity: f64,
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
    let mut state_temp_init = ConductorState::new(cond_type, t_ambient);
    state_temp_init.properties.epsilon = emissivity;
    state_temp_init.properties.alpha = absorptivity;
    let initial_steady_temp = state_temp_init.calculate_steady_state_temp(initial_current, &env);

    // 2. Initialize simulation state
    let mut state = ConductorState::new(cond_type, initial_steady_temp);
    state.properties.epsilon = emissivity;
    state.properties.alpha = absorptivity;

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

/// Finds the emergency loading current required to reach the target temperature limit (t_max)
/// at target_minutes of elapsed simulation time, assuming step change starts immediately (step_time = 0).
#[wasm_bindgen]
pub fn find_emergency_loading_by_duration(
    name: &str,
    t_ambient: f64,
    wind_speed: f64,
    wind_angle_deg: f64,
    elevation: f64,
    solar_radiation: f64,
    initial_current: f64,
    t_max: f64,
    target_minutes: f64,
    duration_mins: f64,
    emissivity: f64,
    absorptivity: f64,
) -> Result<f64, String> {
    let step_time_mins = 0.0;
    if duration_mins < target_minutes {
        return Err("Target minutes outside simulation range".to_string());
    }

    let target_idx = target_minutes.round() as usize;

    // Calculate steady-state ampacity to get a baseline for high bound
    let ampacity = calculate_ampacity(
        name,
        t_max,
        t_ambient,
        wind_speed,
        wind_angle_deg,
        elevation,
        solar_radiation,
        emissivity,
        absorptivity,
    )?;

    let mut low = initial_current;
    let mut high = ampacity * 5.0;
    let mut best_mid = ampacity * 1.2;

    for _ in 0..30 {
        let mid = (low + high) / 2.0;
        let temps = simulate_transient_temp(
            name,
            t_ambient,
            wind_speed,
            wind_angle_deg,
            elevation,
            solar_radiation,
            initial_current,
            mid,
            step_time_mins,
            duration_mins,
            emissivity,
            absorptivity,
        )?;

        if temps.len() > target_idx {
            let temp_at_target = temps[target_idx];
            if temp_at_target < t_max {
                low = mid;
            } else {
                high = mid;
            }
            best_mid = mid;
        } else {
            return Err("Simulation results too short".to_string());
        }
    }

    // Verify convergence
    let final_temps = simulate_transient_temp(
        name,
        t_ambient,
        wind_speed,
        wind_angle_deg,
        elevation,
        solar_radiation,
        initial_current,
        best_mid,
        step_time_mins,
        duration_mins,
        emissivity,
        absorptivity,
    )?;

    if final_temps.len() > target_idx {
        let final_temp = final_temps[target_idx];
        if (final_temp - t_max).abs() < 0.5 {
            return Ok(best_mid);
        }
    }

    Err("Search did not converge to within 0.5 degrees".to_string())
}

/// Calculates conductor sag, operating tension, ground clearance, and 2D catenary curve points.
#[wasm_bindgen]
pub fn calculate_conductor_sag(
    conductor_name: &str,
    span_length: f64,
    initial_tension_percent_rts: f64,
    conductor_temp: f64,
    ref_temp: f64,
    structure_height: f64,
) -> Result<sag::ConductorSagResult, JsValue> {
    sag::calculate_sag(
        conductor_name,
        span_length,
        initial_tension_percent_rts,
        conductor_temp,
        ref_temp,
        structure_height,
    ).map_err(|e| JsValue::from_str(&e))
}
