use crate::{Entity, EntityEntry, EntityId, EntityList};
// use std::iter::IntoIterator;

#[derive(Default)]
struct EntitySlice<'a> {
    start: usize,
    slice: &'a mut [EntityEntry],
}

impl<'a> EntitySlice<'a> {
    /// A "dirty" clone that takes mutable reference.
    /// Because it requires mutable reference to self, we cannot implement Clone trait.
    ///
    /// Conceptually, it sounds weird that you need a mutable reference in order to clone,
    /// but in this case what we need is the exclusivity, not the mutability, to ensure that
    /// our internal mutable slice would not have aliases.
    ///
    /// Lifetime annotation is still a bit weird, it should return EntitySlice<'a> since the
    /// underlying EntityEntry lifetime should not change by making a slice to it, but
    /// somehow it fails to compile if I do.
    fn clone(&mut self) -> EntitySlice {
        EntitySlice {
            start: self.start,
            slice: self.slice,
        }
    }
}

/// A structure that allow random access to structure array with possible gaps.
///
/// It uses a SmallVec of slices, which will put the slices inline into the struct and avoid heap allocation
/// up to 2 elements. Most of the time, we only need left and right slices, which are inlined.
/// In rare occasions we want more slices and it will fall back to heap allocation.
/// This design requires a little inconvenience in exchange. That is, explicitly dropping the StructureDynIter before
/// being able to access the structures pointed to, like the example below. It seems to have something to do with the SmallVec's drop check,
/// but I'm not sure.
///
/// ```ignore
/// fn a(structures: &mut [EntityEntry]) {
///     let (_, iter) = StructureDynIter::new(&mut structures);
///     drop(iter);
///     structures[0].dynamic.name();
/// }
/// ```
///
/// It can access internal object in O(n) where n is the number of slices, not the number of objects.
/// It is convenient when you want to have mutable reference to two elements in the array at the same time.
pub(crate) struct EntityDynIter<'a>(Vec<EntitySlice<'a>>);

impl<'a> EntityDynIter<'a> {
    pub(crate) fn new_all(source: &'a mut EntityList) -> Self {
        Self(vec![EntitySlice {
            start: 0,
            slice: &mut source.0,
        }])
    }

    pub(crate) fn new_split(
        source: &'a mut EntityList,
        split_idx: usize,
    ) -> Option<(&'a mut EntityEntry, Self)> {
        let (left, right) = source.0.split_at_mut(split_idx);
        let (center, right) = right.split_first_mut()?;
        Some((
            center,
            Self(vec![
                EntitySlice {
                    start: 0,
                    slice: left,
                },
                EntitySlice {
                    start: split_idx + 1,
                    slice: right,
                },
            ]),
        ))
    }

    pub(crate) fn dyn_iter_id(&self) -> impl Iterator<Item = (EntityId, &Entity)> + '_ {
        self.0
            .iter()
            .flat_map(move |slice| {
                let start = slice.start;
                slice
                    .slice
                    .iter()
                    .enumerate()
                    .map(move |(i, val)| (i + start, val))
            })
            .filter_map(|(id, val)| {
                Some((
                    EntityId {
                        id: id as u32,
                        gen: val.gen,
                    },
                    val.entity.as_ref()?,
                ))
            })
    }

    pub(crate) fn exclude(&mut self, idx: usize) -> Option<&mut EntityEntry> {
        if let Some((slice_idx, _)) = self
            .0
            .iter_mut()
            .enumerate()
            .find(|(_, slice)| slice.start <= idx && idx < slice.start + slice.slice.len())
        {
            let slice = std::mem::take(&mut self.0[slice_idx]);
            let (left, right) = slice.slice.split_at_mut(idx - slice.start);
            let (center, right) = right.split_first_mut()?;
            self.0[slice_idx] = EntitySlice {
                start: slice.start,
                slice: left,
            };
            self.0.push(EntitySlice {
                start: idx + 1,
                slice: right,
            });
            Some(center)
        } else {
            None
        }
    }
}

// struct EntityIter<'d, 'a> {
//     dyn_iter: &'d EntityDynIter<'a>,
//     slice: usize,
//     item: usize,
// }

// impl<'d, 'a> Iterator for EntityIter<'d, 'a>
// where
//     'a: 'd,
// {
//     type Item = &'a EntityEntry;
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.slice < self.dyn_iter.0.len() && self.item < self.dyn_iter.0[self.slice].slice.len()
//         {
//             Some(&self.dyn_iter.0[self.slice].slice[self.item])
//         } else {
//             None
//         }
//     }
// }

// impl<'d, 'a> IntoIterator for &'d EntityDynIter<'a> {
//     type Item = &'a EntityEntry;
//     type IntoIter = EntityIter<'d, 'a>;
//     fn into_iter(self) -> Self::IntoIter {
//         EntityIter{
//             dyn_iter: self,
//             slice: 0,
//             item: 0,
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::EntityDynIter;
    use crate::{Entity, EntityList};
    #[test]
    fn slice_test() {
        let mut el = EntityList::default();
        let a = el.add(Entity { name: "a" });
        let b = el.add(Entity { name: "b" });
        let c = el.add(Entity { name: "c" });

        let dyn_iter = EntityDynIter::new_all(&mut el);
        // Test repeatability
        for _ in 0..2 {
            let mut iter = dyn_iter.dyn_iter_id();
            assert_eq!(iter.next().map(|(id, e)| (id, e.name)), Some((a, "a")));
            assert_eq!(iter.next().map(|(id, e)| (id, e.name)), Some((b, "b")));
            assert_eq!(iter.next().map(|(id, e)| (id, e.name)), Some((c, "c")));
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn slice_split() {
        let mut el = EntityList::default();
        let a = el.add(Entity { name: "a" });
        let b = el.add(Entity { name: "b" });
        let _c = el.add(Entity { name: "c" });
        let d = el.add(Entity { name: "d" });

        let (split_c, dyn_iter) = EntityDynIter::new_split(&mut el, 2).unwrap();
        assert_eq!(split_c.entity.as_ref().map(|e| e.name), Some("c"));
        // Test repeatability
        for _ in 0..2 {
            let mut iter = dyn_iter.dyn_iter_id();
            assert_eq!(iter.next().map(|(id, e)| (id, e.name)), Some((a, "a")));
            assert_eq!(iter.next().map(|(id, e)| (id, e.name)), Some((b, "b")));
            assert_eq!(iter.next().map(|(id, e)| (id, e.name)), Some((d, "d")));
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn slice_multi_split() {
        let mut el = EntityList::default();
        let a = el.add(Entity { name: "a" });
        let _b = el.add(Entity { name: "b" });
        let c = el.add(Entity { name: "c" });
        let _d = el.add(Entity { name: "d" });
        let e = el.add(Entity { name: "e" });

        let (split_b, mut dyn_iter) = EntityDynIter::new_split(&mut el, 1).unwrap();
        let split_d = dyn_iter.exclude(3).unwrap();
        assert_eq!(split_b.entity.as_ref().map(|e| e.name), Some("b"));
        assert_eq!(split_d.entity.as_ref().map(|e| e.name), Some("d"));
        // Test repeatability
        for _ in 0..2 {
            let mut iter = dyn_iter.dyn_iter_id();
            assert_eq!(iter.next().map(|(id, e)| (id, e.name)), Some((a, "a")));
            assert_eq!(iter.next().map(|(id, e)| (id, e.name)), Some((c, "c")));
            assert_eq!(iter.next().map(|(id, e)| (id, e.name)), Some((e, "e")));
            assert_eq!(iter.next(), None);
        }
    }
}
