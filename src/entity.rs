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
    pub fn get_component<T: Sized + Any + Component>(&self) -> Option<&T> {
        if let Some(b) = self.components.get(&TypeId::of::<T>()) {
            b.downcast_ref()
        } else {
            None
        }
    }

    pub fn get_component_mut<T: Sized + Any + Component>(&mut self) -> Option<&mut T> {
        if let Some(b) = self.components.get_mut(&TypeId::of::<T>()) {
            b.downcast_mut()
        } else {
            None
        }
    }

    pub fn has_component<T: Sized + Any + Component>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<T>())
    }

    pub fn remove_component<T: Sized + Any + Component>(&mut self) -> T {
        *self
            .components
            .remove(&TypeId::of::<T>())
            .unwrap()
            .downcast::<T>()
            .unwrap()
    }
}

pub trait FindEntityWithComponent {
    fn find_entity_with_component<T: Sized + Any + Component>(&mut self) -> Option<&Entity>;
}

impl<'a> FindEntityWithComponent for std::slice::Iter<'a, Entity> {
    fn find_entity_with_component<T: Sized + Any + Component>(&mut self) -> Option<&Entity> {
        self.find(|e| e.has_component::<T>())
    }
}

pub trait FindEntityWithComponentMut {
    fn find_entity_with_component_mut<T: Sized + Any + Component>(&mut self)
        -> Option<&mut Entity>;
}

impl<'a> FindEntityWithComponentMut for std::slice::IterMut<'a, Entity> {
    fn find_entity_with_component_mut<T: Sized + Any + Component>(
        &mut self,
    ) -> Option<&mut Entity> {
        self.find(|e| e.has_component::<T>())
    }
}