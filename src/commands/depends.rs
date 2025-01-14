use clap::Parser;
use sprinkles::{
    contexts::ScoopContext,
    packages::reference::{manifest, package},
};

use crate::{
    abandon,
    output::sectioned::{Children, Section, Sections},
};

#[derive(Debug, Clone, Parser)]
/// List the dependencies of a given package, in the order that they will be installed
pub struct Args {
    #[clap(help = "The package to list dependencies for")]
    package: package::Reference,

    #[clap(help = "The bucket of the given package")]
    bucket: Option<String>,

    // TODO: Implement recursion?
    // recursive: bool,
    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(mut self, ctx: &impl ScoopContext) -> Result<(), anyhow::Error> {
        if let Some(bucket) = self.bucket {
            self.package.set_bucket(bucket)?;
        }

        let manifests = self.package.list_manifests(ctx).await?;

        if manifests.is_empty() {
            abandon!("Could not find any packages matching: {}", self.package);
        };

        if self.json {
            println!("{}", serde_json::to_string(&manifests)?);
            return Ok(());
        }

        let output: Sections<manifest::Reference> = manifests
            .into_iter()
            .filter_map(|manifest| {
                Children::from(manifest.depends())
                    .into_option()
                    .map(|children| {
                        Section::new(children).with_title(format!(
                            "Dependencies for '{}' in '{}'",
                            unsafe { manifest.name() },
                            unsafe { manifest.bucket() }
                        ))
                    })
            })
            .collect();

        println!("{output}");

        Ok(())
    }
}

// note to self, use `phpstudy-lagecy-scoop` to test this command
