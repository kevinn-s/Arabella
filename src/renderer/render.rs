
pub struct AtlasConfig {
    /// Initial number of atlases to create.
    pub initial_atlas_count: usize,
    /// Maximum number of atlases to create.
    pub max_atlases: usize,
    /// Size of each atlas texture.
    pub atlas_size: (u32, u32),
    /// Whether to automatically create new atlases when needed.
    pub auto_grow: bool,
}

impl Default for AtlasConfig {
    fn default() -> Self {
        Self {
            // NOTE: When targeting wasm32 with a WebGL/GLES backend, you may want to set
            // `initial_atlas_count` to 2. In WGPU's GLES backend, heuristics are used to decide
            // whether a texture should be treated as D2 or D2Array. However, this can cause a
            // mismatch: when depth_or_array_layers == 1, the backend assumes the texture is D2,
            // even if it was actually created as a D2Array. This issue only occurs with the GLES
            // backend.
            //
            // @see https://github.com/gfx-rs/wgpu/blob/61e5124eb9530d3b3865556a7da4fd320d03ddc5/wgpu-hal/src/gles/mod.rs#L470-L517
            initial_atlas_count: 1,
            max_atlases: 8,
            atlas_size: (4096, 4096),
            auto_grow: false
        }
    }
}
pub struct RenderSettings {
    /// This controls how images are managed in GPU memory through texture atlases.
    /// The atlas system packs multiple images into larger textures to reduce the
    /// number of GPU texture bindings. This config allows customizing atlas parameters such as:
    /// - The number and size of atlases
    /// - How images are allocated across multiple atlases
    /// - Whether new atlases are automatically created when needed
    ///
    /// Adjusting these settings can affect memory usage and rendering performance
    /// depending on your application's image usage patterns.
    pub atlas_config: AtlasConfig,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            atlas_config: AtlasConfig::default(),
        }
    }
}