#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_functions
#import bevy_pbr::prepass_utils

// このシェーダーは、入力テクスチャを指定された色セット（2〜16色）に減色します。
// 次のステップで行われます：
// 1. 入力ピクセルのRGB値を取得します。
// 2. 色セットの各色との距離を計算します。
// 3. 最も近い色を選択します。
// 4. 出力ピクセルのRGB値を選択した色に設定します。
@group(1) @binding(0)
var<uniform> color_set: vec4<f32>;

struct Vertex {
  @location(0) position: vec3<f32>,
}
struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vertex_main(vertex: Vertex) -> @builtin(position) vec4<f32> {
  var out: VertexOutput;
  out.clip_position = mesh_position_local_to_clip(mesh.model, vec4<f32>(vertex.position, 1.0));
  return out;
}

// 距離関数：二乗ユークリッド距離
fn distance_squared(a: vec3<f32>, b: vec3<f32>) -> f32 {
  let diff = a - b;
  return dot(diff, diff);
}

@fragment
fn fragment_main(
    @builtin(position) frag_coord: vec4<f32>,
    @builtin(sample_index) sample_index: u32,
) -> @location(0) vec4<f32> {
    // テクスチャサンプリング
    let color: vec4<f32> = textureSample(sampler, uv);

    // 入力ピクセルのRGB値
    let input_color: vec3<f32> = color.rgb;

    // 色セット内の最も近い色を見つける
    var nearest_color: vec3<f32> = vec3<f32>(0.0);
    var nearest_distance_squared: f32 = 1000000.0;
    for (var i = 0u; i < color_set.length(); i = i + 1u) {
        let distance: f32 = distance_squared(input_color, color_set[i]);
        if (distance < nearest_distance_squared) {
            nearest_distance_squared = distance;
            nearest_color = color_set[i];
        }
    }

    // 出力ピクセルのRGB値
    let output_color: vec3<f32> = nearest_color;

    // 出力ピクセル
    return vec4<f32>(output_color, color.a);
}