Uses the flate2 raw in-memory "Decompress" struct to inflate streams without consuming data after the end of the stream. 

This is useful for decompressing objects from git packfiles which are self-terminating zlib streams. The position of the end of the stream is unknown until you have decompressed it fully, and flate2 will consume data past the end of the stream which makes it not possible to use to read packfiles.

Here are some bugs open on the flate2 github repo that discuss the issue:

https://github.com/rust-lang/flate2-rs/issues/14
https://github.com/rust-lang/flate2-rs/issues/28
