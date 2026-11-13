//! The [`World`]: entity storage with generational ids and typed component
//! stores.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;

/// Marker trait for component data. Any `'static` struct qualifies.
pub trait Component: 'static {}

impl<T: 'static> Component for T {}

/// Entity identifier: slot index + generation.
///
/// `generation` increments every time a slot is reused, so an [`EntityId`]
/// captured before a despawn is invalid afterwards instead of aliasing the
/// new occupant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId {
    index: u32,
    generation: u32,
}

impl EntityId {
    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({}, gen {})", self.index, self.generation)
    }
}

struct Slot {
    generation: u32,
    alive: bool,
}

/// Type-erased component store interface.
trait AnyStore: Any {
    fn remove(&mut self, index: u32) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Concrete per-type store keyed by slot index.
struct TypedStore<T> {
    map: HashMap<u32, T>,
}

impl<T: 'static> AnyStore for TypedStore<T> {
    fn remove(&mut self, index: u32) -> bool {
        self.map.remove(&index).is_some()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// The ECS world.
#[derive(Default)]
pub struct World {
    slots: Vec<Slot>,
    free: Vec<u32>,
    stores: HashMap<TypeId, Box<dyn AnyStore>>,
}

impl World {
    pub fn new() -> World {
        World::default()
    }

    /// Spawn a new entity; use the returned [`EntityBuilder`] to attach
    /// components.
    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        let (index, generation) = if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.generation += 1;
            slot.alive = true;
            (index, slot.generation)
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(Slot {
                generation: 0,
                alive: true,
            });
            (index, 0)
        };
        EntityBuilder {
            world: self,
            id: EntityId { index, generation },
            committed: false,
        }
    }

    /// Whether `id` refers to the current occupant of its slot.
    pub fn is_alive(&self, id: EntityId) -> bool {
        self.slots
            .get(id.index as usize)
            .is_some_and(|slot| slot.alive && slot.generation == id.generation)
    }

    /// Number of live entities.
    pub fn entity_count(&self) -> usize {
        self.slots.iter().filter(|slot| slot.alive).count()
    }

