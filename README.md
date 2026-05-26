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
| `Validate Typst Source` | Compile Typst source text without exporting, returning success, page count, and warnings. |

Primary pins:

| Pin | Type | Use |
| --- | --- | --- |
| `source` | `string` | Main Typst source text compiled as `main.typ` by the source/validate nodes. |
| `source_file` | `FlowPath` | Main `.typ` file used by `Typst File to PDF`. |
| `source_folder` | `FlowPath` | Folder used as the root for relative imports and assets referenced by `source`. |
| `output_file` | `FlowPath` | PDF/SVG file to write. Compile nodes always write to this path. |
| `allow_universe_packages` | `bool` | Enables `@preview/...` Typst Universe package downloads through FlowLike host HTTP. Defaults to `true`; displayed as `Packages`. |

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
| `Typst Template to PDF` | Accept `inputs_json`, expose it to Typst templates, and render PDFs for reports/invoices. |
| `Typst Source to PNG` | Render page images with `typst-render` for previews and thumbnails. |
| `Typst File to SVG` | Storage-based SVG export matching the PDF file workflow. |
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
