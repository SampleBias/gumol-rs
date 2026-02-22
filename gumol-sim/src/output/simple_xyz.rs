// Lumol, an extensible molecular simulation engine
// Copyright (C) Lumol's contributors — BSD license

//! Simple XYZ trajectory output that bypasses chemfiles to avoid SIGFPE

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use super::Output;
use gumol_core::System;

/// Simple XYZ trajectory output that writes directly without using chemfiles.
///
/// This avoids potential SIGFPE issues in the chemfiles C library
/// when converting System to Frame.
pub struct SimpleXYZOutput {
    file: BufWriter<File>,
    path: PathBuf,
}

impl SimpleXYZOutput {
    /// Create a new `SimpleXYZOutput` writing to `filename`.
    /// The file is replaced if it already exists.
    pub fn new<P>(path: P) -> std::io::Result<SimpleXYZOutput>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(path.as_ref())?);
        Ok(SimpleXYZOutput {
            file: file,
            path: path.as_ref().to_owned(),
        })
    }
}

impl Output for SimpleXYZOutput {
    fn write(&mut self, system: &System) {
        // Write XYZ format header
        let header = format!(
            "{}\nLattice=\"{} {} 0 0 {} 0 0 0\"",
            system.size(),
            system.cell.a(),
            system.cell.b(),
            system.cell.c()
        );
        let _ = writeln!(self.file, "{}", header);

        // Write atoms
        let particles = system.particles();
        for i in 0..system.size() {
            let name = &particles.name[i];
            let pos = &particles.position[i];
            let _ = writeln!(self.file, "{} {:.6} {:.6} {:.6}", name, pos[0], pos[1], pos[2]);
        }

        // Flush to ensure data is written
        let _ = self.file.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tests::test_output;

    #[test]
    fn simple_xyz() {
        test_output(
            |path| Box::new(SimpleXYZOutput::new(path).unwrap()),
            "2
Lattice=\"10 0 0 10 0 0 0\"
F 0.000000 0.000000 0.000000
F 1.300000 0.000000 0.000000
",
        );
    }
}
