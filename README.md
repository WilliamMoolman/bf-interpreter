# Brainfuck Interpreter
This project was built as a learning experience with interpreters. The implementation is a fairly standard [Brainfuck](https://en.wikipedia.org/wiki/Brainfuck) interpreter.

## Key points
- The memory is handled by two Vectors, one for positive indices, and one for negative indices. This lets us run any program without constraints on the memory size or only positive indices. 
- The `[` and `]` jump instructions have their offsets precomputed to speed up computation. This is achieve through iterating through the program, and storing the unmatched brackets on a LIFO stack.
- This leads to the entire program needing to be loaded into memory before running. Due to the complexity of using the language, large programs are exceedingly rare, and so this does not detract from the implementation.
- Reading in input was the most difficult part of the implementation, however using Rust's `std::io::stdin().bytes()` method trivialised this.
- Repeating instructions (such as Increment, Increment) can be combined to instead a Add u8 instruction.
- Pushing new values when we reach an out of bound memory is less efficient than using the `Vec::resize` method. Experimenting adding a single push after a resize slows it down, we can hypothesise that this decreased performance is due to the dynamic scaling of the vector.

## Usage
This interpreter can run through cargo, or through the compiled binary
```sh
$ cargo run -r programs/mandelbrot.b
```

## Performance
We can stress test the interpreter using a Mandlebrot set generator brainfuck program written by [Erik Bosman](https://github.com/erikdubbelboer/brainfuck-jit/blob/master/mandelbrot.bf). These times are the average of three runs on my own machine, and serve only to compare different implementations These times are the average of three runs on my own machine, and serve only to compare different implementations. Most implementations (unless stated otherwise) were developed by [pablojorge](https://github.com/pablojorge/brainfuck). By using Rust, we far outperform the python implementation, however we do not match Pablo's implementation. The main difference (apart from better written code, and a fixed sized buffer) is the compression of indentical commands, so instead of `+++` incrementing a bytes three times, we can simply add 3 to the byte. We add this optimisation, which halves our runtime. Using `Vec::resize` instead of `push`ing values one by one saved an additional 2 seconds in release.

| Features | Debug Mode (s) | Release Mode (s) |
| -------- | -------------- | ---------------- |
| Personal +Precompute Jump | 167 | 26 |
| Pablo Rust | 15 | 7 |
| Pablo Python | -- | 1811 |
| Personal +Compression | 83 | 12 | 
| Personal +`resize` | 74 | 10 | 


## Other References
[Daniel B. Cristofani](https://brainfuck.org) for various resources and brainfuck programs
