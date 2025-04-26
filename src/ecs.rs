use std::{any::{Any, TypeId}, collections::{HashMap}};

type EntityId = u16;
type EntityGeneration = u64; // TODO: This is probably overkill, but saves having to check if we've run out of generations. Make a choice later!

#[derive(Debug)]
pub enum EntityError {
    /// An Entity ID was out of bounds.
    OutOfBounds,
    /// An Entity was invalid (either dead or its generation was outdated).
    InvalidEntity,
}

#[derive(Debug)]
pub enum EntityComponentError {
    /// An Entity was invalid (either dead or its generation was outdated).
    InvalidEntity,
    /// A component was expected to be registered, but it was not.
    UnregisteredComponent,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entity {
    id: EntityId,
    generation: EntityGeneration,
}

impl Entity {
    pub fn get_id(&self) -> EntityId {
        self.id
    }

    pub fn get_generation(&self) -> EntityGeneration {
        self.generation
    }
}

/// Provides information about the status of an Entity to EntityAllocator.
struct EntityAllocatorEntry {
    /// Is this Entity currently allocated?
    is_alive: bool,
    /// Generation given to this Entity last time it was allocated.
    generation: EntityGeneration,
}

/// Keeps track of Entities, both living and dead.
struct EntityAllocator {
    /// A Vec the length of all possible Entity IDs.
    /// The index of each entry in this Vec corresponds to the Entity with id=index.
    entries: Vec<EntityAllocatorEntry>,
    /// A list of Entity IDs that may be recycled.
    /// If an Entity was allocated and then deallocated, it will be added to this Vec
    /// until it gets allocated again. This, along with our generational indexing,
    /// helps to keep our `entries` Vec small!
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
                reusable_entry.is_alive = true; // Mark this Entity as alive again
                reusable_entry.generation += 1; // Increment this Entity's generation to make it distinct
                Entity { id: reusable_entity_id as EntityId, generation: reusable_entry.generation }
            }
            None => {
                self.entries.push(EntityAllocatorEntry { is_alive: true, generation: 0 });
                // FIXME: No protection for overflows here! If `self.entries.len()` is > EntityId::MAX,
                // things will probably go very wrong!
                Entity { id: self.entries.len() as EntityId - 1, generation: 0 }
            }
        }
    }

    pub fn deallocate(
        &mut self,
        entity: Entity,
    ) -> Result<(), EntityError> {
        match self.entries.get_mut(entity.id as usize) {
            Some(deallocated_entry) => {
                deallocated_entry.is_alive = false; // Mark this Entity as dead
                self.available_entity_ids.push(entity.id); // Mark this Entity id as reusable
                Ok(())
            }
            None => Err(EntityError::OutOfBounds)
        }
    }
    
    pub fn is_alive(
        &self,
        entity: &Entity,
    ) -> bool {
        match self.entries.get(entity.id as usize) {
            Some(entry) => entry.is_alive,
            None => false,
        }
    }

    pub fn is_valid(
        &self,
        entity: &Entity,
    ) -> bool {
        match self.entries.get(entity.id as usize) {
            Some(entry) => {
                // To be considered valid, the entity must be alive and of the current generation
                entry.is_alive && entry.generation == entity.generation
            }
            None => {
                false
            }
        }
    }

    /// If the Entity with this id is alive, get a new instance of it. Otherwise, get None
    pub fn get_entity_from_id(
        &self,
        id: &EntityId,
    ) -> Option<Entity> {
        let candidate_entity_generation = self.entries.get(*id as usize)?.generation;
        let candidate_entity = Entity { id: *id, generation: candidate_entity_generation };
        if self.is_alive( &candidate_entity ) { Some(candidate_entity) } else { None }
    }
    
    pub fn get_num_entries(&self) -> usize {
        self.entries.len()
    }
}

/// Used to map from a sparsely packed collection of Entities to
/// a densely packed collection of Components
struct ComponentPool<T> {
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

/// Enables registering Entities in type-erased ComponentPools.
trait ComponentStorage: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn register_entity(&mut self, entity: &Entity);
    fn get_entities_with_component(&self) -> &Vec<Entity>;
}

impl<T: 'static> ComponentStorage for ComponentPool<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn register_entity(&mut self, entity: &Entity) {
        let index = entity.id as usize;
        if self.all_entities.len() <= index {
            self.all_entities.resize(index + 1, None);
        }
    }

    fn get_entities_with_component(&self) -> &Vec<Entity> {
        &self.entities_with_component
    }
}

