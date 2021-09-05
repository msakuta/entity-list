#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EntityId {
    id: u32,
    gen: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct Entity {
    // ...
}

struct EntityEntry {
    gen: u32,
    entity: Option<Entity>,
}

#[derive(Default)]
pub struct EntityList(Vec<EntityEntry>);

impl EntityList {
    fn add(&mut self, entity: Entity) -> EntityId {
        for i in 0..self.0.len() {
            if self.0[i].entity.is_none() {
                self.0[i].entity = Some(entity);
                self.0[i].gen += 1;
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
}

#[cfg(test)]
mod tests {
    use super::{Entity, EntityId, EntityList};
    #[test]
    fn it_works() {
        let mut el = EntityList::default();
        let a = el.add(Entity {});
        let b = el.add(Entity {});
        let c = el.add(Entity {});
        assert_eq!(b, EntityId { id: 1, gen: 0 });
        el.remove(b);
        assert_eq!(el.get(a), Some(&Entity {}));
        assert_eq!(el.get(b), None);
        assert_eq!(el.get(c), Some(&Entity {}));
        assert_eq!(el.0.len(), 3);

        let d = el.add(Entity {});
        assert_eq!(d, EntityId { id: 1, gen: 1 });
        assert_eq!(el.get(d), Some(&Entity {}));
        assert_eq!(el.0.len(), 3);
    }
}
