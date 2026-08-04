pub mod text_file;
pub mod fetch;
pub mod zlib;
pub mod logger;
pub mod keyword_hinter;
pub mod levenshtein_distance;

use url::Url;

pub fn chunks_exact<T>(v: Vec<T>, n: usize) -> impl Iterator<Item = Vec<T>> {
    assert!(n > 0);
    let mut iter = v.into_iter();

    std::iter::from_fn(move || {
        let chunk: Vec<_> = iter.by_ref().take(n).collect();
        (chunk.len() == n).then_some(chunk)
    })
}

pub trait StrIterExt<'a>: Iterator<Item = &'a str> {
    fn trimmed(self) -> impl Iterator<Item = &'a str>
    where
        Self: Sized,
    {
        self.map(str::trim).filter(|s| !s.is_empty())
    }
}

impl<'a, I> StrIterExt<'a> for I where I: Iterator<Item = &'a str> {}

pub fn url_host(url: &str) -> Option<String> {
    let utl = Url::parse(url).ok()?;
    let host = utl.host_str()?.trim_start_matches("www.");
    Some(host.to_ascii_lowercase())
}
