# Fixture Origin and Third-Party Notice

The profile record, manifests, source-like input, pseudo-random recipes,
mutation recipes, expected boundaries, and independent checker were created
specifically for Keep and are licensed under the repository's Apache License
2.0.

The boundary algorithm is based on:

> Wen Xia, Yukun Zhou, Hong Jiang, Dan Feng, and Yu Hua. “FastCDC: a Fast and
> Efficient Content-Defined Chunking Approach for Data Deduplication.” 2016
> USENIX Annual Technical Conference.
>
> <https://www.usenix.org/system/files/conference/atc16/atc16-paper-xia.pdf>

The Gear-table generation recipe, table values, scalar algorithm semantics,
and distributed masks were cross-checked against `nlfiedler/fastcdc-rs` tag
`4.0.1`, commit `2e47aa3146c6dbae34896997eebd162b280a7052`:

<https://github.com/nlfiedler/fastcdc-rs/tree/4.0.1>

During acceptance, that pinned implementation independently reproduced the
exact `short-mask-match`, `probe-byte-carry`, and `target-long-transition`
boundary sequences recorded by this corpus.

That source is licensed under the following MIT license:

```text
The MIT License (MIT)

Copyright (c) 2020 Nathan Fiedler

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

The authoritative `gear-table.bin` is reproducible from the language-neutral
recipe in `README.md`. Its first and last entries are, respectively,
`0x3b5d3c7d207e37dc` and `0xaabd2b2a451504e1`.
