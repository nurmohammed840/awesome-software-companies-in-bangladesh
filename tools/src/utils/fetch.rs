use crate::{Result, utils::zlib};
use log::info;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::blocking::Client;
use std::{cell::OnceCell, env, fs, path::PathBuf};
use url::Url;

pub fn url_to_filename(url: &str) -> String {
    utf8_percent_encode(url, NON_ALPHANUMERIC).to_string()
}

pub fn fetch(url: &str) -> Result<String> {
    let url = normalize_url(url)?;
    let cache_path = cache_dir()?.join(url_to_filename(url.as_str()));

    if cache_path.is_file() {
        // info!("[CACHE] {}", url);

        let data = fs::read(&cache_path)
            .map(zlib::decompress)?
            .map(String::from_utf8)??;

        return Ok(data);
    }

    info!("[FETCH] {}", url);

    let data = client().get(url).send()?.error_for_status()?.text()?;
    fs::write(&cache_path, zlib::compress(&data)?)?;
    Ok(data)
}

pub fn normalize_url(url: &str) -> Result<Url> {
    let mut url = Url::parse(url)?;
    url.set_fragment(None);
    url.set_path(&url.path().trim_end_matches('/').to_owned());
    Ok(url)
}

fn client() -> &'static Client {
    use reqwest::{
        blocking::Client,
        header::{HeaderMap, HeaderValue, USER_AGENT},
        redirect::Policy,
    };
    use std::{sync::OnceLock, time::Duration};

    static CLIENT: OnceLock<Client> = OnceLock::new();

    let agents = HeaderValue::from_static(
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36",
    );

    CLIENT.get_or_init(|| {
        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, agents);

        Client::builder()
            .default_headers(headers)
            .danger_accept_invalid_certs(true)
            .redirect(Policy::limited(10))
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(15))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .gzip(true)
            .zstd(true)
            .build()
            .expect("failed to build http client")
    })
}

thread_local! {
    static CACHE_DIR: OnceCell<PathBuf> = const { OnceCell::new() };
}

pub fn cache_dir() -> Result<PathBuf> {
    CACHE_DIR.with(|cell| {
        if let Some(dir) = cell.get() {
            return Ok(dir.clone());
        }

        let dir = env::temp_dir().join("awesome-software-companies-in-bangladesh");
        fs::create_dir_all(&dir)?;
        cell.set(dir.clone()).unwrap();
        Ok(dir)
    })
}

pub fn clear_cache_dir() -> Result {
    let dir = cache_dir()?;

    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        if path.is_file() {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}
