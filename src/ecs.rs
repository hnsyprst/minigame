use std::any::TypeId;

use anymap::AnyMap;

type EntityId = u16;
type EntityGeneration = u64; // TODO: This is probably overkill, but saves having to check if we've run out of generations. Make a choice later!

#[derive(Debug)]
pub enum EntityError {
    /// An Entity ID was out of bounds
    OutOfBounds(Entity),
    /// An Entity was invalid (either dead or its generation was outdated)
    InvalidEntity(Entity),
}

#[derive(Debug)]
pub enum EntityComponentError {
    /// An Entity was invalid (either dead or its generation was outdated)
    InvalidEntity(Entity),
    /// An Entity did not have an expected component
    MissingComponent(Entity, TypeId),
    /// A component was expected to be registered, but it was not
    UnregisteredComponent(TypeId),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: EntityId,
    pub generation: EntityGeneration,
}

struct EntityAllocatorEntry {
    is_live: bool,
    generation: EntityGeneration,
}


pub struct EntityAllocator {
    entries: Vec<EntityAllocatorEntry>,
    available_entity_ids: Vec<EntityId>,
}

impl EntityAllocator {
    pub fn new() -> EntityAllocator {
        EntityAllocator {
            entries: Vec::new(), // TODO: Could be good to allocate with some initial capacity
            available_entity_ids: Vec::new(),
        }
    }
    pub fn allocate(&mut self) -> Entity {
        match self.available_entity_ids.pop() {
            Some(reusable_entity_id) => {
                let reusable_entry = &mut self.entries[reusable_entity_id as usize];
                reusable_entry.is_live = true; // Mark this Entity as alive again
                reusable_entry.generation += 1; // Increment this Entity's generation to make it distinct
                return Entity { id: reusable_entity_id as EntityId, generation: reusable_entry.generation }
            }
            None => {
                self.entries.push(EntityAllocatorEntry { is_live: true, generation: 0 });
                // FIXME: No protection for overflows here! If `self.entries.len()` is > EntityId::MAX,
                // things will probably go very wrong!
                return Entity { id: self.entries.len() as EntityId - 1, generation: 0 }
            }
        }
    }

    pub fn deallocate(&mut self, entity: Entity) -> Result<(), EntityError> {
        match self.entries.get_mut(entity.id as usize) {
            Some(deallocated_entry) => {
                deallocated_entry.is_live = false; // Mark this Entity as dead
                self.available_entity_ids.push(entity.id); // Mark this Entity id as reusable
                return Ok(())
            }
            None => return Err(EntityError::OutOfBounds(entity))
        }
    }
    
    pub fn is_alive(&self, entity: Entity) -> bool {
        match self.entries.get(entity.id as usize) {
            Some(entry) => return entry.is_live,
            None => return false,
        }
    }

    pub fn is_valid(&self, entity: Entity) -> bool {
        match self.entries.get(entity.id as usize) {
            Some(entry) => {
                // To be considered valid, the entity must be alive and of the current generation
                return entry.is_live && entry.generation == entity.generation
            }
            None => {
                return false
            }
        }
    }
    
    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }
}

/// Used to map from a sparsely packed collection of Entities to
/// a densely packed collection of Components
struct SparseSet<T> {
    /// The length of this Vec equals the number of entries in the EntityAllocator.
    /// If an Entity has a Component of type <T>, its index in this Vec will contain
    /// an index to entities_with_component. Othersise, its index in this Vec will contain None.
    /// Use this Vec e.g. to check if a given Entity has a Component of type <T>.
    pub all_entities: Vec<Option<usize>>,
    /// The length of this Vec equals the number of Entities with Component of type <T>.
    /// Each entry contains an Entity.
    /// This Vec is parallel with the components Vec.
    /// Use this Vec e.g. to iterate over all Entities that have a Component of type <T>.
    pub entities_with_component: Vec<Entity>,
    /// The length of this Vec equals the number of Entities with Component of type <T>.
    /// Each entry contains a Component of type <T>.
    /// This Vec is parallel with entities_with_component.
    components: Vec<T>,
}

pub struct World {
    entity_allocator: EntityAllocator,
    components: AnyMap,
}

