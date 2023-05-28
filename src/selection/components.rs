use bevy::prelude::*;

#[derive(Component)]
pub struct Selectable;

#[derive(Component)]
pub struct PendingSelection;

#[derive(Component)]
pub struct SelectedUnit(pub Entity);
