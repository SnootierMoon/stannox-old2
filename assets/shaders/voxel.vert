#version 450

layout(location = 0) in uvec2 data;
layout(location = 0) out vec3 outColor;
layout(push_constant) uniform pushConstants {
    mat4 mvpTransform;
};

vec3 colors[6] = vec3[](
    vec3(1.0, 0.0, 0.0),
    vec3(1.0, 0.5, 0.0),
    vec3(1.0, 1.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0),
    vec3(1.0, 0.0, 1.0));

vec3 faceVertexCoords[6] = vec3[](
    vec3(-0.5, -0.5, 0.5),  // 0
    vec3(-0.5,  0.5, 0.5),  // 1
    vec3( 0.5, -0.5, 0.5),  // 2
    vec3( 0.5,  0.5, 0.5),  // 3
    vec3( 0.5, -0.5, 0.5),  // 2
    vec3(-0.5,  0.5, 0.5)); // 1

float cornerIndicesI[6] = float[](-0.5, -0.5,  0.5, 0.5,  0.5, -0.5);
float cornerIndicesJ[6] = float[](-0.5,  0.5, -0.5, 0.5, -0.5,  0.5);

mat3 faceTransforms[6] = mat3[](        // face plane
    mat3(0, 1, 0, 0, 0, 1,   1, 0, 0),  // +x,  yz
    mat3(0, 0, 1, 0, 1, 0,  -1, 0, 0),  // -x,  zy
    mat3(0, 0, 1, 1, 0, 0,  0,  1, 0),  // +y,  zx
    mat3(1, 0, 0, 0, 0, 1,  0, -1, 0),  // -y,  xz
    mat3(1, 0, 0, 0, 1, 0,  0, 0,  1),  // +z,  xy
    mat3(0, 1, 0, 1, 0, 0,  0, 0, -1)); // -z,  yx

void main() {
    uint chunkCubeX = bitfieldExtract(data.y,  0, 5);
    uint chunkCubeY = bitfieldExtract(data.y,  5, 5);
    uint chunkCubeZ = bitfieldExtract(data.y, 10, 5);
    uint direction =  bitfieldExtract(data.y, 15, 3);

    vec3 chunkVertexCoord = vec3(chunkCubeX, chunkCubeY, chunkCubeZ)
        + faceTransforms[direction] * faceVertexCoords[gl_VertexIndex];

    gl_Position = mvpTransform * vec4(chunkVertexCoord, 1);
    outColor = colors[data.x];
}
