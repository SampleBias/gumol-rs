# gumol - GPU-Accelerated MD SIGFPE Bug Fix Todo List

## Project Analysis
- [x] Create project structure files
- [x] Initialize todo.md, activity.md, and PROJECT_README.md
- [x] Read and understand the GPU-accelerated Argon MD example
- [x] Analyze gumol-gpu module (kernels, memory, dispatcher, profile)
- [x] Understand ForceProvider trait and System::forces() path
- [x] Identify root cause of SIGFPE (Floating Point Exception)
- [x] Determine if issue is in CUDA kernel or CPU path
- [x] Review temperature computation and degrees_of_freedom calculation

## Root Cause Investigation
- [x] Test argon_gpu example to reproduce SIGFPE
- [x] Analyze Temperature::compute() formula: T = 2*K/(Kb*dof)
- [x] Check degrees_of_freedom() for Argon system (particles vs molecules)
- [x] Verify kinetic energy computation
- [x] Check GPU kernel for division by zero issues
- [x] Review ForceProvider fallback logic in GpuForceProvider
- [x] Test both GPU and CPU-only versions
- [x] Identify multiple SIGFPE sources across codebase

## Bug Fix Implementation
- [x] Fix Temperature::compute() division by zero for dof < 1
- [x] Fix MolecularVirial division by zero for overlapping particles
- [x] Fix scale() velocity scaling division by zero for near-zero temperature
- [x] Fix BerendsenThermostat division by zero for near-zero temperature
- [x] Fix CSVRThermostat division by zero for near-zero kinetic energy
- [x] Fix VelocityVerlet division by zero for zero mass particles
- [x] Fix Verlet integrator division by zero for zero mass particles
- [x] Fix LeapFrog integrator division by zero for zero mass particles
- [x] Fix PressureAtTemperature division by zero for zero/negative volume
- [x] Fix StressAtTemperature division by zero for zero/negative volume
- [x] Fix Vector3D::normalized() division by zero for near-zero vectors
- [x] Fix UnitCell::vector_image() division by zero for zero cell dimensions
- [x] Fix UnitExpr::eval() division by zero in unit conversion
- [x] Fix RemoveTranslation control division by zero for zero total mass
- [x] Fix chemfiles velocity/position copy-paste bug in System -> Frame conversion

## Remaining Issues
- [x] Investigated TrajectoryOutput SIGFPE in chemfiles library
- [x] Implemented workaround: SimpleXYZOutput to bypass chemfiles
- [x] Successfully tested workaround with full MD simulation

## Verification
- [x] Run argon_gpu example successfully
- [x] Run CPU-only example for comparison
- [x] Check energy.dat output for reasonable values
- [x] Verify temperature stays around 300K
- [x] Test with different atom counts

## Documentation
- [x] Add reference to SIGFPE_FIX.md in README.md (docs/SIGFPE_FIX.md exists but not linked)
- [x] Add comments explaining the division-by-zero protection (all locations documented in code)
- [x] Update test cases if needed (examples/test_*.rs created and tested successfully)

## Review Section
*This section will be updated upon completion with a summary of all changes made during the session.*

---
*Created: 2026-02-21 10:36*
*Last Updated: 2026-02-21 10:36*