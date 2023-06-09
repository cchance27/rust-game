use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion},
    prelude::*,
};
use bevy_polyline::prelude::*;

pub mod components;
mod resources;
mod systems;

use resources::*;
use systems::*;
//use components::*;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Selecting::default())
            .add_plugin(PolylinePlugin)
            .add_system(
                handle_mouse_input_selection
                    .run_if(on_event::<MouseMotion>().or_else(on_event::<MouseButtonInput>())),
            )
            .add_system(draw_selection_indicator)
            .add_system(draw_selection_box);
    }
}
