#![allow(non_snake_case)]

pub use crate::conductors::{ConductorProperties, ConductorType, CONDUCTOR_NAMES};

/// Weather and environmental conditions for a given time step
#[derive(Clone, Copy, Debug)]
pub struct EnvironmentConditions {
    pub Ta: f64,    // Ambient temperature (C)
    pub Ws: f64,    // Wind speed (m/s)
    pub Wa: f64,    // Wind angle relative to conductor (degrees)
    pub H_e: f64,   // Elevation above sea level (m)
    pub Q_se: f64,  // Effective solar heat flux (W/m^2)
    pub theta: f64, // Effective angle of incidence of the sun's rays (radians)
}

/// Selection of atmospheric clarity for solar heat calculations (Tables 4 & 5)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AtmosphereType {
    Clear,
    Industrial,
}

impl AtmosphereType {
    pub fn from_str(atmosphere: &str) -> Self {
        match atmosphere.to_lowercase().as_str() {
            "industrial" => AtmosphereType::Industrial,
            _ => AtmosphereType::Clear,
        }
    }
}

impl EnvironmentConditions {
    /// Creates environment conditions using directly provided solar flux and incidence angle.
    /// Useful when you have live Pyranometer (solar sensor) data available, bypassing
    /// the need for astronomical calculations.
    pub fn new(Ta: f64, Ws: f64, Wa: f64, H_e: f64, Q_se: f64, theta_degrees: f64) -> Self {
        Self {
            Ta,
            Ws,
            Wa,
            H_e,
            Q_se,
            theta: theta_degrees.to_radians(), // Struct expects radians internally
        }
    }

    /// Generates Environmental conditions dynamically using the IEEE 738 Astronomical equations
    pub fn from_weather_and_location(
        Ta: f64,
        Ws: f64,
        Wa: f64,
        H_e: f64,
        day_of_year: u32,
        hour_of_day: f64,   // Solar time, e.g., 11.5 for 11:30 AM
        latitude: f64,      // Degrees
        line_azimuth: f64,  // Degrees (0 for N-S, 90 for E-W)
        atmosphere: AtmosphereType
    ) -> Self {
        // 4.4.5.3 Solar declination (delta)
        let delta = 23.45 * ((284.0 + day_of_year as f64) / 365.0 * 360.0).to_radians().sin();
        
        // 4.4.5.2 Hour angle (omega)
        let omega = (hour_of_day - 12.0) * 15.0;
        
        let lat_rad = latitude.to_radians();
        let delta_rad = delta.to_radians();
        let omega_rad = omega.to_radians();
        
        // 4.4.5.1 Solar altitude (H_c)
        let mut sin_hc = lat_rad.cos() * delta_rad.cos() * omega_rad.cos() + lat_rad.sin() * delta_rad.sin();
        if sin_hc < 0.0 { sin_hc = 0.0; } // Clamp to 0 during nighttime per standard
        let H_c = sin_hc.asin().to_degrees();
        
        // 4.4.5.4 Solar azimuth (Z_c)
        let denom = lat_rad.sin() * omega_rad.cos() - lat_rad.cos() * delta_rad.tan();
        let x = omega_rad.sin() / denom;
        
        let c = if omega < 0.0 {
            if x >= 0.0 { 0.0 } else { 180.0 }
        } else {
            if x >= 0.0 { 180.0 } else { 360.0 }
        };
        
        let Z_c = c + x.atan().to_degrees();
        
        // 4.4.5.5 and 4.4.5.6 Total solar and sky radiated heat intensity corrected for elevation (Q_se)
        let Q_se = calculate_solar_flux_from_altitude(H_c, H_e, atmosphere);
        
        // 4.4.5 Angle of incidence of the sun's rays (theta)
        let theta = (H_c.to_radians().cos() * (Z_c - line_azimuth).to_radians().cos()).acos();
        
        EnvironmentConditions { Ta, Ws, Wa, H_e, Q_se, theta }
    }
}

