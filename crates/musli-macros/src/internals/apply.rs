use proc_macro2::TokenStream;
use quote::ToTokens;

pub(crate) trait Apply<T> {
    fn apply(&self, value: &T, tokens: &mut TokenStream);
}

impl<F, T> Apply<T> for F
where
    F: Fn(&T, &mut TokenStream),
{
    fn apply(&self, value: &T, tokens: &mut TokenStream) {
        self(value, tokens)
    }
}

pub(crate) struct IterItem<'a, A, T> {
    apply: A,
    value: &'a T,
}

impl<A, T> ToTokens for IterItem<'_, A, T>
where
    A: Apply<T>,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.apply.apply(self.value, tokens);
    }
}

pub(crate) struct Iter<'a, I, T> {
    iter: I,
    value: &'a T,
}

impl<'a, I, T> Iterator for Iter<'a, I, T>
where
    I: Iterator<Item: Apply<T>>,
{
    type Item = IterItem<'a, I::Item, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let f = self.iter.next()?;

        Some(IterItem {
            apply: f,
            value: self.value,
        })
    }
}

/// Apply an iterator of functions to a value.
pub(crate) fn iter<I, T>(iter: I, value: &T) -> Iter<'_, I::IntoIter, T>
where
    I: IntoIterator<Item: Apply<T>>,
{
    Iter {
        iter: iter.into_iter(),
        value,
    }
}
