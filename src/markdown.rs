//! Flow-Like Markdown to safe, styleable Typst body markup.
//!
//! The converter deliberately emits model-authored values only as Typst string
//! literals. This keeps Markdown such as `#pagebreak()` or `$secret$` as text
//! instead of allowing it to become executable Typst when inserted into a
//! document template.

use crate::chart::chart_to_typst;
use pulldown_cmark::{
    Alignment, BlockQuoteKind, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
};

const FLOWLIKE_DIRECTIVES: &[&str] = &["info", "warning", "error", "success", "tip", "spoiler"];
const MAX_DIRECTIVE_DEPTH: usize = 16;
const MAX_FOOTNOTE_EXPANSIONS: usize = 128;
const MAX_ACTIVE_FOOTNOTES: usize = 8;
const MAX_AST_DEPTH: usize = 128;
const MAX_MARKDOWN_BYTES: usize = 4 * 1024 * 1024;
const MAX_TYPST_BYTES: usize = 16 * 1024 * 1024;
const MAX_LINK_DESTINATION_BYTES: usize = 8_000;

#[derive(Clone, Copy)]
struct CalloutStyle {
    label: &'static str,
    icon: &'static str,
    accent: &'static str,
    fill: &'static str,
    stroke: &'static str,
}

fn directive_style(kind: &str) -> CalloutStyle {
    match kind {
        "info" => CalloutStyle {
            label: "Info",
            icon: "i",
            accent: "#1d4ed8",
            fill: "#eff6ff",
            stroke: "#93c5fd",
        },
        "warning" => CalloutStyle {
            label: "Warning",
            icon: "!",
            accent: "#92400e",
            fill: "#fffbeb",
            stroke: "#fbbf24",
        },
        "error" => CalloutStyle {
            label: "Error",
            icon: "x",
            accent: "#991b1b",
            fill: "#fef2f2",
            stroke: "#fca5a5",
        },
        "success" => CalloutStyle {
            label: "Success",
            icon: "+",
            accent: "#166534",
            fill: "#f0fdf4",
            stroke: "#86efac",
        },
        "tip" => CalloutStyle {
            label: "Tip",
            icon: "*",
            accent: "#5b21b6",
            fill: "#f5f3ff",
            stroke: "#c4b5fd",
        },
        "spoiler" => CalloutStyle {
            label: "Details",
            icon: ">",
            accent: "#334155",
            fill: "#f8fafc",
            stroke: "#cbd5e1",
        },
        _ => CalloutStyle {
            label: "Note",
            icon: "i",
            accent: "#334155",
            fill: "#f8fafc",
            stroke: "#cbd5e1",
        },
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Node {
    Document(Vec<Node>),
    Paragraph(Vec<Node>),
    Heading(u8, Vec<Node>),
    BlockQuote(Option<BlockQuoteKind>, Vec<Node>),
    CodeBlock(Option<String>, Vec<Node>),
    HtmlBlock(Vec<Node>),
    List(Option<u64>, Vec<Node>),
    Item(Vec<Node>),
    FootnoteDefinition(String, Vec<Node>),
    DefinitionList(Vec<Node>),
    DefinitionListTitle(Vec<Node>),
    DefinitionListDefinition(Vec<Node>),
    Table(Vec<Alignment>, Vec<Node>),
    TableHead(Vec<Node>),
    TableRow(Vec<Node>),
    TableCell(Vec<Node>),
    Emphasis(Vec<Node>),
    Strong(Vec<Node>),
    Strikethrough(Vec<Node>),
    Superscript(Vec<Node>),
    Subscript(Vec<Node>),
    Link {
        destination: String,
        title: String,
        children: Vec<Node>,
    },
    Image {
        destination: String,
        title: String,
        children: Vec<Node>,
    },
    Metadata(Vec<Node>),
    Text(String),
    Code(String),
    InlineMath(String),
    DisplayMath(String),
    Html(String),
    FootnoteReference(String),
    SoftBreak,
    HardBreak,
    Rule,
    TaskListMarker(bool),
}

#[derive(Debug)]
enum FrameKind {
    Document,
    Paragraph,
    Heading(u8),
    BlockQuote(Option<BlockQuoteKind>),
    CodeBlock(Option<String>),
    HtmlBlock,
    List(Option<u64>),
    Item,
    FootnoteDefinition(String),
    DefinitionList,
    DefinitionListTitle,
    DefinitionListDefinition,
    Table(Vec<Alignment>),
    TableHead,
    TableRow,
    TableCell,
    Emphasis,
    Strong,
    Strikethrough,
    Superscript,
    Subscript,
    Link { destination: String, title: String },
    Image { destination: String, title: String },
    Metadata,
}

#[derive(Debug)]
struct Frame {
    kind: FrameKind,
    children: Vec<Node>,
}

impl Frame {
    fn new(kind: FrameKind) -> Self {
        Self {
            kind,
            children: Vec::new(),
        }
    }

    fn finish(self) -> Node {
        match self.kind {
            FrameKind::Document => Node::Document(self.children),
            FrameKind::Paragraph => Node::Paragraph(self.children),
            FrameKind::Heading(level) => Node::Heading(level, self.children),
            FrameKind::BlockQuote(kind) => Node::BlockQuote(kind, self.children),
            FrameKind::CodeBlock(language) => Node::CodeBlock(language, self.children),
            FrameKind::HtmlBlock => Node::HtmlBlock(self.children),
            FrameKind::List(start) => Node::List(start, self.children),
            FrameKind::Item => Node::Item(self.children),
            FrameKind::FootnoteDefinition(label) => Node::FootnoteDefinition(label, self.children),
            FrameKind::DefinitionList => Node::DefinitionList(self.children),
            FrameKind::DefinitionListTitle => Node::DefinitionListTitle(self.children),
            FrameKind::DefinitionListDefinition => Node::DefinitionListDefinition(self.children),
            FrameKind::Table(alignments) => Node::Table(alignments, self.children),
            FrameKind::TableHead => Node::TableHead(self.children),
            FrameKind::TableRow => Node::TableRow(self.children),
            FrameKind::TableCell => Node::TableCell(self.children),
            FrameKind::Emphasis => Node::Emphasis(self.children),
            FrameKind::Strong => Node::Strong(self.children),
            FrameKind::Strikethrough => Node::Strikethrough(self.children),
            FrameKind::Superscript => Node::Superscript(self.children),
            FrameKind::Subscript => Node::Subscript(self.children),
            FrameKind::Link { destination, title } => Node::Link {
                destination,
                title,
                children: self.children,
            },
            FrameKind::Image { destination, title } => Node::Image {
                destination,
                title,
                children: self.children,
            },
            FrameKind::Metadata => Node::Metadata(self.children),
        }
    }
}

/// Convert Flow-Like-flavoured Markdown into a Typst body fragment.
pub(crate) fn markdown_to_typst(markdown: &str) -> Result<String, String> {
    markdown_to_typst_inner(markdown, 0)
}

fn markdown_to_typst_inner(markdown: &str, directive_depth: usize) -> Result<String, String> {
    if directive_depth > MAX_DIRECTIVE_DEPTH {
        return Err("Flow-Like directives are nested too deeply".to_string());
    }
    if markdown.len() > MAX_MARKDOWN_BYTES {
        return Err(format!(
            "Markdown input is too large (maximum {} MiB)",
            MAX_MARKDOWN_BYTES / (1024 * 1024)
        ));
    }

    let normalized = markdown.replace("\r\n", "\n").replace('\r', "\n");
    let preprocessed = preprocess_directives(&normalized);
    let document = parse(&preprocessed)?;
    validate_ast_depth(&document)?;
    let rendered = Renderer::new(&document, directive_depth).render_document(&document)?;
    if rendered.len() > MAX_TYPST_BYTES {
        return Err(format!(
            "Generated Typst body is too large (maximum {} MiB)",
            MAX_TYPST_BYTES / (1024 * 1024)
        ));
    }
    Ok(rendered)
}

fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_GFM
        | Options::ENABLE_DEFINITION_LIST
        | Options::ENABLE_SUPERSCRIPT
        | Options::ENABLE_SUBSCRIPT
}

fn parse(markdown: &str) -> Result<Node, String> {
    let mut stack = vec![Frame::new(FrameKind::Document)];

    for event in Parser::new_ext(markdown, markdown_options()) {
        match event {
            Event::Start(tag) => stack.push(Frame::new(frame_kind(tag))),
            Event::End(end) => {
                if stack.len() == 1 {
                    return Err(format!(
                        "Markdown parser emitted an unmatched {end:?} end tag"
                    ));
                }
                let frame = stack.pop().expect("the root frame remains on the stack");
                if !end_matches(&frame.kind, end) {
                    return Err(format!(
                        "Markdown parser emitted mismatched container tags: {:?} and {end:?}",
                        frame.kind
                    ));
                }
                push_node(&mut stack, frame.finish());
            }
            Event::Text(text) => push_node(&mut stack, Node::Text(text.into_string())),
            Event::Code(code) => push_node(&mut stack, Node::Code(code.into_string())),
            Event::InlineMath(math) => push_node(&mut stack, Node::InlineMath(math.into_string())),
            Event::DisplayMath(math) => {
                push_node(&mut stack, Node::DisplayMath(math.into_string()))
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                push_node(&mut stack, Node::Html(html.into_string()))
            }
            Event::FootnoteReference(label) => {
                push_node(&mut stack, Node::FootnoteReference(label.into_string()))
            }
            Event::SoftBreak => push_node(&mut stack, Node::SoftBreak),
            Event::HardBreak => push_node(&mut stack, Node::HardBreak),
            Event::Rule => push_node(&mut stack, Node::Rule),
            Event::TaskListMarker(checked) => push_node(&mut stack, Node::TaskListMarker(checked)),
        }
    }

    if stack.len() != 1 {
        return Err("Markdown parser left an unclosed container".to_string());
    }

    Ok(stack.pop().expect("the root frame exists").finish())
}

fn validate_ast_depth(root: &Node) -> Result<(), String> {
    let mut pending = vec![(root, 0usize)];
    while let Some((node, depth)) = pending.pop() {
        if depth > MAX_AST_DEPTH {
            return Err(format!(
                "Markdown structure is nested too deeply (maximum {MAX_AST_DEPTH} levels)"
            ));
        }
        if let Some(children) = node_children(node) {
            pending.extend(children.iter().map(|child| (child, depth + 1)));
        }
    }
    Ok(())
}

fn node_children(node: &Node) -> Option<&[Node]> {
    match node {
        Node::Document(children)
        | Node::Paragraph(children)
        | Node::Heading(_, children)
        | Node::BlockQuote(_, children)
        | Node::CodeBlock(_, children)
        | Node::HtmlBlock(children)
        | Node::List(_, children)
        | Node::Item(children)
        | Node::FootnoteDefinition(_, children)
        | Node::DefinitionList(children)
        | Node::DefinitionListTitle(children)
        | Node::DefinitionListDefinition(children)
        | Node::Table(_, children)
        | Node::TableHead(children)
        | Node::TableRow(children)
        | Node::TableCell(children)
        | Node::Emphasis(children)
        | Node::Strong(children)
        | Node::Strikethrough(children)
        | Node::Superscript(children)
        | Node::Subscript(children)
        | Node::Link { children, .. }
        | Node::Image { children, .. }
        | Node::Metadata(children) => Some(children),
        _ => None,
    }
}

fn push_node(stack: &mut [Frame], node: Node) {
    stack
        .last_mut()
        .expect("the Markdown parser always has a root frame")
        .children
        .push(node);
}

fn frame_kind(tag: Tag<'_>) -> FrameKind {
    match tag {
        Tag::Paragraph => FrameKind::Paragraph,
        Tag::Heading { level, .. } => FrameKind::Heading(heading_level(level)),
        Tag::BlockQuote(kind) => FrameKind::BlockQuote(kind),
        Tag::CodeBlock(kind) => FrameKind::CodeBlock(code_language(kind)),
        Tag::HtmlBlock => FrameKind::HtmlBlock,
        Tag::List(start) => FrameKind::List(start),
        Tag::Item => FrameKind::Item,
        Tag::FootnoteDefinition(label) => FrameKind::FootnoteDefinition(label.into_string()),
        Tag::DefinitionList => FrameKind::DefinitionList,
        Tag::DefinitionListTitle => FrameKind::DefinitionListTitle,
        Tag::DefinitionListDefinition => FrameKind::DefinitionListDefinition,
        Tag::Table(alignments) => FrameKind::Table(alignments),
        Tag::TableHead => FrameKind::TableHead,
        Tag::TableRow => FrameKind::TableRow,
        Tag::TableCell => FrameKind::TableCell,
        Tag::Emphasis => FrameKind::Emphasis,
        Tag::Strong => FrameKind::Strong,
        Tag::Strikethrough => FrameKind::Strikethrough,
        Tag::Superscript => FrameKind::Superscript,
        Tag::Subscript => FrameKind::Subscript,
        Tag::Link {
            dest_url, title, ..
        } => FrameKind::Link {
            destination: dest_url.into_string(),
            title: title.into_string(),
        },
        Tag::Image {
            dest_url, title, ..
        } => FrameKind::Image {
            destination: dest_url.into_string(),
            title: title.into_string(),
        },
        Tag::MetadataBlock(_) => FrameKind::Metadata,
    }
}

fn end_matches(kind: &FrameKind, end: TagEnd) -> bool {
    matches!(
        (kind, end),
        (FrameKind::Paragraph, TagEnd::Paragraph)
            | (FrameKind::Heading(_), TagEnd::Heading(_))
            | (FrameKind::BlockQuote(_), TagEnd::BlockQuote(_))
            | (FrameKind::CodeBlock(_), TagEnd::CodeBlock)
            | (FrameKind::HtmlBlock, TagEnd::HtmlBlock)
            | (FrameKind::List(None), TagEnd::List(false))
            | (FrameKind::List(Some(_)), TagEnd::List(true))
            | (FrameKind::Item, TagEnd::Item)
            | (FrameKind::FootnoteDefinition(_), TagEnd::FootnoteDefinition)
            | (FrameKind::DefinitionList, TagEnd::DefinitionList)
            | (FrameKind::DefinitionListTitle, TagEnd::DefinitionListTitle)
            | (
                FrameKind::DefinitionListDefinition,
                TagEnd::DefinitionListDefinition
            )
            | (FrameKind::Table(_), TagEnd::Table)
            | (FrameKind::TableHead, TagEnd::TableHead)
            | (FrameKind::TableRow, TagEnd::TableRow)
            | (FrameKind::TableCell, TagEnd::TableCell)
            | (FrameKind::Emphasis, TagEnd::Emphasis)
            | (FrameKind::Strong, TagEnd::Strong)
            | (FrameKind::Strikethrough, TagEnd::Strikethrough)
            | (FrameKind::Superscript, TagEnd::Superscript)
            | (FrameKind::Subscript, TagEnd::Subscript)
            | (FrameKind::Link { .. }, TagEnd::Link)
            | (FrameKind::Image { .. }, TagEnd::Image)
            | (FrameKind::Metadata, TagEnd::MetadataBlock(_))
    )
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn code_language(kind: CodeBlockKind<'_>) -> Option<String> {
    let CodeBlockKind::Fenced(info) = kind else {
        return None;
    };

    info.split_whitespace()
        .next()
        .filter(|language| !language.is_empty())
        .map(str::to_string)
}

struct Renderer<'a> {
    footnotes: HashMap<&'a str, &'a [Node]>,
    active_footnotes: RefCell<HashSet<&'a str>>,
    footnote_expansions: Cell<usize>,
    directive_depth: usize,
}

impl<'a> Renderer<'a> {
    fn new(document: &'a Node, directive_depth: usize) -> Self {
        let mut footnotes = HashMap::new();
        collect_footnotes(document, &mut footnotes);
        Self {
            footnotes,
            active_footnotes: RefCell::new(HashSet::new()),
            footnote_expansions: Cell::new(0),
            directive_depth,
        }
    }

    fn render_document(&self, document: &Node) -> Result<String, String> {
        let Node::Document(children) = document else {
            return Err("Markdown root was not a document".to_string());
        };

        let mut rendered = Vec::new();
        for child in children {
            if matches!(child, Node::FootnoteDefinition(_, _) | Node::Metadata(_)) {
                continue;
            }
            let block = self.render_block(child)?;
            if !block.trim().is_empty() {
                rendered.push(block);
            }
        }
        Ok(rendered.join("\n\n"))
    }

    fn render_block(&self, node: &Node) -> Result<String, String> {
        match node {
            Node::Document(_) => self.render_document(node),
            Node::Paragraph(children) => Ok(format!("#par[{}]", self.render_inlines(children)?)),
            Node::Heading(level, children) => Ok(format!(
                "#heading(level: {level})[{}]",
                self.render_inlines(children)?
            )),
            Node::BlockQuote(kind, children) => self.render_quote(*kind, children),
            Node::CodeBlock(language, children) => self.render_code_block(language, children),
            Node::HtmlBlock(children) => Ok(format!(
                "#raw({}, block: true, lang: \"html\")",
                typst_string(&plain_text(children))
            )),
            Node::List(start, children) => self.render_list(*start, children),
            Node::Item(children) => self.render_item(children),
            Node::DefinitionList(children) => self.render_definition_list(children),
            Node::DefinitionListTitle(children) | Node::DefinitionListDefinition(children) => {
                self.render_inlines(children)
            }
            Node::Table(alignments, children) => self.render_table(alignments, children),
            Node::TableHead(children) | Node::TableRow(children) | Node::TableCell(children) => {
                self.render_mixed_content(children)
            }
            Node::DisplayMath(math) => Ok(format!(
                "#raw({}, block: true, lang: \"latex\")",
                typst_string(math)
            )),
            Node::Rule => Ok("#line(length: 100%)".to_string()),
            Node::FootnoteDefinition(_, _) | Node::Metadata(_) => Ok(String::new()),
            _ => Ok(format!("#par[{}]", self.render_inline(node)?)),
        }
    }

    fn render_inlines(&self, nodes: &[Node]) -> Result<String, String> {
        let mut output = String::new();
        for node in nodes {
            match node {
                Node::Paragraph(children) => output.push_str(&self.render_inlines(children)?),
                Node::DisplayMath(_) | Node::Rule | Node::List(_, _) | Node::Table(_, _) => {
                    output.push_str(&self.render_block(node)?);
                }
                _ => output.push_str(&self.render_inline(node)?),
            }
        }
        Ok(output)
    }

    fn render_inline(&self, node: &Node) -> Result<String, String> {
        match node {
            Node::Text(text) => Ok(format!("#text({})", typst_string(text))),
            Node::Code(code) => Ok(format!("#raw({})", typst_string(code))),
            Node::InlineMath(math) => Ok(format!("#raw({}, lang: \"latex\")", typst_string(math))),
            Node::DisplayMath(_) => self.render_block(node),
            Node::Html(html) => Ok(format!("#text({})", typst_string(html))),
            Node::SoftBreak | Node::HardBreak => Ok("#linebreak()".to_string()),
            Node::Rule => self.render_block(node),
            Node::TaskListMarker(checked) => Ok(format!(
                "#text({})",
                typst_string(if *checked { "☒ " } else { "☐ " })
            )),
            Node::Emphasis(children) => Ok(format!("#emph[{}]", self.render_inlines(children)?)),
            Node::Strong(children) => Ok(format!("#strong[{}]", self.render_inlines(children)?)),
            Node::Strikethrough(children) => {
                Ok(format!("#strike[{}]", self.render_inlines(children)?))
            }
            Node::Superscript(children) => {
                Ok(format!("#super[{}]", self.render_inlines(children)?))
            }
            Node::Subscript(children) => Ok(format!("#sub[{}]", self.render_inlines(children)?)),
            Node::Link {
                destination,
                title: _,
                children,
            } => {
                let content = self.render_inlines(children)?;
                let destination = destination.trim();
                if destination.is_empty()
                    || destination.len() > MAX_LINK_DESTINATION_BYTES
                    || has_unsafe_uri_scheme(destination)
                {
                    Ok(content)
                } else {
                    Ok(format!("#link({})[{content}]", typst_string(destination)))
                }
            }
            Node::Image {
                destination,
                title,
                children,
            } => self.render_image(destination, title, children),
            Node::FootnoteReference(label) => self.render_footnote(label),
            Node::Paragraph(children)
            | Node::TableCell(children)
            | Node::DefinitionListTitle(children)
            | Node::DefinitionListDefinition(children) => self.render_inlines(children),
            _ => self.render_block(node),
        }
    }

    fn render_mixed_content(&self, children: &[Node]) -> Result<String, String> {
        let mut output = String::new();
        for child in children {
            match child {
                Node::Paragraph(inlines) => output.push_str(&self.render_inlines(inlines)?),
                Node::List(_, _)
                | Node::Table(_, _)
                | Node::BlockQuote(_, _)
                | Node::CodeBlock(_, _)
                | Node::Rule => output.push_str(&self.render_block(child)?),
                _ => output.push_str(&self.render_inline(child)?),
            }
        }
        Ok(output)
    }

    fn render_quote(
        &self,
        kind: Option<BlockQuoteKind>,
        children: &[Node],
    ) -> Result<String, String> {
        let body = self.render_blocks_inside(children)?;
        let Some(kind) = kind else {
            return Ok(format!("#quote(block: true)[{body}]"));
        };
        let (style, title) = match kind {
            BlockQuoteKind::Note => (directive_style("info"), "Note"),
            BlockQuoteKind::Tip => (directive_style("tip"), "Tip"),
            BlockQuoteKind::Important => (directive_style("warning"), "Important"),
            BlockQuoteKind::Warning => (directive_style("warning"), "Warning"),
            BlockQuoteKind::Caution => (directive_style("error"), "Caution"),
        };
        Ok(render_callout(style, title, &body))
    }

    fn render_code_block(
        &self,
        language: &Option<String>,
        children: &[Node],
    ) -> Result<String, String> {
        let body = plain_text(children);
        if let Some(language @ ("nivo" | "plotly")) = language.as_deref() {
            return match chart_to_typst(language, &body) {
                Ok(chart) => Ok(format!("#block(width: 100%)[{chart}]")),
                Err(error) => Ok(chart_fallback(language, &body, &error)),
            };
        }

        if let Some(language) = language.as_deref() {
            if let Some(kind) = language.strip_prefix("directive-") {
                if FLOWLIKE_DIRECTIVES.contains(&kind) {
                    return self.render_directive(kind, &body);
                }
            }
        }

        let mut args = format!("{}, block: true", typst_string(&body));
        if let Some(language) = language {
            args.push_str(&format!(", lang: {}", typst_string(language)));
        }
        // Typst's `raw(block: true)` controls raw layout, but the expression
        // itself can still participate in the surrounding flow. The explicit
        // full-width block keeps a fenced block from sharing a heading line.
        Ok(format!("#block(width: 100%)[#raw({args})]"))
    }

    fn render_directive(&self, kind: &str, body: &str) -> Result<String, String> {
        let (custom_title, markdown) = body
            .split_once("\n---\n")
            .map_or((None, body), |(title, content)| {
                (Some(title.trim()), content)
            });
        let style = directive_style(kind);
        let title = custom_title.filter(|title| !title.is_empty()).map_or_else(
            || style.label.to_string(),
            |title| format!("{}: {title}", style.label),
        );
        let content = markdown_to_typst_inner(markdown, self.directive_depth + 1)?;
        Ok(render_callout(style, &title, &content))
    }

    fn render_blocks_inside(&self, children: &[Node]) -> Result<String, String> {
        let mut output = String::new();
        for child in children {
            if matches!(child, Node::FootnoteDefinition(_, _) | Node::Metadata(_)) {
                continue;
            }
            output.push_str(&self.render_block(child)?);
        }
        Ok(output)
    }

    fn render_list(&self, start: Option<u64>, children: &[Node]) -> Result<String, String> {
        let uses_task_markers = start.is_none()
            && children.iter().any(|child| match child {
                Node::Item(item_children) => task_list_state(item_children).is_some(),
                _ => false,
            });
        let mut items = Vec::new();
        for child in children {
            let content = match child {
                Node::Item(item_children) if uses_task_markers => {
                    let task_state = task_list_state(item_children);
                    let marker = match task_state {
                        Some(true) => "☒",
                        Some(false) => "☐",
                        None => "•",
                    };
                    format!(
                        "#box(width: 1.35em)[#text({})]{}",
                        typst_string(marker),
                        self.render_item_content(item_children, task_state.is_some())?
                    )
                }
                Node::Item(item_children) => self.render_item(item_children)?,
                other => self.render_mixed_content(std::slice::from_ref(other))?,
            };
            items.push(format!("[{content}]"));
        }

        let function = if start.is_some() { "enum" } else { "list" };
        let mut args = String::new();
        if let Some(start) = start {
            args.push_str(&format!("start: {start}"));
            if !items.is_empty() {
                args.push_str(",\n");
            }
        } else if uses_task_markers {
            args.push_str("marker: []");
            if !items.is_empty() {
                args.push_str(",\n");
            }
        }
        args.push_str(&items.join(",\n"));
        Ok(format!("#block(width: 100%)[#{function}(\n{args}\n)]"))
    }

    fn render_item(&self, children: &[Node]) -> Result<String, String> {
        self.render_item_content(children, false)
    }

    fn render_item_content(
        &self,
        children: &[Node],
        omit_leading_task_marker: bool,
    ) -> Result<String, String> {
        let mut output = String::new();
        let mut paragraph_count = 0usize;
        let mut task_marker_omitted = false;
        for child in children {
            match child {
                Node::TaskListMarker(_) if omit_leading_task_marker && !task_marker_omitted => {
                    task_marker_omitted = true;
                }
                Node::Paragraph(inlines) => {
                    if paragraph_count > 0 {
                        output.push_str("#parbreak()");
                    }
                    let inlines = if omit_leading_task_marker && !task_marker_omitted {
                        task_marker_omitted = true;
                        inlines
                            .strip_prefix(&[Node::TaskListMarker(false)])
                            .or_else(|| inlines.strip_prefix(&[Node::TaskListMarker(true)]))
                            .unwrap_or(inlines)
                    } else {
                        inlines
                    };
                    output.push_str(&self.render_inlines(inlines)?);
                    paragraph_count += 1;
                }
                Node::List(_, _) => {
                    output.push_str("#linebreak()");
                    output.push_str(&self.render_block(child)?);
                }
                _ => output.push_str(&self.render_mixed_content(std::slice::from_ref(child))?),
            }
        }
        Ok(output)
    }

    fn render_definition_list(&self, children: &[Node]) -> Result<String, String> {
        let mut output = Vec::new();
        let mut index = 0usize;
        while index < children.len() {
            let title = match &children[index] {
                Node::DefinitionListTitle(nodes) => self.render_inlines(nodes)?,
                other => self.render_mixed_content(std::slice::from_ref(other))?,
            };
            let definition = children.get(index + 1).map_or_else(
                || Ok(String::new()),
                |node| match node {
                    Node::DefinitionListDefinition(nodes) => self.render_mixed_content(nodes),
                    other => self.render_mixed_content(std::slice::from_ref(other)),
                },
            )?;
            output.push(format!(
                "#block(above: 0.35em, below: 0.35em)[#strong[{title}]#block(inset: (left: 1em), above: 0.15em)[{definition}]]"
            ));
            index += 2;
        }
        Ok(output.join("\n"))
    }

    fn render_table(&self, alignments: &[Alignment], children: &[Node]) -> Result<String, String> {
        let mut header: Vec<&[Node]> = Vec::new();
        let mut rows: Vec<Vec<&[Node]>> = Vec::new();

        for child in children {
            match child {
                Node::TableHead(head_children) => {
                    if let Some(Node::TableRow(cells)) = head_children.first() {
                        header = table_cells(cells);
                    } else {
                        header = table_cells(head_children);
                    }
                }
                Node::TableRow(cells) => rows.push(table_cells(cells)),
                _ => {}
            }
        }

        let columns = alignments
            .len()
            .max(header.len())
            .max(rows.iter().map(Vec::len).max().unwrap_or(0));
        if columns == 0 {
            return Ok(String::new());
        }

        let mut aligned = Vec::with_capacity(columns);
        for index in 0..columns {
            aligned.push(
                match alignments.get(index).copied().unwrap_or(Alignment::None) {
                    Alignment::None | Alignment::Left => "left",
                    Alignment::Center => "center",
                    Alignment::Right => "right",
                },
            );
        }
        let align_tuple = if columns == 1 {
            format!("({},)", aligned[0])
        } else {
            format!("({})", aligned.join(", "))
        };

        let mut cells = Vec::new();
        if !header.is_empty() {
            let mut rendered = Vec::with_capacity(columns);
            for index in 0..columns {
                rendered.push(format!(
                    "[{}]",
                    self.render_table_cell(header.get(index).copied().unwrap_or(&[]))?
                ));
            }
            cells.push(format!("table.header({})", rendered.join(", ")));
        }
        for row in rows {
            for index in 0..columns {
                cells.push(format!(
                    "[{}]",
                    self.render_table_cell(row.get(index).copied().unwrap_or(&[]))?
                ));
            }
        }

        Ok(format!(
            "#table(\ncolumns: {columns},\nalign: {align_tuple},\n{}\n)",
            cells.join(",\n")
        ))
    }

    fn render_table_cell(&self, nodes: &[Node]) -> Result<String, String> {
        let mut output = String::new();
        for node in nodes {
            match node {
                Node::Paragraph(children) => output.push_str(&self.render_inlines(children)?),
                _ => output.push_str(&self.render_inline(node)?),
            }
        }
        Ok(output)
    }

    fn render_image(
        &self,
        destination: &str,
        title: &str,
        children: &[Node],
    ) -> Result<String, String> {
        let alt = plain_text(children);
        let destination = destination.trim();
        if destination.is_empty() || destination.len() > MAX_LINK_DESTINATION_BYTES {
            let label = if alt.trim().is_empty() {
                "Image"
            } else {
                alt.trim()
            };
            return Ok(format!("#text({})", typst_string(label)));
        }
        if is_safe_remote_link(destination) {
            let label = if alt.trim().is_empty() {
                destination
            } else {
                alt.trim()
            };
            return Ok(format!(
                "#link({})[#text({})]",
                typst_string(destination),
                typst_string(label)
            ));
        }

        if has_uri_scheme(destination) {
            let label = if alt.trim().is_empty() {
                "Image"
            } else {
                alt.trim()
            };
            return Ok(format!("#text({})", typst_string(label)));
        }

        let alt = if alt.is_empty() { title } else { &alt };
        Ok(format!(
            "#image({}, width: 100%, alt: {})",
            typst_string(destination),
            typst_string(alt)
        ))
    }

    fn render_footnote(&self, label: &str) -> Result<String, String> {
        let Some((defined_label, children)) = self.footnotes.get_key_value(label) else {
            return Ok(format!("#text({})", typst_string(&format!("[^{label}]"))));
        };
        if self.footnote_expansions.get() >= MAX_FOOTNOTE_EXPANSIONS {
            return Ok(format!("#text({})", typst_string(&format!("[^{label}]"))));
        }
        {
            let mut active = self.active_footnotes.borrow_mut();
            if active.len() >= MAX_ACTIVE_FOOTNOTES || !active.insert(*defined_label) {
                return Ok(format!("#text({})", typst_string(&format!("[^{label}]"))));
            }
        }
        self.footnote_expansions
            .set(self.footnote_expansions.get() + 1);

        let rendered = self.render_mixed_content(children);
        self.active_footnotes.borrow_mut().remove(*defined_label);
        Ok(format!("#footnote[{}]", rendered?))
    }
}

fn collect_footnotes<'a>(node: &'a Node, output: &mut HashMap<&'a str, &'a [Node]>) {
    match node {
        Node::FootnoteDefinition(label, children) => {
            output.insert(label, children);
        }
        Node::Document(children)
        | Node::Paragraph(children)
        | Node::Heading(_, children)
        | Node::BlockQuote(_, children)
        | Node::CodeBlock(_, children)
        | Node::HtmlBlock(children)
        | Node::List(_, children)
        | Node::Item(children)
        | Node::DefinitionList(children)
        | Node::DefinitionListTitle(children)
        | Node::DefinitionListDefinition(children)
        | Node::Table(_, children)
        | Node::TableHead(children)
        | Node::TableRow(children)
        | Node::TableCell(children)
        | Node::Emphasis(children)
        | Node::Strong(children)
        | Node::Strikethrough(children)
        | Node::Superscript(children)
        | Node::Subscript(children)
        | Node::Link { children, .. }
        | Node::Image { children, .. }
        | Node::Metadata(children) => {
            for child in children {
                collect_footnotes(child, output);
            }
        }
        _ => {}
    }
}

