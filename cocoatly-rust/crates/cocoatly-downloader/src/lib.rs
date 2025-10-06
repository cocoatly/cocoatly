use cocoatly_core::error::{CocoatlyError, Result};
use cocoatly_core::config::NetworkConfig;
use reqwest::{Client, Response};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use tokio::time::sleep;
use futures::stream::StreamExt;

pub struct Downloader {
    client: Client,
    config: NetworkConfig,
}

impl Downloader {
    pub fn new(config: NetworkConfig) -> Result<Self> {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .connect_timeout(Duration::from_secs(30));

        if config.use_proxy {
            if let Some(proxy_url) = &config.proxy_url {
                let proxy = reqwest::Proxy::all(proxy_url)
                    .map_err(|e| CocoatlyError::DownloadFailed(
                        format!("Failed to configure proxy: {}", e)
                    ))?;
                client_builder = client_builder.proxy(proxy);
            }
        }

        let client = client_builder
            .build()
            .map_err(|e| CocoatlyError::DownloadFailed(
                format!("Failed to create HTTP client: {}", e)
            ))?;

        Ok(Self { client, config })
    }

    pub async fn download<P: AsRef<Path>>(
        &self,
        url: &str,
        destination: P,
        progress_callback: Option<Box<dyn Fn(u64, u64) + Send>>,
    ) -> Result<DownloadResult> {
        let mut attempts = 0;
        let max_attempts = self.config.retry_attempts;

        loop {
            attempts += 1;

            match self.download_internal(url, destination.as_ref(), progress_callback.as_ref()).await {
                Ok(result) => return Ok(result),
                Err(e) if attempts < max_attempts => {
                    sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn download_internal(
        &self,
        url: &str,
        destination: &Path,
        progress_callback: Option<&Box<dyn Fn(u64, u64) + Send>>,
    ) -> Result<DownloadResult> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| CocoatlyError::DownloadFailed(
                format!("Failed to download from {}: {}", url, e)
            ))?;

        if !response.status().is_success() {
            return Err(CocoatlyError::DownloadFailed(
                format!("HTTP error {}: {}", response.status(), url)
            ));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;

        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = File::create(destination)?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| CocoatlyError::DownloadFailed(
                format!("Failed to read response chunk: {}", e)
            ))?;

            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            if let Some(callback) = progress_callback {
                callback(downloaded, total_size);
            }
        }

        file.flush()?;

        Ok(DownloadResult {
            url: url.to_string(),
            destination: destination.to_path_buf(),
            size: downloaded,
            total_size,
        })
    }

    pub async fn download_multiple(
        &self,
        downloads: Vec<DownloadTask>,
    ) -> Result<Vec<DownloadResult>> {
        let max_concurrent = self.config.max_concurrent_downloads;
        let mut results = Vec::new();

        for chunk in downloads.chunks(max_concurrent) {
            let futures: Vec<_> = chunk
                .iter()
                .map(|task| self.download(&task.url, &task.destination, None))
                .collect();

            let chunk_results = futures::future::join_all(futures).await;

            for result in chunk_results {
                results.push(result?);
            }
        }

        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub url: String,
    pub destination: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub url: String,
    pub destination: PathBuf,
    pub size: u64,
    pub total_size: u64,
}
