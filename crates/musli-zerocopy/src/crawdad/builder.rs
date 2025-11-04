use crate::error::{Error, ErrorKind};

use super::mapper::CodeMapper;
use super::{END_CODE, END_MARKER, INVALID_IDX, MAX_VALUE, OFFSET_MASK};
use super::{MpTrie, Node, Trie, utils};

use core::cmp::Ordering;

use alloc::vec::Vec;

// The default parameter for free blocks to be searched in `find_base`.
const DEFAULT_NUM_FREE_BLOCKS: u32 = 16;

#[derive(Default)]
struct Record {
    key: Vec<char>,
    value: u32,
}

#[derive(Default, Debug, PartialEq, Eq)]
struct Suffix {
    key: Vec<char>,
    value: u32,
}

pub(super) struct Builder {
    records: Vec<Record>,
    mapper: CodeMapper,
    nodes: Vec<Node>,
    suffixes: Option<Vec<Suffix>>,
    labels: Vec<u32>,
    head_idx: u32,
    block_len: u32,
    num_free_blocks: u32,
}

impl Builder {
    pub(super) fn new() -> Self {
        Self {
            records: Vec::new(),
            mapper: CodeMapper::default(),
            nodes: Vec::new(),
            suffixes: None,
            labels: Vec::new(),
            head_idx: 0,
            block_len: 0,
            num_free_blocks: DEFAULT_NUM_FREE_BLOCKS,
        }
    }

    pub(super) fn minimal_prefix(mut self) -> Self {
        self.suffixes = Some(Vec::new());
        self
    }

    pub(super) fn build_from_keys<I, K>(self, keys: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        self.build_from_records(
            keys.into_iter()
                .enumerate()
                .map(|(i, k)| (k, i.try_into().unwrap())),
        )
    }

