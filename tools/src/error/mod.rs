use std::{
    borrow::Cow,
    fmt,
    ops::{self, Range},
};

use annotate_snippets::{renderer::DecorStyle, *};
use log::log;
use serde::{Deserialize, Serialize};

pub struct Report<'a> {
    source: &'a str,
    renderer: Renderer,
}

impl<'a> Report<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            renderer: Renderer::styled().decor_style(DecorStyle::Unicode),
        }
    }

    fn emit(
        &self,
        level: Level<'a>,
        span: Range<usize>,
        message: impl Into<Cow<'a, str>>,
        label: impl Into<OptionCow<'a>>,
        hint: Option<String>,
    ) {
        let lvl = match level {
            Level::ERROR => log::Level::Error,
            Level::WARNING => log::Level::Warn,
            _ => log::Level::Info,
        };

        let mut report = level.primary_title(message).element(
            Snippet::source(self.source)
                .annotation(AnnotationKind::Primary.span(span).label(label)),
        );

        if let Some(hint) = hint {
            report = report.element(Level::HELP.message(hint));
        }

        log!(lvl, "{}\n", self.renderer.render(&[report]),);
    }

    pub fn error(
        &self,
        span: Range<usize>,
        message: impl Into<Cow<'a, str>>,
        label: impl Into<OptionCow<'a>>,
        hint: Option<String>,
    ) {
        self.emit(Level::ERROR, span, message, label, hint);
    }

    pub fn warning(
        &self,
        span: Range<usize>,
        message: impl Into<Cow<'a, str>>,
        label: impl Into<OptionCow<'a>>,
    ) {
        self.emit(Level::WARNING, span, message, label, None);
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Span<T>(toml::Spanned<T>);

impl<T> Span<T> {
    pub fn new_with(span: Range<usize>, value: T) -> Self {
        Self(toml::Spanned::new(span, value))
    }

    pub fn new(value: T) -> Self {
        Self::new_with(0..0, value)
    }

    pub fn span(&self) -> Range<usize> {
        self.0.span()
    }

    pub fn _into_inner(self) -> T {
        self.0.into_inner()
    }
}

impl<T: fmt::Debug> fmt::Debug for Span<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_ref().fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for Span<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_ref().fmt(f)
    }
}

impl<T> ops::Deref for Span<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.get_ref()
    }
}

impl<T> ops::DerefMut for Span<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.get_mut()
    }
}

impl<T: Default> Default for Span<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}
