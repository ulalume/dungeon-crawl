#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_functions
#import bevy_pbr::prepass_utils

@group(1) @binding(0)
var<uniform> scale: f32;
@group(1) @binding(1)
var<uniform> depth_thresh: f32;
@group(1) @binding(2)
var<uniform> depth_normal_thresh: f32;
@group(1) @binding(3)
var<uniform> depth_normal_thresh_scale: f32;
@group(1) @binding(4)
var<uniform> normal_thresh: f32;


struct Vertex {
  @location(0) position: vec3<f32>,
}
struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
  var out: VertexOutput;
  out.clip_position = mesh_position_local_to_clip(mesh.model, vec4<f32>(vertex.position, 1.0));
  return out;
}

@fragment
fn fragment(
  @builtin(position) frag_coord: vec4<f32>,
  @builtin(sample_index) sample_index: u32,
) -> @location(0) vec4<f32> {
  let half_scale_floor = floor(scale * 0.5);
  let half_scale_ceil = ceil(scale * 0.5);

  let uv_bottom_left = frag_coord + vec4(1.0, 1.0, 0.0, 0.0) * half_scale_floor;
  let uv_top_right = frag_coord + vec4(1.0, 1.0, 0.0, 0.0) * half_scale_ceil;
  let uv_bottom_right = frag_coord + vec4(1.0 * half_scale_ceil, -1.0 * half_scale_floor, 0.0, 0.0);
  let uv_top_left = frag_coord + vec4(-1.0 * half_scale_floor, 1.0 * half_scale_ceil, 0.0, 0.0);

  // calculate edge depth
  let depth0 = prepass_depth(uv_bottom_left, sample_index);
  let depth1 = prepass_depth(uv_top_right, sample_index);
  let depth2 = prepass_depth(uv_bottom_right, sample_index);
  let depth3 = prepass_depth(uv_top_left, sample_index);

  // calculate normal depth
  let normal0 = prepass_normal(uv_bottom_left, sample_index);
  let normal1 = prepass_normal(uv_top_right, sample_index);
  let normal2 = prepass_normal(uv_bottom_right, sample_index);
  let normal3 = prepass_normal(uv_top_left, sample_index);


  let view_front = normalize(view.inverse_view_proj * vec4(0.0, 0.0, 1.0, 1.0));
  var n_dot_v = dot(normal0, view_front.xyz);
  let normal_thresh01 = saturate((n_dot_v - depth_normal_thresh) / (1.0 - depth_normal_thresh));
  let normal_thresh_ = normal_thresh01 * depth_normal_thresh_scale + 1.0;

  let depth_thresh_ = depth_thresh * depth0 * normal_thresh_;
  let depth_finite_difference0 = depth1 - depth0;
  let depth_finite_difference1 = depth3 - depth2;
  var edge_depth = sqrt(pow(depth_finite_difference0, 2.0) + pow(depth_finite_difference1, 2.0)) * 100.0;
  if edge_depth > depth_thresh_ {
    edge_depth = 1.0;
  } else {
    edge_depth = 0.0;
  }

  let normal_finite_difference0 = normal1 - normal0;
  let normal_finite_difference1 = normal3 - normal2;
  var edge_normal = sqrt(dot(normal_finite_difference0, normal_finite_difference0) + dot(normal_finite_difference1, normal_finite_difference1));
  if edge_normal > normal_thresh {
    edge_normal = 1.0;
  } else {
    edge_normal = 0.0;
  }

  let edge = max(edge_depth, edge_normal);
  return vec4(edge, edge, edge, 1.0);
}
