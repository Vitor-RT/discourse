use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use tar::{Builder, Header};

const MAX_LINKS: u32 = 1000;

pub struct ArchiveWriter<W: Write> {
    builder: Builder<W>,
    hardlinks: HashMap<(u64, u64), (String, u32)>,
}

impl<W: Write> ArchiveWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            builder: Builder::new(writer),
            hardlinks: HashMap::new(),
        }
    }

    pub fn add_file<P: AsRef<Path>>(&mut self, source: P, name: &str) -> io::Result<()> {
        let mut file = File::open(source)?;
        let metadata = file.metadata()?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if metadata.nlink() > 1 {
                let inode = (metadata.dev(), metadata.ino());

                let mut should_add_hardlink = None;

                if let Some((target, count)) = self.hardlinks.get_mut(&inode) {
                    if *count < MAX_LINKS {
                        should_add_hardlink = Some(target.clone());
                        *count += 1;
                    }
                } else {
                    self.hardlinks.insert(inode, (name.to_string(), 1));
                }

                if let Some(target) = should_add_hardlink {
                    self.add_hardlink(name, &target)?;
                    return Ok(());
                }
            }
        }

        self.builder.append_file(name, &mut file)?;
        Ok(())
    }

    pub fn add_directory(&mut self, name: &str) -> io::Result<()> {
        self.builder.append_dir(name, ".")?;
        Ok(())
    }

    pub fn add_hardlink(&mut self, name: &str, target: &str) -> io::Result<()> {
        let mut header = Header::new_gnu();
        header.set_entry_type(tar::EntryType::Link);
        header.set_size(0);
        self.builder.append_link(&mut header, name, target)?;
        Ok(())
    }

    pub fn finish(mut self) -> io::Result<()> {
        self.builder.finish()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_archive_writer_file() {
        let mut archive_data = Vec::new();
        let mut writer = ArchiveWriter::new(&mut archive_data);

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello world").unwrap();

        writer.add_file(temp_file.path(), "hello.txt").unwrap();
        writer.finish().unwrap();

        // Read back
        let mut archive = tar::Archive::new(&archive_data[..]);
        let mut entries = archive.entries().unwrap();
        let mut entry = entries.next().unwrap().unwrap();

        assert_eq!(entry.path().unwrap().to_str().unwrap(), "hello.txt");
        let mut content = String::new();
        entry.read_to_string(&mut content).unwrap();
        assert_eq!(content, "Hello world");
    }

    #[test]
    fn test_archive_writer_directory() {
        let mut archive_data = Vec::new();
        let mut writer = ArchiveWriter::new(&mut archive_data);

        writer.add_directory("test_dir/").unwrap();
        writer.finish().unwrap();

        let mut archive = tar::Archive::new(&archive_data[..]);
        let mut entries = archive.entries().unwrap();
        let entry = entries.next().unwrap().unwrap();

        assert_eq!(entry.path().unwrap().to_str().unwrap(), "test_dir/");
        assert_eq!(entry.header().entry_type(), tar::EntryType::Directory);
    }
}
