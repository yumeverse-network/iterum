use bevy_ecs::entity::Entity;

#[derive(Default)]
pub struct EditorSelection {
    pub selected: Option<Entity>,
}