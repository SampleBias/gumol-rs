# Gumol - GPU-Accelerated Radiation Simulation Engine

Gumol is a GPU-accelerated molecular simulation engine designed for studying
the effects of ionizing radiation on biological systems, with particular focus
on Extracellular Superoxide Dismutase (EC-SOD) protection in space radiation
environments.

**Based on**: [Lumol](https://github.com/lumol-org/lumol) (BSD-3-Clause license)

## Overview

Gumol extends the flexible molecular simulation framework of Lumol with specialized
GPU acceleration and radiation-specific algorithms. The engine is designed to:

- Simulate radiation damage to DNA, proteins, and cell membranes
- Model EC-SOD protective mechanisms in space environments
- Provide high-performance computation through adaptive CPU/GPU acceleration
- Support radiation physics at the molecular level

## Key Features

- **Hybrid CPU/GPU Acceleration**: Automatic selection between CPU and GPU compute paths based on system size and characteristics
- **CUDA Support**: Optimized kernels for NVIDIA GPUs (Turing architecture and newer)
- **Radiation Physics**: Specialized algorithms for modeling ionizing radiation damage
- **Cell-Linked Lists**: O(N) neighbor list construction for efficient force calculations
- **EC-SOD Library**: Coarse-grained models for superoxide dismutase simulation
- **Flexible Simulation**: Molecular dynamics (NVE, NVT, NPT), Monte Carlo, energy minimization

## Hardware Requirements

- **GPU**: NVIDIA GPU with CUDA support (GTX 1050 / Pascal 6.1 or newer)
  - GTX 1050: Supported, optimized profile available
  - RTX 3050 Ti / Turing: Recommended for larger systems
- **CUDA**: Toolkit 11.0 or higher
- **CPU**: Multi-core processor for parallel CPU computation
- **Memory**: 8GB+ RAM recommended for large simulations

## Architecture

The Gumol engine uses an adaptive computation strategy:

- **GPU Path**: Homogeneous systems with >500 atoms, non-bonded force calculations, Lennard-Jones potentials
- **CPU Path**: Bonded interactions (bonds, angles, dihedrals), electrostatics (Ewald), small systems, integrators

### Key Components

- `gumol-core`: Core data structures, potentials, and force computations
- `gumol-sim`: Molecular dynamics, Monte Carlo, and minimization algorithms
- `gumol-input`: TOML-based input system for simulations
- `gumol-gpu`: CUDA kernels and GPU memory management

## Installation

### Prerequisites

```bash
# Install CUDA toolkit 11.0+ (Linux)
sudo apt install nvidia-cuda-toolkit

# Verify installation
nvidia-smi
nvcc --version
```

### From Source

```bash
# Clone the repository
git clone https://github.com/SampleBias/gumol-rs.git
cd gumol-rs

# Build (CPU only)
cargo build --release

# Build with GPU support (requires CUDA toolkit)
cargo build --release --features gpu

# The binary will be at target/release/gumol
```

### GPU Profile Configuration

For different NVIDIA GPUs, use the appropriate profile:

```rust
use gumol_gpu::{GpuAccelerator, GpuProfile, GpuForceProvider};

// GTX 1050 (Pascal) - default for older GPUs
let profile = GpuProfile::gtx_1050();

// RTX 3050 Ti (Turing) - when you upgrade
let profile = GpuProfile::rtx_3050_ti();

// Custom profile for your GPU
let profile = GpuProfile::custom("My GPU")
    .compute_capability(61)  // Pascal
    .block_size(128)
    .min_atoms_for_gpu(1000)
    .build();

let accelerator = GpuAccelerator::with_profile(profile)?;
system.set_force_provider(std::sync::Arc::new(GpuForceProvider::new(accelerator)));
```

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
gumol = { git = "https://github.com/SampleBias/gumol-rs" }
```

## Quick Start

### Command Line

```bash
# Run a simulation
gumol simulation.toml

# Run with GPU acceleration (if available)
gumol --gpu simulation.toml
```

### As a Library

```rust
use gumol_core::*;
use gumol_sim::*;
use gumol_input::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load system from file
    let system = System::from_file("water.pdb")?;

    // Set up simulation
    let mut simulation = MolecularDynamics::new(
        VelocityVerlet::new(1.0), // timestep in fs
        BerendsenThermostat::new(300.0, 100.0) // temperature and coupling
    );

    // Run simulation
    simulation.run(&mut system, 10000)?;
    Ok(())
}
```

## Project Status

**Current Phase**: Initial refactoring (Version 0.1.0)

Roadmap:
- [x] Project setup and package renaming
- [ ] Neighbor list system (cell-linked list)
- [ ] GPU kernel infrastructure
- [ ] Radiation damage models
- [ ] EC-SOD coarse-grained library
- [ ] Performance optimization and benchmarking

## Contributing

Gumol is in active development. Contributions are welcome! Areas of interest:

- GPU kernel optimization
- Radiation physics algorithms
- EC-SOD model development
- Testing and validation
- Documentation

Please open an issue to discuss major changes before starting work.

## Documentation

Full documentation is available at: https://samplebias.github.io/gumol-rs

- User manual: Concepts, tutorials, and examples
- Input reference: TOML configuration guide
- API documentation: Library function reference

## License

This software is licensed under the BSD-3-Clause license (retained from Lumol).
See the [LICENSE](LICENSE) file for details.

## Acknowledgments

Gumol is a fork of [Lumol](https://github.com/lumol-org/lumol), maintained by
the Lumol development team. We thank them for the solid foundation they've
provided.

## Contact

- **Issues**: https://github.com/SampleBias/gumol-rs/issues
- **Discussions**: https://github.com/SampleBias/gumol-rs/discussions