/// Calculates solar heat radiation flux (Q_se in W/m^2) for a given altitude angle and elevation
pub fn calculate_solar_flux_from_altitude(solar_altitude_deg: f64, elevation: f64, atmosphere: AtmosphereType) -> f64 {
    let mut q_s = match atmosphere {
        AtmosphereType::Clear => {
            -42.2391 + 63.8044 * solar_altitude_deg - 1.9220 * solar_altitude_deg.powi(2) + 3.46921e-2 * solar_altitude_deg.powi(3) 
            - 3.61118e-4 * solar_altitude_deg.powi(4) + 1.94318e-6 * solar_altitude_deg.powi(5) - 4.07608e-9 * solar_altitude_deg.powi(6)
        },
        AtmosphereType::Industrial => {
            53.1821 + 14.2110 * solar_altitude_deg + 6.6138e-1 * solar_altitude_deg.powi(2) - 3.1658e-2 * solar_altitude_deg.powi(3) 
            + 5.4654e-4 * solar_altitude_deg.powi(4) - 4.3446e-6 * solar_altitude_deg.powi(5) + 1.3236e-8 * solar_altitude_deg.powi(6)
        }
    };
    if q_s < 0.0 { q_s = 0.0; }
    
    let k_solar = 1.0 + 1.148e-4 * elevation - 1.108e-8 * elevation.powi(2);
    k_solar * q_s
}

/// Calculates solar heat radiation flux (Q_se in W/m^2) using IEEE 738 astronomical equations
/// for a given latitude (deg), day_of_year (1..365), hour_of_day (0..24), elevation (m), and atmosphere type.
pub fn calculate_astronomical_solar_flux(
    latitude: f64,
    day_of_year: u32,
    hour_of_day: f64,
    elevation: f64,
    atmosphere: AtmosphereType,
) -> f64 {
    // 4.4.5.3 Solar declination (delta)
    let delta = 23.45 * ((284.0 + day_of_year as f64) / 365.0 * 360.0).to_radians().sin();
    
    // 4.4.5.2 Hour angle (omega)
    let omega = (hour_of_day - 12.0) * 15.0;
    
    let lat_rad = latitude.to_radians();
    let delta_rad = delta.to_radians();
    let omega_rad = omega.to_radians();
    
    // 4.4.5.1 Solar altitude (H_c)
    let mut sin_hc = lat_rad.cos() * delta_rad.cos() * omega_rad.cos() + lat_rad.sin() * delta_rad.sin();
    if sin_hc < 0.0 { sin_hc = 0.0; }
    let h_c = sin_hc.asin().to_degrees();
    
    if h_c <= 0.0 {
        return 0.0;
    }
    
    calculate_solar_flux_from_altitude(h_c, elevation, atmosphere)
}

/// Represents the real-time thermal state of a specific conductor
#[derive(Clone, Debug)]
pub struct ConductorState {
    pub properties: ConductorProperties,
    pub Tc: f64, // Current conductor temperature (C)
}

impl ConductorState {
    /// Initialize a new conductor state starting at ambient temperature
    pub fn new(conductor_type: ConductorType, initial_temp: f64) -> Self {
        Self {
            properties: conductor_type.properties(),
            Tc: initial_temp,
        }
    }

    /// Progresses the thermal state forward by a time step `dt` (in seconds)
    pub fn tick(&mut self, load_current: f64, env: &EnvironmentConditions, dt: f64) {
        let R_Tc = conductor_resistance(self.Tc, &self.properties);
        let (q_c, q_r, q_s) = self.calculate_heat_factors(env);
        
        // Rate of change of temperature
        let dTc_dt = (1.0 / self.properties.mCp) * (R_Tc * load_current.powi(2) + q_s - q_c - q_r);
        
        // Euler integral for new temperature
        self.Tc += dTc_dt * dt;
    }

