//! Chunk postings that archive to the same bytes on every run.

use rkyv::collections::swiss_table::HashMapResolver;
use rkyv::rancor::{Fallible, Source};
use rkyv::ser::{Allocator, Writer};
use rkyv::{Archive, Archived, Place, Serialize};

use crate::types::{ChunkPostings, Posting};

/// Archived form of a chunk. Readers access chunk files as this type.
type ArchivedChunk = Archived<ChunkPostings>;

/// Load factor of rkyv's archived `HashMap`. It matches rkyv's own `HashMap` impl.
const LOAD_FACTOR: (usize, usize) = (7, 8);

/// Term-to-postings entries that archive as [`ChunkPostings`] in sorted term order.
///
/// rkyv hashes keys with a fixed hasher. It places colliding keys in the order it
/// receives them. A std `HashMap` iterates in a random order per instance, so its
/// archive bytes differ between runs. This type feeds entries in term order, so the
/// bytes repeat. The archived type equals `Archived<ChunkPostings>`, so a reader of
/// `ChunkPostings` reads the output unchanged.
pub struct SortedChunk<'a> {
    entries: Vec<(&'a String, &'a Vec<Posting>)>,
}

impl<'a> SortedChunk<'a> {
    /// Collects the entries and sorts them by term.
    pub fn new(entries: impl IntoIterator<Item = (&'a String, &'a Vec<Posting>)>) -> Self {
        let mut entries: Vec<(&'a String, &'a Vec<Posting>)> = entries.into_iter().collect();
        entries.sort_unstable_by_key(|&(term, _)| term);
        Self { entries }
    }

    /// Returns the terms in archive order.
    pub fn terms(&self) -> impl Iterator<Item = &'a String> + '_ {
        self.entries.iter().map(|(term, _)| *term)
    }
}

impl Archive for SortedChunk<'_> {
    type Archived = ArchivedChunk;
    type Resolver = HashMapResolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        ArchivedChunk::resolve_from_len(self.entries.len(), LOAD_FACTOR, resolver, out);
    }
}

impl<S> Serialize<S> for SortedChunk<'_>
where
    S: Fallible + Writer + Allocator + ?Sized,
    S::Error: Source,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        ArchivedChunk::serialize_from_iter::<_, _, _, String, Vec<Posting>, _>(
            self.entries.iter().copied(),
            LOAD_FACTOR,
            serializer,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn posting(doc_id: u32) -> Vec<Posting> {
        vec![Posting {
            doc_id,
            score: 1.0,
            positions: vec![doc_id],
        }]
    }

    #[test]
    fn sorted_chunk_reads_back_as_chunk_postings() {
        let mut map: ChunkPostings = HashMap::new();
        for (i, term) in ["beta", "alpha", "gamma", "delta"].iter().enumerate() {
            map.insert((*term).to_string(), posting(i as u32));
        }

        let chunk = SortedChunk::new(&map);
        let terms: Vec<&String> = chunk.terms().collect();
        assert_eq!(terms, vec!["alpha", "beta", "delta", "gamma"]);

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&chunk).unwrap();
        let back: ChunkPostings = rkyv::from_bytes::<_, rkyv::rancor::Error>(&bytes).unwrap();
        assert_eq!(back.len(), 4);
        assert_eq!(back["gamma"][0].doc_id, 2);
        assert_eq!(back["alpha"][0].positions, vec![1]);
    }

    #[test]
    fn sorted_chunk_bytes_ignore_insertion_order() {
        let terms: Vec<String> = (0..200).map(|i| format!("term{i:03}")).collect();

        let mut forward: ChunkPostings = HashMap::new();
        for (i, term) in terms.iter().enumerate() {
            forward.insert(term.clone(), posting(i as u32));
        }
        let mut backward: ChunkPostings = HashMap::new();
        for (i, term) in terms.iter().enumerate().rev() {
            backward.insert(term.clone(), posting(i as u32));
        }

        let a = rkyv::to_bytes::<rkyv::rancor::Error>(&SortedChunk::new(&forward)).unwrap();
        let b = rkyv::to_bytes::<rkyv::rancor::Error>(&SortedChunk::new(&backward)).unwrap();
        assert_eq!(a.as_slice(), b.as_slice());
    }
}
