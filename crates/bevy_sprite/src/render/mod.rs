use crate::{ColorMaterial, Sprite, TextureAtlas, TextureAtlasSprite};
use bevy_asset::{Assets, Handle};
use bevy_ecs::Resources;
use bevy_render::{
    pipeline::{
        BindGroupDescriptor, BindType::*, BindingDescriptor, BindingShaderStage, BlendDescriptor,
        BlendFactor, BlendOperation, ColorStateDescriptor, ColorWrite, CompareFunction, CullMode,
        DepthStencilStateDescriptor, FrontFace, InputStepMode::*, PipelineDescriptor,
        RasterizationStateDescriptor, StencilStateDescriptor, StencilStateFaceDescriptor,
        UniformProperty::*, VertexAttributeDescriptor, VertexBufferDescriptor, VertexFormat::*,
    },
    render_graph::{base, AssetRenderResourcesNode, RenderGraph, RenderResourcesNode},
    shader::{Shader, ShaderLayout, ShaderStage, ShaderStages},
    texture::{TextureComponentType, TextureFormat, TextureViewDimension::*},
};
use std::borrow::Cow;

use bevy_transform::prelude::GlobalTransform;

pub const SPRITE_PIPELINE_HANDLE: Handle<PipelineDescriptor> =
    Handle::from_u128(278534784033876544639935131272264723170);

pub const SPRITE_SHEET_PIPELINE_HANDLE: Handle<PipelineDescriptor> =
    Handle::from_u128(90168858051802816124217444474933884151);

#[cfg(not(target_arch = "wasm32"))]
macro_rules! glsl_source {
    ($filename:expr) => {
        include_str!($filename)
    };
}

#[cfg(target_arch = "wasm32")]
macro_rules! glsl_source {
    ($filename:expr) => {
        include_str!(concat!("v300/", $filename))
    };
}