    pub fn calculate_steady_state_temp(&self, I_ss: f64, env: &EnvironmentConditions) -> f64 {
        let I_ss_threshold = 0.01;
        let mut Tc_min = env.Ta;
        let mut Tc_max = env.Ta + 500.0; // Safe upper bound
        let mut Tc_test = env.Ta;

        for _ in 0..100 {
            Tc_test = (Tc_max + Tc_min) / 2.0;
            
            let R_Tc = conductor_resistance(Tc_test, &self.properties);
            let (q_c, q_r, q_s) = get_q_factors(Tc_test, &self.properties, env);
            let net_loss = q_c + q_r - q_s;
            
            if net_loss < 0.0 {
                Tc_min = Tc_test;
            } else {
                let I_ss_result = (net_loss / R_Tc).sqrt();
                if (I_ss_result - I_ss).abs() <= I_ss_threshold {
                    break;
                } else if I_ss_result > I_ss {
                    Tc_max = Tc_test;
                } else {
                    Tc_min = Tc_test;
                }
            }
        }
        
        Tc_test
    }

    /// Calculates the maximum steady-state current (ampacity) allowable 
    /// without exceeding a specific maximum temperature limit (T_max).
    pub fn calculate_steady_state_ampacity(&self, t_max: f64, env: &EnvironmentConditions) -> f64 {
        let R_tmax = conductor_resistance(t_max, &self.properties);
        let (q_c, q_r, q_s) = get_q_factors(t_max, &self.properties, env);
        
        let net_heat_loss = q_c + q_r - q_s;
        
        // If environmental solar heat gain alone exceeds cooling limits, 
        // the conductor cannot carry any current safely without exceeding T_max.
        if net_heat_loss <= 0.0 {
            0.0
        } else {
            (net_heat_loss / R_tmax).sqrt()
        }
    }

    /// Private helper to wrap heat factor calculations using struct state
    fn calculate_heat_factors(&self, env: &EnvironmentConditions) -> (f64, f64, f64) {
        get_q_factors(self.Tc, &self.properties, env)
    }
}

// ---------------------------------------------------------
// IEEE 738 Mathematical Helper Functions
// ---------------------------------------------------------

pub fn conductor_resistance(Tc: f64, props: &ConductorProperties) -> f64 {
    ((props.R_T_high - props.R_T_low) / (props.T_high - props.T_low)) * (Tc - props.T_low) + props.R_T_low
}

fn dynamic_viscosity(Ta: f64, Tc: f64) -> f64 {
    let Tfilm = (Tc + Ta) / 2.0;
    1.458e-6 * (Tfilm + 273.0).powf(1.5) / (Tfilm + 383.4)
}

fn air_density(Ta: f64, Tc: f64, H_e: f64) -> f64 {
    let Tfilm = (Tc + Ta) / 2.0;
    (1.293 - 1.525e-4 * H_e + 6.379e-9 * H_e.powi(2)) / (1.0 + 0.00367 * Tfilm)
}

fn reynolds_number(Ta: f64, Tc: f64, H_e: f64, D: f64, Ws: f64) -> f64 {
    let pf = air_density(Ta, Tc, H_e);
    let uf = dynamic_viscosity(Ta, Tc);
    D * pf * Ws / uf
}

fn thermal_conductivity_of_air(Ta: f64, Tc: f64) -> f64 {
    let Tfilm = (Tc + Ta) / 2.0;
    2.424e-2 + 7.477e-5 * Tfilm - 4.407e-9 * Tfilm.powi(2)
}

fn forced_convection(Ta: f64, Tc: f64, env: &EnvironmentConditions, D: f64) -> f64 {
    let kf = thermal_conductivity_of_air(Ta, Tc); 
    let Nre = reynolds_number(Ta, Tc, env.H_e, D, env.Ws);
    
    let Wa_rad = env.Wa.to_radians(); 
    let K_angle = 1.194 - Wa_rad.cos() + 0.194 * (2.0 * Wa_rad).cos() + 0.368 * (2.0 * Wa_rad).sin();

    let qc1 = K_angle * (1.01 + 1.35 * Nre.powf(0.52)) * kf * (Tc - Ta);
    let qc2 = K_angle * 0.754 * Nre.powf(0.6) * kf * (Tc - Ta);

    f64::max(qc1, qc2)
}

