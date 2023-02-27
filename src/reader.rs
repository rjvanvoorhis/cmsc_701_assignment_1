use std::{
    fs::{self, File},
    io::{self, BufRead},
    path::Path,
};

use rand::{rngs::ThreadRng, seq::IteratorRandom, thread_rng};

use crate::record::Record;
use eyre::eyre;

pub const START_CHARACTER: char = '>';

pub struct Reader {
    reader: io::BufReader<fs::File>,
    buffer: String,
}

impl Reader {
    pub fn new(reader: io::BufReader<fs::File>) -> Self {
        Self {
            reader,
            buffer: String::new(),
        }
    }

    pub fn from_file<P>(filename: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(Self::new(io::BufReader::new(file)))
    }

    fn sanitize_line(&self, line: &str, rng: &mut ThreadRng) -> String {
        let converted: String = line
            .trim_end()
            .to_uppercase()
            .chars()
            .map(|x| match x {
                'A' | 'C' | 'T' | 'G' => x,
                _ => "ACTG"
                    .chars()
                    .choose(rng)
                    .expect("Expected to choose a random character"),
            })
            .collect();
        converted
    }

    pub fn read(&mut self, record: &mut Record) -> eyre::Result<()> {
        record.clear();
        let mut rng: ThreadRng = thread_rng();
        if self.buffer.trim_end().is_empty() {
            self.reader.read_line(&mut self.buffer)?;
            if self.buffer.trim_end().is_empty() {
                return Ok(());
            }
        }

        if !self.buffer.starts_with(START_CHARACTER) {
            return Err(eyre!("invalid start character in line: {}", &self.buffer));
        }

        record.set_header(self.buffer[1..].trim_end().to_owned());
        loop {
            self.buffer.clear();
            self.reader.read_line(&mut self.buffer)?;
            let next_part = self.buffer.trim_end();
            if next_part.is_empty() || next_part.starts_with(START_CHARACTER) {
                break;
            }
            record.push_sequence_part(&self.sanitize_line(next_part, &mut rng));
        }

        Ok(())
    }
}

impl Iterator for Reader {
    type Item = eyre::Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut record = Record::new();
        match self.read(&mut record) {
            Ok(()) => {
                if record.is_empty() {
                    None
                } else {
                    Some(Ok(record))
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}
