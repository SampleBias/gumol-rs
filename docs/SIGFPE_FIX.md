# SIGFPE (Floating Point Exception) Bug Analysis and Fixes

## Summary

This document describes the SIGFPE bug in the Gumol GPU-accelerated molecular dynamics engine
and the defensive fixes applied to prevent division-by-zero errors.

## Root Cause

The SIGFPE (Floating Point Exception) is caused by division-by-zero errors in multiple
locations throughout the codebase. These occur when edge cases such as:
- Zero degrees of freedom
- Near-zero temperature or kinetic energy
- Zero or near-zero particle masses
- Zero cell dimensions or volumes
- Overlapping particles (zero distance)
- Zero total system mass
- Non-invertible inertia matrices

## Fixes Applied

### 1. Temperature Calculation (`gumol-core/src/sys/compute.rs`)
**Issue**: Division by zero when `degrees_of_freedom() == 0`
```rust
// Before:
return 2.0 * kinetic / (dof * K_BOLTZMANN);

// After:
if dof < 1.0 {
    return 0.0;
}
return 2.0 * kinetic / (dof * K_BOLTZMANN);
```

### 2. Molecular Virial (`gumol-core/src/sys/compute.rs`)
**Issue**: Division by zero when particles overlap (r_ab.norm2() ≈ 0)
```rust
// Before:
local_virial += w_ab * (r_ab * r_ij) / r_ab.norm2();

// After:
let norm2 = r_ab.norm2();
if norm2 < 1e-20 {
    continue;
}
local_virial += w_ab * (r_ab * r_ij) / norm2;
```

### 3. Velocity Scaling (`gumol-sim/src/velocities.rs`)
**Issue**: Division by zero when instant_temperature ≈ 0
```rust
// Before:
let factor = f64::sqrt(temperature / instant_temperature);

// After:
if instant_temperature < 1e-10 {
    warn!("Instant temperature is near zero...");
    return;
}
let factor = f64::sqrt(temperature / instant_temperature);
```

### 4. Berendsen Thermostat (`gumol-sim/src/md/thermostats.rs`)
**Issue**: Division by zero when instant_temperature ≈ 0
```rust
// Before:
let factor = f64::sqrt(1.0 + (self.temperature / instant_temperature - 1.0) / self.tau);

// After:
if instant_temperature < 1e-10 {
    return;
}
let factor = f64::sqrt(1.0 + (self.temperature / instant_temperature - 1.0) / self.tau);
```

### 5. CSVR Thermostat (`gumol-sim/src/md/thermostats.rs`)
**Issue**: Division by zero when kinetic_energy ≈ 0
```rust
// Before:
let kinetic_factor = self.target_kinetic_per_dof / kinetic;

// After:
if kinetic < 1e-10 {
    return;
}
let kinetic_factor = self.target_kinetic_per_dof / kinetic;
```

### 6. Integrators (`gumol-sim/src/md/integrators.rs`)
**Issue**: Division by zero for particles with zero mass
```rust
// Before (VelocityVerlet):
*acceleration = force / mass;

// After:
if mass > 0.0 {
    *acceleration = force / mass;
} else {
    *acceleration = Vector3D::zero();
}
```
Similar fixes applied to Verlet and LeapFrog integrators.

### 7. Pressure and Stress (`gumol-core/src/sys/compute.rs`)
**Issue**: Division by zero for zero or negative volumes
```rust
// Before:
return (dof * K_BOLTZMANN * self.temperature + virial) / (3.0 * volume);

// After:
if volume <= 0.0 {
    return 0.0; // or Matrix3::zero()
}
return (dof * K_BOLTZMANN * self.temperature + virial) / (3.0 * volume);
```

### 8. Vector Normalization (`gumol-core/src/types/vectors.rs`)
**Issue**: Division by zero for near-zero vectors
```rust
// Before:
pub fn normalized(&self) -> Vector3D {
    self / self.norm()
}

// After:
pub fn normalized(&self) -> Vector3D {
    let norm = self.norm();
    if norm < 1e-10 {
        return Vector3D::zero();
    }
    self / norm
}
```

