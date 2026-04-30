use std::ops::Range;
use std::sync::Arc;

use crate::text::header::BlockHeaderSize;
use crate::Action;

pub mod weight {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum CustomWeight {
        Thin,
        ExtraLight,
        Light,
        Medium,
        Semibold,
        Bold,
        ExtraBold,
        Black,
    }
}

use weight::CustomWeight;

#[derive(Clone, Debug, Default)]
pub struct FormattedText {
    pub lines: Vec<FormattedTextLine>,
}

impl FormattedText {
    pub fn new<I>(lines: I) -> Self
    where
        I: IntoIterator<Item = FormattedTextLine>,
    {
        Self {
            lines: lines.into_iter().collect(),
        }
    }

    pub fn append_line(mut self, line: FormattedTextLine) -> Self {
        self.lines.push(line);
        self
    }
}

#[derive(Clone, Debug)]
pub enum FormattedTextLine {
    Heading(HeadingLine),
    Line(Vec<FormattedTextFragment>),
    TaskList(TaskListLine),
    OrderedList(OrderedListLine),
    UnorderedList(UnorderedListLine),
    CodeBlock(CodeBlockLine),
    Table(TableBlock),
    LineBreak,
    HorizontalRule,
    Embedded(String),
    Image(String),
}

impl FormattedTextLine {
    pub fn set_weight(&mut self, weight: Option<CustomWeight>) {
        for fragment in self.fragments_mut() {
            fragment.styles.weight = weight;
        }
    }

    pub fn hyperlinks(&self, _include_images: bool) -> Vec<(Range<usize>, Hyperlink)> {
        let mut links = Vec::new();
        let mut offset = 0;

        for fragment in self.fragments() {
            let len = fragment.text.chars().count();
            if let Some(link) = &fragment.link {
                links.push((offset..offset + len, link.clone()));
            }
            offset += len;
        }

        links
    }

    fn fragments(&self) -> &[FormattedTextFragment] {
        match self {
            Self::Heading(line) => &line.text,
            Self::Line(text)
            | Self::OrderedList(OrderedListLine {
                indented_text: IndentedText { text, .. },
                ..
            })
            | Self::UnorderedList(UnorderedListLine { text, .. })
            | Self::TaskList(TaskListLine { text, .. }) => text,
            Self::CodeBlock(_)
            | Self::Table(_)
            | Self::LineBreak
            | Self::HorizontalRule
            | Self::Embedded(_)
            | Self::Image(_) => &[],
        }
    }

    fn fragments_mut(&mut self) -> &mut [FormattedTextFragment] {
        match self {
            Self::Heading(line) => &mut line.text,
            Self::Line(text)
            | Self::OrderedList(OrderedListLine {
                indented_text: IndentedText { text, .. },
                ..
            })
            | Self::UnorderedList(UnorderedListLine { text, .. })
            | Self::TaskList(TaskListLine { text, .. }) => text,
            Self::CodeBlock(_)
            | Self::Table(_)
            | Self::LineBreak
            | Self::HorizontalRule
            | Self::Embedded(_)
            | Self::Image(_) => &mut [],
        }
    }
}

#[derive(Clone, Debug)]
pub struct HeadingLine {
    pub heading_size: BlockHeaderSize,
    pub text: Vec<FormattedTextFragment>,
}

#[derive(Clone, Debug)]
pub struct TaskListLine {
    pub text: Vec<FormattedTextFragment>,
    pub checked: bool,
}

#[derive(Clone, Debug)]
pub struct OrderedListLine {
    pub indented_text: IndentedText,
    pub number: usize,
}

#[derive(Clone, Debug)]
pub struct UnorderedListLine {
    pub text: Vec<FormattedTextFragment>,
    pub indent_level: usize,
}

#[derive(Clone, Debug)]
pub struct IndentedText {
    pub text: Vec<FormattedTextFragment>,
    pub indent_level: usize,
}

#[derive(Clone, Debug)]
pub struct CodeBlockLine {
    pub language: Option<String>,
    pub code: String,
}

#[derive(Clone, Debug, Default)]
pub struct TableBlock {
    pub rows: Vec<Vec<String>>,
}

