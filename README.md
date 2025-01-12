livetex
=

A TeX auto builder and deployer.

## Usage Example

```shell
./livetex -r ~/latex -a 0.0.0.0:8000 -c xelatex --output-format=pdf --halt-on-error
```

After running the command above, PDF preview is available on <http://localhost:8000>.
And, everytime the file `~/latex/*.tex` changes, the webpage will reload automatically.

## CLI Usage

<pre>A live integrated server that compiles TeX and serve its PDF automatically on source changes

<u style="text-decoration-style:solid"><b>Usage:</b></u> <b>livetex</b> [OPTIONS] <b>--root</b> &lt;ROOT&gt;

<u style="text-decoration-style:solid"><b>Options:</b></u>
  <b>-a</b>, <b>--addr</b> &lt;ADDR&gt;
          Server address [default: 0.0.0.0:8080]
  <b>-r</b>, <b>--root</b> &lt;ROOT&gt;
          Root directory to serve
  <b>-c</b>, <b>--build-command</b> &lt;BUILD_COMMAND&gt;...
          Command to build a TeX file. This argument should be present last
  <b>-h</b>, <b>--help</b>
          Print help</pre>

## Alternatives

In order to achieve automatic TeX compilation + PDF reloading, I've discovered some other approaches:

**Compilation**

- [autotex](https://crates.io/crates/autotex)
- `latexmk`
  ```shell
  latexmk -pdf -pvc -pdflatex='xelatex --halt-on-error' a.tex
  ```

**Previewing**

- [pdf-live-server](https://crates.io/crates/pdf-live-server)

However, none of them reflect compilation error on the PDF previewing side. Plus, I want
auto-preview with the browser's native PDF viewer (it's simple, just use `<embed>`) instead of using something like `pdf.js`, so
finally I created my version of this little toolchain.