fn table_cells(nodes: &[Node]) -> Vec<&[Node]> {
    nodes
        .iter()
        .filter_map(|node| match node {
            Node::TableCell(children) => Some(children.as_slice()),
            _ => None,
        })
        .collect()
}

fn plain_text(nodes: &[Node]) -> String {
    let mut output = String::new();
    for node in nodes {
        match node {
            Node::Text(text)
            | Node::Code(text)
            | Node::InlineMath(text)
            | Node::DisplayMath(text)
            | Node::Html(text)
            | Node::FootnoteReference(text) => output.push_str(text),
            Node::SoftBreak | Node::HardBreak => output.push('\n'),
            Node::TaskListMarker(checked) => {
                output.push_str(if *checked { "[x] " } else { "[ ] " })
            }
            Node::Rule => output.push_str("---"),
            Node::Document(children)
            | Node::Paragraph(children)
            | Node::Heading(_, children)
            | Node::BlockQuote(_, children)
            | Node::CodeBlock(_, children)
            | Node::HtmlBlock(children)
            | Node::List(_, children)
            | Node::Item(children)
            | Node::FootnoteDefinition(_, children)
            | Node::DefinitionList(children)
            | Node::DefinitionListTitle(children)
            | Node::DefinitionListDefinition(children)
            | Node::Table(_, children)
            | Node::TableHead(children)
            | Node::TableRow(children)
            | Node::TableCell(children)
            | Node::Emphasis(children)
            | Node::Strong(children)
            | Node::Strikethrough(children)
            | Node::Superscript(children)
            | Node::Subscript(children)
            | Node::Link { children, .. }
            | Node::Image { children, .. }
            | Node::Metadata(children) => output.push_str(&plain_text(children)),
        }
    }
    output
}

