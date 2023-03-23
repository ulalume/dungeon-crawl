use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Debug, Clone, AsBindGroup, TypeUuid, Reflect)]
#[uuid = "8f597a4e-ffbb-4422-8706-00fe2928f1d4"]
pub struct OutlineMaterial {
    #[uniform(0)]
    pub scale: f32,
    #[uniform(1)]
    pub depth_thresh: f32,
    #[uniform(2)]
    pub depth_normal_thresh: f32,
    #[uniform(3)]
    pub depth_normal_thresh_scale: f32,
    #[uniform(4)]
    pub normal_thresh: f32,
}
impl Default for OutlineMaterial {
    fn default() -> Self {
        OutlineMaterial {
            scale: 0.5,
            depth_thresh: 1.0,
            depth_normal_thresh: 1.5,
            depth_normal_thresh_scale: 7.0,
            normal_thresh: 0.3,
        }
    }
}
impl Material for OutlineMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/outline.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/outline.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
