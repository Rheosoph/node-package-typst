# Flow-Like Directives

Flow-Like directive blocks are converted to static callouts in Typst documents.

:::info
This informational note uses its default title and supports **Markdown**.
:::

:::tip Model Prompting Tip
Ask the model to use simple tables and explicit chart types when document fidelity matters.
:::

:::success Validation Complete
- Markdown parsed
- Typst generated
- PDF compiled
:::

:::warning Review Required
| Check | Result |
| --- | --- |
| Brand colors | Pending |
| Page breaks | Passed |
:::

:::error Export Failed
Use this style for errors that must remain visible in a static document.
:::

:::spoiler Supporting Detail
Spoilers are rendered as a static **Details** callout because PDF output is not interactive.
:::

## Directives inside code stay literal

```markdown
:::warning
This is example source inside a fenced block, not a live directive.
:::
```

:::info Code Inside a Directive
The directive preprocessor also ignores delimiter-looking text inside its fenced code:

```text
::: is data here, not the end of the directive
```

Content after the code block remains inside the callout.
:::