fn is_safe_remote_link(destination: &str) -> bool {
    let lower = destination.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("ftp://")
}

fn has_uri_scheme(destination: &str) -> bool {
    let Some((scheme, _)) = destination.split_once(':') else {
        return false;
    };
    !scheme.is_empty()
        && scheme.chars().enumerate().all(|(index, character)| {
            character.is_ascii_alphabetic()
                || (index > 0
                    && (character.is_ascii_digit()
                        || character == '+'
                        || character == '-'
                        || character == '.'))
        })
}

fn has_unsafe_uri_scheme(destination: &str) -> bool {
    if !has_uri_scheme(destination) {
        return false;
    }
    let lower = destination.to_ascii_lowercase();
    !(lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
        || lower.starts_with("ftp://"))
}

fn task_list_state(children: &[Node]) -> Option<bool> {
    children.iter().find_map(|child| match child {
        Node::TaskListMarker(checked) => Some(*checked),
        Node::Paragraph(inlines) => match inlines.first() {
            Some(Node::TaskListMarker(checked)) => Some(*checked),
            _ => None,
        },
        _ => None,
    })
}

fn render_callout(style: CalloutStyle, title: &str, content: &str) -> String {
    format!(
        concat!(
            "#block(",
            "width: 100%, ",
            "breakable: true, ",
            "fill: rgb({fill}), ",
            "stroke: 0.8pt + rgb({stroke}), ",
            "radius: 4pt, ",
            "inset: (x: 10pt, y: 8pt), ",
            "above: 0.5em, ",
            "below: 0.5em",
            ")[",
            "#grid(",
            "columns: (1.4em, 1fr), ",
            "column-gutter: 0.4em, ",
            "align: (center, left), ",
            "[#box(width: 1.4em, height: 1.4em, fill: rgb({accent}), radius: 50%)[",
            "#align(center + horizon)[#text(fill: white, size: 0.72em, weight: \"bold\", {icon})]",
            "]], ",
            "[#text(fill: rgb({accent}), weight: \"bold\", {title})]",
            ")",
            "#block(above: 0.35em)[{content}]",
            "]"
        ),
        fill = typst_string(style.fill),
        stroke = typst_string(style.stroke),
        accent = typst_string(style.accent),
        icon = typst_string(style.icon),
        title = typst_string(title),
        content = content,
    )
}