/// Maps from a Component type to its ComponentPool.
/// Supports `ComponentStorage` behaviours (e.g., registering a new Entity)
/// without downcasting to a ComponentPool with a concrete <T>.
struct ComponentMap {
    map: HashMap<TypeId, Box<dyn ComponentStorage>>,
}

impl ComponentMap {
    fn new() -> Self {
        ComponentMap {
            map: HashMap::new(),
        }
    }

    fn insert<T> (
        &mut self,
        value: T,
    ) 
    where
        T: ComponentStorage + Any + 'static
    {
        self.map.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Get an object with type T (will downcast).
    /// To avoid downcasting, try:
    /// ```
    /// self.map.get(&TypeId::of::<T>())
    /// ```
    fn get_typed<T> (&self) -> Option<&T>
    where
        T: ComponentStorage + Any + 'static
    {
        let value = self.map.get(&TypeId::of::<T>())?;
        value
            .as_any()
            .downcast_ref::<T>()
    }

    /// Get a mutable object with type T (will downcast).
    /// To avoid downcasting, try:
    /// ```
    /// self.map.get_mut(&TypeId::of::<T>())
    /// ```
    fn get_typed_mut<T> (&mut self) -> Option<&mut T>
    where
        T: ComponentStorage + Any + 'static
    {
        let value = self.map.get_mut(&TypeId::of::<T>())?;
        value
            .as_any_mut()
            .downcast_mut::<T>()
    }
}

pub struct World {
    /// Used to create and destroy entities
    entity_allocator: EntityAllocator,
    /// Used for typed component access (e.g., for entity-component queries)
    component_pools: ComponentMap,
}

impl World {
    pub fn new() -> World {
        World {
            entity_allocator: EntityAllocator::new(),
            component_pools: ComponentMap::new(),
        }
    }

    /// Create a new Entity, and register it with all ComponentPools
    pub fn create_entity(&mut self) -> Entity {
        let entity = self.entity_allocator.allocate();
        self.component_pools.map
            .iter_mut()
            .for_each(| (_, component_storage) | {
                component_storage.register_entity(&entity);
            });
        entity
    }

    /// Destroy an Entity.
    /// Note: to save time, will not deregister this Entity in any ComponentPools.
    /// Make sure you check Entity validity before any access!
    pub fn destroy_entity(
        &mut self,
        entity: Entity,
    ) {
        // Won't error if the Entity is not alive, will just log
        match self.entity_allocator.deallocate(entity){
            Ok(()) => (),
            Err(EntityError::OutOfBounds) => println!("Tried to deallocate an Entity with an out-of-bounds ID!"),
            Err(EntityError::InvalidEntity) => println!("Tried to deallocate an invalid Entity!"),
        }
    }

    pub fn register_component<T: 'static>(&mut self) {
        self.component_pools.insert(
            ComponentPool::<T> {
                all_entities: vec![None; self.entity_allocator.get_num_entries()],
                entities_with_component: Vec::new(),
                components: Vec::new(),
            }
        );
    }

