# rust-game

This is an early developmental / experimental game being built in the Bevy language.

I'm using this project to learn Bevy and Rust as I begin to experiment outside of my comfort zone of managed languages (C#/JS/PHP, etc) to system style languages (Go/Rust/Zig). As such if your hear looking for "good clean idiomatic rust" you might need to look elsewhere as it's likely to be a bit of a mess as I discover and learn the ins and outs of Bevy. Rust I've used in some private projects but nothing as big as a game before, so excitement will ensue, and maybe even some WGSL.

## Possible MODs for speeding up development. 

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
  
## Concepts for learning/implementing possibly
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

## Some cool bevy links
- https://github.com/zkat/awesome-bevy