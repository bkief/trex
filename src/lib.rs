pub mod conductors;
pub mod ieee738;
pub mod sag;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "python")]
use pyo3::prelude::*;

use ieee738::{ConductorType, EnvironmentConditions, ConductorState, CONDUCTOR_NAMES};

/// A physical conductor
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Debug)]
pub struct Conductor {
    pub(crate) name: String,
    pub(crate) inner: ConductorType,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Conductor {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Result<Conductor, String> {
        let inner = ConductorType::from_name(name)
            .ok_or_else(|| format!("Conductor '{}' not found", name))?;
        Ok(Conductor {
            name: name.to_string(),
            inner,
        })
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
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
        let inner = ConductorType::from_name(name)
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err(format!("Conductor '{}' not found", name)))?;
        Ok(Conductor {
            name: name.to_string(),
            inner,
        })
    }

    #[getter]
    pub fn name(&self) -> String {
        self.name.clone()
    }
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
    pub fn custom(
        name: &str,
        diameter: f64,
        mass_kg_m: f64,
        heat_capacity: f64,
        r_t_high: f64,
        r_t_low: f64,
        t_high: f64,
        t_low: f64,
        epsilon: f64,
        alpha: f64,
        rated_strength_newtons: Option<f64>,
    ) -> Self {
        use crate::conductors::ConductorProperties;
        let props = ConductorProperties {
            D: diameter,
            mass_kg_m,
            mCp: heat_capacity,
            R_T_high: r_t_high,
            R_T_low: r_t_low,
            T_high: t_high,
            T_low: t_low,
            epsilon,
            alpha,
            rated_strength_newtons,
        };
        Conductor {
            name: name.to_string(),
            inner: ConductorType::Custom(props),
        }
    }
}

// --- WASM Bindings ---

/// Exposes the list of standard Southwire conductor names.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_conductor_list() -> Vec<String> {
    CONDUCTOR_NAMES.iter().map(|s| s.to_string()).collect()
}


/// Calculates solar heat radiation flux (Q_se in W/m^2) using IEEE 738 Table 4 (Clear) 
/// or Table 5 (Industrial) equations, corrected for elevation (m) and solar altitude (degrees).
#[cfg(feature = "wasm")]
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
#[cfg(feature = "wasm")]
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
#[cfg(feature = "wasm")]
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


/// Calculates the maximum steady-state current (ampacity) in Amperes
/// using the IEEE 738 standard math.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn calculate_ampacity(
    conductor: &Conductor,
    t_max: f64,
    t_ambient: f64,
    wind_speed: f64,
    wind_angle_deg: f64,
    elevation: f64,
    solar_radiation: f64,
    emissivity: f64,
    absorptivity: f64,
) -> f64 {
    let mut state = ConductorState::new(conductor.inner, t_ambient);
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

    let ampacity = state.calculate_steady_state_ampacity(t_max, &env);
    ampacity
}

/// Calculates the steady-state operating temperature in °C
/// for a given load current using the IEEE 738 binary search math.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn calculate_steady_state_temp(
    conductor: &Conductor,
    current: f64,
    t_ambient: f64,
    wind_speed: f64,
    wind_angle_deg: f64,
    elevation: f64,
    solar_radiation: f64,
    emissivity: f64,
    absorptivity: f64,
) -> f64 {
    let mut state = ConductorState::new(conductor.inner, t_ambient);
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
    temp
}

/// Simulates transient conductor temperature over a time duration (in minutes).
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn simulate_transient_temp(
    conductor: &Conductor,
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
) -> Vec<f64> {
    let env = EnvironmentConditions::new(
        t_ambient,
        wind_speed,
        wind_angle_deg,
        elevation,
        solar_radiation,
        90.0,
    );

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
        let current = if t_secs >= step_time_secs {
            stepped_current
        } else {
            initial_current
        };

        state.tick(current, &env, dt);

        if s % 60 == 0 {
            temperatures.push(state.Tc);
        }
    }

    temperatures
}

