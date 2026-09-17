
# Iterum Engine

An engine focused on large open worlds.

## License

Iterum is licensed under the [MIT License](https://mit-license.org/).

Please see `LICENSE` for more detail.

## Roadmap

### Editor

- [ ] Inspector
- [ ] Files / Assets
- [ ] Hierarchy
- [ ] Play, Pause, Stop
- [ ] Export / Build
- [ ] Hot Reload
- [ ] Asset Importer
- [ ] Undo / Redo
- [ ] Scene Saving (and loading)
- [ ] Material Editor (shader tweaking etc. for runtime and editor)
- [ ] Basic Object Shapes (collision on by default)
- [ ] Asset Previews (Considering)
- [ ] Console Log
- [ ] Multiplayer Editing (Considering)
- [ ] Render / Frame Graph (frame time, FPS, latency, Memory usage in RAM and VRAM)

### Nodes

- [ ] Camera
- [ ] RigidBody
- [ ] CharacterBody
- [ ] Directional Light
- [ ] Lights
  - [ ] Spotlight
  - [ ] Point Light
- [ ] CollisionBox (can act as a trigger zone)
- [ ] AudioSource
- [ ] AudioSource3D
- [ ] NaturalIK
- [ ] HumanoidIK
- [ ] RoboticIK
- [ ] AnimationPlayer
- [ ] AnimationTree (like Godot)
- [ ] UI
  - [ ] Blur
  - [ ] Button
  - [ ] Scrollable
  - [ ] Vertical Alignment Box
  - [ ] Horizontal Alignment Box
  - [ ] Label
  - [ ] Input
  - [ ] Slider

### Graphics

- [ ] Realtime Shadows / Lighting
- [ ] Cell Shading
- [ ] Shadow LODs
- [ ] PCSS (softer shadows with distance) / PCF
- [ ] Deferred Rendering For World
- [ ] Forward Rendering For Characters (glass, hair strands, particle fx, transparent)
- [ ] Bindless Materials & Textures
- [ ] Light Culling
- [ ] Volumetric Fog
- [ ] Post Processing (Bloom, Tone Mapping, Color Grading)
- [ ] Anti-Aliasing (MSAA, FXAA, TAA)
- [ ] GPU Particles
- [ ] Physically Based Sky (no skybox)
- [ ] SSR
- [ ] LOD Dithering
- [ ] Automatic LODs
- [ ] GPU Only Rendering
- [ ] Occlusion Culling
- [ ] Virtual Texturing (Stream textures based on what is visible)
- [ ] GPU-Driven Visibility Culling
- [ ] Variable Rate Shading (shade less in flat areas)
- [ ] FSR Upscaling
- [ ] Animations
- [ ] Shader Hot Reload
- [ ] Async Asset Streaming
- [ ] World Streaming (world LODs)

### Animation

- [ ] Skeletal Import (glTF, etc.)
- [ ] Blend Shapes
- [ ] State Machines
- [ ] Animation Blending

### Audio

- [ ] Formats (ogg, wav, flac)
- [ ] Effects (reverb, delay, filter, limiter)
- [ ] Buses
- [ ] HRTF, Doppler, Attenuation

### Utilities

- [ ] Automatic Asset Import
- [ ] Automatic Texture Compression (ASTC, BC7, etc.)
- [ ] Input API
- [ ] Custom Asset Pack Format

### Scripting

- [ ] C# Scripting (netcorehost, csbindgen)
- [ ] Get & set velocity (RigidBody)
- [ ] Get & set position (object, RigidBody)
- [ ] Get & set rotation (object, RigidBody)
- [ ] Get & set size (object, RigidBody)
- [ ] Release (cleanup and removal of a node; similar to `destroy()` in other engines)
- [ ] Delta (function)
- [ ] Void (function)
- [ ] Get child
- [ ] Set child
- [ ] Export variables
- [ ] Tween

### Platforms

- [ ] Windows x86_64
- [ ] Windows Arm64 (Considering)
- [ ] Linux x86_64 - Wayland & X11
- [ ] Linux Arm64 - Wayland & X11
- [ ] macOS - MoltenVK or native

## Installation

Iterum is currently source-only and does not have prebuilt binaries yet.

```bash
cargo install --path .
```

## Contributing

Contributions are always welcome!

There is currently no contributing guide. Please open an issue or pull request if you'd like to contribute.

## Authors

- [@zpeaxirious](https://www.github.com/zpeaxirious)
