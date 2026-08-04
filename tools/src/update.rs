use crate::{
    PathBuf, Result,
    data::{Companies, CompanyData, Schema},
    repos::tech_companies_in_bangladesh::{self, Link, normalize_tag},
};
use std::{
    collections::{BTreeMap as Map, BTreeSet as Set},
    fs,
};

pub fn repos(schema: &Schema, companies: &mut Companies<'_>, dir: &PathBuf) -> Result {
    let technologies: Map<_, _> = schema
        .technologies
        .iter()
        .map(|(tag, _)| tag)
        .map(|s| (s.to_lowercase(), s))
        .collect();

    let tech_companies =
        fs::read_to_string(dir.join("./repos/tech-companies-in-bangladesh/README.adoc"))?;

    for c in tech_companies_in_bangladesh::parse(&tech_companies) {
        let Some(url) = c.website() else {
            continue;
        };

        let mut data = CompanyData::default();
        for link in c.links.iter() {
            match link {
                Link::Website(url) => data.set_website(*url),
                Link::Facebook(url) => data.set_facebook(*url),
                Link::LinkedIn(url) => data.set_linkedin(*url),
                Link::Twitter(url) => data.set_twitter(*url),
                Link::YouTube(url) => data.set_youtube(*url),
                Link::Github(url) => data.set_github(*url),
                Link::Instagram(_) => &mut data,
            };
        }

        let mut list: Vec<_> = c
            .technologies
            .iter()
            .filter_map(|tech| technologies.get(&normalize_tag(tech)?))
            .map(|s| s.to_string())
            .collect();

        {
            let mut implied = Set::new();

            list.iter()
                .for_each(|tech| schema.collect_implied_technologies(tech, &mut implied));

            list.retain(|tech| !implied.contains(tech.as_str()));
        }
        data.tech = list;

        match companies.find_by_website(url) {
            Some((_name, company)) => company.update_from(data),
            None => companies.add(c.name, data),
        }
    }

    Ok(())
}
