# rust-game

This is an early developmental / experimental game being built in the Bevy language.

I'm using this project to learn Bevy and Rust as I begin to experiment outside of my comfort zone of managed languages (C#/JS/PHP, etc) to system style languages (Go/Rust/Zig). As such if your hear looking for "good clean idiomatic rust" you might need to look elsewhere as it's likely to be a bit of a mess as I discover and learn the ins and outs of Bevy. Rust I've used in some private projects but nothing as big as a game before, so excitement will ensue, and maybe even some WGSL.

## Possible MODs for speeding up development. 
- full engine for mod ideas: https://github.com/janhohenheim/foxtrot
- bevy_hanabi - particles
- kayak-ui - ui
- bevy_asset_loader - extended asset loading
- oxidized_navigation - path finding
- bevy_kira_audio - audio 
- Kurinji - input mapping
- bevy_tweening (bevy_easings) - easing between things, pulsing etc
- seldom_state - state management 
- bevy_wave_collapse - wave collapse meshes
- naga - shader translator
- naia - networking
- bevy_app_compute - compute shaders
- cosmic-text - better text replaced in 0.11
- bevy_spatial - spatial awareness
- navmesh 
  - https://vleue.github.io/bevy_pathmesh/ 
  - https://github.com/TheGrimsey/oxidized_navigation
  - https://github.com/Seldom-SE/seldom_map_nav
  
## Concepts for learning/implementing possibly
- terrain generation
  - http://clynamen.github.io/blog/2021/01/04/terrain_generation_bevy/
- Spatial hash grid vs aabb for distance checking
- Implementing LOD 
- Tilescaper
- Spiderman windows with internals
- distance fields vs raycast
- raymarching
- deferred decals / deferred textures 
  - https://therealmjp.github.io/posts/bindless-texturing-for-deferred-rendering-and-decals/
- infinite wave function collapse / model synthesis 
  - https://paulmerrell.org/model-synthesis/ 
  - https://www.youtube.com/watch?v=DrTYmUtWWw4
- nvidia new hair design https://3dvf.com/en/nvidia-unveils-an-interactive-hair-simulation-technique-powered-by-ai/
- field of view
- decals from mipmaps?
  - https://github.com/DGriffin91/bevy_sponza_scene
- shaders 
  - https://www.shadertoy.com/results?query=&sort=newest&filter=
- distance fields vs ray marching
- using tokio with bevy ecs
  - https://gist.github.com/shanesveller/81993a56931265b3b3f9f7b4ad707351#file-main-rs
  
## Some cool bevy links
- https://github.com/zkat/awesome-bevy