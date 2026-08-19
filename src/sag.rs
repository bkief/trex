//! Conductor Sag & Tension Calculations
//!
//! Solves exact catenary sag and horizontal tension using Newton-Raphson iteration
//! on the Catenary Change-of-State equation for overhead electrical conductors.

use crate::conductors::ConductorType;

const MAX_NEWTON_ITERATIONS: usize = 100;
const NEWTON_GRADIENT_TOLERANCE: f64 = 1e-12;
const NEWTON_STEP_TOLERANCE: f64 = 1e-4;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Result struct holding conductor sag, tension, and 2D catenary curve coordinates
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Debug)]
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

#[cfg(feature = "wasm")]
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

#[cfg(feature = "python")]
#[pymethods]
impl ConductorSagResult {
    #[getter]
    pub fn initial_sag(&self) -> f64 { self.initial_sag }
    #[getter]
    pub fn operating_sag(&self) -> f64 { self.operating_sag }
    #[getter]
    pub fn sag_percent(&self) -> f64 { self.sag_percent }
    #[getter]
    pub fn clearance(&self) -> f64 { self.clearance }
    #[getter]
    pub fn initial_tension(&self) -> f64 { self.initial_tension }
    #[getter]
    pub fn operating_tension(&self) -> f64 { self.operating_tension }
    #[getter]
    pub fn initial_tension_percent_rts(&self) -> f64 { self.initial_tension_percent_rts }
    #[getter]
    pub fn operating_tension_percent_rts(&self) -> f64 { self.operating_tension_percent_rts }
    #[getter]
    pub fn rated_strength(&self) -> f64 { self.rated_strength }
    #[getter]
    pub fn weight_n_per_m(&self) -> f64 { self.weight_n_per_m }
    #[getter]
    pub fn curve_x(&self) -> Vec<f64> { self.curve_x.clone() }
    #[getter]
    pub fn curve_y(&self) -> Vec<f64> { self.curve_y.clone() }
    #[getter]
    pub fn curve_y_initial(&self) -> Vec<f64> { self.curve_y_initial.clone() }

    fn __repr__(&self) -> String {
        format!(
            "<ConductorSagResult operating_sag={:.2}m clearance={:.2}m operating_tension={:.1}% RTS>",
            self.operating_sag, self.clearance, self.operating_tension_percent_rts
        )
    }
}

/// Solves the exact Catenary Change-of-State cubic equation for horizontal tension H2 (N)
/// using Newton-Raphson iteration:
///
/// f(H2) = H2^3 + A_coeff * H2^2 - K_const = 0
///
/// where:
/// K_const = (E * A * w^2 * L^2) / 24
/// A_coeff = (K_const / H1^2) + E * A * alpha * (T2 - T1) - H1
pub fn solve_state_change_newton_raphson(
    w: f64,          // Linear weight (N/m)
    span_l: f64,     // Span length (m)
    h1: f64,         // Initial stringing tension (N)
    ea: f64,         // Conductor elastic modulus x area (N)
    alpha: f64,      // Linear coefficient of thermal expansion (1/C)
    delta_t: f64,    // Temperature change T2 - T1 (C)
) -> f64 {
    let k_const = (ea * w * w * span_l * span_l) / 24.0;
    let a_coeff = (k_const / (h1 * h1)) + (ea * alpha * delta_t) - h1;

    // Initial guess for H2
    let mut h2 = h1;

    // Newton-Raphson iteration loop
    for _ in 0..MAX_NEWTON_ITERATIONS {
        let f = h2.powi(3) + a_coeff * h2.powi(2) - k_const;
        let f_prime = 3.0 * h2.powi(2) + 2.0 * a_coeff * h2;

        if f_prime.abs() < NEWTON_GRADIENT_TOLERANCE {
            break;
        }

        let delta_h = f / f_prime;
        h2 -= delta_h;

        if delta_h.abs() < NEWTON_STEP_TOLERANCE {
            break;
        }
    }

    h2.max(100.0) // Ensure positive physical horizontal tension
}

