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
                    gen: self.0[i].gen,
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
}
