# Binario

An asynchronous zero-fluff binary encoder / decoder

## Why?

This is very similar to [bincode](https://github.com/bincode-org/bincode) however it has been designed from the group up to work with [Tokio](https://tokio.rs)'s [AsyncRead](https://docs.rs/tokio/latest/tokio/io/trait.AsyncRead.html) [AsyncWrite](https://docs.rs/tokio/latest/tokio/io/trait.AsyncWrite.html) traits. This allows it to perform better when used with [TcpStream](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html)'s than equivalent bincode code.

**Benchmarking is still in progress but so far I am seeing about a 3 times performance win over bincode when used over a TCP connection. Binario does and will likely will continue to loose on in-memory head to head performance due to the overhead of async.**

## Usage with Tokio

It is **highly** recommended that you use Tokio's buffering utilities in conjunction with this crate to greatly improve performance. Refer to [BufReader](https://docs.rs/tokio/latest/tokio/io/struct.BufReader.html) and [BufWriter](https://docs.rs/tokio/latest/tokio/io/struct.BufWriter.html) for more information.