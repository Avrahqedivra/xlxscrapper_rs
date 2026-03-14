/*
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 *  of this software and associated documentation files (the 'Software'), to deal
 *  in the Software without restriction, including without limitation the rights
 *  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 *  copies of the Software, and to permit persons to whom the Software is
 *  furnished to do so, subject to the following conditions:
 *
 *  The above copyright notice and this permission notice shall be included in
 *   all copies or substantial portions of the Software.
 *
 *   THE SOFTWARE IS PROVIDED 'AS IS', WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 *   IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 *   FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 *   AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 *   LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 *   OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 *   THE SOFTWARE.
 *
 *  Copyright (c) 2026 F4JDN - Jean-Michel Cohen
 *
 *
 */

use anyhow::{Context, Result};
use chrono::Local;
use log::{debug, error, info, warn};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::Deserialize;
use simple_logger::SimpleLogger;
use std::fs::{self, File};
use std::io::Write;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct DownloaderConfig {
    pub(crate) url: String,
    pub(crate) interval_seconds: u64,
    pub(crate) output_dir: String,
    pub(crate) output_file: String,
    pub(crate) user_agent: String,
    pub(crate) timeout_seconds: u64,
}

impl Default for DownloaderConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            interval_seconds: 1,
            output_dir: "downloaded_pages".to_string(),
            output_file: "xlx.json".to_string(),
            user_agent: "Mozilla/5.0 (compatible; RustScraper/1.0)".to_string(),
            timeout_seconds: 30,
        }
    }
}

pub struct PageDownloader {
    client: Client,
    pub(crate) config: Arc<DownloaderConfig>,
    counter: u64,
}

impl PageDownloader {
    pub(crate) fn new(config: DownloaderConfig) -> Result<Self> {
        // Setup custom headers
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, config.user_agent.parse()?);

        // Create HTTP client with timeout
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            config: Arc::new(config),
            counter: 0,
        })
    }

    pub(crate) fn download_page(&self) -> Result<String> {
        debug!("Downloading: {}", self.config.url);

        let response = self.client
            .get(&self.config.url)
            .send()
            .context(format!("Failed to download {}", self.config.url))?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {} - {}", response.status(), self.config.url);
        }

        let content = response.text()
            .context("Failed to read response body")?;

        debug!("Downloaded {} bytes", content.len());
        Ok(content)
    }

    fn process_iteration(&mut self) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.counter += 1;

        info!("[{}] Iteration #{} - Downloading...", timestamp, self.counter);

        match self.download_page() {
            Ok(content) => {
                info!("[{}] Successfully downloaded {} bytes", timestamp, content.len());
                Ok(())
            }
            Err(e) => {
                error!("[{}] Download failed: {}", timestamp, e);
                Err(e)
            }
        }
    }

    fn run(&mut self) -> ! {
        info!("Starting downloader for: {}", self.config.url);
        info!("Interval: {} second(s)", self.config.interval_seconds);

        loop {
            let start_time = std::time::Instant::now();

            // Process one iteration
            let _ = self.process_iteration(); // Ignore errors to continue running

            // Calculate sleep time to maintain exact interval
            let elapsed = start_time.elapsed();
            let sleep_duration = Duration::from_secs(self.config.interval_seconds)
                .saturating_sub(elapsed);

            if sleep_duration > Duration::from_millis(0) {
                thread::sleep(sleep_duration);
            }
        }
    }
}
