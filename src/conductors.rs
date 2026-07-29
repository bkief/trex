#![allow(non_snake_case)]

/// Core properties of a physical conductor (all values in SI units)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConductorProperties {
    pub D: f64,                              // Conductor diameter (m)
    pub mass_kg_m: f64,                      // Conductor mass (kg/m)
    pub mCp: f64,                            // Conductor total heat capacity (J/m-C)
    pub R_T_high: f64,                       // Resistance at high temperature reference (ohm/m)
    pub R_T_low: f64,                        // Resistance at low temperature reference (ohm/m)
    pub T_high: f64,                         // High temperature reference (C)
    pub T_low: f64,                          // Low temperature reference (C)
    pub epsilon: f64,                        // Emissivity (0.0 to 1.0)
    pub alpha: f64,                          // Solar absorptivity (0.0 to 1.0)
    pub rated_strength_newtons: Option<f64>, // Rated breaking strength (N)
}

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

    /// Helper to generate IEEE 738 properties purely from SI Units.
    /// * `dia_m` - Complete Cable Diameter (meters)
    /// * `mass_kg_m` - Linear mass (kg/m)
    /// * `pct_al` - Aluminum Content by Weight (%)
    /// * `r_dc_20_ohm_m` - DC Resistance at 20C (Ohms/m)
    /// * `r_ac_75_ohm_m` - AC Resistance at 75C (Ohms/m)
    /// * `rated_strength_newtons` - Rated Breaking Strength (Newtons)
    fn from_si_data(dia_m: f64, mass_kg_m: f64, pct_al: f64, r_dc_20_ohm_m: f64, r_ac_75_ohm_m: f64, rated_strength_newtons: Option<f64>) -> ConductorProperties {
        // Standard Specific Heat values (J/kg-C)
        let cp_al = 897.0; 
        let cp_st = 476.0; 
        
        // Equivalent heat capacity for the composite core based on mass percentages
        let al_ratio = pct_al / 100.0;
        let st_ratio = 1.0 - al_ratio;
        let cp_eq = (al_ratio * cp_al) + (st_ratio * cp_st);
        
        ConductorProperties {
            D: dia_m,                     // meters
            mass_kg_m,                    // kg/m
            mCp: mass_kg_m * cp_eq,       // Total Heat Capacity (J/m-C)
            R_T_low: r_dc_20_ohm_m,       // Ohms/m
            R_T_high: r_ac_75_ohm_m,      // Ohms/m
            T_low: 20.0,                  // Base reference
            T_high: 75.0,                 // Base reference
            epsilon: 0.5,                 // Standard assumed emissivity
            alpha: 0.5,                   // Standard assumed absorptivity
            rated_strength_newtons,       // Newtons
        }
    }

    /// Returns the physical properties for the selected conductor
    pub fn properties(&self) -> ConductorProperties {
        // Default properties use alpha/epsilon of 0.5 (typical for new/moderate weathering).
        // Heavily weathered conductors often approach 0.8. Users can override these parameters 
        // by instantiating via standard enum, then directly modifying the returned struct.
        match self {
            //                         Dia(m)    Mass(kg/m) %Al     Rdc(ohm/m)   Rac(ohm/m)   Strength(N)
            ConductorType::Turkey    => Self::from_si_data(0.00503, 0.05357, 67.88, 0.00210302, 0.00264436, Some(5293.4)),
            ConductorType::Swan      => Self::from_si_data(0.00635, 0.08483, 67.87, 0.00132218, 0.00168963, Some(8273.7)),
            ConductorType::Swanate   => Self::from_si_data(0.00653, 0.09971, 58.10, 0.00130906, 0.00170276, Some(10497.8)),
            ConductorType::Sparrow   => Self::from_si_data(0.00803, 0.13542, 67.90, 0.00108924, 0.00140420, Some(12677.4)),
            ConductorType::Sparate   => Self::from_si_data(0.00826, 0.15923, 58.12, 0.00110892, 0.00143045, Some(15390.8)),
            ConductorType::Robin     => Self::from_si_data(0.00899, 0.17114, 67.88, 0.00082349, 0.00106627, Some(15791.2)),
            ConductorType::Raven     => Self::from_si_data(0.01011, 0.21578, 67.89, 0.00065945, 0.00085630, Some(19483.2)),
            ConductorType::Quail     => Self::from_si_data(0.01135, 0.27233, 67.88, 0.00052165, 0.00067913, Some(23620.0)),
            ConductorType::Pigeon    => Self::from_si_data(0.01275, 0.34228, 67.87, 0.00041339, 0.00054134, Some(29447.2)),
            ConductorType::Penguin   => Self::from_si_data(0.01430, 0.43305, 67.88, 0.00032808, 0.00042979, Some(37142.6)),
            ConductorType::Waxwing   => Self::from_si_data(0.01547, 0.43008, 86.43, 0.00021096, 0.00025820, Some(30603.8)),
            ConductorType::Partridge => Self::from_si_data(0.01631, 0.54615, 68.51, 0.00020899, 0.00025558, Some(50264.9)),
            ConductorType::Ostrich   => Self::from_si_data(0.01727, 0.61312, 68.51, 0.00018602, 0.00022736, Some(56492.4)),
            ConductorType::Merlin    => Self::from_si_data(0.01737, 0.54169, 86.43, 0.00016732, 0.00020505, Some(38610.6)),
            ConductorType::Linnet    => Self::from_si_data(0.01829, 0.68753, 68.51, 0.00016568, 0.00020276, Some(62720.0)),
            ConductorType::Oriole    => Self::from_si_data(0.01882, 0.78277, 60.35, 0.00016470, 0.00020112, Some(76954.2)),
            ConductorType::Chickadee => Self::from_si_data(0.01887, 0.64140, 86.43, 0.00014173, 0.00017356, Some(44215.3)),
            ConductorType::Brant     => Self::from_si_data(0.01961, 0.76045, 73.21, 0.00014108, 0.00017257, Some(64944.0)),
            ConductorType::Ibis      => Self::from_si_data(0.01989, 0.81254, 68.51, 0.00014042, 0.00017159, Some(72506.0)),
            ConductorType::Lark      => Self::from_si_data(0.02047, 0.92564, 60.35, 0.00013944, 0.00017028, Some(90298.9)),
            ConductorType::Pelican   => Self::from_si_data(0.02068, 0.66521, 86.44, 0.00011811, 0.00014501, Some(52489.0)),
            ConductorType::Flicker   => Self::from_si_data(0.02149, 0.91373, 73.21, 0.00011745, 0.00014403, Some(76509.4)),
            ConductorType::Hawk      => Self::from_si_data(0.02179, 0.97623, 68.51, 0.00011680, 0.00014304, Some(86740.3)),
            ConductorType::Hen       => Self::from_si_data(0.02243, 1.11017, 60.35, 0.00011614, 0.00014206, Some(105867.7)),
            ConductorType::Osprey    => Self::from_si_data(0.02233, 0.89736, 86.43, 0.00010105, 0.00012434, Some(60940.6)),
            ConductorType::Parakeet  => Self::from_si_data(0.02322, 1.06552, 73.21, 0.00010072, 0.00012336, Some(88074.8)),
            ConductorType::Dove      => Self::from_si_data(0.02355, 1.13844, 68.51, 0.00010039, 0.00012303, Some(100529.8)),
            ConductorType::Eagle     => Self::from_si_data(0.02421, 1.29619, 60.35, 0.00009941, 0.00012205, Some(120546.8)),
            ConductorType::Peacock   => Self::from_si_data(0.02421, 1.15928, 73.20, 0.00009252, 0.00011352, Some(98305.7)),
            ConductorType::Squab     => Self::from_si_data(0.02454, 1.23815, 68.51, 0.00009219, 0.00011319, Some(109871.1)),
            ConductorType::WoodDuck  => Self::from_si_data(0.02525, 1.40780, 60.35, 0.00009154, 0.00011220, Some(131222.5)),
            ConductorType::Teal      => Self::from_si_data(0.02525, 1.39738, 60.86, 0.00009154, 0.00011220, Some(129888.1)),
            ConductorType::Kingbird  => Self::from_si_data(0.02388, 1.02683, 86.43, 0.00008858, 0.00010892, Some(75619.8)),
            ConductorType::Swift     => Self::from_si_data(0.02362, 0.95689, 92.72, 0.00008891, 0.00010958, Some(85405.9)),
            ConductorType::Rook      => Self::from_si_data(0.02482, 1.21731, 73.22, 0.00008793, 0.00010827, Some(101864.3)),
            ConductorType::Grosbeak  => Self::from_si_data(0.02517, 1.30065, 68.51, 0.00008760, 0.00010761, Some(112095.2)),
            ConductorType::Scoter    => Self::from_si_data(0.02588, 1.48072, 60.35, 0.00008399, 0.00010663, Some(137005.2)),
            ConductorType::Egret     => Self::from_si_data(0.02588, 1.46881, 60.85, 0.00008727, 0.00010696, Some(137894.9)),
            ConductorType::Flamingo  => Self::from_si_data(0.02540, 1.27684, 73.21, 0.00008399, 0.00010335, Some(114319.3)),
            ConductorType::Gannet    => Self::from_si_data(0.02576, 1.36315, 68.51, 0.00008366, 0.00010269, Some(122326.1)),
            ConductorType::Stilt     => Self::from_si_data(0.02631, 1.36911, 73.21, 0.00007841, 0.00009646, Some(118322.7)),
            ConductorType::Starling  => Self::from_si_data(0.02670, 1.46435, 68.51, 0.00007808, 0.00009580, Some(127219.1)),
            ConductorType::Redwing   => Self::from_si_data(0.02746, 1.65186, 60.85, 0.00007743, 0.00009514, Some(152129.2)),
            ConductorType::Coot      => Self::from_si_data(0.02642, 1.19648, 92.72, 0.00007119, 0.00008793, Some(116543.4)),
            ConductorType::Drake     => Self::from_si_data(0.02812, 1.62656, 68.51, 0.00007021, 0.00008629, Some(140119.0)),
            ConductorType::Tern      => Self::from_si_data(0.02700, 1.33190, 83.67, 0.00007087, 0.00008825, Some(97860.9)),
            ConductorType::Condor    => Self::from_si_data(0.02774, 1.52239, 73.21, 0.00007054, 0.00008924, Some(126329.5)),
            ConductorType::Mallard   => Self::from_si_data(0.02896, 1.83639, 60.86, 0.00006988, 0.00008563, Some(169922.1)),
            ConductorType::Ruddy     => Self::from_si_data(0.02873, 1.50751, 83.67, 0.00006266, 0.00007841, Some(130332.9)),
            ConductorType::Canary    => Self::from_si_data(0.02952, 1.72329, 73.22, 0.00006234, 0.00007907, Some(116543.4)),
            ConductorType::Rail      => Self::from_si_data(0.02959, 1.59828, 83.67, 0.00005906, 0.00007382, Some(136560.4)),
            ConductorType::Cardinal  => Self::from_si_data(0.03038, 1.82597, 73.21, 0.00005873, 0.00007480, Some(150349.9)),
            ConductorType::Ortolan   => Self::from_si_data(0.03079, 1.73073, 83.67, 0.00005479, 0.00006857, Some(143232.7)),
            ConductorType::Curlew    => Self::from_si_data(0.03162, 1.97925, 73.21, 0.00005413, 0.00006923, Some(169922.1)),
            ConductorType::Bluejay   => Self::from_si_data(0.03195, 1.86466, 83.67, 0.00005085, 0.00006365, Some(152574.0)),
            ConductorType::Finch     => Self::from_si_data(0.03282, 2.12658, 73.72, 0.00005052, 0.00006463, Some(172146.2)),
            ConductorType::Bunting   => Self::from_si_data(0.03307, 1.99860, 83.67, 0.00004724, 0.00005971, Some(162360.1)),
            ConductorType::Grackle   => Self::from_si_data(0.03396, 2.27837, 73.72, 0.00004724, 0.00006037, Some(183266.7)),
            ConductorType::Bittern   => Self::from_si_data(0.03416, 2.13104, 83.67, 0.00004429, 0.00005610, Some(172591.0)),
            ConductorType::Pheasant  => Self::from_si_data(0.03508, 2.43016, 73.71, 0.00004429, 0.00005676, Some(196611.4)),
            ConductorType::Dipper    => Self::from_si_data(0.03520, 2.26349, 83.67, 0.00004167, 0.00005315, Some(182377.1)),
            ConductorType::Martin    => Self::from_si_data(0.03617, 2.58196, 73.72, 0.00004167, 0.00005348, Some(207287.1)),
            ConductorType::Bobolink  => Self::from_si_data(0.03625, 2.39743, 83.67, 0.00003937, 0.00005020, Some(192608.0)),
            ConductorType::Lapwing   => Self::from_si_data(0.03820, 2.66381, 83.67, 0.00003543, 0.00004560, Some(215738.7)),
            ConductorType::Falcon    => Self::from_si_data(0.03922, 3.03734, 73.72, 0.00003543, 0.00004593, Some(244652.2)),
            ConductorType::Chukar    => Self::from_si_data(0.04069, 3.08347, 81.35, 0.00003182, 0.00004101, Some(226859.3)),
            ConductorType::Bluebird  => Self::from_si_data(0.04476, 3.73231, 81.34, 0.00002628, 0.00003445, Some(270451.9)),
            ConductorType::Kiwi      => Self::from_si_data(0.04407, 3.42277, 89.17, 0.00002628, 0.00003478, Some(236200.6)),
            
            // High Mechanical Strength (Page 4)
            ConductorType::Grouse    => Self::from_si_data(0.00932, 0.22174, 50.40, 0.00067749, 0.00094751, Some(23130.8)),
            ConductorType::Petrel    => Self::from_si_data(0.01171, 0.37799, 37.80, 0.00051936, 0.00081791, Some(50709.7)),
            ConductorType::Minorca   => Self::from_si_data(0.01222, 0.41073, 37.80, 0.00047703, 0.00076476, Some(55158.0)),
            ConductorType::Leghorn   => Self::from_si_data(0.01346, 0.50002, 37.80, 0.00039272, 0.00065617, Some(67168.1)),
            ConductorType::Guinea    => Self::from_si_data(0.01463, 0.58931, 37.80, 0.00033268, 0.00057644, Some(79178.3)),
            ConductorType::Dotterel  => Self::from_si_data(0.01542, 0.65628, 37.80, 0.00029888, 0.00053084, Some(88074.8)),
            ConductorType::Dorking   => Self::from_si_data(0.01603, 0.70836, 37.80, 0.00027723, 0.00050197, Some(95192.0)),
            ConductorType::Brahma    => Self::from_si_data(0.01814, 1.00451, 28.40, 0.00025066, 0.00049180, Some(146791.3)),
            ConductorType::Cochin    => Self::from_si_data(0.01687, 0.78426, 37.80, 0.00025033, 0.00046260, Some(105422.9)),
            
            ConductorType::Custom(props) => *props,
        }
    }
}