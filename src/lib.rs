mod dyn_iter;
mod iter;

pub use iter::EntityDynIter;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EntityId {
    id: u32,
    gen: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct Entity {
    name: &'static str,
}

struct EntityEntry {
    gen: u32,
    entity: Option<Entity>,
}

impl EntityEntry {
    fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        if self.gen == id.gen {
            self.entity.as_mut()
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct EntityList(Vec<EntityEntry>);

impl EntityList {
    fn add(&mut self, entity: Entity) -> EntityId {
        for (i, entry) in self.0.iter_mut().enumerate() {
            if entry.entity.is_none() {
                entry.entity = Some(entity);
                entry.gen += 1;
                return EntityId {
                    id: i as u32,
                    gen: entry.gen,
                };
            }
        }

        self.0.push(EntityEntry {
            gen: 0,
            entity: Some(entity),
        });
        EntityId {
            id: self.0.len() as u32 - 1,
            gen: 0,
        }
    }

    fn remove(&mut self, id: EntityId) -> Option<Entity> {
        self.0
            .get_mut(id.id as usize)
            .and_then(|entry| entry.entity.take())
    }

    fn get(&self, id: EntityId) -> Option<&Entity> {
        self.0.get(id.id as usize).and_then(|e| {
            if e.gen == id.gen {
                e.entity.as_ref()
            } else {
                None
            }
        })
    }

    fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.0.get_mut(id.id as usize).and_then(|e| {
            if e.gen == id.gen {
                e.entity.as_mut()
            } else {
                None
            }
        })
    }

    fn get_pair_mut(
        &mut self,
        a: EntityId,
        b: EntityId,
    ) -> (Option<&mut Entity>, Option<&mut Entity>) {
        if a.id < b.id {
            let (left, right) = self.0.split_at_mut(b.id as usize);
            (
                left.get_mut(a.id as usize)
                    .and_then(|entry| entry.get_mut(a)),
                right.first_mut().and_then(|s| s.get_mut(b)),
            )
        } else if b.id < a.id {
            let (left, right) = self.0.split_at_mut(a.id as usize);
            (
                right.first_mut().and_then(|s| s.get_mut(a)),
                left.get_mut(b.id as usize)
                    .and_then(|entry| entry.get_mut(b)),
            )
            // The following cases are when a and b points to the same index. In that case we want to return
            // only the one with valid generation.
        } else if self
            .0
            .get(a.id as usize)
            .map(|a_obj| a.gen != a_obj.gen)
            .unwrap_or(true)
        {
            (
                None,
                self.0
                    .get_mut(b.id as usize)
                    .and_then(|entry| entry.get_mut(b)),
            )
        } else {
            (
                self.0
                    .get_mut(a.id as usize)
                    .and_then(|entry| entry.get_mut(a)),
                None,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Entity, EntityId, EntityList};

    #[test]
    fn it_works() {
        let mut el = EntityList::default();
        let a = el.add(Entity { name: "a" });
        let b = el.add(Entity { name: "b" });
        let c = el.add(Entity { name: "c" });
        assert_eq!(b, EntityId { id: 1, gen: 0 });
        el.remove(b);
        assert_eq!(el.get(a), Some(&Entity { name: "a" }));
        assert_eq!(el.get(b), None);
        assert_eq!(el.get(c), Some(&Entity { name: "c" }));
        assert_eq!(el.0.len(), 3);

        let d = el.add(Entity { name: "d" });
        assert_eq!(d, EntityId { id: 1, gen: 1 });
        assert_eq!(el.get(d), Some(&Entity { name: "d" }));
        assert_eq!(el.0.len(), 3);

        if let Some(a) = el.get_mut(a) {
            a.name = "A";
        }
        assert_eq!(el.get(a), Some(&Entity { name: "A" }));
    }

    #[test]
    fn get_pair() {
        let mut el = EntityList::default();
        let a = el.add(Entity { name: "a" });
        let b = el.add(Entity { name: "b" });
        let c = el.add(Entity { name: "c" });

        assert_eq!(
            el.get_pair_mut(a, b),
            (
                Some(&mut Entity { name: "a" }),
                Some(&mut Entity { name: "b" })
            )
        );
        assert_eq!(
            el.get_pair_mut(b, c),
            (
                Some(&mut Entity { name: "b" }),
                Some(&mut Entity { name: "c" })
            )
        );
        assert_eq!(
            el.get_pair_mut(c, a),
            (
                Some(&mut Entity { name: "c" }),
                Some(&mut Entity { name: "a" })
            )
        );

        el.remove(a);

        let d = el.add(Entity { name: "d" });

        assert_eq!(
            el.get_pair_mut(d, a),
            (Some(&mut Entity { name: "d" }), None)
        );

        el.remove(d);

        let _e = el.add(Entity { name: "e" });
        assert_eq!(el.get_pair_mut(d, a), (None, None));
    }
}
