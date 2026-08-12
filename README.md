# FlowLike Typst Nodes

WASM node package for compiling Typst documents inside FlowLike workflows.
The package compiles Typst source text into PDF/SVG artifacts, resolves relative
imports/assets from a FlowLike source folder, and returns structured compile
metadata.

## Nodes

Implemented in this package:

| Node | Purpose |
| --- | --- |
| `Typst Source to PDF` | Compile Typst source text into PDF and write the PDF to an output `FlowPath`. |
| `Typst File to PDF` | Read a `.typ` file from FlowLike storage and compile it into PDF. |
| `Typst Source to SVG` | Compile Typst source text into SVG and write the SVG to an output `FlowPath`. |
| `Typst File to SVG` | Read a `.typ` file from FlowLike storage and compile it into SVG. |
| `Typst Source to PDF + SVG` | Compile Typst source text once and write both output formats. |
| `Typst File to PDF + SVG` | Read a `.typ` file, compile once, and write both output formats. |
| `Markdown to Typst` | Convert Flow-Like Markdown into a safe, styleable Typst body fragment, including static Nivo and Plotly charts. |
| `Typst Source with Template to PDF` | Apply an optional imported `.typ` template function to source text and write PDF. |
| `Typst File with Template to PDF` | Read a `.typ` document file, apply an optional imported `.typ` template function, and write PDF. |
| `Typst Source with Template to SVG` | Apply an optional imported `.typ` template function to source text and write SVG. |
| `Typst File with Template to SVG` | Read a `.typ` document file, apply an optional imported `.typ` template function, and write SVG. |
| `Typst Source with Template to PDF + SVG` | Apply an optional imported `.typ` template function to source text and write both output formats. |
| `Typst File with Template to PDF + SVG` | Read a `.typ` document file, apply an optional imported `.typ` template function, and write both output formats. |
| `Validate Typst Source` | Compile Typst source text without exporting, returning success, page count, and warnings. |
| `Validate Typst File` | Read a `.typ` file and validate it without exporting. |

Primary pins:

| Pin | Type | Use |
| --- | --- | --- |
| `markdown` | `string` | Flow-Like Markdown input for `Markdown to Typst`. |
| `typst_source` | `string` | Generated Typst body fragment for a format/template node. |
| `source` | `string` | Main Typst document text used by source nodes. |
| `source_file` | `FlowPath` | Main `.typ` document source file used by file nodes. |
| `template_file` | `FlowPath` | Optional `.typ` module exporting the template function to import and apply. |
| `template_function` | `string` | Exported function name to import from `template_file`. Defaults to `template`. |
| `template_data` | `string` | JSON object exposed to template functions as named arguments through Typst `sys.inputs`. |
| `source_folder` | `FlowPath` | Folder used as the root for relative imports and assets referenced by `source`. |
| `output_file` | `FlowPath` | PDF/SVG file to write. Compile nodes always write to this path. |
| `pdf_output_file` | `FlowPath` | PDF output path for the `PDF + SVG` nodes. |
| `svg_output_file` | `FlowPath` | SVG output path for the `PDF + SVG` nodes. |
| `allow_universe_packages` | `bool` | Enables `@preview/...` Typst Universe package downloads through FlowLike host HTTP. Defaults to `true`; displayed as `Packages`. |

## Model-generated Markdown bodies

`Markdown to Typst` is a pure conversion node intended for model output. A
typical workflow is:

```text
Model Markdown -> Markdown to Typst -> Format String -> Typst Source to PDF
```

The user-owned Typst document keeps the global page, text, heading, table, list,
quote, and other show/set rules. Leave a distinctive Format String placeholder
where the generated body belongs, for example:

```typst
#set page(paper: "a4", margin: 2cm)
#set text(font: "Libertinus Serif", size: 11pt)
#show heading.where(level: 1): set text(fill: rgb("#c2410c"))

{markdown_body}
```

Connect `Markdown to Typst.typst_source` to the Format String input named
`markdown_body`, then connect `formatted_string` to `Typst Source to PDF.source`.
Markdown-authored text is emitted through Typst string literals, so text such as
`#pagebreak()` remains text rather than becoming executable Typst.

