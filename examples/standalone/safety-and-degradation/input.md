# Safety and Graceful Degradation

Model-authored Typst-looking text is displayed literally: `#pagebreak()` in code and #pagebreak() in prose do not execute. Brackets [like these], dollar signs $secret$, angle brackets <unsafe-tag>, quotes "quoted", and backslashes \ remain content.

## Links and images

[HTTPS links work](https://example.com), [email links work](mailto:hello@example.com), and [unsafe JavaScript destinations degrade to plain text](javascript:alert(1)).

![A remote image is represented by a link](https://example.com/image.png)

![A data URI cannot become an image](data:image/svg+xml,bad)

## Malformed chart

An invalid chart does not abort the entire document. Its escaped source and diagnostic are shown instead.

```nivo
{ this is not valid JSON }
```

## Cyclic footnote

This reference is safe even though its definition refers to itself.[^loop]

[^loop]: The cycle ends as a literal [^loop] marker.

