use std::path::PathBuf;
use clap::Args;
use log::info;
use anyhow::{Context, Result};

use crate::{
    ChargeDensity,
    ChargeType,
    OptProcess,
};

#[derive(Debug, Args)]
/// Sum multiple CHGCAR charge densities: CHG_SUM = CHG1 + CHG2 + ...
pub struct ChgSum {
    /// Input CHGCAR files
    #[arg(required = true)]
    input: Vec<PathBuf>,

    #[arg(short, long, default_value = "CHG_SUM.vasp")]
    output: PathBuf,
}

impl OptProcess for ChgSum {
    fn process(&self) -> Result<()> {
        use rayon::prelude::*;

        if self.input.is_empty() {
            anyhow::bail!("At least one CHGCAR file is required.");
        }

        let mut chg_list: Vec<anyhow::Result<ChargeDensity>> =
            vec![Err(anyhow::anyhow!("placeholder")); self.input.len()];

        rayon::scope(|s| {
            for (i, path) in self.input.iter().enumerate() {
                s.spawn(|_| {
                    chg_list[i] = ChargeDensity::from_file(path, ChargeType::Chgcar)
                        .with_context(|| format!("Failed to load CHGCAR from {:?}", path));
                });
            }
        });

        let mut chg_list = chg_list.into_iter().collect::<Result<Vec<_>>>()?;

        let mut total = chg_list.pop().context("No CHGCAR files successfully loaded.")?;
        for chg in chg_list {
            total = (total + chg)?;
        }

        info!("Writing summed charge density to {:?}", self.output);
        total.to_file(&self.output)?;

        Ok(())
    }
}

