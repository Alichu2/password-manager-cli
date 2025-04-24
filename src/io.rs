use std::io::{BufRead, Write};

pub struct IoHandler<R, W> {
    pub reader: R,
    pub writer: W,
}

impl<R, W> IoHandler<R, W>
where
    R: BufRead,
    W: Write,
{
    pub fn echo(&mut self, ouput: &str) {
        write!(self.writer, "{}", ouput).expect("Error writing to output.");
    }

    pub fn prompt(&mut self, question: &str) -> String {
        self.echo(question);

        let mut input = String::new();
        self.reader
            .read_line(&mut input)
            .expect("Error reading line");

        input
    }
}
