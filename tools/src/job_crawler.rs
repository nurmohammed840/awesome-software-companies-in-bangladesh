//! Website
//!   │
//!   ▼
//! Normalize HTML -> Markdown
//!   │
//!   ▼
//! LLM Extraction (qwen3:4b-instruct local)
//!   │
//!   ├── JobPost[] ──► Store
//!   │
//!   └── Url[] ──► Queue for Crawling
//!                      │
//!                      ▼
//!                 Fetch Website
//!                      │
//!                      └── repeat
#![allow(unused)]
use crate::Result;
use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest, ChatResponseFormat};
use url::Url;

#[derive(Debug)]
pub struct JobPost {
    pub title: String,

    /// Normalized role (e.g. "Software Engineer").
    pub role: String,
    pub employment_type: EmploymentType,
    pub posted_at: Option<PostedAt>,

    /// Markdown version of the job description.
    pub description: String,

    pub location: JobLocation,

    /// Technologies or skills.
    pub skills: Vec<String>,

    /// Ways to apply.
    pub apply: Vec<ApplicationMethod>,

    /// Original job posting URL.
    pub source: Url,
    pub confidence: f32,
}

#[derive(Debug)]
pub enum PostedAt {
    Absolute(chrono::DateTime<chrono::Utc>),
    Relative(String),
}

#[derive(Debug)]
pub enum ApplicationMethod {
    Email(String),
    Website(Url),
}

#[derive(Debug)]
pub enum EmploymentType {
    FullTime,
    PartTime,
    Contract,
    Temporary,
    Internship,
    Freelance,
}

#[derive(Debug)]
pub enum JobLocation {
    Remote,
    Hybrid(String),
    OnSite(String),
}

#[derive(Debug, Default)]
pub struct LLMOutput {
    pub posts: Vec<JobPost>,
    pub urls: Vec<CrawlUrl>,
}

#[derive(Debug)]
pub struct CrawlUrl {
    pub url: String,
    pub kind: CrawlKind,
}

#[derive(Debug)]
pub enum CrawlKind {
    Job,
    Careers,
}

const LLM_INPUT: &str = r#"
You are an information extraction engine.

Your task is to extract:

1. Job postings.
2. URLs to individual job posting pages (`kind: "Job"`).
3. URLs to career pages that may contain additional job postings (`kind: "Careers"`).

Rules:
- Return ONLY valid JSON matching the schema.
- Never guess; use `null` or `[]` when unknown.
- If no jobs are found, return `"posts": []`.
- If no crawlable URLs are found, return `"urls": []`.
- Ignore navigation and unrelated links (about, blog, login, privacy, terms, social, etc.).
- Keep `title` unchanged.
- Format `description` as Markdown. Reorganize if needed, but do not add or remove information.
- Include a confidence score (0.0–1.0) for each extracted job.

Schema:

```ts
export interface Output {
  posts: JobPost[];
  urls: CrawlUrl[];
}

interface JobPost {
  confidence: number;

  /** Original job posting URL. */
  source: string;

  title: string;
  role: string;

  /** Job description formatted as Markdown. */
  description: string;

  employmentType:
    | "FullTime"
    | "PartTime"
    | "Contract"
    | "Temporary"
    | "Internship"
    | "Freelance"
    | null;

  postedAt:
    | { kind: "Absolute"; value: string }
    | { kind: "Relative"; value: string }
    | null;

  location:
    | { kind: "Remote" }
    | { kind: "Hybrid"; value: string }
    | { kind: "OnSite"; value: string };

  skills: string[];

  apply: ApplicationMethod[];
}

type ApplicationMethod =
  | { kind: "Email"; value: string }
  | { kind: "Website"; value: string };

interface CrawlUrl {
  url: string;
  kind: "Job" | "Careers";
}
```
"#;

#[tokio::main]
pub async fn main() -> Result {
    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .new_headless_mode()
            // .with_head()
            .build()?,
    )
    .await?;

    let handle = tokio::spawn(async move {
        while let Some(_ev) = handler.next().await {
            // ...
        }
    });

    load_page(&browser, "https://brainstation-23.easy.jobs/").await?;

    browser.close().await?;
    handle.await?;

    Ok(())
}

// https://aistudio.google.com/api-keys

async fn load_page(browser: &Browser, url: &str) -> Result {
    let page = browser.new_page(url).await?;
    let html = page.wait_for_navigation().await?.content().await?;

    page.close().await?;

    let markdown = normalize_md(&html)?;
    log::info!("{url}: {markdown}");

    let client = genai::Client::default();

    let options = ChatOptions::default()
        .with_temperature(0.0)
        // .with_max_tokens(2048)
        .with_response_format(ChatResponseFormat::JsonMode);

    let chat_req = ChatRequest::new(vec![
        ChatMessage::system(LLM_INPUT.to_string()),
        ChatMessage::user(format!("Extract from this markdown.\n\n{markdown}")),
    ]);

    log::info!("[LLM-CALL]: {url}");
    let response = client
        .exec_chat("gemini-3.5-flash-lite", chat_req, Some(&options))
        .await?;

    log::info!("LLM Output:\n{}", response.texts().join("\n"));

    Ok(())
}

fn normalize_md(html: &str) -> Result<String> {
    let converter = htmd::HtmlToMarkdown::builder()
        .skip_tags(vec![
            "script", "style", "noscript", // Executable / styling
            "iframe", "canvas", "svg", "object", "embed", // Embedded content
            "picture", "source", "video", "audio", // Media
            "head", "template", // Other
        ])
        .build();

    Ok(converter.convert(html)?)
}