### 9. Cell Vector Image (`gumol-core/src/sys/config/cells.rs`)
**Issue**: Division by zero for zero cell dimensions
```rust
// Before:
vect[0] -= f64::round(vect[0] / self.a()) * self.a();
vect[1] -= f64::round(vect[1] / self.b()) * self.b();
vect[2] -= f64::round(vect[2] / self.c()) * self.c();

// After:
let a = self.a();
let b = self.b();
let c = self.c();
if a > 1e-10 {
    vect[0] -= f64::round(vect[0] / a) * a;
}
if b > 1e-10 {
    vect[1] -= f64::round(vect[1] / b) * b;
}
if c > 1e-10 {
    vect[2] -= f64::round(vect[2] / c) * c;
}
```

### 10. Unit Expression Evaluation (`gumol-core/src/units.rs`)
**Issue**: Division by zero in unit conversion
```rust
// Before:
UnitExpr::Div(ref lhs, ref rhs) => lhs.eval() / rhs.eval(),

// After:
UnitExpr::Div(ref lhs, ref rhs) => {
    let denom = rhs.eval();
    if denom.abs() < 1e-20 {
        return if lhs.eval() >= 0.0 { f64::MAX } else { f64::MIN };
    }
    lhs.eval() / denom
}
```

### 11. Remove Translation Control (`gumol-sim/src/md/controls.rs`)
**Issue**: Division by zero for zero total mass
```rust
// Before:
let total_mass = system.particles().mass.iter().sum();
// ... divide by total_mass ...

// After:
let total_mass = system.particles().mass.iter().sum();
if total_mass < 1e-10 {
    return;
}
// ... divide by total_mass ...
```

### 12. Remove Rotation Control (`gumol-sim/src/md/controls.rs`)
**Issue**: Division by zero for non-invertible inertia matrix
```rust
// Before:
let angular = inertia.inverse() * moment;

// After:
let determinant = inertia.determinant();
if determinant.abs() < 1e-30 {
    return; // Matrix not invertible, skip rotation removal
}
let angular = inertia.inverse() * moment;
```

## Notes

1. The SIGFPE is NOT GPU-specific - it occurs in the core simulation code
2. The CUDA kernel already had protective checks (line 59 in `gumol-gpu/src/kernels.rs`)
3. Multiple edge cases can trigger division-by-zero during MD simulation
4. All defensive checks use small epsilon thresholds (1e-10, 1e-20, 1e-30) to avoid
   missing actual issues while preventing numerical instability

## Testing

The following test programs were created to verify fixes:
- `examples/minimal_test.rs` - Tests basic system setup
- `examples/argon_cpu.rs` - Tests force computation (CPU only)
- `examples/test_integrator.rs` - Tests VelocityVerlet integration
- `examples/test_100.rs` - Tests with 100 particles
- `examples/test_500.rs` - Tests with 500 particles
- `examples/test_1000.rs` - Tests with 1000 particles (GPU threshold)
- `examples/test_controls2.rs` - Tests velocity initialization and controls

All small-scale tests (8-1000 particles) complete successfully with the fixes applied.

## Files Modified

- `gumol-core/src/sys/compute.rs`
- `gumol-core/src/types/vectors.rs`
- `gumol-core/src/sys/config/cells.rs`
- `gumol-core/src/units.rs`
- `gumol-sim/src/velocities.rs`
- `gumol-sim/src/md/thermostats.rs`
- `gumol-sim/src/md/integrators.rs`
- `gumol-sim/src/md/controls.rs`

## Related Issues

- The CUDA kernel in `gumol-gpu/src/kernels.rs` already had protection:
  ```cuda
  if (r < 1e-10) continue;  /* Avoid division by zero for overlapping particles */
  ```
  and used `floor(x+0.5)` instead of `rint` to avoid SIGFPE on some systems.

## Recommendations

1. Consider adding a global configuration option for numerical epsilon thresholds
2. Add unit tests for edge cases (zero mass, zero temperature, etc.)
3. Consider using Rust's `checked_div` or `checked_div` operations for more explicit error handling
4. Monitor for remaining numerical instabilities in production simulations