impl TableBlock {
    pub fn to_plain_text(&self) -> String {
        self.rows
            .iter()
            .map(|row| row.join("\t"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Clone, Debug)]
pub struct FormattedTextFragment {
    pub text: String,
    pub styles: FragmentStyles,
    pub link: Option<Hyperlink>,
}

impl FormattedTextFragment {
    pub fn plain_text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            styles: FragmentStyles::default(),
            link: None,
        }
    }

    pub fn hyperlink_url(text: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            styles: FragmentStyles {
                underline: true,
                ..Default::default()
            },
            link: Some(Hyperlink::Url(url.into())),
        }
    }

    pub fn hyperlink_action<A>(text: impl Into<String>, action: A) -> Self
    where
        A: Action + 'static,
    {
        Self {
            text: text.into(),
            styles: FragmentStyles {
                underline: true,
                ..Default::default()
            },
            link: Some(Hyperlink::Action(Arc::new(action))),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FragmentStyles {
    pub weight: Option<CustomWeight>,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub inline_code: bool,
}

#[derive(Clone, Debug)]
pub enum Hyperlink {
    Url(String),
    Action(Arc<dyn Action>),
}

pub fn parse_markdown(markdown: &str) -> Result<FormattedText, String> {
    let mut lines = Vec::new();
    let mut code_block_language = None;
    let mut code_block = String::new();

    for line in markdown.lines() {
        if let Some(language) = line.strip_prefix("```") {
            if code_block_language.is_some() {
                lines.push(FormattedTextLine::CodeBlock(CodeBlockLine {
                    language: code_block_language.take().filter(|value| !value.is_empty()),
                    code: code_block.trim_end_matches('\n').to_owned(),
                }));
                code_block.clear();
            } else {
                code_block_language = Some(language.trim().to_owned());
            }
            continue;
        }

        if code_block_language.is_some() {
            code_block.push_str(line);
            code_block.push('\n');
            continue;
        }

        lines.push(parse_line(line));
    }

    if code_block_language.is_some() {
        lines.push(FormattedTextLine::CodeBlock(CodeBlockLine {
            language: code_block_language.filter(|value| !value.is_empty()),
            code: code_block.trim_end_matches('\n').to_owned(),
        }));
    }

    Ok(FormattedText::new(lines))
}

fn parse_line(line: &str) -> FormattedTextLine {
    if line.is_empty() {
        return FormattedTextLine::LineBreak;
    }

    if let Some((level, text)) = heading(line) {
        return FormattedTextLine::Heading(HeadingLine {
            heading_size: BlockHeaderSize::try_from(level).unwrap_or(BlockHeaderSize::Header6),
            text: parse_fragments(text),
        });
    }

    if let Some(text) = line.strip_prefix("* ").or_else(|| line.strip_prefix("- ")) {
        return FormattedTextLine::UnorderedList(UnorderedListLine {
            text: parse_fragments(text),
            indent_level: 0,
        });
    }

    FormattedTextLine::Line(parse_fragments(line))
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let hashes = line.chars().take_while(|ch| *ch == '#').count();
    if (1..=6).contains(&hashes) && line.as_bytes().get(hashes) == Some(&b' ') {
        Some((hashes, line[hashes + 1..].trim()))
    } else {
        None
    }
}

fn parse_fragments(text: &str) -> Vec<FormattedTextFragment> {
    let mut fragments = Vec::new();
    let mut remaining = text;

    while let Some(open_label) = remaining.find('[') {
        let before = &remaining[..open_label];
        if !before.is_empty() {
            fragments.push(FormattedTextFragment::plain_text(before));
        }

        let after_open_label = &remaining[open_label + 1..];
        let Some(close_label) = after_open_label.find(']') else {
            fragments.push(FormattedTextFragment::plain_text(after_open_label));
            return fragments;
        };

        let after_close_label = &after_open_label[close_label + 1..];
        if !after_close_label.starts_with('(') {
            fragments.push(FormattedTextFragment::plain_text("["));
            remaining = after_open_label;
            continue;
        }

        let Some(close_url) = after_close_label[1..].find(')') else {
            fragments.push(FormattedTextFragment::plain_text("["));
            remaining = after_open_label;
            continue;
        };

        let label = &after_open_label[..close_label];
        let url = &after_close_label[1..1 + close_url];
        fragments.push(FormattedTextFragment::hyperlink_url(label, url));
        remaining = &after_close_label[close_url + 2..];
    }

    if !remaining.is_empty() {
        fragments.push(FormattedTextFragment::plain_text(remaining));
    }

    fragments
}
