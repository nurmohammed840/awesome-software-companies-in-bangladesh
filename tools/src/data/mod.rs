mod companies;
mod schema;

pub use companies::*;
pub use schema::*;

use serde::{Deserialize, Deserializer};
use std::collections::{BTreeMap as Map, BTreeSet as Set};

#[derive(Deserialize)]
#[serde(untagged)]
enum Strings {
    One(String),
    Many(Vec<String>),
}

fn parse_map_strings<'de, D>(de: D) -> Result<Map<String, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let list = Map::<String, Strings>::deserialize(de)?
        .into_iter()
        .map(|(key, strings)| {
            let list = match strings {
                Strings::One(s) => vec![s],
                Strings::Many(s) => s,
            };
            (key, list)
        });

    Ok(list.collect())
}
