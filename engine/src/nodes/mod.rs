use bevy_ecs::component::Component;

pub mod light;
pub mod planet;
pub mod sky;

#[derive(Component)]
pub struct Name(pub String);