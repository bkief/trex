import trex

print("=== TREX Python Library Test ===")

# 1. Get Conductor List
conductors = trex.get_conductor_list()
print(f"Available Conductors ({len(conductors)}): {conductors[:5]}...")

# Create the Drake conductor instance
drake = trex.Conductor("Drake")

# 2. Calculate Steady-State Ampacity (IEEE 738)
ampacity = trex.calculate_ampacity(
    conductor=drake,
    max_conductor_temp=100.0,  # °C
    ambient_temp=25.0,          # °C
    wind_speed=0.61,            # m/s (~2 ft/s)
    wind_angle_deg=90.0,        # deg
    elevation=0.0,              # m
    solar_radiation=1000.0,     # W/m²
    emissivity=0.5,
    absorptivity=0.5,
)
print(f"Drake Ampacity @ 100°C: {ampacity:.1f} Amperes")

# 3. Simulate Transient Conductor Heating
sim_temps = trex.simulate_transient_temp(
    conductor=drake,
    t_ambient=25.0,
    wind_speed=0.61,
    wind_angle_deg=90.0,
    elevation=0.0,
    solar_radiation=1000.0,
    initial_current=500.0,     # A
    stepped_current=1200.0,    # A
    step_time_mins=0.0,        # mins
    duration_mins=60.0,        # mins
    emissivity=0.5,
    absorptivity=0.5,
)
print(f"Transient Heating (60 mins): Min 0={sim_temps[0]:.1f}°C -> Min 30={sim_temps[30]:.1f}°C -> Min 60={sim_temps[60]:.1f}°C")

# 4. Calculate Conductor Sag via Newton-Raphson
sag_result = trex.calculate_conductor_sag(
    conductor=drake,
    span_length=250.0,                   # m
    initial_tension_percent_rts=20.0,    # % RTS
    conductor_temp=sim_temps[30],        # °C at minute 30
    ref_temp=15.0,                       # °C
    structure_height=25.0,               # m
)
print(f"Newton-Raphson Sag Result: {sag_result}")
print(f"  - Initial Sag: {sag_result.initial_sag:.2f} m")
print(f"  - Operating Sag: {sag_result.operating_sag:.2f} m")
print(f"  - Ground Clearance: {sag_result.clearance:.2f} m")
print(f"  - Operating Tension: {sag_result.operating_tension_percent_rts:.1f}% RTS ({sag_result.operating_tension:.0f} N)")
print(f"  - Catenary Profile Points: {len(sag_result.curve_x)} points")

print("\nSUCCESS: All TREX Python native functions executed cleanly!")