/// Calculate conductor sag, tension, ground clearance, and 2D catenary curve via Newton-Raphson
pub fn calculate_sag(
    conductor_type: ConductorType,
    span_length_m: f64,
    initial_tension_percent_rts: f64,
    conductor_temp_c: f64,
    ref_temp_c: f64,
    structure_height_m: f64,
) -> Result<ConductorSagResult, String> {
    let props = conductor_type.properties();
    
    // Conductor linear weight (N/m) = mass (kg/m) * g (m/s^2)
    let weight_n_per_m = props.mass_kg_m * 9.80665;
    
    // Rated Breaking Strength (N)
    let rated_strength = props.rated_strength_newtons.unwrap_or(100_000.0);

    // Initial stringing horizontal tension (N)
    let clamped_tension_pct = initial_tension_percent_rts.clamp(5.0, 50.0);
    let initial_tension = (clamped_tension_pct / 100.0) * rated_strength;

    // Initial catenary sag S0 = (H0 / w) * (cosh(w * L / (2 * H0)) - 1)
    let initial_sag = (initial_tension / weight_n_per_m) * ((weight_n_per_m * span_length_m / (2.0 * initial_tension)).cosh() - 1.0);

    // Thermal & Elastic parameters
    let alpha_thermal = 19.1e-6; // Linear expansion coefficient (1/°C) for ACSR
    let modulus_e = 70e9;        // Effective Modulus of Elasticity (Pa = N/m^2) for ACSR
    let area_a = (std::f64::consts::PI / 4.0) * props.D * props.D; // Cross-sectional area (m^2)
    let ea = modulus_e * area_a; // Elastic stiffness EA (N)

    let delta_t = conductor_temp_c - ref_temp_c;

    // Solve operating tension H2 via Newton-Raphson iteration
    let operating_tension = solve_state_change_newton_raphson(
        weight_n_per_m,
        span_length_m,
        initial_tension,
        ea,
        alpha_thermal,
        delta_t,
    );

    // Operating catenary sag S_op = (H2 / w) * (cosh(w * L / (2 * H2)) - 1)
    let operating_sag = (operating_tension / weight_n_per_m) * ((weight_n_per_m * span_length_m / (2.0 * operating_tension)).cosh() - 1.0);

    let operating_tension_percent_rts = (operating_tension / rated_strength) * 100.0;
    let sag_percent = (operating_sag / span_length_m) * 100.0;
    let clearance = (structure_height_m - operating_sag).max(0.0);

    // Generate 51 points along span for exact 2D Catenary curve rendering
    let num_points = 51;
    let mut curve_x = Vec::with_capacity(num_points);
    let mut curve_y = Vec::with_capacity(num_points);
    let mut curve_y_initial = Vec::with_capacity(num_points);

    let a_operating = operating_tension / weight_n_per_m;
    let a_initial = initial_tension / weight_n_per_m;

    for i in 0..num_points {
        let x = (i as f64 / (num_points - 1) as f64) * span_length_m;
        let x_rel = x - (span_length_m / 2.0);
        
        // Exact Catenary curve profile: y(x) = H_struct - S + a * (cosh(x_rel / a) - 1)
        let y_operating = structure_height_m - operating_sag + a_operating * ((x_rel / a_operating).cosh() - 1.0);
        let y_initial = structure_height_m - initial_sag + a_initial * ((x_rel / a_initial).cosh() - 1.0);
        
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newton_raphson_sag_calculation() {
        // Test Drake conductor over a 250m span at 20% RTS stringing tension (15°C -> 100°C)
        let res = calculate_sag(ConductorType::Drake, 250.0, 20.0, 100.0, 15.0, 25.0).unwrap();

        // 1. Initial stringing sag should be less than operating thermal sag
        assert!(res.operating_sag > res.initial_sag, "Operating thermal sag must exceed initial stringing sag");

        // 2. Operating tension (% RTS) should drop as thermal elongation occurs
        assert!(res.operating_tension < res.initial_tension, "Operating tension should drop below initial tension");

        // 3. Ground clearance check: clearance = struct_height - operating_sag
        assert!((res.clearance - (25.0 - res.operating_sag)).abs() < 1e-4, "Clearance math mismatch");

        // 4. 2D Catenary curve points test
        assert_eq!(res.curve_x.len(), 51);
        assert_eq!(res.curve_y.len(), 51);

        // Attachment points at x = 0 and x = 250m must equal structure height (25m)
        assert!((res.curve_y[0] - 25.0).abs() < 1e-3, "Start attachment height should be 25m");
        assert!((res.curve_y[50] - 25.0).abs() < 1e-3, "End attachment height should be 25m");

        // Mid-span point at x = 125m must equal clearance (25m - sag)
        assert!((res.curve_y[25] - res.clearance).abs() < 1e-3, "Mid-span clearance height mismatch");
    }

    #[test]
    fn test_newton_raphson_state_change_convergence() {
        // Test Newton-Raphson solver algorithm directly
        let w = 1.628 * 9.80665; // Drake weight N/m
        let span = 300.0;
        let h1 = 0.20 * 140000.0; // 20% RTS
        let ea = 70e9 * (std::f64::consts::PI / 4.0) * 0.02814 * 0.02814;
        let alpha = 19.1e-6;
        let delta_t = 75.0; // 15°C to 90°C

        let h2 = solve_state_change_newton_raphson(w, span, h1, ea, alpha, delta_t);

        // Cubic equation check: f(h2) = h2^3 + A * h2^2 - K = 0
        let k_const = (ea * w * w * span * span) / 24.0;
        let a_coeff = (k_const / (h1 * h1)) + (ea * alpha * delta_t) - h1;
        let f_val = h2.powi(3) + a_coeff * h2.powi(2) - k_const;

        assert!(f_val.abs() < 1.0, "Newton-Raphson cubic equation failed convergence: f(h2) = {}", f_val);
    }
}
