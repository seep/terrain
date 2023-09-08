use std::cmp::Ordering;
use std::collections::BinaryHeap;

use ordered_float::OrderedFloat;

pub struct PriorityQueue<T> {
    heap: BinaryHeap<PriorityQueueEntry<T>>,
}

impl<T> PriorityQueue<T>
where
    T: Eq,
    T: PartialEq,
{
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, value: T, score: f32) {
        self.heap.push(PriorityQueueEntry {
            score: OrderedFloat(score),
            value,
        });
    }

    pub fn pop(&mut self) -> Option<T> {
        match self.heap.pop() {
            Some(n) => Some(n.value),
            None => None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct PriorityQueueEntry<T> {
    score: OrderedFloat<f32>,
    value: T,
}

impl<T> Ord for PriorityQueueEntry<T>
where
    T: Eq,
    T: PartialEq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl<T> PartialOrd for PriorityQueueEntry<T>
where
    T: Eq,
    T: PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
