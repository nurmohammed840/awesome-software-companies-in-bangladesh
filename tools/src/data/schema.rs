use super::{Map, Set, parse_map_strings};
use crate::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Schema {
    #[serde(rename = "Company-Type")]
    pub company_type: Set<String>,

    #[serde(rename = "Technology", deserialize_with = "parse_map_strings")]
    pub technologies: Map<String, Vec<String>>,
}

impl Schema {
    pub fn parse(txt: &str) -> Result<Schema> {
        let this: Schema = toml::from_str(txt)?;
        this.check()?;
        Ok(this)
    }

    fn check(&self) -> Result<(), String> {
        for (name, list) in self.technologies.iter() {
            self.check_cycle(name, list, &mut Vec::new())?;
        }
        Ok(())
    }

    pub fn collect_implied_technologies<'a>(&'a self, name: &str, set: &mut Set<&'a str>) {
        let Some(list) = self.technologies.get(name) else {
            return;
        };

        for name in list {
            if set.insert(name) {
                self.collect_implied_technologies(name, set);
            }
        }
    }

    pub fn is_unknown_company_type(&self, name: &str) -> bool {
        !self.company_type.contains(name)
    }

    pub fn is_unknown_technology(&self, tech: &str) -> bool {
        !self.technologies.contains_key(tech)
    }

    fn check_cycle<'a>(
        &'a self,
        technology: &'a str,
        technology_list: &'a [String],
        visiting: &mut Vec<&'a str>,
    ) -> Result<(), String> {
        if visiting.contains(&technology) {
            visiting.push(technology);
            return Err(format!("cyclic dependency: {}", visiting.join(" -> ")));
        }

        visiting.push(technology);

        for name in technology_list {
            let list = self.technologies.get(name).ok_or_else(|| {
                format!("technology '{technology}' references unknown technology '{name}'")
            })?;

            self.check_cycle(name, list, visiting)?;
        }

        visiting.pop();

        Ok(())
    }
}