/// Finds the emergency loading current required to reach the target temperature limit.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn find_emergency_loading_by_duration(
    conductor: &Conductor,
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

    let ampacity = calculate_ampacity(
        conductor,
        t_max,
        t_ambient,
        wind_speed,
        wind_angle_deg,
        elevation,
        solar_radiation,
        emissivity,
        absorptivity,
    );

    let mut low = initial_current;
    let mut high = ampacity * 5.0;
    let mut best_mid = ampacity * 1.2;

    for _ in 0..30 {
        let mid = (low + high) / 2.0;
        let temps = simulate_transient_temp(
            conductor,
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
        conductor,
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
    );

    if final_temps.len() > target_idx {
        let final_temp = final_temps[target_idx];
        if (final_temp - t_max).abs() < 0.5 {
            return Ok(best_mid);
        }
    }

    Err("Search did not converge to within 0.5 degrees".to_string())
}

/// Calculates conductor sag, operating tension, ground clearance, and 2D catenary curve points.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn calculate_conductor_sag(
    conductor: &Conductor,
    span_length: f64,
    initial_tension_percent_rts: f64,
    conductor_temp: f64,
    ref_temp: f64,
    structure_height: f64,
) -> Result<sag::ConductorSagResult, JsValue> {
    sag::calculate_sag(
        conductor.inner,
        span_length,
        initial_tension_percent_rts,
        conductor_temp,
        ref_temp,
        structure_height,
    ).map_err(|e| JsValue::from_str(&e))
}

// --- Python Bindings (PyO3) ---

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "get_conductor_list")]
fn py_get_conductor_list() -> Vec<String> {
    CONDUCTOR_NAMES.iter().map(|s| s.to_string()).collect()
}


#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "calculate_solar_radiation")]
fn py_calculate_solar_radiation(atmosphere: &str, solar_altitude_deg: f64, elevation: f64) -> f64 {
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

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "calculate_astronomical_solar")]
fn py_calculate_astronomical_solar(latitude: f64, day_of_year: u32, hour_of_day: f64, elevation: f64, atmosphere: &str) -> f64 {
    let atmosphere_enum = match atmosphere.to_lowercase().as_str() {
        "industrial" => ieee738::AtmosphereType::Industrial,
        _ => ieee738::AtmosphereType::Clear,
    };
    ieee738::calculate_astronomical_solar_flux(latitude, day_of_year, hour_of_day, elevation, atmosphere_enum)
}

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "calculate_daily_solar_curve")]
fn py_calculate_daily_solar_curve(latitude: f64, day_of_year: u32, elevation: f64, atmosphere: &str) -> Vec<f64> {
    let atmosphere_enum = match atmosphere.to_lowercase().as_str() {
        "industrial" => ieee738::AtmosphereType::Industrial,
        _ => ieee738::AtmosphereType::Clear,
    };
    let mut curve = Vec::with_capacity(24);
    for hour in 0..24 {
        let hour_f = hour as f64 + 0.5;
        let q_se = ieee738::calculate_astronomical_solar_flux(latitude, day_of_year, hour_f, elevation, atmosphere_enum);
        curve.push(q_se);
    }
    curve
}

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "calculate_ampacity")]
fn py_calculate_ampacity(
    conductor: &Conductor,
    max_conductor_temp: f64,
    ambient_temp: f64,
    wind_speed: f64,
    wind_angle_deg: f64,
    elevation: f64,
    solar_radiation: f64,
    emissivity: f64,
    absorptivity: f64,
) -> f64 {
    let mut state = ConductorState::new(conductor.inner, ambient_temp);
    state.properties.epsilon = emissivity;
    state.properties.alpha = absorptivity;
    let env = EnvironmentConditions::new(ambient_temp, wind_speed, wind_angle_deg, elevation, solar_radiation, 90.0);
    state.calculate_steady_state_ampacity(max_conductor_temp, &env)
}

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "simulate_transient_temp")]
fn py_simulate_transient_temp(
    conductor: &Conductor,
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

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "calculate_conductor_sag")]
fn py_calculate_conductor_sag(
    conductor: &Conductor,
    span_length: f64,
    initial_tension_percent_rts: f64,
    conductor_temp: f64,
    ref_temp: f64,
    structure_height: f64,
) -> PyResult<sag::ConductorSagResult> {
    sag::calculate_sag(
        conductor.inner,
        span_length,
        initial_tension_percent_rts,
        conductor_temp,
        ref_temp,
        structure_height,
    ).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
}

#[cfg(feature = "python")]
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
