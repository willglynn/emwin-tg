use crate::{Error, Product};
use bytes::Bytes;
use std::collections::{btree_map::Entry, BTreeMap};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct StreamState {
    last_seen_at: BTreeMap<String, Instant>,
}

impl StreamState {
    pub fn new_products_in(&mut self, bytes: Bytes) -> Result<Vec<Result<Product, Error>>, Error> {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;

        let mut names: Vec<_> = archive.file_names().map(String::from).collect();
        names.sort();

        let names = self.add_filenames_in(names);

        log::info!("{} of {} products are new", names.len(), archive.len());

        Ok(names
            .into_iter()
            .map(|name| Product::new(archive.by_name(&name)))
            .collect())
    }

    fn add_filenames_in(&mut self, names: Vec<String>) -> Vec<String> {
        let mut out = Vec::new();
        let now = Instant::now();

        // Loop over all the filenames, one at a time, in order
        for name in names {
            // Check if we've seen this before
            match self.last_seen_at.entry(name) {
                Entry::Occupied(mut e) => {
                    // We have seen this before
                    // Update the timestamp
                    e.insert(now);

                    // Skip this file
                    continue;
                }
                Entry::Vacant(e) => {
                    // We have not seen this before
                    // Add to the list
                    log::trace!("new file: {}", e.key());
                    out.push(e.key().clone());

                    // Insert this record
                    e.insert(now);
                }
            };
        }

        // Cull everything we haven't seen in a while
        let before_count = self.last_seen_at.len();
        self.last_seen_at
            .retain(|_, value| now.duration_since(*value) < Duration::from_secs(6 * 3600));
        let after_count = self.last_seen_at.len();

        log::trace!(
            "last_seen_at culled from {} -> {} entries",
            before_count,
            after_count
        );

        out
    }
}

impl StreamState {
    pub fn new() -> Self {
        Self {
            last_seen_at: BTreeMap::new(),
        }
    }
}
