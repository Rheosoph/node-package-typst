# Flow-Like Markdown: Core Formatting

This document is compiled directly from the Markdown-to-Typst body fragment without a user template.
This second source line becomes a visible line break, matching Flow-Like's `remarkBreaks` behavior.

## Emphasis and inline content

Use **bold**, *italic*, ***bold italic***, ~~strikethrough~~, and `inline code` in the same paragraph.

A safe link points to [Flow-Like](https://github.com/flow-like/flow-like), while a remote image becomes a link: ![Flow-Like logo](https://example.com/flow-like.png).

---

## Heading levels

### Heading level three

#### Heading level four

##### Heading level five

###### Heading level six

## Fenced code

```rust
fn main() {
    println!("Markdown stays readable, Typst stays valid.");
}
```

```json
{
  "node": "typst_markdown_to_typst",
  "pure": true,
  "permissions": []
}
```