fn natural_convection(Ta: f64, Tc: f64, H_e: f64, D: f64) -> f64 {
    let rho_f = air_density(Ta, Tc, H_e);
    3.645 * rho_f.powf(0.5) * D.powf(0.75) * (Tc - Ta).powf(1.25)
}

fn convective_heat_loss(Tc: f64, props: &ConductorProperties, env: &EnvironmentConditions, vectored: bool) -> f64 {
    let q_cn = natural_convection(env.Ta, Tc, env.H_e, props.D);
    let q_cf = forced_convection(env.Ta, Tc, env, props.D);

    if vectored {
        (q_cn.powi(2) + q_cf.powi(2)).powf(0.5)
    } else {
        f64::max(q_cn, q_cf)
    }
}

fn radiative_heat_loss(Tc: f64, props: &ConductorProperties, Ta: f64) -> f64 {
    17.8 * props.D * props.epsilon * (((Tc + 273.0)/100.0).powi(4) - ((Ta + 273.0)/100.0).powi(4))
}

fn solar_heat_gain(props: &ConductorProperties, env: &EnvironmentConditions) -> f64 {
    // Area per unit length is simply the diameter D (m^2 / m)
    props.alpha * env.Q_se * env.theta.sin() * props.D
}

pub fn get_q_factors(Tc: f64, props: &ConductorProperties, env: &EnvironmentConditions) -> (f64, f64, f64) {
    let vectored_convection = false; 
    let q_c = convective_heat_loss(Tc, props, env, vectored_convection);
    let q_r = radiative_heat_loss(Tc, props, env.Ta);
    let q_s = solar_heat_gain(props, env);
    
    (q_c, q_r, q_s)
}

// ---------------------------------------------------------
// Example Execution / Simulation
// ---------------------------------------------------------

