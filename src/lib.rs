pub mod conductors;
pub mod ieee738;
pub mod sag;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "python")]
use pyo3::prelude::*;

use ieee738::{ConductorType, EnvironmentConditions, ConductorState, CONDUCTOR_NAMES};

// ---------------------------------------------------------
// Core Structs
// ---------------------------------------------------------

/// A physical conductor
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Debug)]
pub struct Conductor {
    pub(crate) name: String,
    pub(crate) inner: ConductorType,
}

#[cfg(all(feature = "wasm", not(feature = "python")))]
#[wasm_bindgen]
impl Conductor {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Result<Conductor, String> {
        Conductor::new_internal(name)
    }
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String { self.name.clone() }
    #[wasm_bindgen(getter)]
    pub fn diameter(&self) -> f64 { self.inner.properties().D }
    #[wasm_bindgen(getter)]
    pub fn mass_kg_m(&self) -> f64 { self.inner.properties().mass_kg_m }
    #[wasm_bindgen(getter)]
    pub fn heat_capacity(&self) -> f64 { self.inner.properties().mCp }
    #[wasm_bindgen(getter)]
    pub fn resistance_high(&self) -> f64 { self.inner.properties().R_T_high }
    #[wasm_bindgen(getter)]
    pub fn resistance_low(&self) -> f64 { self.inner.properties().R_T_low }
    #[wasm_bindgen(getter)]
    pub fn t_high(&self) -> f64 { self.inner.properties().T_high }
    #[wasm_bindgen(getter)]
    pub fn t_low(&self) -> f64 { self.inner.properties().T_low }
    #[wasm_bindgen(getter)]
    pub fn emissivity(&self) -> f64 { self.inner.properties().epsilon }
    #[wasm_bindgen(getter)]
    pub fn absorptivity(&self) -> f64 { self.inner.properties().alpha }
    #[wasm_bindgen(getter)]
    pub fn rated_strength_newtons(&self) -> Option<f64> { self.inner.properties().rated_strength_newtons }
}

#[cfg(feature = "python")]
#[pymethods]
impl Conductor {
    #[new]
    pub fn new(name: &str) -> PyResult<Self> {
        Conductor::new_internal(name)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }
    #[getter]
    pub fn name(&self) -> String { self.name.clone() }
    #[getter]
    pub fn diameter(&self) -> f64 { self.inner.properties().D }
    #[getter]
    pub fn mass_kg_m(&self) -> f64 { self.inner.properties().mass_kg_m }
    #[getter]
    pub fn heat_capacity(&self) -> f64 { self.inner.properties().mCp }
    #[getter]
    pub fn resistance_high(&self) -> f64 { self.inner.properties().R_T_high }
    #[getter]
    pub fn resistance_low(&self) -> f64 { self.inner.properties().R_T_low }
    #[getter]
    pub fn t_high(&self) -> f64 { self.inner.properties().T_high }
    #[getter]
    pub fn t_low(&self) -> f64 { self.inner.properties().T_low }
    #[getter]
    pub fn emissivity(&self) -> f64 { self.inner.properties().epsilon }
    #[getter]
    pub fn absorptivity(&self) -> f64 { self.inner.properties().alpha }
    #[getter]
    pub fn rated_strength_newtons(&self) -> Option<f64> { self.inner.properties().rated_strength_newtons }

    #[staticmethod]
    #[pyo3(signature = (name, diameter, mass_kg_m, heat_capacity, r_t_high, r_t_low, t_high, t_low, epsilon, alpha, rated_strength_newtons=None))]
    pub fn custom(
        name: &str, diameter: f64, mass_kg_m: f64, heat_capacity: f64,
        r_t_high: f64, r_t_low: f64, t_high: f64, t_low: f64,
        epsilon: f64, alpha: f64, rated_strength_newtons: Option<f64>,
    ) -> Self {
        use crate::conductors::ConductorProperties;
        let props = ConductorProperties {
            D: diameter, mass_kg_m, mCp: heat_capacity, R_T_high: r_t_high,
            R_T_low: r_t_low, T_high: t_high, T_low: t_low, epsilon, alpha,
            rated_strength_newtons,
        };
        Conductor {
            name: name.to_string(),
            inner: ConductorType::Custom(props),
        }
    }
}

impl Conductor {
    /// Pure Rust internal constructor
    pub fn new_internal(name: &str) -> Result<Conductor, String> {
        let inner = ConductorType::from_name(name)
            .ok_or_else(|| format!("Conductor '{}' not found", name))?;
        Ok(Conductor {
            name: name.to_string(),
            inner,
        })
    }
}

