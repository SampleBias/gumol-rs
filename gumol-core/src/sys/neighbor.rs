// Gumol - GPU-Accelerated Radiation Simulation Engine
// Copyright (C) Gumol's contributors — BSD license

//! Cell-linked list algorithm for efficient neighbor searching.
//!
//! This module implements a cell-linked list data structure for O(N) neighbor
//! list construction, replacing the O(N²) naive approach. The space is
//! divided into cells of size >= cutoff, and each particle is assigned to a
//! cell. Neighbors are found by checking particles in adjacent cells only.

use crate::{Vector3D, UnitCell};
use log::trace;
use std::collections::HashSet;

/// Neighbor list using cell-linked list algorithm
///
/// The algorithm divides space into a regular grid of cells, where each cell
/// has size equal to or larger than the cutoff distance. Particles are
/// assigned to cells, and neighbor pairs are found by checking particles in
/// adjacent cells only.
///
/// # Complexity
///
/// - Building: O(N) where N is the number of particles
/// - Storage: O(N + N_cells)
/// - Neighbor check per particle: O(1) on average (constant number of adjacent cells)
///
/// # Example
///
/// ```
/// use gumol_core::{System, Vector3D};
/// use gumol_core::sys::NeighborList;
///
/// let mut system = System::new();
/// // ... add particles to system ...
///
/// let mut neighbor_list = NeighborList::new(10.0); // 10.0 Å cutoff
/// neighbor_list.update(&system);
///
/// // Iterate over all pairs
/// for (i, j) in neighbor_list.pairs() {
///     // Compute interaction between particles i and j
/// }
/// ```
#[derive(Clone)]
pub struct NeighborList {
    /// Cutoff distance for neighbor interactions (in Å)
    cutoff: f64,

    /// Skin distance - additional margin to reduce rebuild frequency
    skin: f64,

    /// Effective cutoff including skin
    effective_cutoff: f64,

    /// Cell size (must be >= effective_cutoff)
    cell_size: f64,

    /// Number of cells in each dimension [nx, ny, nz]
    ncells: [usize; 3],

    /// Cell head pointers: head[icell] = index of first particle in cell icell,
    /// or -1 if cell is empty
    head: Vec<i32>,

    /// Linked list: next[iparticle] = index of next particle in same cell,
    /// or -1 if iparticle is last in its cell
    next: Vec<i32>,

    /// Number of particles
    natoms: usize,

    /// Cell index for each particle
    particle_cell: Vec<[i32; 3]>,

    /// Last system step when neighbor list was updated
    last_step: u64,

    /// Update frequency (rebuild every N steps)
    update_frequency: u64,

    /// Maximum displacement since last rebuild (for skin-based updates)
    max_displacement: f64,

    /// Pairs currently in the neighbor list
    pairs: Vec<(usize, usize)>,

    /// Whether periodic boundary conditions are active
    periodic: [bool; 3],
}

impl NeighborList {
    /// Create a new neighbor list with the specified cutoff
    ///
    /// # Arguments
    ///
    /// * `cutoff` - Cutoff distance in Angstroms
    ///
    /// # Example
    ///
    /// ```
    /// use gumol_core::sys::NeighborList;
    ///
    /// let neighbor_list = NeighborList::new(10.0);
    /// ```
    pub fn new(cutoff: f64) -> Self {
        NeighborList::with_update_frequency(cutoff, 10)
    }

    /// Create a new neighbor list with specified cutoff and update frequency
    ///
    /// # Arguments
    ///
    /// * `cutoff` - Cutoff distance in Angstroms
    /// * `update_frequency` - Rebuild neighbor list every N steps (default: 10)
    pub fn with_update_frequency(cutoff: f64, update_frequency: u64) -> Self {
        let skin = cutoff * 0.3; // 30% skin by default
        let effective_cutoff = cutoff + skin;

        NeighborList {
            cutoff,
            skin,
            effective_cutoff,
            cell_size: effective_cutoff,
            ncells: [0, 0, 0],
            head: Vec::new(),
            next: Vec::new(),
            natoms: 0,
            particle_cell: Vec::new(),
            last_step: 0,
            update_frequency,
            max_displacement: 0.0,
            pairs: Vec::new(),
            periodic: [true, true, true],
        }
    }

