#![allow(dead_code)]

use std::fmt;
use std::slice::Iter;

use crate::utils::StrIterExt;
use crate::utils::chunks_exact;

#[derive(Debug)]
pub struct Table<'a> {
    pub header: Vec<&'a str>,
    pub rows: Vec<Vec<Text<'a>>>,
}

pub enum Text<'a> {
    Inline(&'a str),
    Multiline(Vec<&'a str>),
}

pub enum TextIter<'a, 'b> {
    Inline(Option<&'a str>),
    Multiline(Iter<'b, &'a str>),
}

impl<'a, 'b> Iterator for TextIter<'a, 'b> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TextIter::Inline(iter) => iter.take(),
            TextIter::Multiline(iter) => iter.next().copied(),
        }
    }
}

impl<'a> Text<'a> {
    pub fn iter(&self) -> TextIter<'a, '_> {
        match self {
            Text::Inline(s) => TextIter::Inline(Some(*s)),
            Text::Multiline(v) => TextIter::Multiline(v.iter()),
        }
    }
}

impl<'a> Text<'a> {
    fn first(&self) -> Option<&'a str> {
        self.iter().next()
    }
}

impl<'a> fmt::Debug for Text<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inline(v) => v.fmt(f),
            Self::Multiline(v) => v.fmt(f),
        }
    }
}

impl<'a> Text<'a> {
    fn push(&mut self, line: &'a str) {
        match self {
            Text::Inline(prev) => *self = Text::Multiline(vec![prev, line]),
            Text::Multiline(lines) => lines.push(line),
        }
    }
}

fn parse_table_rows(table: &str) -> (Vec<&str>, Vec<Text<'_>>) {
    let mut lines = table.lines().trimmed();

    let header = lines
        .next()
        .iter()
        .flat_map(|h| h.split('|'))
        .trimmed()
        .collect();

    let mut rows = Vec::new();

    for line in lines {
        if let Some(line) = line.strip_prefix('|') {
            rows.push(Text::Inline(line));
            continue;
        }

        if let Some(last) = rows.last_mut() {
            last.push(line)
        }
    }

    (header, rows)
}

fn parse_table(table: &str) -> Table<'_> {
    let (header, rows) = parse_table_rows(table);
    let column_count = header.len();

    Table {
        header,
        rows: chunks_exact(rows, column_count).collect(),
    }
}

pub fn parse_tables(input: &str) -> impl Iterator<Item = Table<'_>> {
    input.split("|===").skip(1).step_by(2).map(parse_table)
}

#[derive(Debug, Clone)]
pub struct Company<'a> {
    pub name: &'a str,
    pub technologies: Vec<&'a str>,
    pub links: Vec<Link<'a>>,
}

#[derive(Debug, Clone)]
pub enum Link<'a> {
    Website(&'a str),
    Facebook(&'a str),
    LinkedIn(&'a str),
    Twitter(&'a str),
    YouTube(&'a str),
    Github(&'a str),
    Instagram(&'a str),
}

impl<'a> Link<'a> {
    pub fn parse(s: &'a str) -> Option<Self> {
        let (url, kind) = s.rsplit_once('[')?;

        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        Some(match kind.strip_suffix(']')?.to_lowercase().as_str() {
            "website" => Link::Website(url),
            "facebook" => Link::Facebook(url),
            "linkedin" => Link::LinkedIn(url),
            "twitter" => Link::Twitter(url),
            "youtube" => Link::YouTube(url),
            "github" => Link::Github(url),
            "instagram" => Link::Instagram(url),
            _ty => {
                // eprintln!("Unknown link type: {_ty}");
                return None;
            }
        })
    }
}

impl<'a> Company<'a> {
    pub fn website(&self) -> Option<&str> {
        self.links.iter().find_map(|link| match link {
            Link::Website(url) => Some(*url),
            _ => None,
        })
    }

    fn get_from(row: &[Text<'a>]) -> Option<Self> {
        Some(Self {
            name: row.first()?.first()?,
            technologies: row.get(2)?.first()?.split(',').trimmed().collect(),
            links: row
                .get(3)?
                .iter()
                .flat_map(|a| a.split_whitespace())
                .filter_map(Link::parse)
                .collect(),
        })
    }
}

pub fn parse(txt: &str) -> impl Iterator<Item = Company<'_>> {
    parse_tables(txt)
        .flat_map(|table| table.rows)
        .flat_map(|row| Company::get_from(&row))
}

pub fn normalize_tag(tag: &str) -> Option<String> {
    let normalized = tag
        .split(['/', '(', '-', ')'])
        .next()
        .unwrap()
        .replace(['.', ' '], "")
        .to_lowercase();

    if TECHNOLOGY_IGNORE.contains(&normalized.as_str()) {
        return None;
    }

    let tag = TECHNOLOGY_ALIAS
        .iter()
        .find(|(alias, _)| *alias == normalized)
        .map(|(_, canonical)| canonical.to_lowercase());

    Some(tag.unwrap_or(normalized))
}

pub static TECHNOLOGY_ALIAS: &[(&str, &str)] = &[
    ("golang", "Go"),
    ("vue", "VueJS"),
    ("node", "NodeJS"),
    ("mongo", "MongoDB"),
    ("postgres", "PostgreSQL"),
    ("machinelearning", "ML"),
    ("net", ".Net"),
];

pub static TECHNOLOGY_IGNORE: &[&str] = &["css", "sqa"];
