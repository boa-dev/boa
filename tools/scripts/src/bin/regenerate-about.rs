//! Regenerates the `ABOUT.md` document for all publishable crates.

use std::{error::Error, path::Path};

use log::{info, warn};

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new().env().init()?;

    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .exec()?;

    for member_id in &metadata.workspace_members {
        let member = &metadata[member_id];

        info!("Checking member `{}`", member.manifest_path);

        if member.publish.as_ref().is_some_and(Vec::is_empty) {
            warn!("Skipping unpublishable member...");
            continue;
        }

        let clone_path = member.manifest_path.with_file_name("ABOUT.md");

        info!("Cloning ABOUT.md into path `{clone_path}`");

        std::fs::copy(Path::new("./ABOUT.md"), clone_path)?;
    }

    Ok(())
}
