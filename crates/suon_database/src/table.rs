//! Typed table resources used by Bevy systems.
//!
//! Each type implementing [`Table`] is wrapped in a [`Tables<T>`] resource and
//! accessed through the [`Db<T>`] / [`DbMut<T>`] system parameters. The wrapper
//! tracks dirty mutations automatically: every mutable dereference of
//! `DbMut<T>` (or of `Tables<T>`) bumps the table's dirty epoch, which the
//! persistence layer uses to schedule background saves without requiring
//! callers to call any `mark_dirty` method by hand.

use bevy::{ecs::system::SystemParam, prelude::*};

/// Marks a structure as a typed database table resource.
///
/// Combine with `#[derive(Table)]` from `suon_macros` to register a struct as
/// a [`Tables<T>`] resource.
pub trait Table: Send + Sync + 'static {}

/// Resource wrapper that owns a typed table and tracks dirty mutations.
///
/// Direct mutations through [`DbMut<T>`] (or any other code that mutably
/// dereferences `Tables<T>`) bump [`dirty_epoch`](Self::dirty_epoch) so the
/// persistence layer can detect changes without explicit hooks.
#[derive(Resource)]
pub struct Tables<T: Table> {
    inner: T,
    dirty_epoch: u64,
}

impl<T: Table + Default> Default for Tables<T> {
    fn default() -> Self {
        Self {
            inner: T::default(),
            dirty_epoch: 0,
        }
    }
}

impl<T: Table> Tables<T> {
    /// Wraps an inner table value in a fresh, clean resource.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            dirty_epoch: 0,
        }
    }

    /// Returns the current dirty epoch, or zero when the table is clean.
    pub fn dirty_epoch(&self) -> u64 {
        self.dirty_epoch
    }

    /// Returns whether the table has unsaved mutations since the last persist.
    pub fn is_dirty(&self) -> bool {
        self.dirty_epoch != 0
    }

    /// Bumps the dirty epoch without going through `DerefMut`.
    ///
    /// Most callers should mutate the inner table directly through `DbMut`,
    /// which bumps the epoch automatically. Use this only when the table state
    /// is updated through interior mutability that the wrapper cannot observe.
    pub fn mark_dirty(&mut self) {
        self.dirty_epoch = self.dirty_epoch.saturating_add(1).max(1);
    }

    /// Clears the dirty flag if no further mutations happened after `epoch`.
    ///
    /// The persistence layer captures the epoch at the time a save is queued
    /// and passes it back here once the write completes; if the table was
    /// mutated again in the meantime, the dirty flag stays set so a follow-up
    /// save is scheduled.
    pub fn mark_persisted(&mut self, epoch: u64) {
        if self.dirty_epoch == epoch {
            self.dirty_epoch = 0;
        }
    }

    /// Replaces the inner table value without bumping the dirty epoch.
    ///
    /// Used by load systems after pulling rows from storage so that loading
    /// data does not falsely flag the table as needing a save.
    pub(crate) fn replace_loaded(&mut self, inner: T) {
        self.inner = inner;
        self.dirty_epoch = 0;
    }
}

impl<T: Table> std::ops::Deref for Tables<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T: Table> std::ops::DerefMut for Tables<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.dirty_epoch = self.dirty_epoch.saturating_add(1).max(1);
        &mut self.inner
    }
}

/// Read-only system parameter for accessing a typed table.
#[derive(SystemParam)]
pub struct Db<'w, T: Table> {
    #[system_param(validation_message = "Table not initialized")]
    tables: Res<'w, Tables<T>>,
}

impl<T: Table> std::ops::Deref for Db<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.tables
    }
}

/// Mutable system parameter that auto-tracks dirty mutations.
#[derive(SystemParam)]
pub struct DbMut<'w, T: Table> {
    #[system_param(validation_message = "Table not initialized")]
    tables: ResMut<'w, Tables<T>>,
}

impl<T: Table> std::ops::Deref for DbMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.tables
    }
}