// ---------------------------------------------------------
// Pure Rust API (High-Level Front End)
// ---------------------------------------------------------

pub fn get_conductor_list() -> Vec<String> {
    CONDUCTOR_NAMES.iter().map(|s| s.to_string()).collect()
}

pub fn calculate_solar_radiation(atmosphere: &str, solar_altitude_deg: f64, elevation: f64) -> f64 {
    let atmosphere_enum = ieee738::AtmosphereType::from_str(atmosphere);
    let h_c = solar_altitude_deg.clamp(0.0, 90.0);
    ieee738::calculate_solar_flux_from_altitude(h_c, elevation, atmosphere_enum)
}

pub fn calculate_astronomical_solar(
    latitude: f64, day_of_year: u32, hour_of_day: f64, elevation: f64, atmosphere: &str,
) -> f64 {
    let atmosphere_enum = ieee738::AtmosphereType::from_str(atmosphere);
    ieee738::calculate_astronomical_solar_flux(latitude, day_of_year, hour_of_day, elevation, atmosphere_enum)
}

pub fn calculate_daily_solar_curve(
    latitude: f64, day_of_year: u32, elevation: f64, atmosphere: &str,
) -> Vec<f64> {
    let atmosphere_enum = ieee738::AtmosphereType::from_str(atmosphere);
    let mut curve = Vec::with_capacity(24);
    for hour in 0..24 {
        let hour_f = hour as f64 + 0.5;
        let q_se = ieee738::calculate_astronomical_solar_flux(latitude, day_of_year, hour_f, elevation, atmosphere_enum);
        curve.push(q_se);
    }
    curve
}

pub fn calculate_ampacity(
    conductor: &Conductor, t_max: f64, t_ambient: f64, wind_speed: f64,
    wind_angle_deg: f64, elevation: f64, solar_radiation: f64,
    emissivity: f64, absorptivity: f64,
) -> f64 {
    let mut state = ConductorState::new(conductor.inner, t_ambient);
    state.properties.epsilon = emissivity;
    state.properties.alpha = absorptivity;
    let env = EnvironmentConditions::new(t_ambient, wind_speed, wind_angle_deg, elevation, solar_radiation, 90.0);
    state.calculate_steady_state_ampacity(t_max, &env)
}

pub fn calculate_steady_state_temp(
    conductor: &Conductor, current: f64, t_ambient: f64, wind_speed: f64,
    wind_angle_deg: f64, elevation: f64, solar_radiation: f64,
    emissivity: f64, absorptivity: f64,
) -> f64 {
    let mut state = ConductorState::new(conductor.inner, t_ambient);
    state.properties.epsilon = emissivity;
    state.properties.alpha = absorptivity;
    let env = EnvironmentConditions::new(t_ambient, wind_speed, wind_angle_deg, elevation, solar_radiation, 90.0);
    state.calculate_steady_state_temp(current, &env)
}

pub fn simulate_transient_temp(
    conductor: &Conductor, t_ambient: f64, wind_speed: f64, wind_angle_deg: f64,
    elevation: f64, solar_radiation: f64, initial_current: f64,
    stepped_current: f64, step_time_mins: f64, duration_mins: f64,
    emissivity: f64, absorptivity: f64,
) -> Vec<f64> {
    let env = EnvironmentConditions::new(t_ambient, wind_speed, wind_angle_deg, elevation, solar_radiation, 90.0);
    let mut state_temp_init = ConductorState::new(conductor.inner, t_ambient);
    state_temp_init.properties.epsilon = emissivity;
    state_temp_init.properties.alpha = absorptivity;
    let initial_steady_temp = state_temp_init.calculate_steady_state_temp(initial_current, &env);

    let mut state = ConductorState::new(conductor.inner, initial_steady_temp);
    state.properties.epsilon = emissivity;
    state.properties.alpha = absorptivity;

    let dt = 1.0;
    let total_steps = (duration_mins * 60.0) as usize;
    let step_time_secs = step_time_mins * 60.0;
    
    let mut temperatures = Vec::new();
    temperatures.push(state.Tc);

    for s in 1..=total_steps {
        let t_secs = s as f64;
        let current = if t_secs >= step_time_secs { stepped_current } else { initial_current };
        state.tick(current, &env, dt);
        if s % 60 == 0 {
            temperatures.push(state.Tc);
        }
    }
    temperatures
}

