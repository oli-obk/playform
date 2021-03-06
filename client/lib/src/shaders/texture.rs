//! Draw textures using a projection matrix.

use gl;
use yaglw::gl_context::GLContext;
use yaglw::shader::Shader;

/// Draw textures using a projection matrix.
pub struct TextureShader<'a> {
  #[allow(missing_docs)]
  pub shader: Shader<'a>,
}

impl<'a> TextureShader<'a> {
  #[allow(missing_docs)]
  pub fn new<'b>(gl: &'b GLContext) -> Self where 'a: 'b {
    let components = vec!(
      (gl::VERTEX_SHADER, "
        #version 330 core

        uniform mat4 projection_matrix;

        in vec3 position;
        in vec2 texture_position;

        out vec2 tex_position;

        void main() {
          tex_position = texture_position;
          gl_Position = projection_matrix * vec4(position, 1.0);
        }".to_owned()),
      (gl::FRAGMENT_SHADER, "
        #version 330 core

        uniform sampler2D texture_in;
        uniform float alpha_threshold;

        in vec2 tex_position;

        out vec4 frag_color;

        void main() {
          vec4 c = texture(texture_in, tex_position);
          if (c.a < alpha_threshold) {
            discard;
          }
          frag_color = c;
        }".to_owned()),
    );
    TextureShader {
      shader: Shader::new(gl, components.into_iter()),
    }
  }
}