impl<T: Table> std::ops::DerefMut for DbMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // ResMut::deref_mut bumps Bevy's change tick; Tables::deref_mut bumps
        // the dirty epoch. Both effects compose through the auto-deref
        // coercion the compiler applies to the return value.
        &mut self.tables
    }
}

/// Extension trait registering typed tables on a Bevy [`App`].
pub trait AppDbExt {
    /// Initializes a typed table resource with its default value when missing.
    fn init_db_table<T: Table + Default>(&mut self) -> &mut Self;

    /// Inserts or replaces a typed table resource with a concrete value.
    fn insert_db_table<T: Table>(&mut self, table: T) -> &mut Self;
}

impl AppDbExt for App {
    fn init_db_table<T: Table + Default>(&mut self) -> &mut Self {
        self.init_resource::<Tables<T>>();
        self
    }

    fn insert_db_table<T: Table>(&mut self, table: T) -> &mut Self {
        self.insert_resource(Tables::new(table));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MyTable {
        value: bool,
    }

    impl Table for MyTable {}

    #[test]
    fn should_initialize_table_resource() {
        let mut app = App::new();
        app.init_db_table::<MyTable>();

        assert!(
            app.world().contains_resource::<Tables<MyTable>>(),
            "init_db_table should create a Tables<MyTable> resource",
        );
    }

    #[test]
    fn should_default_to_clean_dirty_epoch() {
        let table = Tables::<MyTable>::default();

        assert_eq!(table.dirty_epoch(), 0);
        assert!(!table.is_dirty());
    }

    #[test]
    fn should_bump_dirty_epoch_on_mutable_deref() {
        let mut table = Tables::<MyTable>::default();
        let _: &mut MyTable = &mut table;

        assert!(table.is_dirty(), "DerefMut should bump the dirty epoch");
    }

    #[test]
    fn should_clear_dirty_epoch_when_persist_matches() {
        let mut table = Tables::<MyTable>::default();
        let _: &mut MyTable = &mut table;
        let epoch = table.dirty_epoch();

        table.mark_persisted(epoch);

        assert!(
            !table.is_dirty(),
            "mark_persisted should clear the matching epoch",
        );
    }

    #[test]
    fn should_keep_dirty_when_mutation_outraces_persist() {
        let mut table = Tables::<MyTable>::default();
        let _: &mut MyTable = &mut table;
        let queued_epoch = table.dirty_epoch();

        let _: &mut MyTable = &mut table;
        table.mark_persisted(queued_epoch);

        assert!(
            table.is_dirty(),
            "mark_persisted should keep the table dirty if a later mutation happened",
        );
    }

    #[test]
    fn should_not_dirty_when_replacing_loaded_state() {
        let mut table = Tables::new(MyTable { value: false });
        table.replace_loaded(MyTable { value: true });

        assert!(table.value);
        assert!(!table.is_dirty());
    }

    #[test]
    fn should_access_table_through_system_params() {
        let mut app = App::new();
        app.insert_db_table(MyTable { value: false });

        app.add_systems(PreUpdate, |table: Db<MyTable>| {
            assert!(!table.value, "Db should expose initial values to systems");
        })
        .add_systems(Update, |mut table: DbMut<MyTable>| {
            table.value = true;
        })
        .add_systems(PostUpdate, |table: Db<MyTable>| {
            assert!(table.value, "Db should observe values written through DbMut");
        });

        app.update();

        let resource = app
            .world()
            .resource::<Tables<MyTable>>();
        assert!(resource.value);
        assert!(
            resource.is_dirty(),
            "DbMut should bump the dirty epoch on mutation",
        );
    }

    #[test]
    fn should_overwrite_existing_table_when_inserting_again() {
        let mut app = App::new();
        app.insert_db_table(MyTable { value: false });
        app.insert_db_table(MyTable { value: true });

        let table = app.world().resource::<Tables<MyTable>>();
        assert!(table.value);
    }

    #[test]
    fn should_chain_app_db_extension_methods() {
        let mut app = App::new();
        let returned = app
            .init_db_table::<MyTable>()
            .insert_db_table(MyTable { value: true });

        assert!(std::ptr::eq(returned, &app));
    }
}
