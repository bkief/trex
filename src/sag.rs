//! Conductor Sag & Tension Calculations
//!
//! Provides catenary and thermal sag calculations for overhead electrical conductors
//! based on span length, stringing tension, and operating temperature.

use crate::conductors::ConductorType;
use wasm_bindgen::prelude::*;

/// Result struct holding conductor sag, tension, and 2D catenary curve coordinates
#[wasm_bindgen]
pub struct ConductorSagResult {
    pub initial_sag: f64,
    pub operating_sag: f64,
    pub sag_percent: f64,
    pub clearance: f64,
    pub initial_tension: f64,
    pub operating_tension: f64,
    pub initial_tension_percent_rts: f64,
    pub operating_tension_percent_rts: f64,
    pub rated_strength: f64,
    pub weight_n_per_m: f64,
    curve_x: Vec<f64>,
    curve_y: Vec<f64>,
    curve_y_initial: Vec<f64>,
}

#[wasm_bindgen]
impl ConductorSagResult {
    pub fn get_curve_x(&self) -> Vec<f64> {
        self.curve_x.clone()
    }

    pub fn get_curve_y(&self) -> Vec<f64> {
        self.curve_y.clone()
    }

    pub fn get_curve_y_initial(&self) -> Vec<f64> {
        self.curve_y_initial.clone()
    }
}

/// Calculate conductor sag, tension, ground clearance, and 2D catenary curve
pub fn calculate_sag(
    conductor_name: &str,
    span_length_m: f64,
    initial_tension_percent_rts: f64,
    conductor_temp_c: f64,
    ref_temp_c: f64,
    structure_height_m: f64,
) -> Result<ConductorSagResult, String> {
    let conductor_type = ConductorType::from_name(conductor_name)
        .ok_or_else(|| format!("Unknown conductor: {}", conductor_name))?;

    let props = conductor_type.properties();
    
    // Conductor linear weight (N/m) = mass (kg/m) * g (m/s^2)
    let weight_n_per_m = props.mass_kg_m * 9.80665;
    
    // Rated Breaking Strength (N)
    let rated_strength = props.rated_strength_newtons.unwrap_or(100_000.0);

    // Initial stringing horizontal tension (N)
    let clamped_tension_pct = initial_tension_percent_rts.clamp(5.0, 50.0);
    let initial_tension = (clamped_tension_pct / 100.0) * rated_strength;

    // Initial sag S0 = (w * L^2) / (8 * H0)
    let initial_sag = (weight_n_per_m * span_length_m * span_length_m) / (8.0 * initial_tension);

    // Thermal Expansion calculation
    // Standard linear coefficient of thermal expansion for composite conductors (1/C)
    let alpha_thermal = 19.1e-6; // ~19.1 x 10^-6 /°C for ACSR
    let delta_t = (conductor_temp_c - ref_temp_c).max(0.0);

    // Thermal expansion length change delta_L = L * alpha * delta_T
    let delta_l_thermal = span_length_m * alpha_thermal * delta_t;

    // Parabolic state change formula: S_thermal = sqrt(S0^2 + (3/8) * L * delta_L)
    let operating_sag = (initial_sag * initial_sag + (3.0 / 8.0) * span_length_m * delta_l_thermal).sqrt();

    // Operating tension H_op = (w * L^2) / (8 * S_op)
    let operating_tension = if operating_sag > 0.0 {
        (weight_n_per_m * span_length_m * span_length_m) / (8.0 * operating_sag)
    } else {
        initial_tension
    };

    let operating_tension_percent_rts = (operating_tension / rated_strength) * 100.0;
    let sag_percent = (operating_sag / span_length_m) * 100.0;
    let clearance = (structure_height_m - operating_sag).max(0.0);

    // Generate 51 points along span for 2D Catenary curve rendering
    let num_points = 51;
    let mut curve_x = Vec::with_capacity(num_points);
    let mut curve_y = Vec::with_capacity(num_points);
    let mut curve_y_initial = Vec::with_capacity(num_points);

    for i in 0..num_points {
        let x = (i as f64 / (num_points - 1) as f64) * span_length_m;
        let x_rel = x - (span_length_m / 2.0);
        
        // Parabolic profile equation: y(x) = H_struct - S * (1 - 4*(x_rel/L)^2)
        let y_operating = structure_height_m - operating_sag * (1.0 - (4.0 * x_rel * x_rel) / (span_length_m * span_length_m));
        let y_initial = structure_height_m - initial_sag * (1.0 - (4.0 * x_rel * x_rel) / (span_length_m * span_length_m));
        
        curve_x.push(x);
        curve_y.push(y_operating.max(0.0));
        curve_y_initial.push(y_initial.max(0.0));
    }

    Ok(ConductorSagResult {
        initial_sag,
        operating_sag,
        sag_percent,
        clearance,
        initial_tension,
        operating_tension,
        initial_tension_percent_rts: clamped_tension_pct,
        operating_tension_percent_rts,
        rated_strength,
        weight_n_per_m,
        curve_x,
        curve_y,
        curve_y_initial,
    })
}
