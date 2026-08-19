# TREX: Transmission Rating Explorer

[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://opensource.org/licenses/MPL-2.0)
[![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Target: WASM](https://img.shields.io/badge/Target-WebAssembly-purple.svg)](https://webassembly.org/)
[![Target: Python](https://img.shields.io/badge/Target-Python_3-blue.svg)](https://www.python.org/)

**TREX (Transmission Rating Explorer)** is a high-performance library written in Rust with bindings for **WebAssembly (WASM)** and **Python**. It provides core engineering calculations for overhead electrical power transmission lines based on the **IEEE 738 standard** and exact **Newton-Raphson catenary change-of-state** algorithms.

---

## Key Features

- **IEEE 738 Thermal & Ampacity Calculations**:
  - **Steady-State Ampacity**: Calculate maximum allowable current for target conductor operating temperatures under given environmental conditions.
  - **Steady-State Temperature**: Solve for conductor operating equilibrium temperature for a given load current.
  - **Transient Thermal Simulation**: Time-step numerical integration for transient conductor heating under sudden load steps or emergency ampacity increases.
- **Solar Irradiance & Atmospheric Modeling**:
  - Clear and Industrial atmosphere models based on IEEE 738 Tables 4 & 5.
  - Elevation solar radiation correction.
  - Astronomical solar flux calculation based on latitude, day of year, hour of day, and line azimuth.
  - 24-hour diurnal solar irradiance curve generation.
- **Catenary Sag & Tension Analysis**:
  - Newton-Raphson iterative solver for catenary change-of-state equations under thermal expansion.
  - Calculation of initial sag vs. operating thermal sag, ground clearance, and operating horizontal tension (% RTS - Rated Tensile Strength).
  - High-resolution 2D catenary curve coordinates for visualization.
- **Conductor Database**:
  - Built-in database containing specifications for **Sampling of 95+ standard overhead conductors** (e.g., Drake, Hawk, Dove, Cardinal, Falcon, Osprey, Partridge, etc.).
  - Support for user-defined custom conductor geometry, resistance parameters, heat capacity, and rated breaking strength.
- **Cross-Platform & Multi-Language**:
  - Native Rust library.
  - WebAssembly ES Module output for browser/web-based rating visualization dashboards.
  - Native Python C-extension module (via `pyo3`).

---

## Repository Structure

```
trex/
├── .github/
│   └── workflows/
│       ├── ci.yml      # CI workflow for Rust tests & Python wheel installation check
│       └── release.yml # GitHub Actions workflow to build multi-platform Python wheels & GitHub Releases
├── Cargo.toml          # Rust package configuration & dependencies (wasm-bindgen, pyo3)
├── Cargo.lock          # Dependency lockfile
├── pyproject.toml      # Maturin build configuration for Python wheels
├── build.ps1           # PowerShell script to run unit tests and build WASM bindings
├── test_trex.py        # Python test and demonstration script
├── LICENSE             # Mozilla Public License v2.0 (MPL-2.0)
├── README.md           # Documentation
└── src/
    ├── lib.rs          # Main crate entrypoint with WASM & Python module bindings
    ├── conductors.rs   # Conductor database & ConductorProperties definitions
    ├── ieee738.rs      # IEEE 738 thermal equations, solar flux, & transient solver
    └── sag.rs          # Newton-Raphson catenary sag & tension change-of-state solver
```

---

## Installation & Building

### Prerequisites
- **Rust Toolchain**: [Install Rust](https://www.rust-lang.org/tools/install) (Edition 2021)
- **wasm-bindgen CLI** (for WASM output): `cargo install wasm-bindgen-cli`
- **Python 3.x** and `maturin` or `setuptools-rust` (for Python bindings)

### Building the Rust Library & Running Tests
```bash
cargo build --release
cargo test
```

### Building the WebAssembly (WASM) Module
Run the provided PowerShell script or execute the Cargo build manually:

```powershell
.\build.ps1
```

Or step-by-step:
```bash
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/trex.wasm --out-dir pkg --target web --no-typescript
```
The compiled WASM module and JavaScript wrapper will be exported to `./pkg/`.

### Building Python Bindings
To build the native Python extension module:
```bash
cargo build --release --features python
```
Or use `maturin`:
```bash
maturin develop --features python
```

---

## Usage Examples

### 1. Python Example

```python
import trex

# 1. Inspect available conductors
conductors = trex.get_conductor_list()
print(f"Available Conductors ({len(conductors)}): {conductors[:5]}...")

# 2. Instantiate a Conductor
# You can use a built-in conductor by name...
drake = trex.Conductor("Drake")

# ...or define a custom conductor with explicit properties
custom_conductor = trex.Conductor.custom(
    name="MyCustomConductor",
    diameter=0.02814,           # m
    mass_kg_m=1.628,            # kg/m
    heat_capacity=930.0,        # J/(kg*°C)
    r_t_high=0.0000728,         # Ohm/m
    r_t_low=0.0000624,          # Ohm/m
    t_high=75.0,                # °C
    t_low=25.0,                 # °C
    epsilon=0.5,
    alpha=0.5,
    rated_strength_newtons=140000.0
)

# 3. Calculate Steady-State Ampacity (IEEE 738 Reference Example)
ampacity = trex.calculate_ampacity(
    conductor=drake,           # Pass the conductor instance
    max_conductor_temp=100.0,  # °C (Target max temp)
    ambient_temp=40.0,         # °C (IEEE 738 reference ambient)
    wind_speed=0.61,           # m/s (~2 ft/s crosswind)
    wind_angle_deg=90.0,       # deg
    elevation=0.0,             # m (Sea level)
    solar_radiation=1000.0     # W/m² (Clear day noon)
)
print(f"Drake Ampacity @ 100°C (IEEE Reference): {ampacity:.1f} Amperes")

# 4. Simulate Transient Conductor Heating
sim_temps = trex.simulate_transient_temp(
    conductor=drake,
    t_ambient=25.0,
    wind_speed=0.61,
    wind_angle_deg=90.0,
    elevation=0.0,
    solar_radiation=1000.0,
    initial_current=500.0,    # A
    stepped_current=1200.0,   # A
    step_time_mins=0.0,       # mins
    duration_mins=60.0        # mins
)
print(f"Transient Heating: Min 0={sim_temps[0]:.1f}°C -> Min 30={sim_temps[30]:.1f}°C -> Min 60={sim_temps[60]:.1f}°C")

# 5. Calculate Conductor Sag & Ground Clearance (Newton-Raphson Catenary)
sag_result = trex.calculate_conductor_sag(
    conductor=custom_conductor,         # Works with custom conductors too!
    span_length=250.0,                  # m
    initial_tension_percent_rts=20.0,   # % RTS
    conductor_temp=sim_temps[30],       # °C
    ref_temp=15.0,                      # °C
    structure_height=25.0,              # m
)

print(f"Sag Results:")
print(f"  Operating Sag:      {sag_result.operating_sag:.2f} m")
print(f"  Ground Clearance:   {sag_result.clearance:.2f} m")
print(f"  Operating Tension:  {sag_result.operating_tension_percent_rts:.1f}% RTS")
```

### 2. JavaScript / WebAssembly Example

```javascript
import init, { Conductor, calculate_ampacity } from './pkg/trex.js';

async function run() {
  await init();

  // Instantiate the Conductor
  const drake = new Conductor("Drake");

  // Calculate Ampacity in Browser
  const ampacity = calculate_ampacity(
    drake,  // conductor instance
    100.0,  // t_max (°C)
    25.0,   // t_ambient (°C)
    0.61,   // wind_speed (m/s)
    90.0,   // wind_angle (deg)
    0.0,    // elevation (m)
    1000.0, // solar_radiation (W/m²)
    0.5,    // emissivity
    0.5     // absorptivity
  );
  console.log(`Drake Ampacity: ${ampacity} A`);
}

run();
```

---

## CI/CD & Automated Python Wheel Releases

The repository includes GitHub Actions workflows dedicated exclusively to testing and releasing **Python wheels** (WASM artifacts are excluded from release workflow):

- **Automated Multi-Platform Wheel Builds**: Builds native C-extension wheels for **Linux** (`x86_64`, `aarch64`, `i686`), **Windows** (`x64`, `x86`), and **macOS** (`x86_64`, `aarch64` / Apple Silicon) using `PyO3/maturin-action`.
- **Source Distribution (`sdist`)**: Builds `.tar.gz` source package.
- **GitHub Releases & Package Attachment**: Automatically creates a GitHub Release when pushing a git tag matching `v*` (e.g. `v0.1.0`) or when triggered manually via `workflow_dispatch`, attaching all `.whl` and `.tar.gz` files.

To create a new Python wheel release:
```bash
git tag v0.1.0
git push origin v0.1.0
```

---

## API Summary

| Function | Description |
| :--- | :--- |
| `get_conductor_list()` | Returns array of standard conductor names available in the database. |
| `get_conductor_details(name)` | Fetches conductor physical properties (diameter, heat capacity, resistance). |
| `calculate_ampacity(...)` | Solves steady-state ampacity (Current $I$ in Amperes) per IEEE 738. |
| `calculate_steady_state_temp(...)` | Solves equilibrium operating temperature ($^\circ\text{C}$) for a given current load. |
| `simulate_transient_temp(...)` | Simulates minute-by-minute transient thermal dynamic response to load changes. |
| `calculate_solar_radiation(...)` | Calculates solar radiation heat flux ($Q_{se}$) using clear/industrial tables. |
| `calculate_astronomical_solar(...)` | Computes solar flux from latitude, day of year, hour of day, and elevation. |
| `calculate_daily_solar_curve(...)` | Generates 24-hour diurnal solar irradiance vector ($W/m^2$). |
| `calculate_conductor_sag(...)` | Solves catenary change-of-state for sag, tension (% RTS), clearance, and 2D profile points. |

---

## License

This project is licensed under the **Mozilla Public License v2.0** (MPL-2.0-or-later). See the [LICENSE](./LICENSE) file for full details.
