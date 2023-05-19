use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(test)
        .run();
}

fn test() {
    println!("Hello World!");
}