The converter covers Flow-Like's documented Markdown formatting: headings,
emphasis, strikethrough, inline and fenced code, nested ordered/unordered/task
lists, links, relative images, aligned GFM tables, blockquotes, alerts,
horizontal rules, footnotes, and visual line breaks. Remote Markdown images are
rendered as links because the Typst project resolver does not fetch arbitrary
HTTP image URLs; relative images continue to work when `source_folder` contains
the asset.

Fenced `nivo` and `plotly` blocks accept the same two forms as Flow-Like's
Markdown renderer:

````markdown
```nivo
type: bar
title: Quarterly revenue
showLegend: true
---
quarter,revenue,profit
Q1,120,30
Q2,150,42
Q3,170,55
```
````

or native JSON (`chartType` plus `data` for Nivo; `data`, `layout`, and optional
`config` for Plotly). Supported charts become package-free static SVG embedded
in the Typst fragment, so they require neither a browser nor a Typst Universe
package. Interactive-only options such as animation and hover behavior have no
static PDF equivalent. Chart shapes without a faithful static renderer fall
back to a deterministic data card; malformed chart blocks fall back to escaped
source instead of breaking the whole document.

See the [example gallery](examples/README.md) for standalone Markdown,
Flow-Like Nivo and Plotly configurations, user-owned Format String layouts, an
imported Typst template, and their checked-in Typst, PDF, and SVG outputs.

When `source_folder` is omitted for `Typst File to PDF`, relative files resolve
from the parent folder of `source_file`.

When `source_folder` points to:

```text
reports
```

then source text can reference files below that folder with relative Typst paths:

```typst
#import "templates/styles.typ": *
#image("images/logo.png")
#let csv = read("data/table.csv")
#bibliography("refs.bib")
```

Typst Universe imports work when `allow_universe_packages` is enabled and the
workflow grants outbound HTTP:

```typst
#import "@preview/conchord:0.4.0": smart-chord
```

Package files are fetched from `https://packages.typst.org`, decompressed in
memory for the current node run, and resolved separately from relative project
files.

## Using Templates

Use a template node when the layout stays the same but the document body or
workflow data changes. In Typst, a template is a function: named arguments carry
data such as a title or customer, and one final content argument receives the
document body.

### Quick start: file with template

The repository includes three files you can copy:

- [`report.typ`](examples/template/report.typ) is the reusable layout.
- [`document.typ`](examples/template/document.typ) is the document body.
- [`data.json`](examples/template/data.json) is the workflow data.

Put them in FlowLike storage under one project folder. `data.json` is shown in
the tree for convenience; this example pastes its contents into a string pin
rather than reading the file at runtime.

```text
reports/quarterly/
├── document.typ
├── data.json
├── templates/
│   └── report.typ
└── output/
    └── briefing.pdf
```

Create `reports/quarterly/templates/report.typ`:

```typst
#let report(
  title: "Untitled report",
  client: "",
  items: (),
  draft: false,
  doc,
) = [
  #set page(paper: "a4", margin: 2cm)
  #set text(font: "Libertinus Serif", size: 11pt)
  #set document(title: title)

  #if draft [
    #align(center)[*DRAFT*]
  ]

  = #title

  #if client != "" [
    *Client:* #client
  ]

  #if items.len() > 0 [
    == Highlights

    #for item in items [
      - #item
    ]
  ]

  #doc
]
```

Create `reports/quarterly/document.typ`:

```typst
= Executive Summary

This body is wrapped by the imported report template.
```

Add the **Typst File with Template to PDF** node and connect these values.
FlowPath pins are file/folder objects selected or produced by FlowLike, not
plain strings; the paths below show which objects to select.

| Pin | Value |
| --- | --- |
| `source_file` | FlowPath for `reports/quarterly/document.typ` |
| `template_file` | FlowPath for `reports/quarterly/templates/report.typ` |
| `template_function` | `report` |
| `template_data` | Contents of `data.json`, passed as a string |
| `source_folder` | FlowPath folder `reports/quarterly` |
| `output_file` | Writable FlowPath `reports/quarterly/output/briefing.pdf` |
| `allow_universe_packages` | `true`, unless the project must forbid package downloads |

Paste this string into `template_data`:

```json
{
  "title": "Quarterly Briefing",
  "client": "Rheosoph",
  "items": ["Data", "Automation", "AI"],
  "draft": true
}
```

Run the node. It writes the PDF to `output_file`, returns that FlowPath, and
returns a compile report with page and warning counts.