impl World {
    pub fn new() -> World {
        World {
            entity_allocator: EntityAllocator::new(),
            components: AnyMap::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        return self.entity_allocator.allocate()

    }

    pub fn destroy_entity(
        &mut self,
        entity: Entity,
    ) {
        // Won't error if the Entity is not alive, will just log
        match self.entity_allocator.deallocate(entity){
            Ok(()) => (),
            Err(EntityError::OutOfBounds(_)) => println!("Error: tried to deallocate an Entity with an out-of-bounds ID!"),
            Err(EntityError::InvalidEntity(_)) => println!("Error: tried to deallocate an invalid Entity!"),
        }
    }

    pub fn register_component<T: 'static>(&mut self) {
        self.components.insert(
            SparseSet::<T> {
                all_entities: vec![None; self.entity_allocator.num_entries()],
                entities_with_component: Vec::new(),
                components: Vec::new(),
            }
        );
    }

    pub fn get_component_pool<T: 'static>(&self) -> Result<&SparseSet<T>, EntityComponentError> {
        match self.components.get::<SparseSet<T>>() {
            Some(sparse_set) => return Ok(sparse_set),
            None => Err(EntityComponentError::UnregisteredComponent(TypeId::of::<T>()))
        }
    }

    pub fn get_component_pool_mut<T: 'static>(&mut self) -> Result<&mut SparseSet<T>, EntityComponentError> {
        match self.components.get_mut::<SparseSet<T>>() {
            Some(sparse_set) => return Ok(sparse_set),
            None => Err(EntityComponentError::UnregisteredComponent(TypeId::of::<T>()))
        }
    }

    pub fn get_all_instances_of_component<T: 'static>(&self) -> Result<&Vec<T>, EntityComponentError> {
        match self.components.get::<SparseSet<T>>() {
            Some(component_sparse_set) => return Ok(&component_sparse_set.components),
            None => return Err(EntityComponentError::UnregisteredComponent(TypeId::of::<T>()))
        }
    }

    pub fn get_all_instances_of_component_mut<T: 'static>(&mut self) -> Result<&mut Vec<T>, EntityComponentError> {
        match self.components.get_mut::<SparseSet<T>>() {
            Some(component_sparse_set) => return Ok(&mut component_sparse_set.components),
            None => return Err(EntityComponentError::UnregisteredComponent(TypeId::of::<T>()))
        }
    }

    pub fn entity_has_component<T: 'static>(
        &self,
        entity: Entity,
    ) -> Result<bool, EntityComponentError> {
        // First check if this entity is valid
        if !self.entity_allocator.is_valid(entity) {
            return Err(EntityComponentError::InvalidEntity(entity));
        }
        let component_pool = self.get_component_pool::<T>()?;
        Ok(component_pool.all_entities[entity.id as usize].is_some())
    }

    pub fn get_component_from_entity<T: 'static>(
        &self,
        entity: Entity,
    ) -> Result<Option<&T>, EntityComponentError> {
        // First check if this entity is valid
        if !self.entity_allocator.is_valid(entity) {
            return Err(EntityComponentError::InvalidEntity(entity))
        }
        let component_pool = self.get_component_pool::<T>()?;
        // We can access directly here (without get) because we are confident that the entity exists and is valid
        match component_pool.all_entities[entity.id as usize] {
            // If the value in all_entities is Some, we have our index for the component!
            Some(dense_data_index) => return Ok(Some(&component_pool.components[dense_data_index])),
            None => return Ok(None),
        }
    }

    pub fn get_component_from_entity_mut<T: 'static>(
        &mut self,
        entity: Entity,
    ) -> Result<Option<&T>, EntityComponentError> {
        // First check if this entity is valid
        if !self.entity_allocator.is_valid(entity) {
            return Err(EntityComponentError::InvalidEntity(entity))
        }
        let component_pool = self.get_component_pool_mut::<T>()?;
        // We can access directly here (without get) because we are confident that the entity exists and is valid
        match component_pool.all_entities[entity.id as usize] {
            // If the value in all_entities is Some, we have our index for the component!
            Some(dense_data_index) => return Ok(Some(&component_pool.components[dense_data_index])),
            None => return Ok(None),
        }
    }

    pub fn add_component_to_entity<T: 'static>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), EntityComponentError> {
        // First check if this entity is valid
        if !self.entity_allocator.is_valid(entity) {
            return Err(EntityComponentError::InvalidEntity(entity))
        }
        let component_pool = self.get_component_pool_mut::<T>()?;
        match component_pool.all_entities[entity.id as usize] {
            // If the value in sparse is Some, the Entity already has this component
            Some(_) => return Ok(()),
            None => {
                component_pool.entities_with_component.push(entity);
                component_pool.components.push(component);
                return Ok(())
            },
        }
    }       
}