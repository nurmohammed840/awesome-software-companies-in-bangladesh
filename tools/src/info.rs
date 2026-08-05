use scraper::*;
use serde_json as json;
use std::{collections::BTreeMap, path::Path, thread};

use crate::{
    Result,
    data::{Companies, Company},
    utils::{fetch::fetch, text_file::TextFile},
};

fn fetch_info_from(company: &Company) -> Result<Vec<json::Value>> {
    let html = fetch(&company.links.website)?;
    let selector = Selector::parse(r#"script[type="application/ld+json"]"#).unwrap();

    let values = Html::parse_document(&html)
        .select(&selector)
        .filter_map(|script| {
            let json: String = script.text().collect();
            json::from_str(&json).ok()
        })
        .collect();

    Ok(values)
}

pub fn fetch_info(companies: &Companies<'_>, dir: &Path) -> Result {
    let batch: Vec<_> = companies.iter().collect();

    let mut output = BTreeMap::new();

    for companies in batch.chunks(10) {
        thread::scope(|scope| {
            companies
                .iter()
                .map(|(name, company)| {
                    scope.spawn(move || {
                        fetch_info_from(company).map(|data| (name.to_string(), data))
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .filter_map(|handle| handle.join().ok()?.ok())
                .filter(|(_, list)| !list.is_empty())
                .for_each(|(key, value)| {
                    output.insert(key, value);
                });
        });
    }

    TextFile::read(dir.join("./data/info.json"))?.write(json::to_string_pretty(&output)?)?;

    Ok(())
}