For an inline body, use the **Typst Source with Template to PDF** node and put
the contents of `document.typ` in the `source` pin. Keep the other pin values
the same.

### What the node generates

The node effectively adds the import and show rule for you:

```typst
#import "templates/report.typ": report as __flowlike_template
#show: __flowlike_template.with(..sys.inputs)

= Executive Summary

This body is wrapped by the imported report template.
```

Do not add that import/show pair to the body as well, or the template will be
applied twice.

For inline bodies, FlowLike creates a collision-safe internal entry file at the
project root. You do not choose or reserve its name, and a stored template can
safely be named `main.typ`.

### Source with Template or File with Template?

All six template nodes apply data in the same way. Choose the input style first,
then PDF, SVG, or both:

| Input style | Body comes from | Best for |
| --- | --- | --- |
| Source with Template | The `source` string pin | Typst text generated earlier in a workflow. |
| File with Template | The `.typ` file selected by `source_file` | Documents and multi-file projects kept in FlowLike storage. |

The combined PDF + SVG variant uses `pdf_output_file` and `svg_output_file`;
the single-format variants use `output_file`.

### Template reference

#### Function contract

The function exported by `template_file` must:

- Be defined with `#let`; its name must match `template_function`.
- Accept each top-level `template_data` key as a named parameter. Defaults are
  recommended for optional data.
- Accept the document body as a positional content parameter such as `doc`.
- Insert `doc` where the caller's body should appear.

This follows Typst's standard `#show: function.with(...)` template pattern. The
show rule supplies the final document body automatically.

#### How JSON values reach Typst

`template_data` must be one JSON object. Each top-level key becomes a named
template argument. FlowLike converts values to native Typst values:

| JSON | Typst |
| --- | --- |
| string | `str` |
| integer or decimal | `int` or `float` |
| boolean | `bool` |
| array | `array` |
| object | `dictionary` |
| `null` | `none` |

This is intentionally richer than the Typst CLI's `--input`, which exposes
command-line values as strings. The template above can therefore loop over
`items` directly and test `draft` as a boolean; it does not need to parse JSON.

#### Direct `sys.inputs` mode

For compatibility with the original template nodes, `template_file` is
optional. When it is unconnected, the node compiles the body unchanged and
still makes `template_data` available as `sys.inputs`. `template_function` is
then unused.

For example, set `source` to:

```typst
#let title = sys.inputs.at("title", default: "Untitled report")
#let items = sys.inputs.at("items", default: ())
#let draft = sys.inputs.at("draft", default: false)

#if draft [*DRAFT*]

= #title

#for item in items [
- #item
]
```

Use this mode for existing documents that already read `sys.inputs`, or for a
small one-off document. Connect `template_file` for a reusable layout that stays
separate from its content.

#### Project roots and paths

`source_folder` is the Typst project root and file-access boundary. For
predictable multi-file projects, connect it explicitly:

- `source_file` and `template_file` must be inside `source_folder` and use the
  same FlowLike storage backend as it.
- FlowPath pin paths must identify stored files directly; do not put `.` or `..`
  segments in them or start them with `/`.
- A relative path in an inline `source` body starts at `source_folder`. In a
  stored body it starts at the parent folder of `source_file`.
- Inside Typst code, a relative path is resolved from the `.typ` file containing
  that path. For example, `image("../assets/logo.svg")` inside
  `templates/report.typ` resolves from the `templates` folder.
- A leading `/` in Typst code means the project root, not the operating-system
  root. With the root above, `image("/assets/logo.svg")` starts at
  `reports/quarterly`.
- `output_file` is not part of Typst's read root and may be any writable
  FlowPath.

If `source_folder` is omitted, a file node normally uses the parent of
`source_file`; an inline template node can use the parent of `template_file`.
That default is convenient only when all referenced files are below that
folder. When body, template, or assets are in sibling folders, select their
common parent explicitly.

An empty body is valid: use it when the template itself produces all visible
content.

#### Common errors

