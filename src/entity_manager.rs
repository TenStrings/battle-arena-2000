use crate::Entity;

#[derive(Default)]
pub struct EntityManager {
    next: u32,
    deleted: std::collections::HashSet<u32>,
}

pub struct EntityIterator<'a> {
    current: u32,
    upper: u32,
    deleted: &'a std::collections::HashSet<u32>,
}

impl EntityManager {
    pub fn new() -> EntityManager {
        EntityManager {
            next: 0,
            deleted: std::collections::HashSet::new(),
        }
    }

    pub fn next_entity(&mut self) -> Entity {
        let e = dbg!(&self.deleted).iter().cloned().next();
        if let Some(e) = e {
            self.deleted.remove(&e);
            Entity(e)
        } else {
            let next = self.next;
            self.next += 1;

            Entity(next)
        }
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        if let Some(last) = self.next.checked_sub(1) {
            use std::cmp::Ordering;
            match entity.0.cmp(&last) {
                Ordering::Equal => {
                    self.next = entity.0;
                }
                Ordering::Less => {
                    self.deleted.insert(entity.0);
                }
                Ordering::Greater => (),
            }
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Entity> + 'a {
        EntityIterator {
            upper: self.next,
            current: 0,
            deleted: &self.deleted,
        }
    }
}

impl<'a> Iterator for EntityIterator<'a> {
    type Item = Entity;
    fn next(&mut self) -> Option<Entity> {
        if self.current < self.upper {
            let entity = self.current;
            self.current += 1;
            if self.deleted.contains(&entity) {
                // TODO: rewrite iterative?
                self.next()
            } else {
                Some(Entity(entity))
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::collection;
    use proptest::prelude::*;

    #[derive(Clone, Debug)]
    enum EntityManagerAction {
        NewEntity,
        DeleteEntity(Entity),
    }

    fn entity_manager_action(max_entities: u32) -> BoxedStrategy<EntityManagerAction> {
        prop_oneof![
            Just(EntityManagerAction::NewEntity),
            any::<u32>()
                .prop_map(move |a| EntityManagerAction::DeleteEntity(Entity(a % max_entities)))
        ]
        .boxed()
    }

    proptest! {
        #[test]
        fn entity_manager_iterator(actions in collection::vec(entity_manager_action(150), 10..150)) {
            let mut entity_manager = EntityManager::new();
            let mut reference = std::collections::BTreeSet::new();

            for action in actions.iter() {
                match action {
                    EntityManagerAction::NewEntity => {
                        let e = entity_manager.next_entity();
                        reference.insert(e);
                    },
                    EntityManagerAction::DeleteEntity(e) => {
                        entity_manager.delete_entity(*e);
                        println!("deleting entity {:?}", e);
                        reference.remove(&e);
                    },
                }
            }

            let actual: Vec<Entity> = entity_manager.iter().collect();
            let expected: Vec<Entity> = reference.iter().cloned().collect();

            // dbg!(&expected);
            // dbg!(&actual);

            assert_eq!(actual, expected);
        }
    }

    #[ignore]
    #[test]
    fn manual() {
        use EntityManagerAction::*;
        let actions = [
            NewEntity,
            NewEntity,
            NewEntity,
            NewEntity,
            NewEntity,
            NewEntity,
            NewEntity,
            NewEntity,
            DeleteEntity(Entity(8)),
            NewEntity,
        ];
        let mut entity_manager = EntityManager::new();
        let mut reference = std::collections::BTreeSet::new();

        for action in actions.iter() {
            match action {
                EntityManagerAction::NewEntity => {
                    let e = entity_manager.next_entity();
                    println!("inserting entity {:?}", e);
                    reference.insert(e);
                }
                EntityManagerAction::DeleteEntity(e) => {
                    entity_manager.remove_entity(*e);
                    println!("deleting entity {:?}", e);
                    reference.remove(&e);
                }
            }
        }

        let actual: Vec<Entity> = entity_manager.iter().collect();
        let expected: Vec<Entity> = reference.iter().cloned().collect();

        // dbg!(&expected);
        // dbg!(&actual);

        assert_eq!(actual, expected);
    }
}
