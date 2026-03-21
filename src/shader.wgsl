struct Uniforms {
    // 0: none, 1: 1.5x, 2: 2x
    zoom_level: f32,
    // 0: not flipped, 1: flipped
    flipped: f32,
    // 0: cover, 1: contain, 2: fill
    fit_mode: f32,
    // 0: circle, 1: rounded-rect, 2: none (fullscreen)
    shape: f32,
    video_aspect: f32,
    window_aspect: f32,
    // Menu overlay rect in pixels (x, y)
    menu_x: f32,
    menu_y: f32,
    // Menu overlay rect size in pixels (w, h)
    menu_w: f32,
    menu_h: f32,
    // Surface size in pixels
    surface_w: f32,
    surface_h: f32,
}

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var video_texture: texture_2d<f32>;
@group(0) @binding(2) var video_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Full-screen triangle (covers the quad with 3 vertices)
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let pos = positions[vertex_index];

    var out: VertexOutput;
    out.position = vec4<f32>(pos, 0.0, 1.0);
    // Convert clip space [-1,1] to UV [0,1], flip Y for texture
    out.uv = vec2<f32>(pos.x * 0.5 + 0.5, 1.0 - (pos.y * 0.5 + 0.5));
    return out;
}

// Signed distance function for a rounded rectangle
fn sd_rounded_rect(p: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(p) - half_size + vec2<f32>(radius);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;

    // Apply zoom (crop toward center)
    var zoom_inset = 0.0;
    if u.zoom_level == 1.0 {
        zoom_inset = 1.0 / 6.0;
    } else if u.zoom_level == 2.0 {
        zoom_inset = 1.0 / 4.0;
    }
    let zoom_min = zoom_inset;
    let zoom_max = 1.0 - zoom_inset;
    uv = uv * (zoom_max - zoom_min) + zoom_min;

    // Apply flip
    if u.flipped == 1.0 {
        uv.x = 1.0 - uv.x;
    }

    // Apply fit mode (adjust UVs for aspect ratio)
    let va = u.video_aspect;
    let wa = u.window_aspect;

    if u.fit_mode == 0.0 {
        // Cover: fill window, crop excess
        if wa > va {
            let scale = wa / va;
            uv.y = (uv.y - 0.5) * scale + 0.5;
        } else {
            let scale = va / wa;
            uv.x = (uv.x - 0.5) * scale + 0.5;
        }
    } else if u.fit_mode == 1.0 {
        // Contain: fit within window, letterbox
        if wa > va {
            let scale = va / wa;
            uv.x = (uv.x - 0.5) / scale + 0.5;
        } else {
            let scale = wa / va;
            uv.y = (uv.y - 0.5) / scale + 0.5;
        }
    }
    // fit_mode == 2.0 is fill/stretch: UVs unchanged

    // Check if UV is out of bounds (for contain mode letterboxing)
    if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let color = textureSample(video_texture, video_sampler, uv);

    // Apply shape clipping
    let centered = in.uv - vec2<f32>(0.5);

    if u.shape == 0.0 {
        // Circle
        let dist = length(centered);
        let alpha = 1.0 - smoothstep(0.495, 0.5, dist);
        return vec4<f32>(color.rgb, color.a * alpha);
    } else if u.shape == 1.0 {
        // Rounded rectangle
        let radius = 0.05;
        let half_size = vec2<f32>(0.5, 0.5);
        let d = sd_rounded_rect(centered, half_size, radius);
        let alpha = 1.0 - smoothstep(-0.005, 0.0, d);
        return vec4<f32>(color.rgb, color.a * alpha);
    }

    // No shape (fullscreen mode)
    return color;
}

// Menu overlay shader - renders a positioned texture quad
@vertex
fn vs_menu(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let pos = positions[vertex_index];
    var out: VertexOutput;
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = vec2<f32>(pos.x * 0.5 + 0.5, 1.0 - (pos.y * 0.5 + 0.5));
    return out;
}

@fragment
fn fs_menu(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert UV to pixel coordinates
    let px = in.uv * vec2<f32>(u.surface_w, u.surface_h);

    // Check if pixel is within menu rectangle
    let menu_min = vec2<f32>(u.menu_x, u.menu_y);
    let menu_max = menu_min + vec2<f32>(u.menu_w, u.menu_h);

    if px.x < menu_min.x || px.x > menu_max.x || px.y < menu_min.y || px.y > menu_max.y {
        discard;
    }

    // Map to UV within menu texture
    let menu_uv = (px - menu_min) / vec2<f32>(u.menu_w, u.menu_h);
    return textureSample(video_texture, video_sampler, menu_uv);
}
