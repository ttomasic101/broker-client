use quinn::ClientConfigBuilder;
use anyhow::Result;
use std::{
    path::PathBuf,
    fs, io
};
use tracing::info;

pub fn setup_security(client_config: &mut ClientConfigBuilder, path: &Option<PathBuf>) -> Result<()> {
    if let Some(ca_path) = path {
        client_config
            .add_certificate_authority(quinn::Certificate::from_der(&fs::read(&ca_path)?)?)?;
    } else {
        let dirs = directories::ProjectDirs::from("arg", "quinn", "quinn-examples").unwrap();
        match fs::read(dirs.data_local_dir().join("cert.der")) {
            Ok(cert) => {
                client_config.add_certificate_authority(quinn::Certificate::from_der(&cert)?)?;
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                info!("Local server certificate not found");
                return Err(anyhow::Error::from(e));
            },
            Err(e) => {
                info!("Failed to open local server certificate: {}", e);
                return Err(anyhow::Error::from(e));
            }
        }
    }

    Ok(())
}