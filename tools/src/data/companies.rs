use super::{Map, Set};

use serde::{Deserialize, Serialize};
use std::{fmt, format as fmt, mem, ops};
use url::Url;

use crate::{
    Result,
    data::Schema,
    error::{Report, Span},
    utils::{keyword_hinter::HintDex, url_host},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Company {
    pub tech: Span<Set<Span<String>>>,

    #[serde(rename = "type")]
    pub ty: Span<Vec<Span<String>>>,

    #[serde(flatten)]
    pub links: Links,
}

impl Company {
    pub fn update_from(&mut self, data: CompanyData) {
        let mut new = Self::from(data);

        self.links = new.links;
        self.tech.append(&mut new.tech);
    }

    fn check_non_empty_list(&self, name: &str, report: &Report<'_>) {
        if self.ty.is_empty() {
            report.warning(
                self.ty.span(),
                fmt!("company '{name}' type shouldn't empty"),
                "expected at least one company type",
            );
        }

        if self.tech.is_empty() {
            report.warning(
                self.tech.span(),
                fmt!("company '{name}' tech shouldn't empty"),
                "expected at least one technology",
            );
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Links {
    pub website: Url,
    pub github: Option<Url>,
    pub twitter: Option<Url>,
    pub linkedin: Option<Url>,
    pub facebook: Option<Url>,
    pub youtube: Option<Url>,
}

impl fmt::Display for Links {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;

        for (label, url) in [
            ("Web", Some(&self.website)),
            ("GitHub", self.github.as_ref()),
            ("YouTube", self.youtube.as_ref()),
            ("LinkedIn", self.linkedin.as_ref()),
            // ("Facebook", self.facebook.as_ref()),
            // ("Twitter", self.twitter.as_ref()),
        ] {
            let Some(url) = url else { continue };

            if !mem::take(&mut first) {
                write!(f, " <br> ")?;
            }

            write!(f, "[{label}]({url})")?;
        }

        Ok(())
    }
}

impl fmt::Debug for Links {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ds = f.debug_struct("Links");
        for (name, value) in [
            ("website", Some(&self.website)),
            ("github", self.github.as_ref()),
            ("twitter", self.twitter.as_ref()),
            ("linkedin", self.linkedin.as_ref()),
            ("facebook", self.facebook.as_ref()),
            ("youtube", self.youtube.as_ref()),
        ] {
            if let Some(value) = value {
                ds.field(name, value);
            }
        }
        ds.finish()
    }
}

pub struct Companies<'a> {
    pub report: Report<'a>,
    pub list: Map<String, Company>,
}

impl<'a> Companies<'a> {
    pub fn add(&mut self, name: &str, data: CompanyData) {
        self.list.insert(name.to_owned(), data.into());
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&self.list)
    }

    pub fn find_by_website(&mut self, url: &Url) -> Option<(&String, &mut Company)> {
        let target = url_host(url)?;
        self.iter_mut().find(|(_, company)| {
            url_host(&company.links.website).is_some_and(|host| host == target)
        })
    }

    pub fn parse(txt: &'a str) -> Result<Self> {
        let report = Report::new(txt);

        let list = toml::from_str(txt)?;
        let this = Self { report, list };

        for (name, c) in this.iter() {
            c.check_non_empty_list(name, &this.report);
        }

        Ok(this)
    }

    pub fn check_no_redundant_technologies(&self, schema: &Schema) {
        let hint = HintDex::from(schema);

        for (name, company) in self.iter() {
            company
                .tech
                .iter()
                .filter(|tech| schema.is_unknown_technology(tech))
                .for_each(|tech| {
                    self.report.error(
                        tech.span(),
                        fmt!("unknown technology '{tech}'"),
                        fmt!("in company '{name}'"),
                        Some(hint.closest(tech, 3).join(", ")),
                    );
                });

            let mut implied = Set::new();

            company
                .tech
                .iter()
                .for_each(|tech| schema.collect_implied_technologies(tech, &mut implied));

            company
                .tech
                .iter()
                .filter(|tech| implied.contains(tech.as_str()))
                .for_each(|tech| {
                    self.report.error(
                        tech.span(),
                        fmt!("remove redundant technology '{tech}'"),
                        fmt!("in company '{name}'"),
                        None,
                    );
                });
        }
    }

    pub fn check_known_company_type(&self, schema: &Schema) {
        for (name, company) in self.iter() {
            company
                .ty
                .iter()
                .filter(|ty| schema.is_unknown_company_type(ty))
                .for_each(|ty| {
                    self.report.error(
                        ty.span(),
                        fmt!("unknown company type '{ty}'"),
                        fmt!("in company '{name}'"),
                        None,
                    );
                });
        }
    }
}

impl<'a> ops::Deref for Companies<'a> {
    type Target = Map<String, Company>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl<'a> ops::DerefMut for Companies<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

impl<'a> fmt::Debug for Companies<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.list.fmt(f)
    }
}

impl<'a> fmt::Display for Companies<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "| # | Company | Type | Technologies | Link |")?;
        writeln!(f, "|:-:| ------- | ---- | ------------ | ---- |")?;

        for (i, (name, company)) in self.iter().enumerate() {
            let ty = company
                .ty
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            let tech: String = company
                .tech
                .iter()
                .map(|s| format!("`{}` ", s.as_str()))
                .collect();

            let no = i + 1;
            let ty = if ty.is_empty() { "—" } else { &ty };
            let tech = if tech.is_empty() { "—" } else { tech.trim() };

            writeln!(f, "| {no} | {name} | {ty} | {tech} | {} |", company.links)?;
        }

        Ok(())
    }
}

pub struct CompanyData {
    pub tech: Vec<String>,
    pub ty: Vec<String>,
    pub links: Links,
}

impl Default for CompanyData {
    fn default() -> Self {
        Self {
            tech: Vec::new(),
            ty: Vec::new(),
            links: Links {
                website: Url::parse("https://example.com").unwrap(),
                github: None,
                twitter: None,
                linkedin: None,
                facebook: None,
                youtube: None,
            },
        }
    }
}

impl CompanyData {
    pub fn _new() -> Self {
        Self::default()
    }

    pub fn _add_tech(&mut self, tech: impl Into<String>) -> &mut Self {
        self.tech.push(tech.into());
        self
    }

    pub fn _add_type(&mut self, ty: impl Into<String>) -> &mut Self {
        self.ty.push(ty.into());
        self
    }

    pub fn set_website(&mut self, url: &Url) -> &mut Self {
        self.links.website = url.clone();
        self
    }

    pub fn set_github(&mut self, url: &Url) -> &mut Self {
        self.links.github = Some(url.clone());
        self
    }

    pub fn set_linkedin(&mut self, url: &Url) -> &mut Self {
        self.links.linkedin = Some(url.clone());
        self
    }

    pub fn set_twitter(&mut self, url: &Url) -> &mut Self {
        self.links.twitter = Some(url.clone());
        self
    }

    pub fn set_facebook(&mut self, url: &Url) -> &mut Self {
        self.links.facebook = Some(url.clone());
        self
    }

    pub fn set_youtube(&mut self, url: &Url) -> &mut Self {
        self.links.youtube = Some(url.clone());
        self
    }
}

impl From<CompanyData> for Company {
    fn from(data: CompanyData) -> Self {
        Company {
            tech: Span::new(data.tech.into_iter().map(Span::new).collect()),
            ty: Span::new(data.ty.into_iter().map(Span::new).collect()),
            links: data.links,
        }
    }
}
