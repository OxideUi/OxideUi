use std::marker::PhantomData;
use std::ops::AddAssign;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeekBias {
    Left,
    Right,
}

pub trait Item {
    type Summary: Clone + Default;

    fn summary(&self) -> Self::Summary;
}

pub trait Dimension<'a, Summary> {
    fn add_summary(&mut self, summary: &'a Summary);
}

#[derive(Clone, Debug)]
pub struct SumTree<T: Item + Clone> {
    items: Vec<T>,
}

impl<T: Item + Clone> SumTree<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn push_tree(&mut self, mut tree: Self) {
        self.items.append(&mut tree.items);
    }

    pub fn extend<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.items.extend(items);
    }

    pub fn summary(&self) -> T::Summary
    where
        for<'a> T::Summary: AddAssign<&'a T::Summary>,
    {
        summarize::<T>(&self.items)
    }

    pub fn cursor<SeekDimension, StartDimension>(&self) -> Cursor<T, SeekDimension, StartDimension>
    where
        StartDimension: Default,
    {
        Cursor {
            items: self.items.clone(),
            position: 0,
            start: StartDimension::default(),
            _seek_dimension: PhantomData,
        }
    }
}

impl<T: Item + Clone> Default for SumTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Cursor<T, SeekDimension, StartDimension>
where
    T: Item + Clone,
{
    items: Vec<T>,
    position: usize,
    start: StartDimension,
    _seek_dimension: PhantomData<SeekDimension>,
}

impl<T, SeekDimension, StartDimension> Cursor<T, SeekDimension, StartDimension>
where
    T: Item + Clone,
    StartDimension: Clone + Default,
    for<'a> StartDimension: Dimension<'a, T::Summary>,
{
    pub fn seek(&mut self, target: &SeekDimension, bias: SeekBias)
    where
        SeekDimension: Clone + Default + PartialOrd,
        for<'a> SeekDimension: Dimension<'a, T::Summary>,
    {
        let (position, start) =
            seek_position::<T, SeekDimension, StartDimension>(&self.items, target, bias);
        self.position = position;
        self.start = start;
    }

    pub fn start(&self) -> &StartDimension {
        &self.start
    }

    pub fn item(&self) -> Option<&T> {
        self.items.get(self.position)
    }

    pub fn next(&mut self) {
        if let Some(item) = self.items.get(self.position) {
            self.start.add_summary(&item.summary());
            self.position += 1;
        }
    }

    pub fn slice(&mut self, target: &SeekDimension, bias: SeekBias) -> SumTree<T>
    where
        SeekDimension: Clone + Default + PartialOrd,
        for<'a> SeekDimension: Dimension<'a, T::Summary>,
    {
        self.seek(target, bias);
        SumTree {
            items: self.items[..self.position].to_vec(),
        }
    }

    pub fn suffix(&self) -> SumTree<T> {
        SumTree {
            items: self.items[self.position..].to_vec(),
        }
    }

    pub fn enumerate(&self) -> impl Iterator<Item = (usize, T)> + '_ {
        self.items[self.position..].iter().cloned().enumerate()
    }
}

fn summarize<T>(items: &[T]) -> T::Summary
where
    T: Item + Clone,
    for<'a> T::Summary: AddAssign<&'a T::Summary>,
{
    let mut summary = T::Summary::default();
    for item in items {
        summary += &item.summary();
    }
    summary
}

fn seek_position<T, SeekDimension, StartDimension>(
    items: &[T],
    target: &SeekDimension,
    bias: SeekBias,
) -> (usize, StartDimension)
where
    T: Item + Clone,
    SeekDimension: Clone + Default + PartialOrd,
    StartDimension: Clone + Default,
    for<'a> SeekDimension: Dimension<'a, T::Summary>,
    for<'a> StartDimension: Dimension<'a, T::Summary>,
{
    let mut seek_dimension = SeekDimension::default();
    let mut start_dimension = StartDimension::default();

    for (index, item) in items.iter().enumerate() {
        let summary = item.summary();
        let mut next_seek_dimension = seek_dimension.clone();
        next_seek_dimension.add_summary(&summary);

        let found = match bias {
            SeekBias::Left => next_seek_dimension >= *target,
            SeekBias::Right => next_seek_dimension > *target,
        };

        if found {
            return (index, start_dimension);
        }

        seek_dimension = next_seek_dimension;
        start_dimension.add_summary(&summary);
    }

    (items.len(), start_dimension)
}
