use alloc::vec::Vec;

use anyhow::Result;

use crate::{Error, OwnedBuf};

use super::{Builder, store};

#[test]
fn regular_trie() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let mut trie = Builder::new();

    let key = buf.store_unsized("work")?;
    trie.insert(&buf, key, 1)?;
    let key = buf.store_unsized("worker")?;
    trie.insert(&buf, key, 2)?;
    let key = buf.store_unsized("workers")?;
    trie.insert(&buf, key, 3)?;
    let key = buf.store_unsized("working")?;
    trie.insert(&buf, key, 4)?;
    let key = buf.store_unsized("working")?;
    trie.insert(&buf, key, 5)?;
    let key = buf.store_unsized("working man")?;
    trie.insert(&buf, key, 6)?;

    let trie = trie.build(&mut buf)?;

    assert_eq!(trie.get(&buf, "aard")?, None);
    assert_eq!(trie.get(&buf, "worker")?, Some(&[2][..]));
    assert_eq!(trie.get(&buf, "working")?, Some(&[4, 5][..]));
    Ok(())
}

#[test]
fn disorederly_trie() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let mut trie = Builder::new();

    let key = buf.store_unsized("working")?;
    trie.insert(&buf, key, 4)?;
    let key = buf.store_unsized("working man")?;
    trie.insert(&buf, key, 6)?;
    let key = buf.store_unsized("work")?;
    trie.insert(&buf, key, 1)?;
    let key = buf.store_unsized("worker")?;
    trie.insert(&buf, key, 2)?;
    let key = buf.store_unsized("workers")?;
    trie.insert(&buf, key, 3)?;
    let key = buf.store_unsized("working")?;
    trie.insert(&buf, key, 5)?;

    let trie = trie.build(&mut buf)?;

    assert_eq!(trie.get(&buf, "aard")?, None);
    assert_eq!(trie.get(&buf, "worker")?, Some(&[2][..]));
    assert_eq!(trie.get(&buf, "working")?, Some(&[4, 5][..]));
    Ok(())
}

#[test]
fn trie_problem() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();
    let a = buf.store_unsized("食べなかった")?;
    let b = buf.store_unsized("食べない")?;

    let mut trie = Builder::new();

    trie.insert(&buf, b, 2)?;
    trie.insert(&buf, a, 1)?;

    let trie = trie.build(&mut buf)?;

    assert_eq!(trie.get(&buf, "食べなかった")?, Some(&[1][..]));
    assert_eq!(trie.get(&buf, "食べない")?, Some(&[2][..]));
    Ok(())
}

#[test]
fn trie_prefix() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let values = [
        (buf.store_unsized("work")?, 1),
        (buf.store_unsized("worker")?, 2),
        (buf.store_unsized("workers")?, 3),
        (buf.store_unsized("working")?, 4),
        (buf.store_unsized("working")?, 5),
        (buf.store_unsized("working man")?, 6),
        (buf.store_unsized("run")?, 7),
        (buf.store_unsized("running")?, 8),
    ];

    let trie = store(&mut buf, values)?;

    let values = trie
        .values_in(&buf, "workin")
        .collect::<Result<Vec<_>, _>>()?;
    assert!(values.into_iter().copied().eq([4, 5, 6]));

    let values = trie.values_in(&buf, "wor").collect::<Result<Vec<_>, _>>()?;
    assert!(values.into_iter().copied().eq([1, 2, 3, 4, 5, 6]));

    let values = trie
        .values_in(&buf, "runn")
        .collect::<Result<Vec<_>, _>>()?;
    assert!(values.into_iter().copied().eq([8]));

    let values = trie
        .values_in(&buf, "asdf")
        .collect::<Result<Vec<_>, _>>()?;
    assert!(values.is_empty());
    Ok(())
}

#[test]
fn entries() -> Result<()> {
    use std::str::from_utf8;

    // Helper to convert output to utf-8.
    fn to_utf8<'buf, E>(result: Result<(&'buf [u8], &'buf i32), E>) -> Result<(&'buf str, i32)>
    where
        anyhow::Error: From<E>,
    {
        let (k, v) = result?;
        Ok((from_utf8(k)?, *v))
    }

    let mut buf = OwnedBuf::new();

    let values = [
        (buf.store_unsized("work")?, 1),
        (buf.store_unsized("worker")?, 2),
        (buf.store_unsized("workers")?, 3),
        (buf.store_unsized("working")?, 4),
        (buf.store_unsized("working")?, 5),
        (buf.store_unsized("working man")?, 6),
        (buf.store_unsized("run")?, 7),
        (buf.store_unsized("running")?, 8),
    ];

    let trie = store(&mut buf, values)?;

    let values = trie
        .iter_in(&buf, "workin")
        .map(to_utf8)
        .collect::<Result<Vec<_>>>()?;
    assert_eq!(values, [("working", 4), ("working", 5), ("working man", 6)]);

    let values = trie
        .iter_in(&buf, "wor")
        .map(to_utf8)
        .collect::<Result<Vec<_>>>()?;

    assert_eq!(
        values,
        [
            ("work", 1),
            ("worker", 2),
            ("workers", 3),
            ("working", 4),
            ("working", 5),
            ("working man", 6),
        ]
    );

    let values = trie
        .iter_in(&buf, "runn")
        .map(to_utf8)
        .collect::<Result<Vec<_>>>()?;

    assert_eq!(values, [("running", 8),]);
    Ok(())
}
