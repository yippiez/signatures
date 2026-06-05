//! Multi-line generic signatures, where-clauses, associated types, and
//! higher-ranked trait bounds.  All headers span multiple source lines.

use std::fmt;
use std::marker::PhantomData;

pub const INITIAL_CAPACITY: usize = 16;

/// A typed key-value store parameterised over key and value types.
pub struct TypedMap<K, V>
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    inner: std::collections::HashMap<K, V>,
}

impl<K, V> TypedMap<K, V>
where
    K: Eq + std::hash::Hash + fmt::Debug,
    V: Clone + fmt::Debug,
{
    pub fn new() -> Self {
        TypedMap { inner: std::collections::HashMap::with_capacity(INITIAL_CAPACITY) }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }
}

impl<K, V> Default for TypedMap<K, V>
where
    K: Eq + std::hash::Hash + fmt::Debug,
    V: Clone + fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

/// A generic result combinator.
pub trait Combinable<T, E> {
    type Output;

    fn combine<U, F>(self, other: Result<U, E>, f: F) -> Self::Output
    where
        F: FnOnce(T, U) -> T,
        E: fmt::Display;
}

/// A pipeline stage that transforms items of type `In` to type `Out`.
pub trait Stage<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    type Error: fmt::Debug;

    fn process(&mut self, input: In) -> Result<Out, Self::Error>;

    fn chain<S, Next>(self, next: S) -> Chained<Self, S>
    where
        Self: Sized,
        S: Stage<Out, Next>,
        Next: Send + 'static;
}

/// Two stages chained together.
pub struct Chained<A, B> {
    first: A,
    second: B,
}

impl<A, B, In, Mid, Out> Stage<In, Out> for Chained<A, B>
where
    A: Stage<In, Mid>,
    B: Stage<Mid, Out>,
    In: Send + 'static,
    Mid: Send + 'static,
    Out: Send + 'static,
    A::Error: Into<B::Error>,
{
    type Error = B::Error;

    fn process(&mut self, input: In) -> Result<Out, Self::Error> {
        let mid = self.first.process(input).map_err(Into::into)?;
        self.second.process(mid)
    }

    fn chain<S, Next>(self, next: S) -> Chained<Self, S>
    where
        Self: Sized,
        S: Stage<Out, Next>,
        Next: Send + 'static,
    {
        Chained { first: self, second: next }
    }
}

/// A phantom-typed handle used to associate a compile-time tag with a resource.
pub struct Tagged<T, Tag> {
    pub value: T,
    _tag: PhantomData<Tag>,
}

impl<T, Tag> Tagged<T, Tag>
where
    T: fmt::Debug,
{
    pub fn new(value: T) -> Self {
        Tagged { value, _tag: PhantomData }
    }

    pub fn map<U, F>(self, f: F) -> Tagged<U, Tag>
    where
        F: FnOnce(T) -> U,
        U: fmt::Debug,
    {
        Tagged { value: f(self.value), _tag: PhantomData }
    }
}

/// Merge two iterators of the same item type, applying a predicate.
pub fn merge_filtered<I, J, T, P>(
    left: I,
    right: J,
    predicate: P,
) -> impl Iterator<Item = T>
where
    I: Iterator<Item = T>,
    J: Iterator<Item = T>,
    T: Ord,
    P: Fn(&T) -> bool,
{
    left.chain(right).filter(move |item| predicate(item))
}

/// Higher-ranked trait bound: a callback valid for any lifetime `'a`.
pub fn apply_to_str<F>(s: &str, f: F) -> String
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    f(s).to_owned()
}
