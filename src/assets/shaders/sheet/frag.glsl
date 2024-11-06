uniform sampler2D texture_data;

uniform float tile_w;
uniform float tile_h;
uniform float tile_x;
uniform float tile_y;

uniform float inv_x;

void main()
{
    float tcx = inv_x + (1.0 - 2.0 * inv_x) * vertex_texcoord.x;
    vec2 tc = vec2(tile_x + tcx * tile_w, tile_y + (1.0 - vertex_texcoord.y) * tile_h);
    vec2 tcfull = vec2(tc.x, tc.y);
    vec4 texel = texture(texture_data, tcfull);
    if (texel.a != 1.0) {
        discard;
    }
    frag_color = texel;
} 