fn chart_fallback(language: &str, body: &str, error: &str) -> String {
    let content = format!(
        "#par[#text({})]#block(above: 0.45em)[#raw({}, block: true, lang: {})]",
        typst_string(error),
        typst_string(body),
        typst_string(language)
    );
    render_callout(
        directive_style("error"),
        &format!("{} chart could not be rendered", title_case(language)),
        &content,
    )
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

fn preprocess_directives(markdown: &str) -> String {
    let lines: Vec<&str> = markdown.split('\n').collect();
    let mut output = Vec::with_capacity(lines.len());
    let mut index = 0usize;
    let mut active_fence: Option<MarkdownFence> = None;
    let mut no_more_directive_closers = false;

    while index < lines.len() {
        let line = lines[index];
        if let Some(fence) = active_fence {
            output.push(line.to_string());
            if closes_markdown_fence(line, fence) {
                active_fence = None;
            }
            index += 1;
            continue;
        }
        if let Some(fence) = opens_markdown_fence(line) {
            active_fence = Some(fence);
            output.push(line.to_string());
            index += 1;
            continue;
        }
        if no_more_directive_closers {
            output.push(line.to_string());
            index += 1;
            continue;
        }

        let Some(rest) = line.strip_prefix(":::") else {
            output.push(line.to_string());
            index += 1;
            continue;
        };
        let mut fields = rest.splitn(2, char::is_whitespace);
        let kind = fields.next().unwrap_or_default().to_ascii_lowercase();
        let title = fields.next().unwrap_or_default().trim();
        if !FLOWLIKE_DIRECTIVES.contains(&kind.as_str()) {
            output.push(line.to_string());
            index += 1;
            continue;
        }

        let Some(closing) = find_directive_closer(&lines, index + 1) else {
            // No subsequent directive opener can have a closer either. Keep
            // the remainder literal without repeatedly scanning to EOF.
            no_more_directive_closers = true;
            output.push(line.to_string());
            index += 1;
            continue;
        };

        let body = lines[index + 1..closing].join("\n");
        let fence = "`".repeat(longest_backtick_run(&body).max(2) + 1);
        output.push(format!("{fence}directive-{kind}"));
        if !title.is_empty() {
            output.push(format!("{title}\n---\n{body}"));
        } else {
            output.push(body);
        }
        output.push(fence);
        index = closing + 1;
    }

    output.join("\n")
}

#[derive(Clone, Copy)]
struct MarkdownFence {
    marker: u8,
    length: usize,
}

fn find_directive_closer(lines: &[&str], start: usize) -> Option<usize> {
    let mut active_fence: Option<MarkdownFence> = None;
    for (index, line) in lines.iter().enumerate().skip(start) {
        if let Some(fence) = active_fence {
            if closes_markdown_fence(line, fence) {
                active_fence = None;
            }
            continue;
        }
        if let Some(fence) = opens_markdown_fence(line) {
            active_fence = Some(fence);
        } else if line.trim() == ":::" {
            return Some(index);
        }
    }
    None
}

fn opens_markdown_fence(line: &str) -> Option<MarkdownFence> {
    let line = strip_markdown_indent(line)?;
    let marker = *line.as_bytes().first()?;
    if !matches!(marker, b'`' | b'~') {
        return None;
    }
    let length = line.bytes().take_while(|byte| *byte == marker).count();
    if length < 3 || (marker == b'`' && line[length..].contains('`')) {
        return None;
    }
    Some(MarkdownFence { marker, length })
}

fn closes_markdown_fence(line: &str, fence: MarkdownFence) -> bool {
    let Some(line) = strip_markdown_indent(line) else {
        return false;
    };
    let length = line
        .bytes()
        .take_while(|byte| *byte == fence.marker)
        .count();
    length >= fence.length && line[length..].trim().is_empty()
}

fn strip_markdown_indent(line: &str) -> Option<&str> {
    let spaces = line.bytes().take_while(|byte| *byte == b' ').count();
    (spaces <= 3).then(|| &line[spaces..])
}

fn longest_backtick_run(value: &str) -> usize {
    let mut longest = 0usize;
    let mut current = 0usize;
    for byte in value.bytes() {
        if byte == b'`' {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    longest
}

pub(crate) fn typst_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                output.push_str(&format!("\\u{{{:X}}}", character as u32));
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{compile_project, TypstProject, MAIN_FILE};

    fn convert(markdown: &str) -> String {
        markdown_to_typst(markdown).expect("Markdown should convert")
    }

    fn assert_compiles(markdown: &str) -> String {
        let source = convert(markdown);
        let project = TypstProject {
            main_path: MAIN_FILE.to_string(),
            source: source.clone(),
            source_folder: None,
            allow_universe_packages: false,
        };
        compile_project(project).unwrap_or_else(|error| {
            panic!("generated Typst did not compile:\n{source}\n\n{error}")
        });
        source
    }

    #[test]
    fn escapes_model_text_instead_of_executing_it() {
        let output = convert("Hello #pagebreak() [x] <tag> $secret$ \\\"quote\\\"");
        assert!(
            output.contains("#text(\"Hello #pagebreak() \")"),
            "unexpected generated source: {output}"
        );
        assert!(output.contains("#text(\"<tag>\")"));
        assert!(!output.starts_with("Hello #pagebreak"));
        assert_compiles("Hello #pagebreak() [x] <tag> $secret$ \\\"quote\\\"");
    }

    #[test]
    fn converts_flowlike_core_markdown() {
        let output = assert_compiles(
            r#"# Heading

Text with **bold**, *italic*, ~~gone~~, and `code`.
Second visual line.

> A quote
>> A nested quote

---

- Parent
  1. First
  2. Second
- [x] Done
- [ ] Open

[Link](https://example.com) and ![remote](https://example.com/image.png).

```rust
fn main() { println!("safe"); }
```
"#,
        );
        assert!(output.contains("#heading(level: 1)"));
        assert!(output.contains("#strong["));
        assert!(output.contains("#emph["));
        assert!(output.contains("#strike["));
        assert!(output.contains("#list("));
        assert!(output.contains("#enum("));
        assert!(output.contains("☒"));
        assert!(output.contains("marker: []"));
        assert!(output.contains("#text(\"☒\")"));
        assert!(!output.contains("#text(\"☒ \""));
        assert!(output.contains("#link("));
        assert!(output.contains("lang: \"rust\""));
        assert!(output.contains("#block(width: 100%)[#raw("));
    }

    #[test]
    fn task_lists_emit_exactly_one_visual_marker_per_item() {
        let output = assert_compiles("- ordinary\n- [x] finished\n- [ ] pending");
        assert!(output.contains("marker: []"));
        assert_eq!(output.matches("#text(\"☒\")").count(), 1);
        assert_eq!(output.matches("#text(\"☐\")").count(), 1);
        assert_eq!(output.matches("#text(\"•\")").count(), 1);
        assert!(!output.contains("#text(\"☒ \""));
        assert!(!output.contains("#text(\"☐ \""));
    }

    #[test]
    fn converts_aligned_gfm_tables() {
        let output = assert_compiles(
            "| Left | Center | Right |\n|:-----|:------:|------:|\n| **A** | `B` | C |",
        );
        assert!(output.contains("columns: 3"));
        assert!(output.contains("align: (left, center, right)"));
        assert!(output.contains("table.header("));
    }

    #[test]
    fn converts_local_images_and_links_remote_images() {
        let local = convert("![Logo](images/logo.svg \"Company\")");
        assert!(local.contains("#image(\"images/logo.svg\""));
        let remote = convert("![Logo](https://example.com/logo.svg)");
        assert!(remote.contains("#link(\"https://example.com/logo.svg\")"));
        assert!(!remote.contains("#image("));
    }

    #[test]
    fn converts_footnotes_and_gfm_alerts() {
        let output = assert_compiles(
            "Statement.[^source]\n\n[^source]: Evidence.\n\n> [!WARNING]\n> Check this.",
        );
        assert!(output.contains("#footnote["));
        assert!(output.contains("Warning"));
        assert!(output.contains("rgb(\"#fffbeb\")"));
        assert!(!output.contains("#quote(block: true)"));
    }

    #[test]
    fn self_referential_footnotes_are_rendered_safely() {
        let output = assert_compiles("Statement.[^loop]\n\n[^loop]: Again [^loop].");
        assert_eq!(output.matches("#footnote[").count(), 1);
        assert!(output.contains("[^loop]"));
    }

    #[test]
    fn converts_flowlike_directives_with_inner_markdown() {
        let output = assert_compiles(
            ":::warning Data Residency\n**Review** this setting.\n\n```json\n{\"ok\": true}\n```\n:::",
        );
        assert!(output.contains("Warning: Data Residency"));
        assert!(output.contains("#strong["));
        assert!(output.contains("lang: \"json\""));
        assert!(output.contains("rgb(\"#fffbeb\")"));
        assert!(output.contains("rgb(\"#92400e\")"));
        assert!(!output.contains("#quote(block: true)"));
    }

    #[test]
    fn directives_have_distinct_semantic_static_styles() {
        let markdown = [
            ("info", "#eff6ff", "Info"),
            ("warning", "#fffbeb", "Warning"),
            ("error", "#fef2f2", "Error"),
            ("success", "#f0fdf4", "Success"),
            ("tip", "#f5f3ff", "Tip"),
            ("spoiler", "#f8fafc", "Details"),
        ]
        .into_iter()
        .map(|(kind, _, _)| format!(":::{kind}\nBody.\n:::"))
        .collect::<Vec<_>>()
        .join("\n\n");
        let output = assert_compiles(&markdown);

        for (_, fill, title) in [
            ("info", "#eff6ff", "Info"),
            ("warning", "#fffbeb", "Warning"),
            ("error", "#fef2f2", "Error"),
            ("success", "#f0fdf4", "Success"),
            ("tip", "#f5f3ff", "Tip"),
            ("spoiler", "#f8fafc", "Details"),
        ] {
            assert!(output.contains(&format!("rgb(\"{fill}\")")));
            assert!(output.contains("#text(fill: rgb(") && output.contains(title));
        }
        assert_eq!(output.matches("width: 100%").count(), 6);
    }

    #[test]
    fn directives_respect_markdown_code_fences() {
        let literal = assert_compiles("```markdown\n:::warning\nnot a directive\n:::\n```");
        assert!(literal.contains("lang: \"markdown\""));
        assert!(!literal.contains("#quote(block: true)"));

        let nested = assert_compiles(
            ":::info\n```text\n::: is code, not the directive closer\n```\nAfter code.\n:::",
        );
        assert!(nested.contains("rgb(\"#eff6ff\")"));
        assert!(nested.contains("lang: \"text\""));
        assert!(nested.contains("After code."));
    }

    #[test]
    fn inserts_into_a_user_styled_typst_document() {
        let body = convert(
            "# Generated report\n\nA **styled** paragraph.\n\n| Metric | Value |\n| --- | ---: |\n| Revenue | 42 |",
        );
        let template = r##"#set page(width: 18cm, height: 24cm, margin: 1.5cm)
#set text(font: "Libertinus Serif", size: 10pt)
#show heading: it => block(fill: rgb("#eef2ff"), inset: 8pt, it)
#show table: it => block(stroke: rgb("#cbd5e1"), inset: 4pt, it)

{markdown_body}
"##;
        let source = template.replace("{markdown_body}", &body);
        assert!(!source.contains("{markdown_body}"));

        compile_project(TypstProject {
            main_path: MAIN_FILE.to_string(),
            source: source.clone(),
            source_folder: None,
            allow_universe_packages: false,
        })
        .unwrap_or_else(|error| {
            panic!("styled Markdown document did not compile:\n{source}\n\n{error}")
        });
    }

    #[test]
    fn converts_flowlike_chart_fences_to_package_free_images() {
        let nivo = assert_compiles(
            "```nivo\ntype: bar\ntitle: Quarterly revenue\nshowLegend: true\n---\nquarter,revenue,profit\nQ1,120,30\nQ2,150,42\n```",
        );
        assert!(nivo.contains("#image(bytes("));
        assert!(nivo.contains("#block(width: 100%)[#image("));
        assert!(nivo.contains("format: \"svg\""));
        assert!(!nivo.contains("@preview/"));

        let plotly = assert_compiles(
            r#"```plotly
{"data":[{"x":["Jan","Feb"],"y":[10,15],"type":"scatter","mode":"lines+markers","name":"Sales"}],"layout":{"title":"Monthly sales","xaxis":{"title":"Month"},"yaxis":{"title":"Revenue"}}}
```"#,
        );
        assert!(plotly.contains("#image(bytes("));
        assert!(!plotly.contains("@preview/"));
    }

    #[test]
    fn malformed_charts_fall_back_to_safe_source() {
        let output = assert_compiles("```nivo\n{ definitely-not-json }\n```");
        assert!(output.contains("#raw("));
        assert!(output.contains("invalid JSON"));
        assert!(output.contains("Nivo chart could not be rendered"));
        assert!(output.contains("rgb(\"#fef2f2\")"));
        assert!(output.contains("stroke: 0.8pt"));
    }

    #[test]
    fn strips_unsafe_link_and_image_schemes() {
        let output = assert_compiles(
            "[safe](https://example.com) [unsafe](javascript:alert(1)) [empty]() ![payload](data:image/svg+xml,bad) ![missing]()",
        );
        assert!(output.contains("#link(\"https://example.com\")"));
        assert!(!output.contains("#link(\"javascript:"));
        assert!(!output.contains("#link(\"\")"));
        assert!(!output.contains("#image(\"data:"));
        assert!(!output.contains("#image(\"\""));
    }

    #[test]
    fn overlong_link_destinations_degrade_to_plain_content() {
        let markdown = format!("[label](https://example.com/{})", "a".repeat(8_100));
        let output = assert_compiles(&markdown);
        assert!(output.contains("#text(\"label\")"));
        assert!(!output.contains("#link("));
    }

    #[test]
    fn rejects_excessively_nested_syntax_trees() {
        let mut node = Node::Text("leaf".to_string());
        for _ in 0..=MAX_AST_DEPTH {
            node = Node::Emphasis(vec![node]);
        }
        assert!(validate_ast_depth(&node)
            .unwrap_err()
            .contains("nested too deeply"));
    }

    #[test]
    fn typst_string_escapes_control_characters() {
        assert_eq!(
            typst_string("a\\b\"c\nd\t\u{7}"),
            "\"a\\\\b\\\"c\\nd\\t\\u{7}\""
        );
    }
}
