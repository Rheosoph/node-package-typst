# Custom font test fixture

`texgyrecursor-regular.otf` is an unmodified TeX Gyre Cursor Regular font used
only by tests. Its family name is `TeX Gyre Cursor`, its PostScript name is
`TeXGyreCursor-Regular`, and its embedded version is `1.103`.

The file was copied from `examples/fonts/texgyrecursor-regular.otf` in the
published `typst-as-lib` 0.15.4 crate. Its SHA-256 is:

```text
734dde78cd582fb9457c959f3bf455fb439b100b5ce73c21916da2df6f32e8fb
```

This directory includes one font face from the upstream TeX Gyre collection;
the font file and its embedded notices have not been changed. The complete
upstream distribution and documentation are available from
[CTAN](https://ctan.org/tex-archive/fonts/tex-gyre/), with project information
on the [TeX Gyre Cursor page](https://ctan.org/pkg/tex-gyre-cursor).

## License and copyright

The font is distributed under the GUST Font License, which applies the LaTeX
Project Public License version 1.3c or later. This directory includes verbatim
copies of both license texts:

- [GUST-FONT-LICENSE.txt](GUST-FONT-LICENSE.txt), obtained from the
  [upstream CTAN distribution](https://mirrors.ctan.org/fonts/tex-gyre/doc/GUST-FONT-LICENSE.txt).
- [LPPL-1.3c.txt](LPPL-1.3c.txt), obtained from the
  [LaTeX Project](https://www.latex-project.org/lppl/lppl-1-3c.txt).

The font's embedded copyright notice reads:

> Copyright (URW)++, copyright 1999 by (URW)++ Design & Development; Cyrillic
> glyphs added by Valek Filippov, copyright 2001-2002; Vietnamese characters
> were added by Han The Thanh; copyright 2006, 2008 for TeX Gyre extensions by
> B. Jackowski and J.M. Nowacki (on behalf of TeX users groups). This work is
> released under the GUST Font License -- see
> http://tug.org/fonts/licenses/GUST-FONT-LICENSE.txt for details.

This repository maintains its test integration. Upstream font authors do not
provide support for that integration.
