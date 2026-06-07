//! Type-erased resource container for dependency injection.
//!
//! # Overview
//!
//! [`Resources`] stores exactly one value of each resource type, keyed by
//! [`TypeId`].  Resources are inserted at startup and accessed by systems
//! and tasks during the application's lifecycle.
//!
//! # Resource trait
//!
//! Any `Send + Sync + 'static` type can be a resource by implementing
//! [`Resource`].  The [`suon_macros`] crate provides a derive macro for
//! convenience:
//!
//! ```ignore
//! use suon_macros::Resource;
//!
//! #[derive(Resource)]
//! struct Config { port: u16 }
//! ```
//!
//! # Type erasure
//!
//! Internally, resources are stored as `Box<dyn Any + Send + Sync>` in a
//! [`HashMap<TypeId, …>`].  Lookup is O(1) on average and panics on
//! missing resources, keeping access code ergonomic.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// Marker trait for types that can be stored in [`Resources`].
///
/// Automatically implemented for any `Send + Sync + 'static` type.
pub trait Resource: Send + Sync + 'static {}

/// Type-erased container that stores exactly one value per resource type.
///
/// # Panics
///
/// [`get`](Resources::get) and [`get_mut`](Resources::get_mut) panic when
/// the requested type has not been inserted.  Use [`init`](Resources::init)
/// to insert a default value when the resource may or may not already exist.
#[derive(Default)]
pub struct Resources {
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Resources {
    /// Inserts a `T::default()` value if `T` is not already present.
    ///
    /// If `T` is already stored, the existing value is **overwritten**
    /// with the default.
    pub fn init<T: Resource + Default>(&mut self) -> &mut Self {
        self.resources
            .insert(TypeId::of::<T>(), Box::new(T::default()));
        self
    }

    /// Inserts (or overwrites) a value for type `T`.
    pub fn insert<T: Resource>(&mut self, resource: T) -> &mut Self {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
        self
    }

    /// Returns a shared reference to the value of type `T`.
    ///
    /// # Panics
    ///
    /// Panics if `T` has not been inserted.
    pub fn get<T: Resource>(&self) -> &T {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T>())
            .unwrap_or_else(|| panic!("Resource `{}` not found", std::any::type_name::<T>()))
    }

    /// Returns a mutable reference to the value of type `T`.
    ///
    /// # Panics
    ///
    /// Panics if `T` has not been inserted.
    pub fn get_mut<T: Resource>(&mut self) -> &mut T {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|b| b.downcast_mut::<T>())
            .unwrap_or_else(|| panic!("Resource `{}` not found", std::any::type_name::<T>()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Default)]
    struct Num(i32);
    impl Resource for Num {}

    #[derive(Debug, PartialEq, Default)]
    struct Label(String);
    impl Resource for Label {}

    #[derive(Debug, PartialEq, Default)]
    struct Score(f64);
    impl Resource for Score {}

    #[derive(Debug, PartialEq)]
    struct Position(f64, f64, f64);
    impl Resource for Position {}

    #[derive(Debug, PartialEq, Default)]
    struct Health(u32);
    impl Resource for Health {}

    #[derive(Debug, PartialEq, Default)]
    struct Mana(u32);
    impl Resource for Mana {}

    #[test]
    fn insert_and_get() {
        let mut resources = Resources::default();
        resources.insert(Num(42));
        assert_eq!(resources.get::<Num>().0, 42);
    }

    #[test]
    fn get_mut_modifies() {
        let mut resources = Resources::default();
        resources.insert(Label(String::from("hello")));
        resources.get_mut::<Label>().0.push_str(" world");
        assert_eq!(resources.get::<Label>().0, "hello world");
    }

    #[test]
    #[should_panic(expected = "Resource `suon_resource::tests::Num` not found")]
    fn missing_resource() {
        let resources = Resources::default();
        resources.get::<Num>();
    }

    #[test]
    fn multiple_types() {
        let mut resources = Resources::default();
        resources.insert(Num(1));
        resources.insert(Score(2.0));
        assert_eq!(resources.get::<Num>().0, 1);
        assert_eq!(resources.get::<Score>().0, 2.0);
    }

    #[test]
    fn insert_overwrites() {
        let mut resources = Resources::default();
        resources.insert(Num(1));
        resources.insert(Num(2));
        assert_eq!(resources.get::<Num>().0, 2);
    }

    #[test]
    fn get_mut_chained() {
        let mut resources = Resources::default();
        resources.insert(Num(0));
        *resources.get_mut::<Num>() = Num(99);
        assert_eq!(resources.get::<Num>().0, 99);
    }

    #[test]
    fn many_resources() {
        let mut resources = Resources::default();
        resources.insert(Num(1));
        resources.insert(Label(String::from("a")));
        resources.insert(Score(1.0));
        resources.insert(Position(0.0, 1.0, 2.0));
        resources.insert(Health(100));
        resources.insert(Mana(50));
        assert_eq!(resources.get::<Num>().0, 1);
        assert_eq!(resources.get::<Health>().0, 100);
        assert_eq!(resources.get::<Position>().1, 1.0);
    }

    #[test]
    fn get_mut_of_many() {
        let mut resources = Resources::default();
        resources.insert(Health(100));
        resources.insert(Mana(50));
        *resources.get_mut::<Health>() = Health(80);
        *resources.get_mut::<Mana>() = Mana(30);
        assert_eq!(resources.get::<Health>().0, 80);
        assert_eq!(resources.get::<Mana>().0, 30);
    }

    #[test]
    fn resources_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Resources>();
    }

    #[test]
    fn resources_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Resources>();
    }

    #[test]
    fn init_creates_default() {
        let mut resources = Resources::default();
        resources.init::<Num>();
        assert_eq!(resources.get::<Num>().0, 0);
    }

    #[test]
    fn init_overwrites_existing() {
        let mut resources = Resources::default();
        resources.insert(Num(42));
        resources.init::<Num>();
        assert_eq!(resources.get::<Num>().0, 0);
    }

    #[test]
    fn init_and_insert_coexist() {
        let mut resources = Resources::default();
        resources.init::<Num>();
        resources.insert(Label(String::from("test")));
        assert_eq!(resources.get::<Num>().0, 0);
        assert_eq!(resources.get::<Label>().0, "test");
    }

    #[test]
    fn chained_insert_and_get() {
        let mut resources = Resources::default();
        resources.insert(Num(1)).insert(Num(2));
        assert_eq!(resources.get::<Num>().0, 2);
    }

    #[test]
    fn chained_init_and_get() {
        let mut resources = Resources::default();
        resources.init::<Num>().init::<Label>();
        assert_eq!(resources.get::<Num>().0, 0);
        assert_eq!(resources.get::<Label>().0, "");
    }

    #[test]
    fn get_after_init_returns_same() {
        let mut resources = Resources::default();
        resources.init::<Num>();
        assert_eq!(resources.get::<Num>().0, 0);
        resources.get_mut::<Num>().0 = 7;
        assert_eq!(resources.get::<Num>().0, 7);
    }
}
