# Hyultis Game Egine

This is a minimal game engine, writed in rust and focusing on vulkan api.
I don't target it to become a "big" thing and don't offer support for it, **go to [bevy](https://bevyengine.org/) or other if you want something ready for making games with
support.**

you can still open an issue if you want some more explanation than here.

The goal was to learn rust/vulkan and create a little game with it : [heatchain](https://github.com/hyultis/heatchain_public)

## structure

The engine is heavily based on multithreading.
I have written 2 library to help me : [singletonThread](https://crates.io/crates/singletonThread) and [HArcMut](https://crates.io/crates/HArcMut)

The main thread is used the engine for window inputs, rendering, calling services, audio, etc

Each component of the engine is split into a "service", a singleton that can be called from everywhere.

### List of services :

* HGEMain : the main service, who initiate and control the engine
* ManagerModels : a 3D storage with simple chunk
* ManagerTexture : a manager and storage for textures, using a "order" system to load, set and update texture in multithreading environnement.
* ManagerInterface : a 2D storage with "page" (chunk) system.
* ManagerFont : a manager for front, based on the texture "font"
* ManagerShaders : glsl shader system for HGE
* ManagerPipeline : vulkan's pipeline storage
* ManagerAudio : an optional service to play audio (need to initialized before the engine, on the main thread, to use it with init: use winit_UserDefinedEventOverride)
* ManagerAnimation : a simple animation system (camera animation is done via HGEMain.Camera_addAnim() for optimisation)

### Directories

* components : basic component ( color, position, offset, uvcoord, etc ), for 2D the engine support "interfacePosition" and for 3D "worldposition"
* configs : HGE configuration structure ( with HGEconfig::defineGeneral()  )
* entities : all default entities, available for 2D or 3D (Cube/loadOBJ/teapot or 3D only)
* fronts : simple connector to windows library (winit or sdl, sdl is unstable, you can also write your own)
* interface : anything about 2D management and specific entities (Bar, Line, Text, Ui<x>)
* Models3D : anything about 3D management
* Pipeline : anything about vulkan pipelines
* Shaders : anything about vulkan glsl shaders
* Texture : anything about texture (TextureDescriptor, TexturePart and Order)
* root : global stuff

### Rendering

Despite the rendering being done on the main thread, it's only calling cache from any ShaderDrawer_Manager existing for any pass ( they are 3 static pass : see HGEsubpassName )

Each cache is updated any time an entity is updated (by ShaderDrawer_Manager::allholder_Update()) but without disturbing the actual rendering by generating all the new cache inside
a singletonThread.

So each entity is controlling its own apparence with ShaderDrawerImpl and something like that.

```rust
ShaderDrawer_Manager::inspect::<HGE_shader_2Dsimple_holder>( move | holder|{
holder.insert(tmp, structure);
});
```

A specific feature "dynamicresolution" use viewport to render at lower resolution and upscale the pre-final render into the native size.
You can use HGEMain::singleton().setWindowHDPI() to change the ratio, but it's just a simple image resize, no dlss or else so below 0.7 is ugly. (android use it for performance)

There's no any lightning/shading/raytracing or any advanced rendering stuff.

### Shaders

Vulkano shader need to be present a compile time, you need to copy the default one from <root>/HGE/tests.
They can be modified be need to keep they default input/ouput.

You can create new one if you want them to have different input, but in this case you also need to create new ShaderStructHolder for it.
Different output implied subpass chcange, it's not supported but adding new pass is not complex.

"screen" shaders is run on all the screen one time.

### Android

The engine support android (thank to winit), but limited to vulkan 1.1 version because of it.

### Example

you can run the winit example with : `run --package simple_2D --bin simple_2D` (you can click on the square button to swap the "page")

the sdl one is a bit buggy atm.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