pub fn find_emergency_loading_by_duration(
    conductor: &Conductor, t_ambient: f64, wind_speed: f64, wind_angle_deg: f64,
    elevation: f64, solar_radiation: f64, initial_current: f64, t_max: f64,
    target_minutes: f64, duration_mins: f64, emissivity: f64, absorptivity: f64,
) -> Result<f64, String> {
    let step_time_mins = 0.0;
    if duration_mins < target_minutes {
        return Err("Target minutes outside simulation range".to_string());
    }

    let target_idx = target_minutes.round() as usize;

    let ampacity = calculate_ampacity(
        conductor, t_max, t_ambient, wind_speed, wind_angle_deg, elevation,
        solar_radiation, emissivity, absorptivity,
    );

    let mut low = initial_current;
    let mut high = ampacity * 5.0;
    let mut best_mid = ampacity * 1.2;

    for _ in 0..30 {
        let mid = (low + high) / 2.0;
        let temps = simulate_transient_temp(
            conductor, t_ambient, wind_speed, wind_angle_deg, elevation,
            solar_radiation, initial_current, mid, step_time_mins,
            duration_mins, emissivity, absorptivity,
        );

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

    let final_temps = simulate_transient_temp(
        conductor, t_ambient, wind_speed, wind_angle_deg, elevation,
        solar_radiation, initial_current, best_mid, step_time_mins,
        duration_mins, emissivity, absorptivity,
    );

    if final_temps.len() > target_idx {
        let final_temp = final_temps[target_idx];
        if (final_temp - t_max).abs() < 0.5 {
            return Ok(best_mid);
        }
    }

    Err("Search did not converge to within 0.5 degrees".to_string())
}

pub fn calculate_conductor_sag(
    conductor: &Conductor, span_length: f64, initial_tension_percent_rts: f64,
    conductor_temp: f64, ref_temp: f64, structure_height: f64,
) -> Result<sag::ConductorSagResult, String> {
    sag::calculate_sag(
        conductor.inner, span_length, initial_tension_percent_rts,
        conductor_temp, ref_temp, structure_height,
    )
}

// ---------------------------------------------------------
// WASM Bindings
// ---------------------------------------------------------

#[cfg(feature = "wasm")]
mod wasm {
    use super::*;


    #[wasm_bindgen(js_name = get_conductor_list)]
    pub fn wasm_get_conductor_list() -> Vec<String> {
        get_conductor_list()
    }

    #[wasm_bindgen(js_name = calculate_astronomical_solar)]
    pub fn wasm_calculate_astronomical_solar(
        latitude: f64, day_of_year: u32, hour_of_day: f64, elevation: f64, atmosphere: &str,
    ) -> f64 {
        calculate_astronomical_solar(latitude, day_of_year, hour_of_day, elevation, atmosphere)
    }

    #[wasm_bindgen(js_name = calculate_ampacity)]
    pub fn wasm_calculate_ampacity(
        conductor: &Conductor, t_max: f64, t_ambient: f64, wind_speed: f64,
        wind_angle_deg: f64, elevation: f64, solar_radiation: f64,
        emissivity: f64, absorptivity: f64,
    ) -> f64 {
        calculate_ampacity(conductor, t_max, t_ambient, wind_speed, wind_angle_deg, elevation, solar_radiation, emissivity, absorptivity)
    }

    #[wasm_bindgen(js_name = simulate_transient_temp)]
    pub fn wasm_simulate_transient_temp(
        conductor: &Conductor, t_ambient: f64, wind_speed: f64, wind_angle_deg: f64,
        elevation: f64, solar_radiation: f64, initial_current: f64,
        stepped_current: f64, step_time_mins: f64, duration_mins: f64,
        emissivity: f64, absorptivity: f64,
    ) -> Vec<f64> {
        simulate_transient_temp(conductor, t_ambient, wind_speed, wind_angle_deg, elevation, solar_radiation, initial_current, stepped_current, step_time_mins, duration_mins, emissivity, absorptivity)
    }

    #[wasm_bindgen(js_name = find_emergency_loading_by_duration)]
    pub fn wasm_find_emergency_loading_by_duration(
        conductor: &Conductor, t_ambient: f64, wind_speed: f64, wind_angle_deg: f64,
        elevation: f64, solar_radiation: f64, initial_current: f64, t_max: f64,
        target_minutes: f64, duration_mins: f64, emissivity: f64, absorptivity: f64,
    ) -> Result<f64, JsValue> {
        find_emergency_loading_by_duration(conductor, t_ambient, wind_speed, wind_angle_deg, elevation, solar_radiation, initial_current, t_max, target_minutes, duration_mins, emissivity, absorptivity)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = calculate_conductor_sag)]
    pub fn wasm_calculate_conductor_sag(
        conductor: &Conductor, span_length: f64, initial_tension_percent_rts: f64,
        conductor_temp: f64, ref_temp: f64, structure_height: f64,
    ) -> Result<sag::ConductorSagResult, JsValue> {
        calculate_conductor_sag(conductor, span_length, initial_tension_percent_rts, conductor_temp, ref_temp, structure_height)
            .map_err(|e| JsValue::from_str(&e))
    }
}

