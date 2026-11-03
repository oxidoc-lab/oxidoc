//! Distinct lowercased words of the corpus, each with its stem, addressed by id.

use std::collections::HashMap;

/// One vocabulary word as byte offsets into [`Vocab::buf`].
///
/// `buf[start..split]` is the word. `buf[split..end]` is its stem. `end == start`
/// marks a stem equal to the word: a word is never empty, so a stored stem always
/// has `end >= split > start`.
struct WordSpan {
    start: usize,
    split: usize,
    end: usize,
}

/// Distinct words with their Snowball stems. A word id indexes this list.
#[derive(Default)]
pub(super) struct Vocab {
    /// Each word, followed by its stem when the stem differs.
    buf: String,
    /// Offsets of each word and stem in `buf`, by word id.
    spans: Vec<WordSpan>,
}

impl Vocab {
    /// Returns the number of words.
    pub(super) fn len(&self) -> usize {
        self.spans.len()
    }

    /// Returns word `id` and its stem. Returns empty strings for an unknown id.
    pub(super) fn get(&self, id: usize) -> (&str, &str) {
        let Some(s) = self.spans.get(id) else {
            return ("", "");
        };
        let word = self.buf.get(s.start..s.split).unwrap_or("");
        let stem = if s.end == s.start {
            word
        } else {
            self.buf.get(s.split..s.end).unwrap_or("")
        };
        (word, stem)
    }
}

/// Assigns word ids while the corpus is read. Each distinct word is stemmed once.
#[derive(Default)]
pub(super) struct VocabBuilder {
    vocab: Vocab,
    /// Id of each word seen so far.
    ids: HashMap<String, usize>,
    /// Per word id: the number of the last list that took the word, plus one.
    last_list: Vec<usize>,
    /// Number of lists built so far.
    lists: usize,
}

impl VocabBuilder {
    /// Returns the ids of the distinct `words`, in first-seen order.
    ///
    /// Scoring takes the best score over a list, so dropping repeats keeps results.
    pub(super) fn distinct_ids<'a>(
        &mut self,
        words: impl IntoIterator<Item = &'a str>,
    ) -> Vec<usize> {
        self.lists += 1;
        let mut out = Vec::new();
        for word in words {
            let id = self.intern(word);
            if let Some(last) = self.last_list.get_mut(id)
                && *last != self.lists
            {
                *last = self.lists;
                out.push(id);
            }
        }
        out.shrink_to_fit();
        out
    }

    /// Returns the id of `word`, adding it and its stem when new.
    fn intern(&mut self, word: &str) -> usize {
        if let Some(&id) = self.ids.get(word) {
            return id;
        }
        let id = self.vocab.spans.len();
        let buf = &mut self.vocab.buf;
        let start = buf.len();
        buf.push_str(word);
        let split = buf.len();
        let stem = oxidoc_text::stem(word);
        let end = if stem == word {
            start
        } else {
            buf.push_str(&stem);
            buf.len()
        };
        self.vocab.spans.push(WordSpan { start, split, end });
        self.last_list.push(0);
        self.ids.insert(word.to_string(), id);
        id
    }

    /// Returns the finished vocabulary.
    pub(super) fn finish(mut self) -> Vocab {
        self.vocab.buf.shrink_to_fit();
        self.vocab.spans.shrink_to_fit();
        self.vocab
    }
}

/// Splits lowercased text into words: runs of alphanumeric chars.
pub(super) fn split_words(lower: &str) -> impl Iterator<Item = &str> {
    lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct_ids_share_words_across_lists() {
        let mut builder = VocabBuilder::default();
        let first = builder.distinct_ids(split_words("running, jumps; running blocks"));
        let second = builder.distinct_ids(split_words("blocks -- running"));
        let empty = builder.distinct_ids(split_words(" -- "));
        let vocab = builder.finish();
        let words =
            |ids: &[usize]| -> Vec<(&str, &str)> { ids.iter().map(|&id| vocab.get(id)).collect() };
        assert_eq!(
            words(&first),
            [("running", "run"), ("jumps", "jump"), ("blocks", "block")]
        );
        assert_eq!(words(&second), [("blocks", "block"), ("running", "run")]);
        assert!(empty.is_empty());
        assert_eq!(vocab.len(), 3);
        assert_eq!(vocab.get(3), ("", ""));
    }

    #[test]
    fn stem_equal_to_word_is_not_stored_twice() {
        let mut builder = VocabBuilder::default();
        let ids = builder.distinct_ids(["tensor", "runs"]);
        let vocab = builder.finish();
        assert_eq!(vocab.get(ids[0]), ("tensor", "tensor"));
        assert_eq!(vocab.get(ids[1]), ("runs", "run"));
        assert_eq!(vocab.buf, "tensorrunsrun");
    }
}