| Error or symptom | Fix |
| --- | --- |
| `template_data must be valid JSON` | Use double quotes, remove trailing commas/comments, and validate the JSON string. |
| `template_data must be a JSON object` | Wrap the data in `{ ... }`; a top-level array or scalar is not accepted. |
| Typst reports `unexpected argument: ...` | Remove that JSON key or add a matching named parameter to the template function. The example's `draft` key is accepted by `draft: false`. |
| The imported function is missing | Make `template_function` exactly match an exported `#let` name in `template_file`; it defaults to `template`. |
| `source_file and template_file must refer to different files` | Keep the reusable template module and document body in separate `.typ` files. |
| A template or source path is outside the root, or uses a different store | Select a common `source_folder` and recreate all project FlowPaths from the same storage backend. |
| An image, bibliography, CSV, or import cannot be found | Resolve a relative path from the file that contains it, or use a `/project-root-relative` Typst path. |
| The template appears twice | Remove any manual template `#show` rule from the body; the template node inserts one. |
| An `@preview/...` package cannot be fetched | Enable `allow_universe_packages` and grant the workflow outbound HTTP, or remove the package import. |

#### Official Typst documentation and GitHub examples

- [Making a Template](https://typst.app/docs/tutorial/making-a-template/)
  explains template functions, everything show rules, named arguments, imports,
  and `.with(...)`.
- [Scripting: Modules](https://typst.app/docs/reference/scripting/#modules)
  documents `import` and `include`.
- [Paths and project roots](https://typst.app/docs/reference/foundations/path/)
  explains relative paths, `/` paths, and the project file-access boundary.
- [`sys.inputs`](https://typst.app/docs/reference/foundations/sys/) documents
  Typst's external input dictionary.
- The Typst team maintains an official
  [template repository](https://github.com/typst/templates); its
  [charged IEEE template](https://github.com/typst/templates/tree/main/charged-ieee)
  is a complete multi-file example.
- The [Typst package repository](https://github.com/typst/packages) contains
  the template packages published in Typst Universe.

## Typst Package Choices

The implementation uses current Typst Rust crates instead of shelling out to
the Typst CLI:

| Crate | Role |
| --- | --- |
| `typst` | Core compiler and document model. |
| `typst-as-lib` | In-memory source/file resolver around Typst. |
| `typst-assets` | Embedded default fonts for sandboxed compilation. |
| `typst-pdf` | PDF export. |
| `typst-svg` | SVG export. |

Typst Universe `@preview/...` packages are resolved through FlowLike host HTTP,
not the native Typst package cache. This keeps the WASM node aligned with
FlowLike permissions and avoids depending on a local filesystem cache.

## Recommended Node Roadmap

High-value follow-up nodes:

| Node | Why |
| --- | --- |
| `Typst Source to PNG` | Render page images with `typst-render` for previews and thumbnails. |
| `Typst Package Cache` | Persist approved Typst Universe packages through FlowLike storage so repeated runs do not need another network download. |
| `Typst Project Bundle to PDF` | Compile a whole project from a directory `FlowPath`, preserving relative imports and assets. |
| `Typst Diagnostics` | Return structured source spans and hints instead of debug-formatted warnings. |
| `Typst HTML Export` | Add HTML output when Typst's HTML export is stable enough for this runtime. |

Useful Typst Universe package categories for FlowLike automations:

| Category | Typical Packages |
| --- | --- |
| Charts and plots | `cetz`, `cetz-plot`, chart/table packages for reports. |
| Tables and data | Table styling, CSV/data helpers, report layout helpers. |
| Invoices/forms | Invoice, letter, resume, and business document templates. |
| Academic documents | Bibliography, theorem, code, and publication templates. |
| Diagrams | Diagram and drawing packages for architecture/report visuals. |

## Build

```bash
rustup target add wasm32-wasip2
cargo build --release
```

With mise:

```bash
mise run build
```

The release component is copied to `node.wasm`; the original artifact is at:

```text
target/wasm32-wasip2/release/flow_like_typst_nodes.wasm
```

## Test

```bash
cargo test --target $(rustc -vV | grep host | awk '{print $2}')
```

## Publishing

`node.wasm` is a generated release artifact and is intentionally ignored by
git. Build it immediately before publishing.

Release checklist:

1. Confirm `Cargo.toml`, `flow-like.toml`, and `README.md` use the same version.
2. Run `cargo test --target $(rustc -vV | grep host | awk '{print $2}')`.
3. Run `mise run build`.
4. Confirm `node.wasm` exists and is newer than the last source edit.
5. In FlowLike Desktop, open **Library -> Packages -> Publish**.
6. Select `node.wasm` and `flow-like.toml`.
7. Submit the package for review.
