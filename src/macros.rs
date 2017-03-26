//! Macros used to generate an ECS!

/// Declares a component with the given name and type.
#[macro_export]
macro_rules! component {
    ( 
        $( #[$meta:meta] )*
        pub $name:ident ( $type:ty );
    ) => {
        $( #[$meta] )*
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        
        impl complecs::traits::CompId for $name {
            type Type = $type;
        }
    }
}

/// Declares multiple components with the given names and types, and 
/// a struct for storing them.
#[macro_export]
macro_rules! components_and_store {
    // No trailing comma.
    (
        $( #[ $store_meta:meta ] )*
        pub struct $store:ident {
            $(
                $( #[ $comp_meta:meta ] )*
                pub $member:ident : $name:ident ( $type:ty )
            ),*
            $(,)*
        }
    ) => {
        
        $(
            component! { 
                $( #[ $comp_meta ] )*
                pub $name ( $type );
            }
        )*
        
        $( #[ $store_meta ] )*
        component_store! {
            pub struct $store {
                $(
                    $member : $name,
                )*
            }
        }
    }
}

/// Declares a component with the members inside a created module.
/// The process takes a set of mutable and immutable components as arguments,
/// as declared with the `mut` and `ref` arguments.
///
/// The first identifier after the ref/mut is a symbol used in the generated code,
/// but with no real influence on the function body.
/// 
/// The body of the run function is executed in a context, in which the
/// components have been loaded and converted to their associated types.
/// The run function can also have extra `ext` arguments declared, that
/// are just passed directly to the scope.
#[macro_export]
macro_rules! process {
    (
        $( #[$meta:meta] )*
        pub mod $mod:ident {
            $( #[$run_meta:meta] )*
            pub fn $proc_id:ident::run(
                // Mutable components, always first.
                $( mut $mut_gensym:ident $mut_arg:ident : &mut $mut_comp:ident, )*
                
                // Immutable components.
                $( ref $gensym:ident $arg:ident : & $comp:ident, )*
                
                // External arguments (relevant here?)
                $( ext $ext_arg:ident : $ext_ty:ty, )*
            ) $body:block
        }
    ) => {
        $( #[$meta] )*
        /// Process definition namespace `[macro-generated]`.
        pub mod $mod {
            use super::complecs::traits;
            use super::complecs::froggy;
            pub use std::fmt::Debug;
            
            // Empty tuple if no components specified
            
            /// Indices to arguments of this process.
            pub type ArgRefs = (
                $( froggy::StorageRc<<super::$mut_comp as traits::CompId>::Type>, )*
                $( froggy::StorageRc<<super::$comp as traits::CompId>::Type>, )* 
            );    
        }
        
        /// An ECS process.
        pub struct $proc_id;
        
        // Mark this as a process identifier.
        impl complecs::traits::ProcId for $proc_id {
            type ArgRefs = self::$mod::ArgRefs;
        }
        
        // Make sure that entities can only be added to this process 
        // inside the right storage types.
        unsafe impl<S> complecs::traits::AddEntityToProcess<S> for $proc_id 
          where S: complecs::traits::HasProcStore<$proc_id>
              $( + complecs::traits::HasCompStore<$mut_comp> )*
              $( + complecs::traits::HasCompStore<$comp> )*
        {}
        
        // Ensure that arguments are only accessed once by this process.
        $(
            impl complecs::traits::HasArg<$mut_comp> for $proc_id {}
        )*
        $(
            impl complecs::traits::HasArg<$comp> for $proc_id {}
        )*
        
        // Add the run function, and ensure that this too can only
        // be run on a simulation type with the right components.
        impl $proc_id {
            $( #[$run_meta] )*
            pub fn run<S>(sim: &mut S $(, $ext_arg : $ext_ty )* )
              where $proc_id: complecs::traits::AddEntityToProcess<S>,
                    S: complecs::traits::HasProcStore<$proc_id>
                  $( + complecs::traits::HasCompStore<$mut_comp> )*
                  $( + complecs::traits::HasCompStore<$comp> )*
            {
                $(  
                    let mut $mut_arg = unsafe {
                        &mut * <S as complecs::traits::HasCompStore<$mut_comp>>::get_mut_components(sim)
                    }.write();
                )*
                $(
                    let $arg = unsafe {
                        & * <S as complecs::traits::HasCompStore<$comp>>::get_components(sim)
                    }.read();
                )*
                
                for &( $( ref $mut_gensym, )* $( ref $gensym, )* )
                in &<S as complecs::traits::HasProcStore<$proc_id>>::process_members(sim).read() {
                    $(
                        let $mut_arg = $mut_arg.get_mut($mut_gensym);
                    )*
                    $(
                        let $arg = $arg.get($gensym);
                    )*
                    $body
                }
            }
        }
        
        // NOTE: The debug clause allows the concatenation of bounds.
        // Ensure that generated entities can be added to this process.
        impl<E> complecs::traits::ProcArgsFrom<E> for $proc_id
          where E: self::$mod::Debug 
                   $( + complecs::traits::HasComp<$mut_comp> )*
                   $( + complecs::traits::HasComp<$comp> )*
        {
            fn from_entity(e: &E) -> self::$mod::ArgRefs {
                (
                    $(<E as complecs::traits::HasComp<$mut_comp>>::get(e).clone() , )* 
                    $(<E as complecs::traits::HasComp<$comp>>::get(e).clone() , )*
                )
            }
        }
    }
}

/// Declares a new entity, with its members contained in the module with the given name.
///
/// # Generation example
/// ```
/// /// <Your docstrings and/or attributes here>
/// pub mod player {
///     // How the data should be stored
///     pub type ProcData = (StorageRc<C> for C in components);
///
///     pub struct Id; // Identifies the struct
///     impl EntityId for Id { 
///         Data = self::ProcData
///     }
///
///     pub struct Data { ... }; // Used to add the entity to a simulation
///     impl<S> AddEntityToStore<self::Id> for self::Data
///       where S: HasEntityStore<self::Id> 
///             (+ HasComp<C> for C in components)
///             (+ HasProc<P> for P in processes)
///     {
///         fn add_to(self, sim: &mut S) { ... }
///     }
///
///     pub struct CompRefs { ... } // Used internally
/// }
///
#[macro_export]
macro_rules! entity {
    (
        $( #[$mod_meta:meta] )*
        pub mod $entity:ident {
            $( #[ $entity_meta:meta ] )*
            pub struct $entity_id:ident {
                $(
                    $comp_name:ident : $comp_id:ident
                ),*
                $(,)*
            }
            
            impl {
                $( $proc_id:ident ),*
                $(,)*
            }
        }
    ) => {
        $( #[$mod_meta] )*
        /// Entity declaration `[macro-generated]`.
        pub mod $entity {
            use super::complecs::traits;
            use super::complecs::froggy;
        
            /// The data that should be stored about this entity to keep it alive.
            pub type ProcData = ( $( froggy::StorageRc<<super::$proc_id as traits::ProcId>::ArgRefs> ),* ,);
        
            $( #[ $entity_meta ] )*
            #[derive(Debug, Clone, Copy)]
            pub struct $entity_id;
        
            impl traits::EntityId for self::$entity_id {
                type Data = self::ProcData;
            }
            
            // Enforce bounds, woo
            $(
                impl traits::EntityHasComp<super::$comp_id> for self::$entity_id {}
            )*
            $(
                impl traits::EntityHasProc<super::$proc_id>for self::$entity_id {}
            )*
            
            impl $entity_id {
                /// Creates the source data for an entity of this type.
                pub fn new_data( $( $comp_name : <super::$comp_id as traits::CompId>::Type ),* ) -> Data {
                    Data::new(
                        $( $comp_name ),*
                    )
                }
            }
        
            // Create the data used to add the item.
            /// Data used to add this entity to a simulation.
            #[derive(Debug)]
            pub struct Data {
                $(
                    /// A component.
                    pub $comp_name : <super::$comp_id as traits::CompId>::Type,
                )*
            }
        
            impl Data {
                /// Creates a new set of entity data.
                pub fn new( $( $comp_name : <super::$comp_id as traits::CompId>::Type ),* ) -> Data {
                    Data {
                        $( $comp_name ),*
                    }
                }
                
                
                pub fn add_to<S>(self, sim: &mut S) 
                  where S: traits::HasEntityStore<self::$entity_id>
                    $(
                        + traits::HasCompStore<super::$comp_id>
                    )*
                    $(
                        + traits::HasProcStore<super::$proc_id>
                    )*
                    $(
                        , super::$proc_id : traits::AddEntityToProcess<S>
                    )*
                {
                    $(
                        let $comp_name = unsafe {
                            &mut * <S as traits::HasCompStore<super::$comp_id>>::get_mut_components(sim)
                        }.write().insert(self.$comp_name);
                    )*
                    let components = CompRefs {
                        $(
                            $comp_name
                        ),*
                    };
                    let entity = ( $(
                        <super::$proc_id as traits::AddEntityToProcess<S>>::add_entity(sim, &components.clone())
                    ),* ,);
                    <S as traits::HasEntityStore<self::$entity_id>>::get_mut_entities(sim).push(entity);
                }
                
            }
        
            /// A struct holding references to the components of this entity inside
            /// a store. 
            /// 
            /// Used internally to access the right components to get the arguments
            /// for a process when adding the entity to it.
            #[derive(Debug, Clone)]
            pub struct CompRefs {
                $(
                    /// A component.
                    pub $comp_name : froggy::StorageRc<<super::$comp_id as traits::CompId>::Type>,
                )*
            }
        
            $(
                impl traits::HasComp<super::$comp_id> for self::CompRefs {
                    fn get(&self) -> &froggy::StorageRc<<super::$comp_id as traits::CompId>::Type> {
                        &self.$comp_name
                    }
                }
            )*
        }
        
        // Export the identifier.
        pub use self::$entity::$entity_id;
    }
}

/// Declares a storage type for the identified components.
#[macro_export]
macro_rules! component_store {
    // No trailing comma
    (
        $( #[$storage_meta:meta] )*
        pub struct $storage:ident {
            $(
                $member:ident : $component:ty
            ),*
            $(,)*
        }
    ) => {
        /// A storage type for components in an ECS system.
        $( #[ $storage_meta ] )*
        #[derive(Debug, Default)]
        pub struct $storage {
            $(
                /// A component [macro-generated].
                pub $member : complecs::froggy::Storage<<$component as complecs::traits::CompId>::Type>
            ),*
        }
        
        $(
            unsafe impl complecs::traits::HasCompStore<$component> for $storage {
                fn get_mut_components(&mut self) -> *mut complecs::froggy::Storage<<$component as complecs::traits::CompId>::Type> {
                    &mut self.$member
                }
    
                fn get_components(&self) -> *const complecs::froggy::Storage<<$component as complecs::traits::CompId>::Type> {
                    &self.$member
                }
            }
        )*
    }
}

/// Declares a storage stype for process arguments (members).
#[macro_export]
macro_rules! process_store {
    // No trailing commas
    (
        $( #[ $storage_meta:meta ] )*
        pub struct $storage:ident {
            $(
                $member:ident : $proc_id:ty
            ),*
            $(,)*
        }
    ) => {
        /// A storage type for arguments to an ECS process.
        $( #[ $storage_meta ] )*
        #[derive(Debug, Default)]
        pub struct $storage {
            $(
                /// A process [macro-generated].
                pub $member : complecs::froggy::Storage<<$proc_id as complecs::traits::ProcId>::ArgRefs>
            ),*
        }
        
        $(
            impl complecs::traits::HasProcStore<$proc_id> for $storage {
                fn process_members_mut(&mut self) -> &mut complecs::froggy::Storage<<$proc_id as complecs::traits::ProcId>::ArgRefs> {
                    &mut self.$member
                }
    
                fn process_members(&self) -> &complecs::froggy::Storage<<$proc_id as complecs::traits::ProcId>::ArgRefs> {
                    &self.$member
                }
            } 
        )*
    }
}

/// Declares a storage stype for entities.
#[macro_export]
macro_rules! entity_store {
    // No trailing commas
    (
        $( #[ $storage_meta:meta ] )*
        pub struct $storage:ident {
            $(
                $member:ident : $mem_id:ty
            ),*
            $(,)*
        }
    ) => {
        /// A storage type for entities in an ECS system.
        $( #[ $storage_meta ] )*
        #[derive(Debug, Default)]
        pub struct $storage {
            $(
                /// A process [macro-generated].
                pub $member : Vec<<$mem_id as complecs::traits::EntityId>::Data>
            ),*
        }
        
        $(
            impl complecs::traits::HasEntityStore<$mem_id> for $storage {
                fn get_mut_entities(&mut self) -> &mut Vec<<$mem_id as complecs::traits::EntityId>::Data> {
                    &mut self.$member
                }
            }
        )*
    };
}

/// Describes that all components stored by the member of the type is also
/// stored by the type.
#[macro_export]
macro_rules! contains_components {
    (
        $type:ident.$member:ident: $comp_type:ty
    ) => {
        unsafe impl<C> complecs::traits::HasCompStore<C> for $type where C: complecs::traits::CompId, $comp_type: complecs::traits::HasCompStore<C> {
            fn get_mut_components(&mut self) -> *mut complecs::froggy::Storage<<C as complecs::traits::CompId>::Type> {
                self.$member.get_mut_components()
            }

            fn get_components(&self) -> *const complecs::froggy::Storage<<C as complecs::traits::CompId>::Type> {
                self.$member.get_components()
            }
        }
    }
}

/// Describes that all processes stored by the member of the type is also
/// stored by the type.
#[macro_export]
macro_rules! contains_processes {
    (
        $type:ident.$member:ident: $proc_type:ty
    ) => {
        impl<P> complecs::traits::HasProcStore<P> for $type where P: complecs::traits::ProcId, $proc_type: complecs::traits::HasProcStore<P> {
            fn process_members_mut(&mut self) -> &mut complecs::froggy::Storage<P::ArgRefs> {
                self.$member.process_members_mut()
            }
    
            /// Returns an immutable reference to the store of arguments to the process.
            fn process_members(&self) -> & complecs::froggy::Storage<P::ArgRefs> {
                self.$member.process_members()
            }
        }
    }
}

/// Describes that all entities stored by the member of the type is also
/// stored by the type.
#[macro_export]
macro_rules! contains_entities {
    (
        $type:ident.$member:ident: $store:ty
    ) => {
        impl<E> complecs::traits::HasEntityStore<E> for $type where E: complecs::traits::EntityId, $store: complecs::traits::HasEntityStore<E> {
            fn get_mut_entities(&mut self) -> &mut Vec<<E as complecs::traits::EntityId>::Data> {
                self.$member.get_mut_entities()
            }
        }
    }
}