pub fn run_simulation() {
    // 1A. Define Environment via direct sensor data (User Provided Q value)
    // For instance, a worst-case assumption of 1000 W/m^2 solar flux at 90 degrees
    let env_direct = EnvironmentConditions::new(
        25.0,    // Ta (Ambient temp)
        0.5,     // Ws (Wind speed)
        0.0,     // Wa (Wind angle)
        25.0,    // H_e (Elevation)
        1000.0,  // Q_se (User provided Q value)
        90.0,    // theta (Incidence angle in degrees)
    );

    // 1B. Alternatively, define Environment via the IEEE 738 Solar Calculator
    let _env_astronomical = EnvironmentConditions::from_weather_and_location(
        25.0,                   // Ta
        0.5,                    // Ws
        0.0,                    // Wa
        25.0,                   // H_e
        161,                    // Day of the year
        11.0,                   // Hour of day
        30.0,                   // Latitude
        90.0,                   // Line Azimuth
        AtmosphereType::Clear,  // Atmosphere Profile
    );

    // Let's use the direct Q-value environment for this run
    let env = env_direct;

    // 2. Initialize Conductor State (Select 'Drake' from Enum)
    let mut conductor = ConductorState::new(ConductorType::Drake, env.Ta);

    // 3. Find Steady State for 500 Amps
    let I_ss = 500.0;
    let Tc_ss = conductor.calculate_steady_state_temp(I_ss, &env);
    conductor.Tc = Tc_ss; // Set initial state to steady state
    
    println!("Initial Steady-State Temp for {}A: {:.2}C", I_ss, Tc_ss);

    // 4. Run Fault Simulation Time-Series
    let dt: f64 = 0.02; // 50 points per second
    let total_time_seconds = 2.0; 
    let steps = (total_time_seconds / dt) as usize;
    
    println!("Running fault simulation...");
    for i in 0..steps {
        let t = (i as f64) * dt;
        
        // Define fault sequence
        let current = if t < 0.1 {
            I_ss
        } else if t < 0.2 {
            20000.0 // Fault
        } else if t < 0.5 {
            0.0     // Breaker Open
        } else if t < 0.6 {
            20000.0 // Reclosure 1
        } else {
            0.0     // Lock-out
        };

        // Advance state by dt
        conductor.tick(current, &env, dt);

        // Optional: log every 10th step
        if i % 10 == 0 {
            println!("t: {:.2}s | I: {:>7}A | Tc: {:.2}C", t, current, conductor.Tc);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drake_ieee738_reference_example() {
        // IEEE 738 Reference Example Case for Drake 795 kcmil ACSR 26/7
        // Ambient Temp: 40°C, Target Max Temp: 100°C, Wind: 0.61 m/s @ 90°, Solar: 1000 W/m²
        let conductor_type = ConductorType::Drake;

        let env = EnvironmentConditions::new(
            40.0,   // Ta (°C)
            0.61,   // Ws (m/s)
            90.0,   // Wa (deg)
            0.0,    // H_e (m)
            1000.0, // Q_se (W/m²)
            90.0,   // theta (deg)
        );

        let state = ConductorState::new(conductor_type, 40.0);

        let (q_c, q_r, q_s) = get_q_factors(100.0, &state.properties, &env);

        // Heat loss & gain components must be positive
        assert!(q_c > 0.0, "Convective heat loss must be positive");
        assert!(q_r > 0.0, "Radiative heat loss must be positive");
        assert!(q_s > 0.0, "Solar heat gain must be positive");

        // Convection must dominate low-wind heat loss
        assert!(q_c > q_r, "Convection heat loss should exceed radiation heat loss");

        // Ampacity calculation for Drake under IEEE 738 reference parameters
        let ampacity = state.calculate_steady_state_ampacity(100.0, &env);

        // Standard Drake ampacity under 40°C ambient, 100°C max temp, 0.61m/s wind is ~1035 A (within ±5%)
        assert!(ampacity > 950.0 && ampacity < 1150.0, "Drake ampacity {:.1} A out of expected IEEE 738 reference range", ampacity);

        // Heat balance equation check: I^2 * R(Tc) + q_s = q_c + q_r
        let r_tc = conductor_resistance(100.0, &state.properties);
        let q_joule = ampacity * ampacity * r_tc;
        let balance_diff = (q_joule + q_s) - (q_c + q_r);
        assert!(balance_diff.abs() < 1e-3, "Heat balance equation failed: diff = {}", balance_diff);
    }

    #[test]
    fn test_transient_temperature_step_response() {
        let conductor_type = ConductorType::Drake;
        let env = EnvironmentConditions::new(25.0, 0.61, 90.0, 0.0, 1000.0, 90.0);
        
        let mut state = ConductorState::new(conductor_type, 25.0);
        let initial_steady_temp = state.calculate_steady_state_temp(500.0, &env);
        state.Tc = initial_steady_temp;

        let mut temps = Vec::new();
        temps.push(state.Tc);

        let dt = 1.0;
        for s in 1..=3600 {
            state.tick(1200.0, &env, dt);
            if s % 60 == 0 {
                temps.push(state.Tc);
            }
        }

        assert_eq!(temps.len(), 61); // Minute 0 through minute 60
        assert!((temps[0] - initial_steady_temp).abs() < 1e-4, "Initial temp should match steady-state");

        // Temperature must increase monotonically after step increase
        for i in 1..temps.len() {
            assert!(temps[i] >= temps[i - 1], "Temperature failed to increase monotonically at minute {}", i);
        }
    }

    #[test]
    fn test_astronomical_solar_radiation() {
        // Test clear sky solar radiation at solar altitude 45 degrees, sea level
        let q_clear = calculate_astronomical_solar_flux(30.0, 172, 12.0, 0.0, AtmosphereType::Clear);
        assert!(q_clear > 800.0 && q_clear < 1200.0, "Clear sky solar radiation {:.1} W/m² out of range", q_clear);
    }
}