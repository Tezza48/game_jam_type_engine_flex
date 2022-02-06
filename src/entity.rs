use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub trait Component {}

pub struct Entity {
    components: HashMap<TypeId, Box<dyn Any>>,
}

impl Entity {
    pub fn new() -> Self {
        Entity {
            components: HashMap::new(),
        }
    }

    pub fn add_component<T: Sized + Any + Component>(&mut self, value: T) {
        self.components.insert(TypeId::of::<T>(), Box::new(value));
    }

    // TODO WT: These could just return Option, makes determining if an entity has a component easier
    pub fn get_component<T: Sized + Any + Component>(&self) -> &T {
        self.components
            .get(&TypeId::of::<T>())
            .unwrap()
            .downcast_ref()
            .unwrap()
    }

    pub fn get_component_mut<T: Sized + Any + Component>(&mut self) -> &mut T {
        self.components
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .downcast_mut()
            .unwrap()
    }

    pub fn has_component<T: Sized + Any + Component>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<T>())
    }

    pub fn remove_component<T: Sized + Any + Component>(&mut self) -> T {
        *self.components.remove(&TypeId::of::<T>())
            .unwrap()
            .downcast::<T>()
            .unwrap()
    }
}