    /// Iterate live entities.
    pub fn entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            if slot.alive {
                Some(EntityId {
                    index: index as u32,
                    generation: slot.generation,
                })
            } else {
                None
            }
        })
    }

    /// Despawn an entity, removing all its components. Returns whether the
    /// id was alive.
    pub fn despawn(&mut self, id: EntityId) -> bool {
        if !self.is_alive(id) {
            return false;
        }
        let slot = &mut self.slots[id.index as usize];
        slot.alive = false;
        slot.generation += 1;
        for store in self.stores.values_mut() {
            store.remove(id.index);
        }
        self.free.push(id.index);
        true
    }

    fn store<T: Component>(&self) -> Option<&TypedStore<T>> {
        self.stores
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.as_any().downcast_ref::<TypedStore<T>>())
    }

    fn store_mut<T: Component>(&mut self) -> &mut TypedStore<T> {
        self.stores
            .entry(TypeId::of::<T>())
            .or_insert_with(|| {
                Box::new(TypedStore::<T> {
                    map: HashMap::new(),
                })
            })
            .as_any_mut()
            .downcast_mut::<TypedStore<T>>()
            .expect("component store type mismatch")
    }

    /// Attach (or replace) component `value` on `id`.
    ///
    /// Does nothing when the entity is not alive.
    pub fn insert<T: Component>(&mut self, id: EntityId, value: T) -> bool {
        if !self.is_alive(id) {
            return false;
        }
        self.store_mut::<T>().map.insert(id.index, value);
        true
    }

    /// Remove component `T` from `id`. Returns whether it was present.
    pub fn remove<T: Component>(&mut self, id: EntityId) -> bool {
        if !self.is_alive(id) {
            return false;
        }
        self.store_mut::<T>().remove(id.index)
    }

    /// Read component `T` of `id`.
    pub fn get<T: Component>(&self, id: EntityId) -> Option<&T> {
        if !self.is_alive(id) {
            return None;
        }
        self.store::<T>()?.map.get(&id.index)
    }

    /// Mutate component `T` of `id`.
    pub fn get_mut<T: Component>(&mut self, id: EntityId) -> Option<&mut T> {
        if !self.is_alive(id) {
            return None;
        }
        self.store_mut::<T>().map.get_mut(&id.index)
    }

    /// Iterate `(EntityId, &mut T)` for every live entity holding `T`.
    pub fn iter_mut<T: Component>(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> + '_ {
        // Snapshot live slot generations first so the closure only borrows
        // the snapshot + the component store, never conflicting with it.
        let generations: HashMap<u32, u32> = self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.alive)
            .map(|(index, slot)| (index as u32, slot.generation))
            .collect();
        let store = self.store_mut::<T>();
        store.map.iter_mut().filter_map(move |(index, value)| {
            let generation = generations.get(index)?;
            Some((
                EntityId {
                    index: *index,
                    generation: *generation,
                },
                value,
            ))
        })
    }

    /// Join: run `f` for every live entity that has both `A` and `B`.
    pub fn for_each2<A: Component, B: Component, F>(&mut self, mut f: F)
    where
        F: FnMut(EntityId, &mut A, &mut B),
    {
        // Collect matching indices first so `f` can mutate either store.
        let mut matches: Vec<u32> = Vec::new();
        {
            let (a_exists, b_exists) = {
                let b = self
                    .store::<B>()
                    .map(|s| s.map.keys().copied().collect::<Vec<_>>());
                let a = self
                    .store::<A>()
                    .map(|s| s.map.keys().copied().collect::<Vec<_>>());
                (a.unwrap_or_default(), b.unwrap_or_default())
            };
            if a_exists.len() <= b_exists.len() {
                for index in a_exists {
                    if b_exists.contains(&index) && self.slots[index as usize].alive {
                        matches.push(index);
                    }
                }
            } else {
                for index in b_exists {
                    if a_exists.contains(&index) && self.slots[index as usize].alive {
                        matches.push(index);
                    }
                }
            }
        }
        for index in matches {
            let id = EntityId {
                index,
                generation: self.slots[index as usize].generation,
            };
            // Take A out so the two stores are never borrowed simultaneously
            // (a HashMap cannot hand out two &mut entries at once).
            let mut a = self
                .store_mut::<A>()
                .map
                .remove(&index)
                .expect("matched A missing");
            let b = self
                .store_mut::<B>()
                .map
                .get_mut(&index)
                .expect("matched B missing");
            f(id, &mut a, b);
            self.store_mut::<A>().map.insert(index, a);
        }
    }
}

impl fmt::Debug for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("World")
            .field("entities", &self.entity_count())
            .field("stores", &self.stores.len())
            .finish()
    }
}

/// Fluent component attach helper returned by [`World::spawn`].
///
/// Components are committed on `build()` — or on drop, so forgetting to
/// call `build()` still commits what was attached.
pub struct EntityBuilder<'w> {
    world: &'w mut World,
    id: EntityId,
    committed: bool,
}

impl<'w> EntityBuilder<'w> {
    /// Attach component `value`; replaces any existing one of the same type.
    pub fn with<T: Component>(self, value: T) -> EntityBuilder<'w> {
        self.world.insert(self.id, value);
        self
    }

    /// Finish building; returns the final [`EntityId`].
    pub fn build(mut self) -> EntityId {
        self.committed = true;
        self.id
    }
}

