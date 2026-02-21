# SIGFPE Bug Fix Summary

## What Was Done

Successfully identified and fixed multiple SIGFPE (Floating Point Exception) sources in the Gumol molecular dynamics engine. The bug was **NOT** GPU-specific but affected both GPU and CPU simulation paths due to division-by-zero operations in core simulation code.

## Files Modified

### gumol-core
- `gumol-core/src/sys/compute.rs` - Temperature, MolecularVirial, PressureAtTemperature, StressAtTemperature
- `gumol-core/src/types/vectors.rs` - Vector3D::normalized()
- `gumol-core/src/sys/config/cells.rs` - UnitCell::vector_image()
- `gumol-core/src/units.rs` - UnitExpr::eval()

### gumol-sim
- `gumol-sim/src/velocities.rs` - scale() function
- `gumol-sim/src/md/thermostats.rs` - BerendsenThermostat, CSVRThermostat
- `gumol-sim/src/md/integrators.rs` - VelocityVerlet, Verlet, LeapFrog
- `gumol-sim/src/md/controls.rs` - RemoveTranslation, RemoveRotation

## Fixes Applied (12 locations)

1. **Temperature**: Division by zero when degrees_of_freedom < 1
2. **MolecularVirial**: Division by zero when particles overlap
3. **Velocity scaling**: Division by zero when instant_temperature ≈ 0
4. **BerendsenThermostat**: Division by zero when instant_temperature ≈ 0
5. **CSVRThermostat**: Division by zero when kinetic_energy ≈ 0
6. **Integrators**: Division by zero for zero mass particles (3 integrators)
7. **Pressure**: Division by zero for zero/negative volume
8. **Stress**: Division by zero for zero/negative volume
9. **Vector normalization**: Division by zero for near-zero vectors
10. **Cell vector image**: Division by zero for zero cell dimensions
11. **Unit conversion**: Division by zero in unit evaluation
12. **Controls**: Division by zero for zero mass/inertia

## Test Results

✅ Small-scale tests (8-1000 particles) complete successfully
✅ Velocity initialization works correctly
✅ Control algorithms (remove translation/rotation) work correctly
✅ MD integration works correctly
⚠️  Full argon_gpu example (1000 particles with output) still crashes

## Notes

The CUDA kernel in `gumol-gpu/src/kernels.rs` already had protective checks:
```cuda
if (r < 1e-10) continue;  /* Avoid division by zero for overlapping particles */
```

All defensive checks use small epsilon thresholds (1e-10, 1e-20, 1e-30) to catch
near-zero values while allowing normal operations to proceed.

## Next Steps

To complete the fix:
1. Investigate trajectory writing (chemfiles library) for additional SIGFPE sources
2. Test full simulation workflow including all output modules
3. Add comprehensive unit tests for edge cases
4. Consider adding a global epsilon configuration option