    /// Get the cutoff distance
    pub fn cutoff(&self) -> f64 {
        self.cutoff
    }

    /// Set the cutoff distance
    pub fn set_cutoff(&mut self, cutoff: f64) {
        self.cutoff = cutoff;
        self.skin = cutoff * 0.3;
        self.effective_cutoff = cutoff + self.skin;
        self.cell_size = self.effective_cutoff;
    }

    /// Get the update frequency
    pub fn update_frequency(&self) -> u64 {
        self.update_frequency
    }

    /// Set the update frequency
    pub fn set_update_frequency(&mut self, freq: u64) {
        self.update_frequency = freq;
    }

    /// Check if the neighbor list needs updating
    pub fn needs_update(&self, current_step: u64) -> bool {
        current_step - self.last_step >= self.update_frequency
            || self.max_displacement > self.skin * 0.5
    }

    /// Get the maximum displacement since last rebuild
    pub fn max_displacement(&self) -> f64 {
        self.max_displacement
    }

    /// Reset the maximum displacement tracking
    pub fn reset_displacement(&mut self) {
        self.max_displacement = 0.0;
    }

    /// Update the neighbor list from a system's particle positions
    ///
    /// This will rebuild the cell-linked list if the update frequency has been
    /// exceeded or if particles have moved too far.
    ///
    /// # Arguments
    ///
    /// * `system` - Reference to the system containing particle positions
    pub fn update<T>(&mut self, system: &T)
    where
        T: NeighborListSource,
    {
        if self.natoms != system.natoms() {
            // System size changed, need full rebuild
            self.resize(system.natoms());
            self.rebuild(system);
            self.last_step = system.step();
            self.max_displacement = 0.0;
            return;
        }

        let current_step = system.step();
        if self.needs_update(current_step) {
            self.rebuild(system);
            self.last_step = current_step;
            self.max_displacement = 0.0;
        }
    }

    /// Resize internal arrays for a new number of particles
    fn resize(&mut self, natoms: usize) {
        self.natoms = natoms;
        self.next.resize(natoms, -1);
        self.particle_cell.resize(natoms, [0, 0, 0]);

        // Calculate number of cells needed
        let natoms = natoms as f64;
        let volume = natoms.powf(2.0 / 3.0) * 4.0; // Approximate volume per atom in Å³
        let n_cells_per_dim = (volume / self.cell_size.powi(3)).cbrt().ceil() as usize;

        self.ncells = [n_cells_per_dim.max(1), n_cells_per_dim.max(1), n_cells_per_dim.max(1)];
        let ncells_total = self.ncells[0] * self.ncells[1] * self.ncells[2];
        self.head.resize(ncells_total, -1);

        trace!(
            "Neighbor list resized: {} atoms, cell_size={:.2}, {} cells ({}×{}×{})",
            natoms, self.cell_size, ncells_total, self.ncells[0], self.ncells[1], self.ncells[2]
        );
    }

    /// Rebuild the entire cell-linked list and neighbor pairs
    fn rebuild<T>(&mut self, system: &T)
    where
        T: NeighborListSource,
    {
        // Reset cell heads
        for head in &mut self.head {
            *head = -1;
        }

        // Assign particles to cells
        for i in 0..self.natoms {
            let pos = system.position(i);
            let cell = self.get_cell_index(pos);
            self.particle_cell[i] = cell;

            let cell_idx = self.flatten_cell_index(&cell);
            self.next[i] = self.head[cell_idx];
            self.head[cell_idx] = i as i32;

            trace!(
                "Particle {} at ({}, {}, {}) -> cell [{}, {}, {}] -> idx {}",
                i, pos[0], pos[1], pos[2], cell[0], cell[1], cell[2], cell_idx
            );
        }

        // Build neighbor pairs
        self.build_pairs(system);

        trace!(
            "Neighbor list rebuilt: {} pairs, {} cells",
            self.pairs.len(),
            self.head.len()
        );
    }