    fn get_component_pool<T: 'static>(&self) -> Result<&ComponentPool<T>, EntityComponentError> {
        match self.component_pools.get_typed::<ComponentPool<T>>() {
            Some(pool) => Ok(pool),
            None => Err(EntityComponentError::UnregisteredComponent)
        }
    }

    fn get_component_pool_mut<T: 'static>(&mut self) -> Result<&mut ComponentPool<T>, EntityComponentError> {
        match self.component_pools.get_typed_mut::<ComponentPool<T>>() {
            Some(pool) => Ok(pool),
            None => Err(EntityComponentError::UnregisteredComponent)
        }
    }

    fn get_all_instances_of_component<T: 'static>(&self) -> Result<&Vec<T>, EntityComponentError> {
        match self.component_pools.get_typed::<ComponentPool<T>>() {
            Some(pool) => Ok(&pool.components),
            None => Err(EntityComponentError::UnregisteredComponent)
        }
    }

    fn get_all_instances_of_component_mut<T: 'static>(&mut self) -> Result<&mut Vec<T>, EntityComponentError> {
        match self.component_pools.get_typed_mut::<ComponentPool<T>>() {
            Some(pool) => Ok(&mut pool.components),
            None => Err(EntityComponentError::UnregisteredComponent)
        }
    }

    /// Get the component of type T for a specific Entity, if it exists.
    /// Outside of this module, use World::query instead.
    fn get_component<T: 'static>(
        &self,
        entity: &Entity,
    ) -> Result<Option<&T>, EntityComponentError> {
        // First check if this entity is valid
        if !self.entity_allocator.is_valid(entity) {
            return Err(EntityComponentError::InvalidEntity)
        }
        let component_pool = self.get_component_pool::<T>()?;
        // We can access directly here (without get) because we are confident that the entity exists and is valid
        match component_pool.all_entities[entity.id as usize] {
            // If the value in all_entities is Some, we have our index for the component!
            Some(dense_data_index) => Ok(Some(&component_pool.components[dense_data_index])),
            None => Ok(None),
        }
    }

    pub fn get_component_mut<T: 'static>(
        &mut self,
        entity: &Entity,
    ) -> Result<Option<&T>, EntityComponentError> {
        // First check if this entity is valid
        if !self.entity_allocator.is_valid(entity) {
            return Err(EntityComponentError::InvalidEntity)
        }
        let component_pool = self.get_component_pool_mut::<T>()?;
        // We can access directly here (without get) because we are confident that the entity exists and is valid
        match component_pool.all_entities[entity.id as usize] {
            // If the value in all_entities is Some, we have our index for the component!
            Some(dense_data_index) => Ok(Some(&component_pool.components[dense_data_index])),
            None => Ok(None),
        }
    }

    /// Get all Entities and components matching a given Query.
    /// See the Query trait and its implementations.
    pub fn query<'a, Q: Query<'a>>(&'a self) -> impl Iterator<Item = (Entity, Q::QueryResult)> + 'a {
        // TODO: Currently, this will iterate over all Entities. Try using Q::get_component_types with ComponentMap to
        // get the component type in the query with the lowest number of components in its component pool,
        // and iterate over that instead
        (0..self.entity_allocator.get_num_entries())
            .filter_map(| id | { self.entity_allocator.get_entity_from_id(&(id as EntityId))})  // FIXME: Not great, allocating an Entity every time
            .filter_map(| entity | {
                Q::execute(self, &entity)
                .map(| query_result | (entity, query_result))
            }) 
    }

    pub fn add_component<T: 'static>(
        &mut self,
        entity: &Entity,
        component: T,
    ) -> Result<(), EntityComponentError> {
        // First check if this entity is valid
        if !self.entity_allocator.is_valid(entity) {
            return Err(EntityComponentError::InvalidEntity)
        }
        let component_pool = self.get_component_pool_mut::<T>()?;
        match component_pool.all_entities[entity.id as usize] {
            // If the value in `all_entities` is Some, the Entity already has this component
            Some(_) => {
                println!("Tried to add a component to an entity that already had it!");
                Ok(())
            },
            None => {
                component_pool.entities_with_component.push(*entity);
                component_pool.components.push(component);
                let entities_with_component_index = component_pool.entities_with_component.len() - 1;
                component_pool.all_entities[entity.id as usize] = Some(entities_with_component_index);
                Ok(())
            },
        }
    }
}

/// Used to create Query trait objects. When used with World::query,
/// gets all Entities that match the given Query.
/// Note: Components in the Query **must** be registered in the World.
/// See World::register_component.
pub trait Query<'a> {
    type QueryResult;

    /// Returns all component types involved in this query.
    fn get_component_types() -> Vec<TypeId>;
    /// Execute the query for this Entity, maybe returning component(s).
    fn execute(world: &'a World, entity: &Entity) -> Option<Self::QueryResult>;
}

/// Get all Entities that have the component A.
/// # Examples:
/// ```
/// world.query::<(&Transform)>()
///     .iter()
///     .for_each(| entity, transform | {
///         ...
///     })
/// ```
impl <'a, A: 'static> Query<'a> for &'a A {
    type QueryResult = &'a A;

    fn get_component_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>()]
    }

    fn execute(
        world: &'a World,
        entity: &Entity,
    ) -> Option<Self::QueryResult> {
        world.get_component::<A>(entity).expect("Invalid entity, or component was not registered!")
    }
}

/// Get all Entities that have component A and component B.
/// # Examples:
/// ```
/// world.query::<(&Transform, &Sprite)>()
///     .iter()
///     .for_each(| entity, (transform, sprite) | {
///         ...
///     })
/// ```
impl <'a, A: 'static, B: 'static> Query<'a> for (&'a A, &'a B) {
    type QueryResult = (&'a A, &'a B);

    fn get_component_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>()]
    }

    fn execute(
        world: &'a World,
        entity: &Entity,
    ) -> Option<Self::QueryResult> {
        let component_a = world.get_component::<A>(entity).expect("Invalid entity, or component was not registered!")?;
        let component_b = world.get_component::<B>(entity).expect("Invalid entity, or component was not registered!")?;
        Some((component_a, component_b))
    }
}