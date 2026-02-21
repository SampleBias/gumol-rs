# gumol - GPU-Accelerated MD SIGFPE Bug Fix Activity Log

## 2026-02-21 10:36 - Project Initialization
- Created project structure files
- Initialized todo.md with project template
- Initialized activity.md for logging
- Generated PROJECT_README.md for context tracking

## 2026-02-21 10:45-11:05 - Bug Investigation and Fixes
- Analyzed the gumol codebase structure (gumol-core, gumol-sim, gumol-gpu)
- Identified multiple potential SIGFPE (Floating Point Exception) sources:
  1. **Temperature::compute()**: Division by zero when degrees_of_freedom = 0
  2. **MolecularVirial**: Division by zero when particles overlap (r_ab.norm2() ≈ 0)
  3. **velocities::scale()**: Division by zero when instant_temperature ≈ 0
  4. **BerendsenThermostat**: Division by zero when instant_temperature ≈ 0
  5. **CSVRThermostat**: Division by zero when kinetic_energy ≈ 0
  6. **Integrators (VelocityVerlet, Verlet, LeapFrog)**: Division by zero for zero mass particles
  7. **PressureAtTemperature & StressAtTemperature**: Division by zero for zero/negative volume
  8. **Vector3D::normalized()**: Division by zero for near-zero vectors
  9. **UnitCell::vector_image()**: Division by zero for zero cell dimensions
  10. **UnitExpr::eval()**: Division by zero in unit conversion
  11. **RemoveTranslation control**: Division by zero for zero total_mass
  12. **Matrix3::inverse()**: Division by zero for near-zero determinant (in RemoveRotation)

- Applied defensive checks to all identified locations
- Created minimal test cases to isolate the issue
- Confirmed SIGFPE occurs in both GPU and CPU versions (not GPU-specific)

### Files Modified
- `gumol-core/src/sys/compute.rs` - Temperature, MolecularVirial, PressureAtTemperature, StressAtTemperature
- `gumol-core/src/types/vectors.rs` - Vector3D::normalized()
- `gumol-core/src/sys/config/cells.rs` - UnitCell::vector_image()
- `gumol-core/src/units.rs` - UnitExpr::eval()
- `gumol-sim/src/velocities.rs` - scale() function
- `gumol-sim/src/md/thermostats.rs` - BerendsenThermostat, CSVRThermostat
- `gumol-sim/src/md/integrators.rs` - VelocityVerlet, Verlet, LeapFrog
- `gumol-sim/src/md/controls.rs` - RemoveTranslation control

### Key Findings
1. The SIGFPE is NOT GPU-specific - it occurs in the core simulation code
2. Multiple edge cases can trigger division by zero during MD simulation
3. The RemoveRotation control calls matrix.inverse() which can fail with near-zero inertia matrix
4. Defensive checks added prevent SIGFPE by checking denominators before division

---
*Activity logging format:*
*## YYYY-MM-DD HH:MM - Action Description*
*- Detailed description of what was done*
*- Files created/modified*
*- Commands executed*
*- Any important notes or decisions*