    pub(super) fn build_from_records<I, K>(mut self, records: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = (K, u32)>,
        K: AsRef<str>,
    {
        self.records = records
            .into_iter()
            .map(|(k, v)| Record {
                key: k.as_ref().chars().collect(),
                value: v,
            })
            .collect();

        self.records.sort_unstable_by(|a, b| a.key.cmp(&b.key));

        for &Record { key: _, value } in &self.records {
            if MAX_VALUE < value {
                return Err(Error::scale("input value", MAX_VALUE));
            }
        }

        self.mapper = CodeMapper::new(&make_freqs(&self.records)?);
        assert_eq!(self.mapper.get(END_MARKER).unwrap(), END_CODE);

        make_prefix_free(&mut self.records)?;

        self.block_len = self.mapper.alphabet_size().next_power_of_two().max(2);
        self.init_array();
        self.arrange_nodes(0, self.records.len(), 0, 0)?;
        self.finish();

        Ok(self)
    }

    pub(super) fn release_trie(self) -> Result<Trie, Error> {
        if self.suffixes.is_some() {
            return Err(Error::new(ErrorKind::MinimalPrefixDisable));
        }

        let Self { nodes, mapper, .. } = self;
        Ok(Trie { nodes, mapper })
    }

    pub(super) fn release_mptrie(self) -> Result<MpTrie, Error> {
        let Self {
            mapper,
            mut nodes,
            suffixes,
            ..
        } = self;

        let Some(suffixes) = suffixes else {
            return Err(Error::new(ErrorKind::MinimalPrefixEnable));
        };

        let mut tails = Vec::new();

        let max_code = mapper.alphabet_size() - 1;
        let code_size = utils::pack_size(max_code);

        let max_value = suffixes.iter().map(|s| s.value).max().unwrap();
        let value_size = utils::pack_size(max_value);

        for node_idx in 0..nodes.len() {
            if nodes[node_idx].is_vacant() {
                continue;
            }
            if !nodes[node_idx].is_leaf() {
                continue;
            }

            debug_assert_eq!(nodes[node_idx].check & !OFFSET_MASK, 0);
            let parent_idx = usize::try_from(nodes[node_idx].check).unwrap();
            let suf_idx = usize::try_from(nodes[node_idx].base & OFFSET_MASK).unwrap();
            let suffix = &suffixes[suf_idx];

            // HasLeaf?
            if nodes[parent_idx].has_leaf() {
                // `node_idx` is indicated from `parent_idx` with END_CODE?
                if usize::try_from(nodes[parent_idx].base).unwrap() == node_idx {
                    assert!(suffix.key.is_empty());
                    nodes[node_idx].base = suffix.value | !OFFSET_MASK;
                    continue;
                }
            }

            let tail_start = if tails.len() <= usize::try_from(OFFSET_MASK).unwrap() {
                u32::try_from(tails.len()).unwrap()
            } else {
                return Err(Error::scale("length of tails", OFFSET_MASK));
            };

            if suffix.key.len() > usize::from(u8::MAX) {
                return Err(Error::scale("length of suffix", u32::from(u8::MAX)));
            }

            nodes[node_idx].base = tail_start | !OFFSET_MASK;
            tails.push(suffix.key.len().try_into().unwrap());
            suffix
                .key
                .iter()
                .map(|&c| mapper.get(c).unwrap())
                .for_each(|c| utils::pack_u32(&mut tails, c, code_size));
            utils::pack_u32(&mut tails, suffix.value, value_size);
        }

        Ok(MpTrie {
            mapper,
            nodes,
            tails,
            code_size,
            value_size,
        })
    }

    #[inline(always)]
    fn num_nodes(&self) -> u32 {
        self.nodes.len().try_into().unwrap()
    }

    fn init_array(&mut self) {
        self.nodes.clear();
        self.nodes
            .resize(usize::try_from(self.block_len).unwrap(), Node::default());

        for i in 0..self.block_len {
            if i == 0 {
                self.set_prev(i, self.block_len - 1);
            } else {
                self.set_prev(i, i - 1);
            }
            if i == self.block_len - 1 {
                self.set_next(i, 0);
            } else {
                self.set_next(i, i + 1);
            }
        }

        self.head_idx = 0;
        self.fix_node(0);
    }

    fn arrange_nodes(
        &mut self,
        spos: usize,
        epos: usize,
        depth: usize,
        node_idx: u32,
    ) -> Result<(), Error> {
        debug_assert!(self.is_fixed(node_idx));

        if let Some(suffixes) = self.suffixes.as_mut() {
            if spos + 1 == epos {
                // It has been checked in build_from_records().
                debug_assert_eq!(self.records[spos].value & !OFFSET_MASK, 0);

                let suffix_idx = if suffixes.len() <= usize::try_from(OFFSET_MASK).unwrap() {
                    u32::try_from(suffixes.len()).unwrap()
                } else {
                    return Err(Error::scale("length of suffixes", OFFSET_MASK));
                };

                self.nodes[usize::try_from(node_idx).unwrap()].base = suffix_idx | !OFFSET_MASK;

                suffixes.push(Suffix {
                    key: pop_end_marker(&self.records[spos].key[depth..]),
                    value: self.records[spos].value,
                });

                return Ok(());
            }
        } else if self.records[spos].key.len() == depth {
            debug_assert_eq!(spos + 1, epos);
            // It has been checked in build_from_records().
            debug_assert_eq!(self.records[spos].value & !OFFSET_MASK, 0);
            // Sets IsLeaf = True
            self.node_mut(node_idx).base = self.records[spos].value | !OFFSET_MASK;
            // Note: HasLeaf must not be set here and should be set in finish()
            // because MSB of check is used to indicate vacant element.
            return Ok(());
        }

        self.fetch_labels(spos, epos, depth);
        let base = self.define_nodes(node_idx)?;

        let mut i1 = spos;
        let mut c1 = self.records[i1].key[depth];

        for i2 in spos + 1..epos {
            let c2 = self.records[i2].key[depth];
            if c1 != c2 {
                let child_idx = base ^ self.mapper.get(c1).unwrap();
                self.arrange_nodes(i1, i2, depth + 1, child_idx)?;
                i1 = i2;
                c1 = c2;
            }
        }

        let child_idx = base ^ self.mapper.get(c1).unwrap();
        self.arrange_nodes(i1, epos, depth + 1, child_idx)
    }

    fn finish(&mut self) {
        self.node_mut(0).check = OFFSET_MASK;
        if self.head_idx != INVALID_IDX {
            let mut node_idx = self.head_idx;
            loop {
                let next_idx = self.get_next(node_idx);
                self.node_mut(node_idx).base = OFFSET_MASK;
                self.node_mut(node_idx).check = OFFSET_MASK;
                node_idx = next_idx;
                if node_idx == self.head_idx {
                    break;
                }
            }
        }
        for node_idx in 0..self.num_nodes() {
            if self.node_ref(node_idx).is_vacant() {
                continue;
            }
            if self.node_ref(node_idx).is_leaf() {
                continue;
            }
            let end_idx = self.node_ref(node_idx).base ^ END_CODE;
            if self.node_ref(end_idx).check == node_idx {
                // Sets HasLeaf = True
                self.node_mut(node_idx).check |= !OFFSET_MASK;
            }
        }
    }

    fn fetch_labels(&mut self, spos: usize, epos: usize, depth: usize) {
        self.labels.clear();
        let mut c1 = self.records[spos].key[depth];
        for i in spos + 1..epos {
            let c2 = self.records[i].key[depth];
            if c1 != c2 {
                self.labels.push(self.mapper.get(c1).unwrap());
                c1 = c2;
            }
        }
        self.labels.push(self.mapper.get(c1).unwrap());
    }

    fn define_nodes(&mut self, node_idx: u32) -> Result<u32, Error> {
        let base = self.find_base(&self.labels);
        if base >= self.num_nodes() {
            self.enlarge()?;
        }

        self.node_mut(node_idx).base = base;
        for i in 0..self.labels.len() {
            let child_idx = base ^ self.labels[i];
            self.fix_node(child_idx);
            self.node_mut(child_idx).check = node_idx;
        }
        Ok(base)
    }

    fn find_base(&self, labels: &[u32]) -> u32 {
        debug_assert!(!labels.is_empty());

        if self.head_idx == INVALID_IDX {
            return self.num_nodes() ^ labels[0];
        }

        let mut node_idx = self.head_idx;
        loop {
            let base = node_idx ^ labels[0];
            if self.verify_base(base, labels) {
                return base;
            }
            node_idx = self.get_next(node_idx);
            if node_idx == self.head_idx {
                break;
            }
        }
        self.num_nodes() ^ labels[0]
    }

    #[inline(always)]
    fn verify_base(&self, base: u32, labels: &[u32]) -> bool {
        for &label in labels {
            let node_idx = base ^ label;
            if self.is_fixed(node_idx) {
                return false;
            }
        }
        true
    }

    #[inline(always)]
    fn fix_node(&mut self, node_idx: u32) {
        debug_assert!(!self.is_fixed(node_idx));

        let next = self.get_next(node_idx);
        let prev = self.get_prev(node_idx);

        self.set_next(prev, next);
        self.set_prev(next, prev);
        self.set_fixed(node_idx);

        if self.head_idx == node_idx {
            if next == node_idx {
                self.head_idx = INVALID_IDX;
            } else {
                self.head_idx = next;
            }
        }
    }

    fn enlarge(&mut self) -> Result<(), Error> {
        let old_len = self.num_nodes();
        let new_len = old_len + self.block_len;

        if OFFSET_MASK < new_len {
            return Err(Error::scale("num_nodes", OFFSET_MASK));
        }

        let num_blocks = old_len / self.block_len;
        if self.num_free_blocks <= num_blocks {
            self.close_block(num_blocks - self.num_free_blocks);
        }

        for i in old_len..new_len {
            self.nodes.push(Node::default());
            self.set_next(i, i + 1);
            self.set_prev(i, i - 1);
        }

        if self.head_idx == INVALID_IDX {
            self.set_prev(old_len, new_len - 1);
            self.set_next(new_len - 1, old_len);
            self.head_idx = old_len;
        } else {
            let head_idx = self.head_idx;
            let tail_idx = self.get_prev(head_idx);
            self.set_prev(old_len, tail_idx);
            self.set_next(tail_idx, old_len);
            self.set_next(new_len - 1, head_idx);
            self.set_prev(head_idx, new_len - 1);
        }

        Ok(())
    }

    /// Note: Assumes all the previous blocks are closed.
    fn close_block(&mut self, block_idx: u32) {
        let beg_idx = block_idx * self.block_len;
        let end_idx = beg_idx + self.block_len;
        while self.head_idx < end_idx {
            // Here, self.head_idx != INVALID_IDX is ensured,
            // because INVALID_IDX is the maximum value in u32.
            debug_assert_ne!(self.head_idx, INVALID_IDX);
            let idx = self.head_idx;
            self.fix_node(idx);
            self.node_mut(idx).base = OFFSET_MASK;
            self.node_mut(idx).check = OFFSET_MASK;
        }
    }

    #[inline(always)]
    fn node_ref(&self, i: u32) -> &Node {
        &self.nodes[usize::try_from(i).unwrap()]
    }

    #[inline(always)]
    fn node_mut(&mut self, i: u32) -> &mut Node {
        &mut self.nodes[usize::try_from(i).unwrap()]
    }

    // If the most significant bit is unset, the state is fixed.
    #[inline(always)]
    fn is_fixed(&self, i: u32) -> bool {
        self.node_ref(i).check & !OFFSET_MASK == 0
    }

    // Unset the most significant bit.
    #[inline(always)]
    fn set_fixed(&mut self, i: u32) {
        debug_assert!(!self.is_fixed(i));
        self.node_mut(i).base = INVALID_IDX;
        self.node_mut(i).check &= OFFSET_MASK;
    }

    #[inline(always)]
    fn get_next(&self, i: u32) -> u32 {
        debug_assert_ne!(self.node_ref(i).base & !OFFSET_MASK, 0);
        self.node_ref(i).base & OFFSET_MASK
    }

    #[inline(always)]
    fn get_prev(&self, i: u32) -> u32 {
        debug_assert_ne!(self.node_ref(i).check & !OFFSET_MASK, 0);
        self.node_ref(i).check & OFFSET_MASK
    }

    #[inline(always)]
    fn set_next(&mut self, i: u32, x: u32) {
        debug_assert_eq!(x & !OFFSET_MASK, 0);
        self.node_mut(i).base = x | !OFFSET_MASK
    }

    #[inline(always)]
    fn set_prev(&mut self, i: u32, x: u32) {
        debug_assert_eq!(x & !OFFSET_MASK, 0);
        self.node_mut(i).check = x | !OFFSET_MASK
    }
}

fn make_freqs(records: &[Record]) -> Result<Vec<u32>, Error> {
    let end_marker = usize::try_from(u32::from(END_MARKER)).unwrap();
    let mut freqs = Vec::new();
    freqs.resize(end_marker + 1, 0);

    for rec in records {
        for &c in &rec.key {
            let c = usize::try_from(u32::from(c)).unwrap();

            if freqs.len() <= c {
                freqs.resize(c + 1, 0);
            }

            freqs[c] += 1;
        }
    }

    if let Some(&freq) = freqs.get(end_marker) {
        if freq != 0 {
            return Err(Error::new(ErrorKind::EndMarkerInKeys));
        }
    }

    freqs[end_marker] = u32::MAX;
    Ok(freqs)
}

fn make_prefix_free(records: &mut [Record]) -> Result<(), Error> {
    if records.is_empty() {
        return Err(Error::new(ErrorKind::EmptyRecords));
    }

    if records[0].key.is_empty() {
        return Err(Error::new(ErrorKind::EmptyKey));
    }

    for i in 1..records.len() {
        let (lcp, cmp) = utils::longest_common_prefix(&records[i - 1].key, &records[i].key);
        match cmp {
            Ordering::Less => {
                // Startswith?
                if lcp == records[i - 1].key.len() {
                    records[i - 1].key.push(END_MARKER);
                }
            }
            Ordering::Equal => {
                return Err(Error::new(ErrorKind::DuplicateKeys));
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn pop_end_marker(x: &[char]) -> Vec<char> {
    match x.split_last() {
        Some((&END_MARKER, elems)) => elems.to_vec(),
        _ => x.to_vec(),
    }
}
