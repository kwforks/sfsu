use clap::Parser;
use rayon::prelude::*;
use serde::Serialize;
use sfsu::{buckets::Bucket, output::structured::Structured, packages::install};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    fn runner(self) -> anyhow::Result<()> {
        let apps = install::Manifest::list_all_unchecked()?;

        let mut outdated: Vec<Outdated> = apps
            .par_iter()
            .flat_map(|app| -> anyhow::Result<Outdated> {
                if let Some(bucket) = &app.bucket {
                    let app_manifest = app.get_manifest()?;
                    // TODO: Add the option to check all buckets and find the highest version (will require semver to order versions)
                    let bucket = Bucket::new(bucket)?;
                    match bucket.get_manifest(&app.name) {
                        Ok(manifest) if app_manifest.version != manifest.version => Ok(Outdated {
                            name: app.name.clone(),
                            current: app_manifest.version.clone(),
                            available: manifest.version.clone(),
                        }),
                        _ => anyhow::bail!("no update available"),
                    }
                } else {
                    anyhow::bail!("no bucket specified")
                }
            })
            .collect();

        if outdated.is_empty() {
            println!("No outdated packages.");
        } else {
            outdated.dedup();
            outdated.par_sort_by(|a, b| a.name.cmp(&b.name));

            let values = outdated
                .par_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?;

            if self.json {
                let output = serde_json::to_string_pretty(&values)?;

                println!("{output}");
            } else {
                let outputs =
                    Structured::new(&["Name", "Current", "Available"], &values).with_max_length(30);

                print!("{outputs}");
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct Outdated {
    name: String,
    current: String,
    available: String,
}
