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

mod mods;

use crate::mods::downloader::{DownloaderConfig, PageDownloader};
use anyhow::{anyhow, Context, Result};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

include!("globals.rs");

// --- Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum CellValue {
    Number(i64),
    Text(String),
}

impl CellValue {
    fn from_str(s: &str) -> Option<Self> {
        if s.is_empty() {
            return None;
        }
        if let Ok(n) = s.parse::<i64>() {
            Some(CellValue::Number(n))
        } else {
            Some(CellValue::Text(s.to_owned()))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ParsedTable {
    headers: Vec<String>,
    rows: Vec<BTreeMap<String, CellValue>>,
}

// Pre-compiled selectors — built once, reused across every parse call.
struct Selectors {
    form: Selector,
    row:  Selector,
    th:   Selector,
    td:   Selector,
    img:  Selector,
    a:    Selector,
}

impl Selectors {
    fn build() -> Self {
        Self {
            form: Selector::parse(r#"form[name="frmFilterCallSign"]"#).unwrap(),
            row:  Selector::parse("tr").unwrap(),
            th:   Selector::parse("th").unwrap(),
            td:   Selector::parse("td").unwrap(),
            img:  Selector::parse("img").unwrap(),
            a:    Selector::parse("a").unwrap(),
        }
    }
}

// --- Bundles a downloader with its per-target error counter ---

struct Target {
    downloader:         Arc<PageDownloader>,
    consecutive_errors: u32,
    /// SHA-256 of the last written JSON — None until first save.
    last_hash:          Option<String>,
}

// --- Main ---

fn main() -> Result<()> {
    print!("{}", __CLEAR__);

    println!("    {}dMP dMP dMP     dMP dMP       {}.dMMMb  .aMMMb  dMMMMb  {}.aMMMb  dMMMMb  dMMMMMP dMMMMb'",__BLUE__,__WHITE__,__RED__);
    println!("   {}dMK.dMP dMP     dMK.dMP       {}dMP' VP dMP'VMP dMP.dMP {}dMP'dMP dMP.dMP dMP     dMP.dMP'",__BLUE__,__WHITE__,__RED__);
    println!("  {}.dMMMK' dMP     .dMMMK'        {}VMMMb  dMP     dMMMMK' {}dMMMMMP dMMMMP' dMMMP   dMMMMK'",__BLUE__,__WHITE__,__RED__);
    println!(" {}dMP'AMF dMP     dMP'AMF       {}dP .dMP dMP.aMP dMP'AMF {}dMP dMP dMP     dMP     dMP'AMF'",__BLUE__,__WHITE__,__RED__);
    println!("{}dMP dMP dMMMMMP dMP dMP        {}VMMMP'  VMMMP' dMP dMP {}dMP dMP dMP     dMMMMMP dMP dMP'",__BLUE__,__WHITE__,__RED__);


    println!("{}", __RESET__);

    println!("RSMonitor v{} (c) 2026 Jean-Michel Cohen, F4JDN <f4jdn@outlook.fr>", VERSION);

    println!();

    let config_path = std::env::args().nth(1)
        .unwrap_or_else(|| "config.json".to_string());

    let configs = load_configs(&config_path)
        .with_context(|| format!("Failed to load config from \"{config_path}\""))?;

    if configs.is_empty() {
        anyhow::bail!("No targets found in \"{config_path}\" — nothing to do");
    }

    // Build one PageDownloader per config; fail fast if any config is invalid.
    let mut targets: Vec<Target> = configs
        .into_iter()
        .enumerate()
        .map(|(i, config)| {
            std::fs::create_dir_all(&config.output_dir)
                .with_context(|| format!("Failed to create output dir for target {i}"))?;
            let downloader = Arc::new(PageDownloader::new(config)?);
            Ok(Target { downloader, consecutive_errors: 0, last_hash: None })
        })
        .collect::<Result<_>>()?;

    let selectors = Arc::new(Selectors::build());

    println!("Scraper started — {} target(s)", targets.len());
    for (i, t) in targets.iter().enumerate() {
        println!("  [{}] {} → {}", i, t.downloader.config.url, t.downloader.config.output_dir);
    }

    loop {
        for (i, target) in targets.iter_mut().enumerate() {
            match run_iteration(target, &selectors) {
                Ok(row_count) => {
                    target.consecutive_errors = 0;
                    println!("[target={i}] {row_count} rows scraped");
                }
                Err(e) => {
                    target.consecutive_errors += 1;
                    eprintln!("[target={i}] Error ({}x): {e:#}",
                              target.consecutive_errors);

                    if target.consecutive_errors > 1 {
                        let secs = 2u64.pow(target.consecutive_errors.min(5));
                        eprintln!("  → target {i} back-off: skipping next {secs}s");
                    }
                }
            }

            // Respect each target's individual interval before moving to the next.
            std::thread::sleep(Duration::from_secs(target.downloader.config.interval_seconds));
        }
    }
}

fn sha256(data: &str) -> String {
    format!("{:x}", Sha256::digest(data.as_bytes()))
}

fn run_iteration(target: &mut Target, selectors: &Selectors) -> Result<usize> {
    let html  = target.downloader.download_page().context("Download failed")?;
    let table = parse_table(&html, selectors).context("Parse failed")?;

    let json     = serde_json::to_string_pretty(&table.rows)?;
    let new_hash = sha256(&json);

    if target.last_hash.as_deref() != Some(new_hash.as_str()) {
        let path = format!("{}/{}", target.downloader.config.output_dir,
                           target.downloader.config.output_file);
        std::fs::write(&path, &json)
            .with_context(|| format!("Could not write {path}"))?;
        println!("  saved → {path}");
        target.last_hash = Some(new_hash);
    } else {
        println!("  unchanged — skipped write");
    }

    Ok(table.rows.len())
}


// --- Config loader ---
fn load_configs(path: &str) -> Result<Vec<DownloaderConfig>> {
    let raw = std::fs::read_to_string(Path::new(path))
        .with_context(|| format!("Cannot read \"{path}\""))?;
    let configs: Vec<DownloaderConfig> = serde_json::from_str(&raw)
        .with_context(|| format!("Invalid JSON in \"{path}\""))?;
    Ok(configs)
}

// --- Parser ---

fn parse_table(html: &str, sel: &Selectors) -> Result<ParsedTable> {
    let document = Html::parse_document(html);

    // 1. Find the anchor form.
    let form = document
        .select(&sel.form)
        .next()
        .ok_or_else(|| anyhow!("form[name=frmFilterCallSign] not found"))?;

    // 2. Walk up to the nearest ancestor <table> whose class contains "table".
    let table_element = ancestors_of(form)
        .find(|el| {
            el.value().name() == "table"
                && el.value()
                .attr("class")
                .map(|c| c.split_whitespace().any(|w| w.contains("table")))
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow!("No ancestor <table> with a class containing 'table' found"))?;

    // 3. Collect headers and data rows.
    let mut headers: Vec<String> = Vec::new();
    let mut rows: Vec<BTreeMap<String, CellValue>> = Vec::new();
    let mut header_found = false;

    for row in table_element.select(&sel.row) {
        let ths: Vec<_> = row.select(&sel.th).collect();

        if !ths.is_empty() {
            if !header_found && ths.len() > 1 {
                headers = ths.iter()
                    .enumerate()
                    .map(|(i, th)| extract_cell(th, sel).text
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| format!("column_{i}")))
                    .collect();
                header_found = true;
            }
            continue;
        }

        if !header_found {
            continue;
        }

        let tds: Vec<_> = row.select(&sel.td).collect();
        if tds.is_empty() {
            continue;
        }

        let mut row_map: BTreeMap<String, CellValue> = BTreeMap::new();

        for (td, header) in tds.iter().zip(headers.iter()) {
            let cell = extract_cell(td, sel);

            if let Some(value) = cell.text.and_then(|t| CellValue::from_str(&t)) {
                row_map.insert(header.clone(), value);
            }
            if let Some(src) = cell.img_src {
                row_map.insert(format!("{header}_img"), CellValue::Text(src));
            }
        }

        if !row_map.is_empty() {
            rows.push(row_map);
        }
    }

    if headers.is_empty() {
        return Err(anyhow!("No headers found — page structure may have changed"));
    }

    Ok(ParsedTable { headers, rows })
}

// --- Helpers ---

/// Iterator over the ancestor `ElementRef`s of a given node.
fn ancestors_of(el: ElementRef<'_>) -> impl Iterator<Item = ElementRef<'_>> {
    std::iter::successors(el.parent(), |node| node.parent())
        .filter_map(ElementRef::wrap)
}

/// Everything we can extract from a single cell.
struct CellData {
    /// Visible text (direct text, link text, or img alt — in that priority order).
    text:    Option<String>,
    /// `src` of the first `<img>` found anywhere in the cell (inside or outside a link).
    img_src: Option<String>,
}

/// Extracts text and image src from a cell independently.
///
/// Text priority: direct text → link text → img alt.
/// Image src: first `<img src>` found anywhere in the cell, regardless of text.
fn extract_cell(cell: &ElementRef, sel: &Selectors) -> CellData {
    // img src is extracted independently of the text path.
    let img_src = cell.select(&sel.img)
        .next()
        .and_then(|img| img.value().attr("src"))
        .map(str::to_owned);

    // Direct text.
    let direct: String = cell.text().collect();
    let direct = direct.trim();
    if !direct.is_empty() {
        return CellData { text: Some(direct.to_owned()), img_src };
    }

    // Text inside <a>, then img alt inside <a>.
    if let Some(link) = cell.select(&sel.a).next() {
        let link_text: String = link.text().collect();
        let link_text = link_text.trim();
        if !link_text.is_empty() {
            return CellData { text: Some(link_text.to_owned()), img_src };
        }
        if let Some(alt) = link.select(&sel.img).next()
            .and_then(|img| img.value().attr("alt"))
        {
            return CellData { text: Some(alt.to_owned()), img_src };
        }
    }

    // img alt as last-resort text.
    let alt_text = cell.select(&sel.img)
        .next()
        .and_then(|img| img.value().attr("alt"))
        .map(str::to_owned);

    CellData { text: alt_text, img_src }
}