pub fn build_sprite_sheet_pipeline(shaders: &mut Assets<Shader>) -> PipelineDescriptor {
    let vert_layout = ShaderLayout {
        bind_groups: vec![
            BindGroupDescriptor::new(
                0,
                vec![BindingDescriptor {
                    name: "Camera".to_string(),
                    index: 0,
                    bind_type: Uniform {
                        dynamic: false,
                        property: Struct(vec![Mat4]),
                    },
                    shader_stage: BindingShaderStage::VERTEX | BindingShaderStage::FRAGMENT,
                }],
            ),
            BindGroupDescriptor::new(
                1,
                vec![
                    BindingDescriptor {
                        name: "TextureAtlas_size".to_string(),
                        index: 0,
                        bind_type: Uniform {
                            dynamic: false,
                            property: Struct(vec![Vec2]),
                        },
                        shader_stage: BindingShaderStage::VERTEX,
                    },
                    BindingDescriptor {
                        name: "TextureAtlas_textures".to_string(),
                        index: 1,
                        bind_type: Uniform {
                            dynamic: false,
                            property: Array(Box::new(Struct(vec![Vec2, Vec2])), 256),
                        },
                        shader_stage: BindingShaderStage::VERTEX,
                    },
                ],
            ),
            BindGroupDescriptor::new(
                2,
                vec![
                    BindingDescriptor {
                        name: "Transform".to_string(),
                        index: 0,
                        bind_type: Uniform {
                            dynamic: false,
                            property: Struct(vec![Mat4]),
                        },
                        shader_stage: BindingShaderStage::VERTEX,
                    },
                    BindingDescriptor {
                        name: "TextureAtlasSprite".to_string(),
                        index: 1,
                        bind_type: Uniform {
                            dynamic: false,
                            property: Struct(vec![Vec4, UInt]),
                        },
                        shader_stage: BindingShaderStage::VERTEX,
                    },
                ],
            ),
        ],
        vertex_buffer_descriptors: vec![VertexBufferDescriptor {
            name: Cow::from("Vertex"),
            stride: 32,
            step_mode: Vertex,
            attributes: vec![
                VertexAttributeDescriptor {
                    name: Cow::from("Vertex_Position"),
                    offset: 0,
                    format: Float3,
                    shader_location: 0,
                },
                VertexAttributeDescriptor {
                    name: "Vertex_Normal".into(),
                    offset: 12,
                    format: Float3,
                    shader_location: 1,
                },
                VertexAttributeDescriptor {
                    name: "Vertex_Uv".into(),
                    offset: 24,
                    format: Float2,
                    shader_location: 2,
                },
            ],
        }],
        entry_point: "main".to_string(),
    };
    let frag_layout = ShaderLayout {
        bind_groups: vec![BindGroupDescriptor::new(
            1,
            vec![
                BindingDescriptor {
                    name: "TextureAtlas_texture".to_string(),
                    index: 2,
                    bind_type: SampledTexture {
                        multisampled: false,
                        dimension: D2,
                        component_type: TextureComponentType::Float,
                    },
                    shader_stage: BindingShaderStage::FRAGMENT,
                },
                BindingDescriptor {
                    name: "TextureAtlas_texture_sampler".to_string(),
                    index: 3,
                    bind_type: Sampler { comparison: false },
                    shader_stage: BindingShaderStage::FRAGMENT,
                },
            ],
        )],
        vertex_buffer_descriptors: vec![VertexBufferDescriptor {
            name: Cow::from("v"),
            stride: 24,
            step_mode: Vertex,
            attributes: vec![
                VertexAttributeDescriptor {
                    name: Cow::from("v_Uv"),
                    offset: 0,
                    format: Float2,
                    shader_location: 0,
                },
                VertexAttributeDescriptor {
                    name: Cow::from("v_Color"),
                    offset: 8,
                    format: Float4,
                    shader_location: 1,
                },
            ],
        }],
        entry_point: "main".to_string(),
    };
    PipelineDescriptor {
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Ccw,
            cull_mode: CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
            clamp_depth: false,
        }),
        depth_stencil_state: Some(DepthStencilStateDescriptor {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::LessEqual,
            stencil: StencilStateDescriptor {
                front: StencilStateFaceDescriptor::IGNORE,
                back: StencilStateFaceDescriptor::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
        }),
        color_states: vec![ColorStateDescriptor {
            format: TextureFormat::Bgra8UnormSrgb,
            color_blend: BlendDescriptor {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha_blend: BlendDescriptor {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            write_mask: ColorWrite::ALL,
        }],
        ..PipelineDescriptor::new(ShaderStages {
            vertex: shaders.add(Shader::from_glsl_and_layout(
                ShaderStage::Vertex,
                glsl_source!("sprite_sheet.vert"),
                vert_layout,
            )),
            fragment: Some(shaders.add(Shader::from_glsl_and_layout(
                ShaderStage::Fragment,
                glsl_source!("sprite_sheet.frag"),
                frag_layout,
            ))),
        })
    }
}

pub fn build_sprite_pipeline(shaders: &mut Assets<Shader>) -> PipelineDescriptor {
    let vert_layout = ShaderLayout {
        bind_groups: vec![
            BindGroupDescriptor::new(
                0,
                vec![BindingDescriptor {
                    name: "Camera".into(),
                    index: 0,
                    bind_type: Uniform {
                        dynamic: false,
                        property: Struct(vec![Mat4]),
                    },
                    shader_stage: BindingShaderStage::VERTEX | BindingShaderStage::FRAGMENT,
                }],
            ),
            BindGroupDescriptor::new(
                2,
                vec![
                    BindingDescriptor {
                        name: "Transform".into(),
                        index: 0,
                        bind_type: Uniform {
                            dynamic: false,
                            property: Struct(vec![Mat4]),
                        },
                        shader_stage: BindingShaderStage::VERTEX,
                    },
                    BindingDescriptor {
                        name: "Sprite_size".into(),
                        index: 1,
                        bind_type: Uniform {
                            dynamic: false,
                            property: Struct(vec![Vec2]),
                        },
                        shader_stage: BindingShaderStage::VERTEX,
                    },
                ],
            ),
        ],
        vertex_buffer_descriptors: vec![VertexBufferDescriptor {
            name: "Vertex".to_string().into(),
            stride: 32,
            step_mode: Vertex,
            attributes: vec![
                VertexAttributeDescriptor {
                    name: "Vertex_Position".into(),
                    offset: 0,
                    format: Float3,
                    shader_location: 0,
                },
                VertexAttributeDescriptor {
                    name: "Vertex_Normal".into(),
                    offset: 12,
                    format: Float3,
                    shader_location: 1,
                },
                VertexAttributeDescriptor {
                    name: "Vertex_Uv".into(),
                    offset: 24,
                    format: Float2,
                    shader_location: 2,
                },
            ],
        }],
        entry_point: "main".to_string(),
    };
    let frag_layout = ShaderLayout {
        bind_groups: vec![BindGroupDescriptor::new(
            1,
            vec![
                BindingDescriptor {
                    name: "ColorMaterial_color".to_string(),
                    index: 0,
                    bind_type: Uniform {
                        dynamic: false,
                        property: Struct(vec![Vec4]),
                    },
                    shader_stage: BindingShaderStage::FRAGMENT,
                },
                BindingDescriptor {
                    name: "ColorMaterial_texture".to_string().into(),
                    index: 1,
                    bind_type: SampledTexture {
                        multisampled: false,
                        dimension: D2,
                        component_type: TextureComponentType::Float,
                    },
                    shader_stage: BindingShaderStage::FRAGMENT,
                },
                BindingDescriptor {
                    name: "ColorMaterial_texture_sampler".into(),
                    index: 2,
                    bind_type: Sampler { comparison: false },
                    shader_stage: BindingShaderStage::FRAGMENT,
                },
            ],
        )],
        vertex_buffer_descriptors: vec![VertexBufferDescriptor {
            name: "v".into(),
            stride: 8,
            step_mode: Vertex,
            attributes: vec![VertexAttributeDescriptor {
                name: "v_Uv".into(),
                offset: 0,
                format: Float2,
                shader_location: 0,
            }],
        }],
        entry_point: "main".into(),
    };

    PipelineDescriptor {
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Ccw,
            cull_mode: CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
            clamp_depth: false,
        }),
        depth_stencil_state: Some(DepthStencilStateDescriptor {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::LessEqual,
            stencil: StencilStateDescriptor {
                front: StencilStateFaceDescriptor::IGNORE,
                back: StencilStateFaceDescriptor::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
        }),
        color_states: vec![ColorStateDescriptor {
            format: TextureFormat::Bgra8UnormSrgb,
            color_blend: BlendDescriptor {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha_blend: BlendDescriptor {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            write_mask: ColorWrite::ALL,
        }],
        ..PipelineDescriptor::new(ShaderStages {
            vertex: shaders.add(Shader::from_glsl_and_layout(
                ShaderStage::Vertex,
                glsl_source!("sprite.vert"),
                vert_layout,
            )),
            fragment: Some(shaders.add(Shader::from_glsl_and_layout(
                ShaderStage::Fragment,
                glsl_source!("sprite.frag"),
                frag_layout,
            ))),
        })
    }
}

pub mod node {
    pub const TRANSFORM: &str = "transform";
    pub const COLOR_MATERIAL: &str = "color_material";
    pub const SPRITE: &str = "sprite";
    pub const SPRITE_SHEET: &str = "sprite_sheet";
    pub const SPRITE_SHEET_SPRITE: &str = "sprite_sheet_sprite";
}

pub trait SpriteRenderGraphBuilder {
    fn add_sprite_graph(&mut self, resources: &Resources) -> &mut Self;
}

impl SpriteRenderGraphBuilder for RenderGraph {
    fn add_sprite_graph(&mut self, resources: &Resources) -> &mut Self {
        self.add_system_node(
            node::TRANSFORM,
            RenderResourcesNode::<GlobalTransform>::new(true),
        );
        self.add_system_node(
            node::COLOR_MATERIAL,
            AssetRenderResourcesNode::<ColorMaterial>::new(false),
        );
        self.add_node_edge(node::COLOR_MATERIAL, base::node::MAIN_PASS)
            .unwrap();

        self.add_system_node(node::SPRITE, RenderResourcesNode::<Sprite>::new(true));
        self.add_node_edge(node::SPRITE, base::node::MAIN_PASS)
            .unwrap();

        self.add_system_node(
            node::SPRITE_SHEET,
            AssetRenderResourcesNode::<TextureAtlas>::new(false),
        );

        self.add_system_node(
            node::SPRITE_SHEET_SPRITE,
            RenderResourcesNode::<TextureAtlasSprite>::new(true),
        );

        let mut pipelines = resources.get_mut::<Assets<PipelineDescriptor>>().unwrap();
        let mut shaders = resources.get_mut::<Assets<Shader>>().unwrap();
        pipelines.set(SPRITE_PIPELINE_HANDLE, build_sprite_pipeline(&mut shaders));
        pipelines.set(
            SPRITE_SHEET_PIPELINE_HANDLE,
            build_sprite_sheet_pipeline(&mut shaders),
        );
        self
    }
}
