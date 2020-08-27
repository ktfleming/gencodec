`gencodec` is a tool to quickly generate [Circe](https://circe.github.io/circe/) encoders/decoders, inspired by [Mr. Boilerplate](https://japgolly.github.io/mr.boilerplate/).

# Installation

You can download a binary from the Github releases page (macOS only at the moment), or build it yourself with Cargo.

# Usage

`gencodec` accepts via stdin a line that contains a case class description (something like `case class Something(number: Int, whatever: String)`) and prints to stdout a companion object for that case class, along with a Circe Encoder and Decoder defined using the `forProductN` methods. For the given example, it would be
```scala
object Something {
  implicit lazy val encoder: Encoder[Something] = Encoder.forProduct2("number", "whatever")(a => (a.number, a.whatever))

  implicit lazy val decoder: Decoder[Something] = Decoder.forProduct2("number", "whatever")(Something.apply)
}
```

That's all it does!

# Integration

Used directly from the command-line, this wouldn't be that useful and Mr. Boilerplate would probably be better. I wrote `gencodec` specifically for the use-case of directly generating encoders and decoders from within Vim. You can put something like this in your `ftplugin/scala.vim` file:

```vim
function! GenCodec() range
  let input = shellescape(join(getline(a:firstline, a:lastline), ""))
  let output = extend([""], split(system('echo '.input.' | gencodec'), "\n"))
  call append(a:lastline, output)
endfunction

command! -range=% -nargs=0 GenCodec :<line1>,<line2>call GenCodec()
```

Then, you can select a case class definition in visual mode (it can span multiple lines) and run `:GenCodec`, at which point the generated companion object will be inserted after the selected range.

[Demonstration of Vim integration](https://asciinema.org/a/VDWBRblQ4K8BZhqb0uMh3rsdS)

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
