use crate::{data::Schema, utils::levenshtein_distance::levenshtein_distance};

pub struct HintDex<'a> {
    keyword_map: Vec<(&'a str, String)>,
}

impl<'a> From<&'a Schema> for HintDex<'a> {
    fn from(schema: &'a Schema) -> Self {
        Self {
            keyword_map: schema
                .technologies
                .keys()
                .map(|keyword| (keyword.as_str(), normalize(keyword)))
                .collect(),
        }
    }
}

impl<'a> HintDex<'a> {
    pub fn closest_match(&self, input: &str) -> Vec<(&'a str, usize)> {
        let input = normalize(input);

        let mut matches: Vec<_> = self
            .keyword_map
            .iter()
            .map(|(keyword, normalized)| (*keyword, levenshtein_distance(&input, normalized)))
            .collect();

        matches.sort_unstable_by_key(|(_, distance)| *distance);
        matches
    }

    pub fn closest(&self, input: &str, take: usize) -> Vec<&'a str> {
        self.closest_match(input)
            .into_iter()
            .take(take)
            .map(|(keyword, _)| keyword)
            .collect()
    }
}

pub fn normalize(keyword: &str) -> String {
    keyword
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
