use std::io::{self, BufRead, Read};

#[derive(Debug)]
enum State {
    Eating(usize),
    Ate(usize, usize, flate2::Status),
    OutputFull,
    Finished(usize),
    DecompressError(flate2::DecompressError),
    IoError(io::Error),
}

pub struct Decompress<T> {
    reader: T,
    decompress: flate2::Decompress,
}

impl<T: BufRead> Decompress<T> {
    pub fn new(reader: T, header: bool) -> Decompress<T> {
        Decompress {
            reader,
            decompress: flate2::Decompress::new(header),
        }
    }

    fn is_reader_empty(&mut self) -> io::Result<bool> {
        self.reader.fill_buf().map(|data| data.is_empty())
    }

    fn eating(&mut self, output: &mut [u8], total: usize) -> State {
        match self.reader.fill_buf() {
            Ok(input) => match eat(&mut self.decompress, input, &mut output[total..]) {
                Ok((status, consumed, emitted)) => State::Ate(total + emitted, consumed, status),
                Err(e) => State::DecompressError(e),
            },
            Err(e) => State::IoError(e),
        }
    }

    fn ate(
        &mut self,
        total: usize,
        consumed: usize,
        status: flate2::Status,
        output_length: usize,
    ) -> State {
        use flate2::Status::*;
        self.reader.consume(consumed);
        let output_full = total == output_length;
        let reader_empty = self.is_reader_empty().unwrap_or(true);
        match status {
            Ok if !output_full => State::Eating(total),
            Ok if output_full => State::OutputFull,
            BufError if !output_full && reader_empty => State::Finished(total),
            BufError if output_full && !reader_empty => State::OutputFull,
            StreamEnd => State::Finished(total),
            _ => unreachable!(),
        }
    }
}

impl<T: BufRead> Read for Decompress<T> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let mut state = State::Eating(0);
        loop {
            state = match state {
                State::Eating(total) => self.eating(output, total),
                State::Ate(total, consumed, status) => {
                    self.ate(total, consumed, status, output.len())
                }
                State::Finished(total) => break Ok(total),
                State::OutputFull => break Ok(output.len()),
                State::IoError(e) => break Err(e),
                State::DecompressError(e) => break Err(io::Error::new(io::ErrorKind::Other, e)),
            };
        }
    }
}

// Implemented as a function to avoid borrowing errors.

fn eat(
    decompress: &mut flate2::Decompress,
    input: &[u8],
    output: &mut [u8],
) -> Result<(flate2::Status, usize, usize), flate2::DecompressError> {
    let i = decompress.total_in();
    let o = decompress.total_out();
    decompress
        .decompress(input, output, flate2::FlushDecompress::None)
        .map(|s| {
            (
                s,
                (decompress.total_in() - i) as usize,
                (decompress.total_out() - o) as usize,
            )
        })
}
