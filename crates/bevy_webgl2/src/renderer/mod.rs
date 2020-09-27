mod webgl2_render_context;
//mod webgl2_render_graph_executor;
mod shader;
mod webgl2_render_resource_context;

pub use webgl2_render_context::*;
//pub use webgl2_render_graph_executor::*;
pub use webgl2_render_resource_context::*;

pub use js_sys;
pub use wasm_bindgen::JsCast;
pub use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlTexture,
    WebGlUniformLocation,
};

pub type Gl = WebGl2RenderingContext;

pub use shader::*;