impl Drop for EntityBuilder<'_> {
    fn drop(&mut self) {
        // Committing happened inline via `with`; nothing pending to flush.
        // The flag exists to document intent and to let `build()` opt out
        // of future drop-time work if it is ever added.
        self.committed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, PartialEq)]
    struct Tag(&'static str);

    #[test]
    fn spawn_get_and_alive() {
        let mut world = World::new();
        let e = world
            .spawn()
            .with(Position { x: 1.0, y: 2.0 })
            .with(Tag("hero"))
            .build();
        assert!(world.is_alive(e));
        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.get::<Position>(e), Some(&Position { x: 1.0, y: 2.0 }));
        assert_eq!(world.get::<Tag>(e), Some(&Tag("hero")));
        assert_eq!(world.get::<u32>(e), None);
    }

    #[test]
    fn despawn_generations_guard_stale_ids() {
        let mut world = World::new();
        let e1 = world.spawn().with(Position { x: 0.0, y: 0.0 }).build();
        assert!(world.despawn(e1));
        assert!(!world.is_alive(e1));
        assert_eq!(world.get::<Position>(e1), None);
        assert!(!world.despawn(e1)); // double despawn is a no-op

        // slot reuse gives the new entity a different generation
        let e2 = world.spawn().with(Position { x: 9.0, y: 9.0 }).build();
        assert_eq!(e2.index(), e1.index());
        assert_ne!(e2.generation(), e1.generation());
        assert_eq!(world.get::<Position>(e1), None); // stale id: rejected
        assert_eq!(
            world.get::<Position>(e2),
            Some(&Position { x: 9.0, y: 9.0 })
        );
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn insert_replace_remove_and_mutate() {
        let mut world = World::new();
        let e = world.spawn().build();
        world.insert(e, Tag("first"));
        world.insert(e, Tag("second"));
        assert_eq!(world.get::<Tag>(e), Some(&Tag("second")));
        let tag = world.get_mut::<Tag>(e).unwrap();
        tag.0 = "third";
        assert_eq!(world.get::<Tag>(e), Some(&Tag("third")));
        assert!(world.remove::<Tag>(e));
        assert_eq!(world.get::<Tag>(e), None);
        assert!(!world.remove::<Tag>(e));
    }

    #[test]
    fn iter_mut_touches_only_live() {
        let mut world = World::new();
        let a = world.spawn().with(Position { x: 1.0, y: 0.0 }).build();
        let b = world.spawn().with(Position { x: 2.0, y: 0.0 }).build();
        let c = world.spawn().with(Position { x: 3.0, y: 0.0 }).build();
        world.despawn(b);
        for (_id, pos) in world.iter_mut::<Position>() {
            pos.x += 100.0;
        }
        assert_eq!(
            world.get::<Position>(a),
            Some(&Position { x: 101.0, y: 0.0 })
        );
        assert_eq!(
            world.get::<Position>(c),
            Some(&Position { x: 103.0, y: 0.0 })
        );
        assert_eq!(world.get::<Position>(b), None);
    }

    #[test]
    fn for_each2_joins_intersecting_components() {
        let mut world = World::new();
        let has_both = world
            .spawn()
            .with(Position { x: 1.0, y: 1.0 })
            .with(Tag("both"))
            .build();
        let only_pos = world.spawn().with(Position { x: 2.0, y: 2.0 }).build();
        let _only_tag = world.spawn().with(Tag("lonely")).build();

        let mut visited: Vec<(EntityId, f32)> = Vec::new();
        world.for_each2::<Position, Tag, _>(|id, pos, _tag| {
            visited.push((id, pos.x));
        });
        assert_eq!(visited, vec![(has_both, 1.0)]);
        assert!(!visited.iter().any(|(id, _)| *id == only_pos));
    }

    #[test]
    fn dead_entities_never_join() {
        let mut world = World::new();
        let doomed = world
            .spawn()
            .with(Position { x: 5.0, y: 5.0 })
            .with(Tag("x"))
            .build();
        world.despawn(doomed);
        let mut visited = 0;
        world.for_each2::<Position, Tag, _>(|_id, _p, _t| visited += 1);
        assert_eq!(visited, 0);
    }
}
