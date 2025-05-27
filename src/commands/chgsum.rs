use std::path::PathBuf;
use clap::Args;
use log::info;
use anyhow::{anyhow, Context};
use rayon::prelude::*;
use crate::{
    types::Result,
    ChargeDensity,
    ChargeType,
    OptProcess,
};

#[derive(Debug, Args)]
/// Calculate charge density sum from multiple CHGCAR files.
///
/// All CHGCARs must have the same grid and lattice.
pub struct Chgsum {
    /// Input CHGCAR files to sum
    #[arg(required = true)]
    input: Vec<PathBuf>,

    /// Output file name (default: CHGSUM.vasp)
    #[arg(short, long, default_value = "CHGSUM.vasp")]
    output: PathBuf,
}

impl OptProcess for Chgsum {
    fn process(&self) -> Result<()> {
        if self.input.len() < 2 {
            return Err(anyhow!("Please provide at least two CHGCAR files."));
        }

        // Load all CHGCARs in parallel
        let chgcars: Result<Vec<ChargeDensity>> = self.input
            .par_iter()
            .map(|path| {
                info!("Reading charge density from {:?}", path);
                ChargeDensity::from_file(path, ChargeType::Chgcar)
                    .with_context(|| format!("Failed to read charge density from {:?}", path))
            })
            .collect();

        let chgcars = chgcars?;

        // Reduce all ChargeDensity files into one
        let sum = chgcars
            .into_iter()
            .reduce(|a, b| (a + b).expect("ChargeDensity addition failed"))
            .ok_or_else(|| anyhow!("Failed to sum CHGCARs"))?;

        info!("Writing summed charge density to {:?}", self.output);
        sum.to_file(&self.output)?;

        Ok(())
    }
}
