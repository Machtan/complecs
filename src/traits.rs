//! Traits encoding internal relationships in an ECS.
use std::fmt::Debug;
use froggy::{Storage, StorageRc};

/// Identifies and describes a component; a data member of an entity.
pub trait CompId {
    /// The data type of this component (not wrapped in anything).
    /// 
    /// Example: `u32`.
    type Type: Debug;
}

/// Identifies and describes an entity; a named collection of components and behaviors.
pub trait EntityId {
    /// The data type that is stored for this entity in the simulation/system/world.
    /// 
    /// This should be a type that contains references to all the process arguments
    /// that 'belong' to this entity.
    ///
    /// Example: `(StorageRc<<RenderProc as ProcId>::Args>, StorageRc<<MoveProc as ProcId>::Args>)`.
    type Data: Debug;
}

// TODO: Processes are 'weaker' than entities now, see if they can be improved
/// Identifies and describes a process.
pub trait ProcId {
    /// What arguments of the process should be stored. (a data type containing `StorageRc`es )
    ///
    /// Think of it as the arguments to the function, but wrapped in storage indices
    /// so for `print_info(age: &u32, name: &String)` this would be:
    /// 
    /// Example: `(StorageRc<u32>, StorageRc<String>)`.
    type ArgRefs: Clone;
}

pub trait ProcArgsFrom<E> : ProcId {
    fn from_entity(e: &E) -> Self::ArgRefs;
}

/// Signifies that the object stores an index to a component of the identified type.
pub trait HasComp<C: CompId> {
    #[inline]
    fn get(&self) -> &StorageRc<C::Type>;
}

/// Signifies that the entity has the given process.
/// 
/// Used in the macros to improve error messages when an entity is declared to have
/// the same process twice.
pub trait EntityHasProc<P: ProcId> : EntityId {}

/// Signifies that the entity has the given component.
/// 
/// Used in the macros to improve error messages when an entity is declared to have
/// the same component twice.
pub trait EntityHasComp<C: CompId> : EntityId {}

/// Signifies that the process has an argument of the given type.
/// 
/// Used in the macros to statically disallow accessing the same component twice.
pub trait HasArg<C: CompId>: ProcId {}

/// Signifies that the object contains a storage for arguments to the
/// identified process. (A list of entity components).
pub trait HasProcStore<P: ProcId> {
    /// Returns a mutable reference to the store of arguments to the process.
    #[inline]
    fn process_members_mut(&mut self) -> &mut Storage<P::ArgRefs>;
    
    /// Returns an immutable reference to the store of arguments to the process.
    #[inline]
    fn process_members(&self) -> & Storage<P::ArgRefs>;
}

/*
/// Signifies that the object has the required components to add entities 
/// to the identified process.
///
/// It is not truly `unsafe` to implement the trait, but can lead to logic errors
/// if the trait bounds are not set correctly.
///
/// `Self` (the implementor) should be constrained to require the components 
/// and processes needed by the identified process
/// 
/// # example
/// ```no_run
/// unsafe impl<T> HasProc<AgePrinter> for T 
///   where T: HasProcStore<AgePrinter>
///          + HasCompStore<AgeId> 
/// {}
/// ```
*/

pub unsafe trait AddEntityToProcess<S: HasProcStore<Self>> : ProcId + Sized {
    #[inline]
    fn add_entity<E>(sim: &mut S, e: &E) -> StorageRc<Self::ArgRefs>
      where Self: ProcArgsFrom<E>
    {
        sim.process_members_mut().write().insert(<Self as ProcArgsFrom<E>>::from_entity(e))
    }
}

/// Signifies that the object contains a storage for components of the identified type.
/// The reason why this trait is unsafe to implement, is that all component stores
/// on an object must be disjoint, in order to not violate aliasing rules in the
/// case that the user wants to modify the stores for two or more components through a process.
pub unsafe trait HasCompStore<C: CompId> {
    /// Returns a mutable reference to the component store.
    #[inline]
    fn get_mut_components(&mut self) -> *mut Storage<C::Type>;
    
    /// Returns an immutable reference to the component store.
    #[inline]
    fn get_components(&self) -> *const Storage<C::Type>;
}

/// Signifies that the object contains a storage for entities of the identified type.
pub trait HasEntityStore<E: EntityId> {
    /// Returns a mutable reference to the entity store.
    #[inline]
    fn get_mut_entities(&mut self) -> &mut Vec<E::Data>;
}

/*
/// Signifies that the entity can be added to a simulation that fulfils a
/// set of requirements.
///
/// It is not truly `unsafe` to implement the trait, but can lead to logic errors
/// if the trait bounds are not set correctly.
///
/// `<S>` should be constrained to include the required components and processes.
///
/// # Example
/// ```no_run
/// unsafe impl<S> AddEntityToStore<EPlayer, S> for Player 
///   where S: HasEntityStore<EPlayer>
///          + HasCompStore<CName>
///          + HasCompStore<CAge>
///          + HasProc<PPrintNameAge>
/// {
///     fn add_to(self, sim: &mut S) { ... }
/// ```
*/
