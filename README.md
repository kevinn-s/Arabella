<div align="center">

# Arabella

**Hybrid CPU/GPU renderer**

[![MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)


</div>

<!-- We use cargo-rdme to update the README with the contents of lib.rs.
To edit the following section, update it in lib.rs, then run:
cargo rdme --workspace-project=vello_hybrid
Full documentation at https://github.com/orium/cargo-rdme -->

<!-- Intra-doc links used in lib.rs should be evaluated here.
See https://linebender.org/blog/doc-include/ for related discussion. -->

<!-- cargo-rdme start -->

A hybrid CPU/GPU renderer for 2D vector graphics.

This crate provides a rendering API that combines CPU and GPU operations for efficient
vector graphics processing.
The hybrid approach balances flexibility and performance by:

- Using the CPU for path processing and initial geometry setup
- Leveraging the GPU for fast rendering and compositing
- Minimizing data transfer between CPU and GPU

## Key Features

- Efficient path rendering with CPU-side processing
- GPU-accelerated compositing and blending
- Support for both windowed and headless rendering

## Feature Flags

- `wgpu` (enabled by default): Enables the GPU rendering backend via wgpu and includes the required sparse shaders.
- `wgpu_default` (enabled by default): Enables wgpu with its default hardware backends (such as Vulkan, Metal, and DX12).
- `text` (enabled by default): Enables glyph rendering ([`Scene::glyph_run`]).
- `webgl`: Enables the WebGL rendering backend for browser support, using GLSL shaders for compatibility.

If you need to customize the set of enabled wgpu features, disable this crate's default features then enable its `wgpu` feature.
You can then depend on wgpu directly, setting the specific features you require.
Don't forget to also disable wgpu's default features.

## Architecture

The renderer is split into several key components:

- `Scene`: Manages the render context and path processing on the CPU
- `Renderer` or `WebGlRenderer`: Handles GPU resource management and executes draw operations
- `Scheduler`: Manages and schedules draw operations on the renderer.

See the individual module documentation for more details on usage and implementation.

<!-- cargo-rdme end -->

## Minimum supported Rust Version (MSRV)

compile with **Rust 1.92** and later.

## Compiling
Terakhir, jalankan : 
```sh
wasm-pack test --chrome --features webgl
```

## License

Licensed under 

- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

[Rust code of conduct]: https://www.rust-lang.org/policies/code-of-conduct
