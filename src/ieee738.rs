#![allow(non_snake_case)]

/// Core properties of a physical conductor
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConductorProperties {
    pub D: f64,          // Conductor diameter (m)
    pub mCp: f64,        // Conductor total heat capacity (J/m-C)
    pub R_T_high: f64,   // Resistance at high temperature reference (ohm/m)
    pub R_T_low: f64,    // Resistance at low temperature reference (ohm/m)
    pub T_high: f64,     // High temperature reference (C)
    pub T_low: f64,      // Low temperature reference (C)
    pub epsilon: f64,    // Emissivity (0.0 to 1.0)
    pub alpha: f64,      // Solar absorptivity (0.0 to 1.0)
}

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
pub enum AtmosphereType {
    Clear,
    Industrial,
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
        
        // 4.4.5.5 Total solar and sky radiated heat intensity at sea level (Q_s)
        let mut Q_s = match atmosphere {
            AtmosphereType::Clear => {
                -42.2391 + 63.8044 * H_c - 1.9220 * H_c.powi(2) + 3.46921e-2 * H_c.powi(3) 
                - 3.61118e-4 * H_c.powi(4) + 1.94318e-6 * H_c.powi(5) - 4.07608e-9 * H_c.powi(6)
            },
            AtmosphereType::Industrial => {
                53.1821 + 14.2110 * H_c + 6.6138e-1 * H_c.powi(2) - 3.1658e-2 * H_c.powi(3) 
                + 5.4654e-4 * H_c.powi(4) - 4.3446e-6 * H_c.powi(5) + 1.3236e-8 * H_c.powi(6)
            }
        };
        if Q_s < 0.0 { Q_s = 0.0; } // Clamp to 0 during nighttime per standard
        
        // 4.4.5.6 Total solar and sky radiated heat intensity corrected for elevation (Q_se)
        let K_solar = 1.0 + 1.148e-4 * H_e - 1.108e-8 * H_e.powi(2);
        let Q_se = K_solar * Q_s;
        
        // 4.4.5 Angle of incidence of the sun's rays (theta)
        let theta = (H_c.to_radians().cos() * (Z_c - line_azimuth).to_radians().cos()).acos();
        