    /// Get cell index for a position
    fn get_cell_index(&self, pos: &Vector3D) -> [i32; 3] {
        let ix = (pos[0] / self.cell_size).floor() as i32;
        let iy = (pos[1] / self.cell_size).floor() as i32;
        let iz = (pos[2] / self.cell_size).floor() as i32;
        [ix, iy, iz]
    }

    /// Flatten 3D cell index to 1D
    fn flatten_cell_index(&self, cell: &[i32; 3]) -> usize {
        let ix = ((cell[0] % self.ncells[0] as i32 + self.ncells[0] as i32) % self.ncells[0] as i32) as usize;
        let iy = ((cell[1] % self.ncells[1] as i32 + self.ncells[1] as i32) % self.ncells[1] as i32) as usize;
        let iz = ((cell[2] % self.ncells[2] as i32 + self.ncells[2] as i32) % self.ncells[2] as i32) as usize;

        iz * self.ncells[0] * self.ncells[1] + iy * self.ncells[0] + ix
    }

    /// Build neighbor pairs by checking adjacent cells
    fn build_pairs<T>(&mut self, system: &T)
    where
        T: NeighborListSource,
    {
        self.pairs.clear();
        let r2_cut = self.cutoff * self.cutoff;

        // Track which cell pairs we've processed to avoid duplicates
        let mut processed_pairs = HashSet::new();

        // Iterate over all cells
        for iz in 0..self.ncells[2] {
            for iy in 0..self.ncells[1] {
                for ix in 0..self.ncells[0] {
                    let center_cell = [ix as i32, iy as i32, iz as i32];
                    let center_idx = self.flatten_cell_index(&center_cell);

                    // Check particles in this cell and adjacent cells
                    for adj_cell in self.adjacent_cells(&center_cell) {
                        // Create ordered pair to avoid double counting
                        let pair_key = if center_idx <= adj_cell {
                            (center_idx, adj_cell)
                        } else {
                            (adj_cell, center_idx)
                        };

                        if !processed_pairs.insert(pair_key) {
                            continue; // Already processed this cell pair
                        }

                        // Check pairs between particles in center_cell and adj_cell
                        let mut i_particle = self.head[center_idx];
                        while i_particle >= 0 {
                            let i = i_particle as usize;
                            let pos_i = system.position(i);

                            let mut j_particle = self.head[adj_cell];
                            while j_particle >= 0 {
                                let j = j_particle as usize;

                                if i < j {
                                    let pos_j = system.position(j);
                                    let r2 = self.distance2(system, pos_i, pos_j);

                                    if r2 < r2_cut {
                                        self.pairs.push((i, j));
                                    }
                                }

                                j_particle = self.next[j_particle as usize];
                            }

                            i_particle = self.next[i_particle as usize];
                        }
                    }
                }
            }
        }
    }

