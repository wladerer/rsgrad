use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use log::info;
use rayon::prelude::*;

use crate::{
    ChargeDensity,
    ChargeType,
    OptProcess,
};

#[derive(Debug, Args)]
/// Sum multiple CHGCAR files to generate a combined charge density.
/// The operation is: `chgcar_sum = chgcar_1 + chgcar_2 + ... + chgcar_n`
pub struct Chgsum {
    /// The input CHGCAR files
    #[arg(required = true)]
    input: Vec<PathBuf>,

    /// The output CHGCAR file path
    #[arg(short, long, default_value = "CHGSUM.vasp")]
    output: PathBuf,
}

impl OptProcess for Chgsum {
    fn process(&self) -> Result<()> {
        if self.input.is_empty() {
            anyhow::bail!("No CHGCAR files provided for summation.");
        }

        info!("Reading {} charge densities in parallel...", self.input.len());

        let charges: Vec<ChargeDensity> = self
            .input
            .par_iter()
            .map(|path| {
                ChargeDensity::from_file(path, ChargeType::Chgcar)
                    .with_context(|| format!("Failed to read CHGCAR from {:?}", path))
            })
            .collect::<Result<_>>()?;

        info!("Successfully read all CHGCAR files. Summing...");

        // Reduce the vector with addition
        let mut chg_sum = charges[0].clone();
        for chg in &charges[1..] {
            chg_sum = (chg_sum.clone() + chg.clone())?;
        }

        info!("Writing combined charge density to {:?}", self.output);
        chg_sum.to_file(&self.output)?;

        Ok(())
    }
}

