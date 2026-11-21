# Key-Helper

Automatically fixes text typed in the wrong keyboard layout on Windows.

## Build

```sh
cargo build --release
```

Binary: `target/release/key-helper.exe`

## Data

Dictionaries in `crates/core/data/` combine [danakt/russian-words](https://github.com/danakt/russian-words) (MIT), [dwyl/english-words](https://github.com/dwyl/english-words) (Unlicense), [hingston/russian](https://github.com/hingston/russian) (CC BY 2.5), [hermitdave/FrequencyWords](https://github.com/hermitdave/FrequencyWords) (content CC-BY-SA-4.0, code MIT), [zuzk/largest-word-list](https://github.com/zuzk/largest-word-list) (MIT), [open-dict-data/wikidict-wordlist](https://github.com/open-dict-data/wikidict-wordlist) (CC0), [jumberties/agk-wordlists](https://github.com/jumberties/agk-wordlists) (MPL-1.1, derived from brown-uk/dict_uk), [LDNOOBWV2/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words_V2](https://github.com/LDNOOBWV2/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words_V2) (CC0), [censor-text/profanity-list](https://github.com/censor-text/profanity-list) (Unlicense) and [readme-SVG/Banned-words](https://github.com/readme-SVG/Banned-words) (Apache-2.0). Supports English, Russian and Ukrainian layouts. See `LICENSE` for code terms.

## License

MIT License

Copyright (c) 2026 timur-code-guy

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