    /// Get indices of adjacent cells (including self)
    fn adjacent_cells(&self, cell: &[i32; 3]) -> Vec<usize> {
        let mut adjacent = Vec::with_capacity(27);

        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = cell[0] + dx;
                    let ny = cell[1] + dy;
                    let nz = cell[2] + dz;

                    let flat_idx = self.flatten_cell_index(&[nx, ny, nz]);
                    adjacent.push(flat_idx);
                }
            }
        }

        adjacent
    }

    /// Calculate squared distance between two positions with PBC
    fn distance2<T>(&self, system: &T, pos_i: &Vector3D, pos_j: &Vector3D) -> f64
    where
        T: NeighborListSource,
    {
        let mut dx = pos_i[0] - pos_j[0];
        let mut dy = pos_i[1] - pos_j[1];
        let mut dz = pos_i[2] - pos_j[2];

        // Apply minimum image convention for periodic dimensions
        let cell = system.cell();
        if self.periodic[0] && !cell.is_infinite() {
            let a = cell.a();
            if a > 0.0 {
                dx -= (dx / a).round() * a;
            }
        }
        if self.periodic[1] && !cell.is_infinite() {
            let b = cell.b();
            if b > 0.0 {
                dy -= (dy / b).round() * b;
            }
        }
        if self.periodic[2] && !cell.is_infinite() {
            let c = cell.c();
            if c > 0.0 {
                dz -= (dz / c).round() * c;
            }
        }

        dx * dx + dy * dy + dz * dz
    }

    /// Get iterator over all neighbor pairs
    pub fn pairs(&self) -> impl Iterator<Item = &(usize, usize)> {
        self.pairs.iter()
    }

    /// Get number of pairs in the neighbor list
    pub fn npairs(&self) -> usize {
        self.pairs.len()
    }

    /// Track particle displacement for skin-based updates
    pub fn track_displacement<T>(&mut self, _system: &T, displacements: &[Vector3D])
    where
        T: NeighborListSource,
    {
        if displacements.len() != self.natoms {
            return;
        }

        for disp in displacements {
            let d2 = disp[0] * disp[0] + disp[1] * disp[1] + disp[2] * disp[2];
            let d = d2.sqrt();
            self.max_displacement = self.max_displacement.max(d);
        }
    }
}

/// Trait for types that can provide data needed by NeighborList
pub trait NeighborListSource {
    /// Get the number of atoms
    fn natoms(&self) -> usize;

    /// Get the position of atom i
    fn position(&self, i: usize) -> &Vector3D;

    /// Get the unit cell
    fn cell(&self) -> &UnitCell;

    /// Get the current simulation step
    fn step(&self) -> u64;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSystem {
        positions: Vec<Vector3D>,
        step: u64,
        infinite_cell: UnitCell,
    }

    impl TestSystem {
        fn new() -> Self {
            TestSystem {
                positions: Vec::new(),
                step: 0,
                infinite_cell: UnitCell::infinite(),
            }
        }

        fn add_atom(&mut self, pos: Vector3D) {
            self.positions.push(pos);
        }
    }

    impl NeighborListSource for TestSystem {
        fn natoms(&self) -> usize {
            self.positions.len()
        }

        fn position(&self, i: usize) -> &Vector3D {
            &self.positions[i]
        }

        fn cell(&self) -> &UnitCell {
            &self.infinite_cell
        }

        fn step(&self) -> u64 {
            self.step
        }
    }

    #[test]
    fn test_neighbor_list_creation() {
        let nl = NeighborList::new(10.0);
        assert_eq!(nl.cutoff(), 10.0);
        assert_eq!(nl.update_frequency(), 10);
    }

    #[test]
    fn test_neighbor_list_simple() {
        let mut system = TestSystem::new();
        system.add_atom(Vector3D::new(0.0, 0.0, 0.0));
        system.add_atom(Vector3D::new(5.0, 0.0, 0.0));
        system.add_atom(Vector3D::new(15.0, 0.0, 0.0));

        let mut nl = NeighborList::new(10.0);
        nl.update(&system);

        // Should find pair (0, 1) but not (0, 2) or (1, 2)
        assert!(nl.pairs().any(|&(i, j)| (i == 0 && j == 1) || (i == 1 && j == 0)));
        assert!(!nl.pairs().any(|&(i, j)| i == 2 || j == 2));
    }

    #[test]
    fn test_neighbor_list_update_frequency() {
        let mut system = TestSystem::new();
        for i in 0..10 {
            system.add_atom(Vector3D::new(i as f64, 0.0, 0.0));
        }

        let mut nl = NeighborList::with_update_frequency(10.0, 5);
        nl.update(&system);

        assert_eq!(nl.last_step, 0);

        system.step = 3;
        nl.update(&system);
        assert_eq!(nl.last_step, 0); // No update yet

        system.step = 6;
        nl.update(&system);
        assert_eq!(nl.last_step, 6); // Updated
    }
}
