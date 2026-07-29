//! This module owns Git path-selection policy for documentation corpora.

const MARKDOWN_PRESENT: [&str; 7] = [
    "ls-files",
    "-z",
    "--cached",
    "--others",
    "--exclude-per-directory=.gitignore",
    "--",
    "*.md",
];
const MARKDOWN_DELETED: [&str; 5] = ["ls-files", "-z", "--deleted", "--", "*.md"];
const WORKFLOW_PRESENT: [&str; 8] = [
    "ls-files",
    "-z",
    "--cached",
    "--others",
    "--exclude-per-directory=.gitignore",
    "--",
    ".github/workflows/*.yml",
    ".github/workflows/*.yaml",
];
const WORKFLOW_DELETED: [&str; 6] = [
    "ls-files",
    "-z",
    "--deleted",
    "--",
    ".github/workflows/*.yml",
    ".github/workflows/*.yaml",
];

#[derive(Clone, Copy)]
pub(super) enum CorpusKind {
    Markdown,
    Workflow,
}

impl CorpusKind {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Markdown => "Markdown",
            Self::Workflow => "GitHub Actions workflow",
        }
    }

    pub(super) const fn present_arguments(self) -> &'static [&'static str] {
        match self {
            Self::Markdown => &MARKDOWN_PRESENT,
            Self::Workflow => &WORKFLOW_PRESENT,
        }
    }

    pub(super) const fn deleted_arguments(self) -> &'static [&'static str] {
        match self {
            Self::Markdown => &MARKDOWN_DELETED,
            Self::Workflow => &WORKFLOW_DELETED,
        }
    }

    pub(super) const fn present_operation(self) -> &'static str {
        match self {
            Self::Markdown => "git Markdown present paths",
            Self::Workflow => "git workflow present paths",
        }
    }

    pub(super) const fn deleted_operation(self) -> &'static str {
        match self {
            Self::Markdown => "git Markdown deleted paths",
            Self::Workflow => "git workflow deleted paths",
        }
    }
}
