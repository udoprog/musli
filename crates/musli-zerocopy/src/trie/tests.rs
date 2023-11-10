use alloc::vec::Vec;

use crate::{Error, OwnedBuf};

use super::{store, Builder};

#[test]
fn regular_trie() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let mut trie = Builder::new();

    let key = buf.store_unsized("work");
    trie.insert(&buf, key, 1)?;
    let key = buf.store_unsized("worker");
    trie.insert(&buf, key, 2)?;
    let key = buf.store_unsized("workers");
    trie.insert(&buf, key, 3)?;
    let key = buf.store_unsized("working");
    trie.insert(&buf, key, 4)?;
    let key = buf.store_unsized("working");
    trie.insert(&buf, key, 5)?;
    let key = buf.store_unsized("working man");
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

    let key = buf.store_unsized("working");
    trie.insert(&buf, key, 4)?;
    let key = buf.store_unsized("working man");
    trie.insert(&buf, key, 6)?;
    let key = buf.store_unsized("work");
    trie.insert(&buf, key, 1)?;
    let key = buf.store_unsized("worker");
    trie.insert(&buf, key, 2)?;
    let key = buf.store_unsized("workers");
    trie.insert(&buf, key, 3)?;
    let key = buf.store_unsized("working");
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
    let a = buf.store_unsized("食べなかった");
    let b = buf.store_unsized("食べない");

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
        (buf.store_unsized("work"), 1),
        (buf.store_unsized("worker"), 2),
        (buf.store_unsized("workers"), 3),
        (buf.store_unsized("working"), 4),
        (buf.store_unsized("working"), 5),
        (buf.store_unsized("working man"), 6),
        (buf.store_unsized("run"), 7),
        (buf.store_unsized("running"), 8),
    ];

    let trie = store(&mut buf, values)?;

    let mut values = Vec::new();

    trie.prefix(&buf, "workin", |n| {
        values.push(*n);
    })?;

    assert_eq!(&values[..], &[4, 5, 6][..]);

    let mut values = Vec::new();

    trie.prefix(&buf, "wor", |n| {
        values.push(*n);
    })?;

    assert_eq!(&values[..], &[1, 2, 3, 4, 5, 6][..]);
    Ok(())
}