// ---------------------------------------------------------
// Python Bindings (PyO3)
// ---------------------------------------------------------

#[cfg(feature = "python")]
mod python {
    use super::*;


    #[pyfunction]
    #[pyo3(name = "get_conductor_list")]
    pub fn py_get_conductor_list() -> Vec<String> {
        get_conductor_list()
    }

    #[pyfunction]
    #[pyo3(name = "calculate_solar_radiation")]
    pub fn py_calculate_solar_radiation(atmosphere: &str, solar_altitude_deg: f64, elevation: f64) -> f64 {
        calculate_solar_radiation(atmosphere, solar_altitude_deg, elevation)
    }

    #[pyfunction]
    #[pyo3(name = "calculate_astronomical_solar")]
    pub fn py_calculate_astronomical_solar(latitude: f64, day_of_year: u32, hour_of_day: f64, elevation: f64, atmosphere: &str) -> f64 {
        calculate_astronomical_solar(latitude, day_of_year, hour_of_day, elevation, atmosphere)
    }

    #[pyfunction]
    #[pyo3(name = "calculate_daily_solar_curve")]
    pub fn py_calculate_daily_solar_curve(latitude: f64, day_of_year: u32, elevation: f64, atmosphere: &str) -> Vec<f64> {
        calculate_daily_solar_curve(latitude, day_of_year, elevation, atmosphere)
    }

    #[pyfunction]
    #[pyo3(name = "calculate_ampacity")]
    pub fn py_calculate_ampacity(
        conductor: &Conductor, max_conductor_temp: f64, ambient_temp: f64, wind_speed: f64,
        wind_angle_deg: f64, elevation: f64, solar_radiation: f64, emissivity: f64, absorptivity: f64,
    ) -> f64 {
        calculate_ampacity(conductor, max_conductor_temp, ambient_temp, wind_speed, wind_angle_deg, elevation, solar_radiation, emissivity, absorptivity)
    }

    #[pyfunction]
    #[pyo3(name = "simulate_transient_temp")]
    pub fn py_simulate_transient_temp(
        conductor: &Conductor, t_ambient: f64, wind_speed: f64, wind_angle_deg: f64,
        elevation: f64, solar_radiation: f64, initial_current: f64, stepped_current: f64,
        step_time_mins: f64, duration_mins: f64, emissivity: f64, absorptivity: f64,
    ) -> Vec<f64> {
        simulate_transient_temp(conductor, t_ambient, wind_speed, wind_angle_deg, elevation, solar_radiation, initial_current, stepped_current, step_time_mins, duration_mins, emissivity, absorptivity)
    }

    #[pyfunction]
    #[pyo3(name = "calculate_conductor_sag")]
    pub fn py_calculate_conductor_sag(
        conductor: &Conductor, span_length: f64, initial_tension_percent_rts: f64,
        conductor_temp: f64, ref_temp: f64, structure_height: f64,
    ) -> PyResult<sag::ConductorSagResult> {
        calculate_conductor_sag(conductor, span_length, initial_tension_percent_rts, conductor_temp, ref_temp, structure_height)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    #[pymodule]
    fn trex(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(py_get_conductor_list, m)?)?;
        m.add_function(wrap_pyfunction!(py_calculate_solar_radiation, m)?)?;
        m.add_function(wrap_pyfunction!(py_calculate_astronomical_solar, m)?)?;
        m.add_function(wrap_pyfunction!(py_calculate_daily_solar_curve, m)?)?;
        m.add_function(wrap_pyfunction!(py_calculate_ampacity, m)?)?;
        m.add_function(wrap_pyfunction!(py_simulate_transient_temp, m)?)?;
        m.add_function(wrap_pyfunction!(py_calculate_conductor_sag, m)?)?;
        m.add_class::<sag::ConductorSagResult>()?;
        m.add_class::<Conductor>()?;
        Ok(())
    }
}
