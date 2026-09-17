import package::common::frame::Frame;
import package::common::object::ObjectUniform;
import package::common::vertex::VertexInput;

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> frame: Frame;

@group(1) @binding(0)
var<storage, read> objects: array<ObjectUniform>;

@vertex
fn vs_main(input: VertexInput, @builtin(instance_index) instance: u32) -> VertexOutput {
    let model = objects[instance].model;

    var output: VertexOutput;
    output.position = frame.light.view_projection * model * vec4<f32>(input.position, 1.0);

    return output;
}