        EnvironmentConditions { Ta, Ws, Wa, H_e, Q_se, theta }
    }
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

    /// Calculates steady-state temperature for a given constant current via binary search
    pub fn calculate_steady_state_temp(&self, I_ss: f64, env: &EnvironmentConditions) -> f64 {
        let I_ss_threshold = 0.01;
        let mut Tc_min = env.Ta;
        let mut Tc_max = env.Ta + 500.0; // Safe upper bound
        let mut Tc_test = env.Ta;

        loop {
            Tc_test = (Tc_max + Tc_min) / 2.0;
            
            let R_Tc = conductor_resistance(Tc_test, &self.properties);
            let (q_c, q_r, q_s) = get_q_factors(Tc_test, &self.properties, env);
            let I_ss_result = ((q_c + q_r - q_s) / R_Tc).abs().sqrt();
            
            if (I_ss_result - I_ss).abs() <= I_ss_threshold {
                break;
            } else if I_ss_result > I_ss {
                Tc_max = Tc_test;
            } else {
                Tc_min = Tc_test;
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
    let env_astronomical = EnvironmentConditions::from_weather_and_location(
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

// =====================================================================
// MODULE EXTRACTION GUIDE
// =====================================================================
// To keep your core mathematical logic clean in a standard Rust project, 
// it is highly recommended to move the `ConductorType` enum and its 
// implementations below into a separate file named `conductors.rs`.
//
// 1. Create a new file in your project: `src/conductors.rs`
// 2. Cut and paste the `ConductorType` enum and `impl ConductorType` block below into it.
// 3. You may also want to move the `ConductorProperties` struct (from the top of this file) 
//    into `conductors.rs` as well, or put it in a shared `types.rs`.
// 4. In your main file (e.g., `lib.rs` or `main.rs`), declare the module and import the items:
//
//      pub mod conductors;
//      use conductors::{ConductorType, ConductorProperties};
//
// This will allow you to access `ConductorType::Drake` effortlessly across your codebase
// without bloating your core physics engine.
// =====================================================================

/// Helper enum to easily select standard conductors or provide custom ones
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConductorType {
    Turkey,
    Swan,
    Swanate,
    Sparrow,
    Sparate,
    Robin,
    Raven,
    Quail,
    Pigeon,
    Penguin,
    Waxwing,
    Partridge,
    Ostrich,
    Merlin,
    Linnet,
    Oriole,
    Chickadee,
    Brant,
    Ibis,
    Lark,
    Pelican,
    Flicker,
    Hawk,
    Hen,
    Osprey,
    Parakeet,
    Dove,
    Eagle,
    Peacock,
    Squab,
    WoodDuck,
    Teal,
    Kingbird,
    Swift,
    Rook,
    Grosbeak,
    Scoter,
    Egret,
    Flamingo,
    Gannet,
    Stilt,
    Starling,
    Redwing,
    Coot,
    Drake,
    Tern,
    Condor,
    Mallard,
    Ruddy,
    Canary,
    Rail,
    Cardinal,
    Ortolan,
    Curlew,
    Bluejay,
    Finch,
    Bunting,
    Grackle,
    Bittern,
    Pheasant,
    Dipper,
    Martin,
    Bobolink,
    Lapwing,
    Falcon,
    Chukar,
    Bluebird,
    Kiwi,
    Grouse,
    Petrel,
    Minorca,
    Leghorn,
    Guinea,
    Dotterel,
    Dorking,
    Brahma,
    Cochin,
    // User Provided
    Custom(ConductorProperties),
}

pub const CONDUCTOR_NAMES: &[&str] = &[
    "Turkey", "Swan", "Swanate", "Sparrow", "Sparate", "Robin", "Raven", "Quail", "Pigeon", "Penguin",
    "Waxwing", "Partridge", "Ostrich", "Merlin", "Linnet", "Oriole", "Chickadee", "Brant", "Ibis", "Lark",
    "Pelican", "Flicker", "Hawk", "Hen", "Osprey", "Parakeet", "Dove", "Eagle", "Peacock", "Squab",
    "WoodDuck", "Teal", "Kingbird", "Swift", "Rook", "Grosbeak", "Scoter", "Egret", "Flamingo", "Gannet",
    "Stilt", "Starling", "Redwing", "Coot", "Drake", "Tern", "Condor", "Mallard", "Ruddy", "Canary",
    "Rail", "Cardinal", "Ortolan", "Curlew", "Bluejay", "Finch", "Bunting", "Grackle", "Bittern", "Pheasant",
    "Dipper", "Martin", "Bobolink", "Lapwing", "Falcon", "Chukar", "Bluebird", "Kiwi", "Grouse", "Petrel",
    "Minorca", "Leghorn", "Guinea", "Dotterel", "Dorking", "Brahma", "Cochin"
];

impl ConductorType {
    /// Resolve a string name to its corresponding ConductorType
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "turkey" => Some(Self::Turkey),
            "swan" => Some(Self::Swan),
            "swanate" => Some(Self::Swanate),
            "sparrow" => Some(Self::Sparrow),
            "sparate" => Some(Self::Sparate),
            "robin" => Some(Self::Robin),
            "raven" => Some(Self::Raven),
            "quail" => Some(Self::Quail),
            "pigeon" => Some(Self::Pigeon),
            "penguin" => Some(Self::Penguin),
            "waxwing" => Some(Self::Waxwing),
            "partridge" => Some(Self::Partridge),
            "ostrich" => Some(Self::Ostrich),
            "merlin" => Some(Self::Merlin),
            "linnet" => Some(Self::Linnet),
            "oriole" => Some(Self::Oriole),
            "chickadee" => Some(Self::Chickadee),
            "brant" => Some(Self::Brant),
            "ibis" => Some(Self::Ibis),
            "lark" => Some(Self::Lark),
            "pelican" => Some(Self::Pelican),
            "flicker" => Some(Self::Flicker),
            "hawk" => Some(Self::Hawk),
            "hen" => Some(Self::Hen),
            "osprey" => Some(Self::Osprey),
            "parakeet" => Some(Self::Parakeet),
            "dove" => Some(Self::Dove),
            "eagle" => Some(Self::Eagle),
            "peacock" => Some(Self::Peacock),
            "squab" => Some(Self::Squab),
            "woodduck" => Some(Self::WoodDuck),
            "teal" => Some(Self::Teal),
            "kingbird" => Some(Self::Kingbird),
            "swift" => Some(Self::Swift),
            "rook" => Some(Self::Rook),
            "grosbeak" => Some(Self::Grosbeak),
            "scoter" => Some(Self::Scoter),
            "egret" => Some(Self::Egret),
            "flamingo" => Some(Self::Flamingo),
            "gannet" => Some(Self::Gannet),
            "stilt" => Some(Self::Stilt),
            "starling" => Some(Self::Starling),
            "redwing" => Some(Self::Redwing),
            "coot" => Some(Self::Coot),
            "drake" => Some(Self::Drake),
            "tern" => Some(Self::Tern),
            "condor" => Some(Self::Condor),
            "mallard" => Some(Self::Mallard),
            "ruddy" => Some(Self::Ruddy),
            "canary" => Some(Self::Canary),
            "rail" => Some(Self::Rail),
            "cardinal" => Some(Self::Cardinal),
            "ortolan" => Some(Self::Ortolan),
            "curlew" => Some(Self::Curlew),
            "bluejay" => Some(Self::Bluejay),
            "finch" => Some(Self::Finch),
            "bunting" => Some(Self::Bunting),
            "grackle" => Some(Self::Grackle),
            "bittern" => Some(Self::Bittern),
            "pheasant" => Some(Self::Pheasant),
            "dipper" => Some(Self::Dipper),
            "martin" => Some(Self::Martin),
            "bobolink" => Some(Self::Bobolink),
            "lapwing" => Some(Self::Lapwing),
            "falcon" => Some(Self::Falcon),
            "chukar" => Some(Self::Chukar),
            "bluebird" => Some(Self::Bluebird),
            "kiwi" => Some(Self::Kiwi),
            "grouse" => Some(Self::Grouse),
            "petrel" => Some(Self::Petrel),
            "minorca" => Some(Self::Minorca),
            "leghorn" => Some(Self::Leghorn),
            "guinea" => Some(Self::Guinea),
            "dotterel" => Some(Self::Dotterel),
            "dorking" => Some(Self::Dorking),
            "brahma" => Some(Self::Brahma),
            "cochin" => Some(Self::Cochin),
            _ => None,
        }
    }

    /// Helper to generate IEEE 738 properties directly from Southwire Spec Sheet tables
    /// * `dia_in` - Complete Cable Diameter (inches)
    /// * `weight_lbs` - Total Weight (lbs/1000 ft)
    /// * `pct_al` - Aluminum Content by Weight (%)
    /// * `r_dc_20` - DC Resistance at 20C (Ohms/1000 ft)
    /// * `r_ac_75` - AC Resistance at 75C (Ohms/1000 ft)
    fn from_southwire(dia_in: f64, weight_lbs: f64, pct_al: f64, r_dc_20: f64, r_ac_75: f64) -> ConductorProperties {
        // Linear Mass Conversion (lbs/1000ft -> kg/m)
        let kg_per_m = weight_lbs * 0.00148816;
        
        // Standard Specific Heat values (J/kg-C)
        let cp_al = 897.0; 
        let cp_st = 476.0; 
        
        // Equivalent heat capacity for the composite core based on mass percentages
        let al_ratio = pct_al / 100.0;
        let st_ratio = 1.0 - al_ratio;
        let cp_eq = (al_ratio * cp_al) + (st_ratio * cp_st);
        
        ConductorProperties {
            D: dia_in * 0.0254,           // inches to meters
            mCp: kg_per_m * cp_eq,        // Total Heat Capacity (J/m-C)
            R_T_low: r_dc_20 / 304.8,     // Ohms/1000ft to Ohms/m
            R_T_high: r_ac_75 / 304.8,    // Ohms/1000ft to Ohms/m
            T_low: 20.0,                  // Southwire table base reference
            T_high: 75.0,                 // Southwire table base reference
            epsilon: 0.5,                 // Standard assumed emissivity
            alpha: 0.5,                   // Standard assumed absorptivity
        }
    }

    /// Returns the physical properties for the selected conductor
    pub fn properties(&self) -> ConductorProperties {
        // Default properties use alpha/epsilon of 0.5 (typical for new/moderate weathering).
        // Heavily weathered conductors often approach 0.8. Users can override these parameters 
        // by instantiating via standard enum, then directly modifying the returned struct.
        match self {
            //                         Dia    Wt      %Al     Rdc     Rac
            ConductorType::Turkey    => Self::from_southwire(0.198, 36.0,   67.88, 0.641,  0.806),
            ConductorType::Swan      => Self::from_southwire(0.250, 57.0,   67.87, 0.403,  0.515),
            ConductorType::Swanate   => Self::from_southwire(0.257, 67.0,   58.10, 0.399,  0.519),
            ConductorType::Sparrow   => Self::from_southwire(0.316, 91.0,   67.90, 0.332,  0.428),
            ConductorType::Sparate   => Self::from_southwire(0.325, 107.0,  58.12, 0.338,  0.436),
            ConductorType::Robin     => Self::from_southwire(0.354, 115.0,  67.88, 0.251,  0.325),
            ConductorType::Raven     => Self::from_southwire(0.398, 145.0,  67.89, 0.201,  0.261),
            ConductorType::Quail     => Self::from_southwire(0.447, 183.0,  67.88, 0.159,  0.207),
            ConductorType::Pigeon    => Self::from_southwire(0.502, 230.0,  67.87, 0.126,  0.165),
            ConductorType::Penguin   => Self::from_southwire(0.563, 291.0,  67.88, 0.100,  0.131),
            ConductorType::Waxwing   => Self::from_southwire(0.609, 289.0,  86.43, 0.0643, 0.0787),
            ConductorType::Partridge => Self::from_southwire(0.642, 367.0,  68.51, 0.0637, 0.0779),
            ConductorType::Ostrich   => Self::from_southwire(0.680, 412.0,  68.51, 0.0567, 0.0693),
            ConductorType::Merlin    => Self::from_southwire(0.684, 364.0,  86.43, 0.0510, 0.0625),
            ConductorType::Linnet    => Self::from_southwire(0.720, 462.0,  68.51, 0.0505, 0.0618),
            ConductorType::Oriole    => Self::from_southwire(0.741, 526.0,  60.35, 0.0502, 0.0613),
            ConductorType::Chickadee => Self::from_southwire(0.743, 431.0,  86.43, 0.0432, 0.0529),
            ConductorType::Brant     => Self::from_southwire(0.772, 511.0,  73.21, 0.0430, 0.0526),
            ConductorType::Ibis      => Self::from_southwire(0.783, 546.0,  68.51, 0.0428, 0.0523),
            ConductorType::Lark      => Self::from_southwire(0.806, 622.0,  60.35, 0.0425, 0.0519),
            ConductorType::Pelican   => Self::from_southwire(0.814, 447.0,  86.44, 0.0360, 0.0442),
            ConductorType::Flicker   => Self::from_southwire(0.846, 614.0,  73.21, 0.0358, 0.0439),
            ConductorType::Hawk      => Self::from_southwire(0.858, 656.0,  68.51, 0.0356, 0.0436),
            ConductorType::Hen       => Self::from_southwire(0.883, 746.0,  60.35, 0.0354, 0.0433),
            ConductorType::Osprey    => Self::from_southwire(0.879, 603.0,  86.43, 0.0308, 0.0379),
            ConductorType::Parakeet  => Self::from_southwire(0.914, 716.0,  73.21, 0.0307, 0.0376),
            ConductorType::Dove      => Self::from_southwire(0.927, 765.0,  68.51, 0.0306, 0.0375),
            ConductorType::Eagle     => Self::from_southwire(0.953, 871.0,  60.35, 0.0303, 0.0372),
            ConductorType::Peacock   => Self::from_southwire(0.953, 779.0,  73.20, 0.0282, 0.0346),
            ConductorType::Squab     => Self::from_southwire(0.966, 832.0,  68.51, 0.0281, 0.0345),
            ConductorType::WoodDuck  => Self::from_southwire(0.994, 946.0,  60.35, 0.0279, 0.0342),
            ConductorType::Teal      => Self::from_southwire(0.994, 939.0,  60.86, 0.0279, 0.0342),
            ConductorType::Kingbird  => Self::from_southwire(0.940, 690.0,  86.43, 0.0270, 0.0332),
            ConductorType::Swift     => Self::from_southwire(0.930, 643.0,  92.72, 0.0271, 0.0334),
            ConductorType::Rook      => Self::from_southwire(0.977, 818.0,  73.22, 0.0268, 0.0330),
            ConductorType::Grosbeak  => Self::from_southwire(0.991, 874.0,  68.51, 0.0267, 0.0328),
            ConductorType::Scoter    => Self::from_southwire(1.019, 995.0,  60.35, 0.0256, 0.0325),
            ConductorType::Egret     => Self::from_southwire(1.019, 987.0,  60.85, 0.0266, 0.0326),
            ConductorType::Flamingo  => Self::from_southwire(1.000, 858.0,  73.21, 0.0256, 0.0315),
            ConductorType::Gannet    => Self::from_southwire(1.014, 916.0,  68.51, 0.0255, 0.0313),
            ConductorType::Stilt     => Self::from_southwire(1.036, 920.0,  73.21, 0.0239, 0.0294),
            ConductorType::Starling  => Self::from_southwire(1.051, 984.0,  68.51, 0.0238, 0.0292),
            ConductorType::Redwing   => Self::from_southwire(1.081, 1110.0, 60.85, 0.0236, 0.0290),
            ConductorType::Coot      => Self::from_southwire(1.040, 804.0,  92.72, 0.0217, 0.0268),
            ConductorType::Drake     => Self::from_southwire(1.107, 1093.0, 68.51, 0.0214, 0.0263),
            ConductorType::Tern      => Self::from_southwire(1.063, 895.0,  83.67, 0.0216, 0.0269),
            ConductorType::Condor    => Self::from_southwire(1.092, 1023.0, 73.21, 0.0215, 0.0272),
            ConductorType::Mallard   => Self::from_southwire(1.140, 1234.0, 60.86, 0.0213, 0.0261),
            ConductorType::Ruddy     => Self::from_southwire(1.131, 1013.0, 83.67, 0.0191, 0.0239),
            ConductorType::Canary    => Self::from_southwire(1.162, 1158.0, 73.22, 0.0190, 0.0241),
            ConductorType::Rail      => Self::from_southwire(1.165, 1074.0, 83.67, 0.0180, 0.0225),
            ConductorType::Cardinal  => Self::from_southwire(1.196, 1227.0, 73.21, 0.0179, 0.0228),
            ConductorType::Ortolan   => Self::from_southwire(1.212, 1163.0, 83.67, 0.0167, 0.0209),
            ConductorType::Curlew    => Self::from_southwire(1.245, 1330.0, 73.21, 0.0165, 0.0211),
            ConductorType::Bluejay   => Self::from_southwire(1.258, 1253.0, 83.67, 0.0155, 0.0194),
            ConductorType::Finch     => Self::from_southwire(1.292, 1429.0, 73.72, 0.0154, 0.0197),
            ConductorType::Bunting   => Self::from_southwire(1.302, 1343.0, 83.67, 0.0144, 0.0182),
            ConductorType::Grackle   => Self::from_southwire(1.337, 1531.0, 73.72, 0.0144, 0.0184),
            ConductorType::Bittern   => Self::from_southwire(1.345, 1432.0, 83.67, 0.0135, 0.0171),
            ConductorType::Pheasant  => Self::from_southwire(1.381, 1633.0, 73.71, 0.0135, 0.0173),
            ConductorType::Dipper    => Self::from_southwire(1.386, 1521.0, 83.67, 0.0127, 0.0162),
            ConductorType::Martin    => Self::from_southwire(1.424, 1735.0, 73.72, 0.0127, 0.0163),
            ConductorType::Bobolink  => Self::from_southwire(1.427, 1611.0, 83.67, 0.0120, 0.0153),
            ConductorType::Lapwing   => Self::from_southwire(1.504, 1790.0, 83.67, 0.0108, 0.0139),
            ConductorType::Falcon    => Self::from_southwire(1.544, 2041.0, 73.72, 0.0108, 0.0140),
            ConductorType::Chukar    => Self::from_southwire(1.602, 2072.0, 81.35, 0.0097, 0.0125),
            ConductorType::Bluebird  => Self::from_southwire(1.762, 2508.0, 81.34, 0.00801,0.0105),
            ConductorType::Kiwi      => Self::from_southwire(1.735, 2300.0, 89.17, 0.00801,0.0106),
            
            // High Mechanical Strength (Page 4)
            ConductorType::Grouse    => Self::from_southwire(0.367, 149.0,  50.40, 0.2065, 0.2888),
            ConductorType::Petrel    => Self::from_southwire(0.461, 254.0,  37.80, 0.1583, 0.2493),
            ConductorType::Minorca   => Self::from_southwire(0.481, 276.0,  37.80, 0.1454, 0.2331),
            ConductorType::Leghorn   => Self::from_southwire(0.530, 336.0,  37.80, 0.1197, 0.2000),
            ConductorType::Guinea    => Self::from_southwire(0.576, 396.0,  37.80, 0.1014, 0.1757),
            ConductorType::Dotterel  => Self::from_southwire(0.607, 441.0,  37.80, 0.0911, 0.1618),
            ConductorType::Dorking   => Self::from_southwire(0.631, 476.0,  37.80, 0.0845, 0.1530),
            ConductorType::Brahma    => Self::from_southwire(0.714, 675.0,  28.40, 0.0764, 0.1499),
            ConductorType::Cochin    => Self::from_southwire(0.664, 527.0,  37.80, 0.0763, 0.1410),
            
            ConductorType::Custom(props) => *props,
        }
    }
}