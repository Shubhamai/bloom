use std::{
    fs::File,
    io::{self, Write},
    os::unix::fs::FileExt,
    path::Path,
};

pub struct BitVec {
    file: File,
    table_size: usize,
}

impl BitVec {
    pub fn new(file_path: String, table_size: usize) -> io::Result<Self> {
        if !Path::new(&file_path).exists() {
            println!("Creating new file : {}", file_path);
            let mut file = File::create(&file_path)?;

            // Save in chunks incase the entire vector can't fit in memory
            let chunk_size = table_size / 20;
            let zeros = vec![0u8; chunk_size];
            for _ in 0..(table_size / chunk_size) {
                file.write_all(&zeros)?;
            }

            file.flush()?;
        } else {
            println!("Using existing file : {file_path}")
        }

        let file = File::options().read(true).write(true).open(file_path)?;

        Ok(BitVec { file, table_size })
    }

    // Since in linux, only an individual byte can be updated in files
    // We first calculate the position of the u8 and then
    // do some bit manipulation to change individual bit in u8
    // We read and write only individual byte using read/write_at
    pub fn set(&mut self, index: usize) -> io::Result<()> {
        let mut buffer = [0u8; 1];
        let position = index / 8;

        self.file.read_at(&mut buffer, position as u64)?;

        let byte = buffer[0] | 1 << (index % 8);
        self.file.write_at(&[byte], position as u64)?;
        self.file.flush()
    }

    // Similar to `set`, we calculate the position of the needed u8 in file
    // then only read that u8
    pub fn get(&mut self, index: usize) -> io::Result<bool> {
        if index >= self.table_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Row index out of bounds",
            ));
        }
        let mut buffer = [0u8; 1];
        let position = index / 8;

        self.file.read_at(&mut buffer, position as u64)?;
        Ok(!(((1 << (index % 8)) & buffer[0]) == 0))
    }

    pub fn len(&self) -> usize {
        self.table_size
    }
}
