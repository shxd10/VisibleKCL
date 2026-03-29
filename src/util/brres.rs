use super::binary::*;
use brres::*;
use brres::{enums::*, json::*};

struct ObjData {
    vertices: Vec<[f32; 3]>,
    groups: Vec<(String, Vec<[usize; 3]>)>,
}

struct MtlFile {
    materials: Vec<MtlData>,
}

struct MtlData {
    name: String,
    color: [f32; 3],
    alpha: f32,
}

fn parse_obj(obj: &str) -> Result<ObjData, String> {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut groups: Vec<(String, Vec<[usize; 3]>)> = Vec::new();
    let mut current_mat = String::new();
    let mut current_faces: Vec<[usize; 3]> = Vec::new();

    for line in obj.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("v") => {
                let x: f32 = parts.next().unwrap().parse().map_err(|e| format!("{e}"))?;
                let y: f32 = parts.next().unwrap().parse().map_err(|e| format!("{e}"))?;
                let z: f32 = parts.next().unwrap().parse().map_err(|e| format!("{e}"))?;
                vertices.push([x, y, z]);
            }
            Some("f") => {
                let idx: Vec<usize> = parts
                    .map(|p| p.split('/').next().unwrap().parse::<usize>().unwrap() - 1)
                    .collect();
                current_faces.push([idx[0], idx[1], idx[2]]);
            }
            Some("usemtl") => {
                if !current_mat.is_empty() {
                    groups.push((current_mat.clone(), std::mem::take(&mut current_faces)));
                }
                current_mat = parts.next().unwrap_or("").to_string();
            }
            _ => {}
        }
    }

    if !current_mat.is_empty() {
        groups.push((current_mat, current_faces));
    }

    Ok(ObjData { vertices, groups })
}

fn parse_mtl(mtl: &str) -> Result<MtlFile, String> {
    let mut materials: Vec<MtlData> = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_color = [1.0f32; 3];
    let mut current_alpha = 1.0f32;

    for line in mtl.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("newmtl") => {
                if let Some(name) = current_name.take() {
                    materials.push(MtlData {
                        name,
                        color: current_color,
                        alpha: current_alpha,
                    });
                }
                current_name = Some(parts.next().unwrap_or("").to_string());
                current_color = [1.0; 3];
                current_alpha = 1.0;
            }
            Some("Kd") => {
                let r: f32 = parts.next().unwrap().parse().map_err(|e| format!("{e}"))?;
                let g: f32 = parts.next().unwrap().parse().map_err(|e| format!("{e}"))?;
                let b: f32 = parts.next().unwrap().parse().map_err(|e| format!("{e}"))?;
                current_color = [r, g, b];
            }
            Some("d") => {
                current_alpha = parts.next().unwrap().parse().map_err(|e| format!("{e}"))?;
            }
            _ => {}
        }
    }

    if let Some(name) = current_name {
        materials.push(MtlData {
            name,
            color: current_color,
            alpha: current_alpha,
        });
    }

    Ok(MtlFile { materials })
}

fn opaque_config() -> (JSONAlphaComparison, JSONZMode, JSONBlendMode, bool, bool) {
    (
        JSONAlphaComparison {
            compLeft: Comparison::ALWAYS,
            refLeft: 0,
            op: AlphaOp::_and,
            compRight: Comparison::ALWAYS,
            refRight: 0,
        },
        JSONZMode {
            compare: true,
            function: Comparison::LEQUAL,
            update: true,
        },
        JSONBlendMode {
            type_: BlendModeType::none,
            source: BlendModeFactor::src_a,
            dest: BlendModeFactor::inv_src_a,
            logic: LogicOp::_copy,
        },
        false, // xlu
        true,  // earlyZComparison
    )
}

fn translucent_config() -> (JSONAlphaComparison, JSONZMode, JSONBlendMode, bool, bool) {
    (
        JSONAlphaComparison {
            compLeft: Comparison::ALWAYS,
            refLeft: 0,
            op: AlphaOp::_and,
            compRight: Comparison::ALWAYS,
            refRight: 0,
        },
        JSONZMode {
            compare: true,
            function: Comparison::LEQUAL,
            update: false,
        },
        JSONBlendMode {
            type_: BlendModeType::blend,
            source: BlendModeFactor::src_a,
            dest: BlendModeFactor::inv_src_a,
            logic: LogicOp::_copy,
        },
        true, // xlu
        true, // earlyZComparison
    )
}

// Builds a vertex data buffer encoding triangle primitives with position and color indices.
// Each face becomes a GX triangle primitive (0x90), with per-vertex index slots for
// position (slot 9) and color0 (slot 13) in a 26-slot `IndexedVertex`.
// https://docs.rs/brres/latest/brres/struct.MatrixPrimitive.html
fn build_vertex_data(faces: &[[usize; 3]]) -> Vec<u8> {
    let mut data = Vec::new();
    let vertex_count = (faces.len() * 3) as u32;
    data.push(0x02); // PrimitiveType::Triangles
    data.push(((vertex_count >> 16) & 0xFF) as u8);
    data.push(((vertex_count >> 8) & 0xFF) as u8);
    data.push((vertex_count & 0xFF) as u8);
    for (face_idx, _face) in faces.iter().enumerate() {
        for i in 0..3 {
            let local_idx = (face_idx * 3 + i) as u16;
            let mut indices = [0u16; 26];
            indices[9] = local_idx; // GX_VA_POS
            indices[10] = local_idx; // GX_VA_NRM
            indices[11] = local_idx; // GX_VA_CLR0
            for idx in indices {
                data.extend_from_slice(&idx.to_le_bytes());
            }
        }
    }
    data
}

// cross product of 2 edges, normalized
fn calculate_normal(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let n = cross(e1, e2);
    let len = dot(n, n).sqrt();
    if len == 0.0 {
        [0.0, 1.0, 0.0]
    } else {
        scale(n, 1.0 / len)
    }
}

pub fn from_obj_replace(brres: &mut Archive, obj: &str, mtl: &str) -> Result<(), String> {
    let obj_file = parse_obj(obj)?;
    let mtl_file = parse_mtl(mtl)?;

    // clone the stuff i dont need to edit
    let model = &brres.models[0];
    let model_name = model.name.clone();
    let model_info = model.info.clone();
    let model_bones = model.bones.clone();
    let model_material = model.materials.clone();
    let model_meshes = model.meshes.clone();
    let model_normals = model.normals.clone();
    let model_texcoords = model.texcoords.clone();
    let model_matrices = model.matrices.clone();

    let mut position_buffer: Vec<VertexPositionBuffer> = Vec::new();
    let mut normal_buffer: Vec<VertexNormalBuffer> = Vec::new();
    let mut material_buffer: Vec<JSONMaterial> = Vec::new();
    let mut color_buffer: Vec<VertexColorBuffer> = Vec::new();
    let mut mesh_buffer: Vec<Mesh> = Vec::new();

    for (i, (mat_name, faces)) in obj_file.groups.iter().enumerate() {
        // get mtl data from the file
        let mtl_data = mtl_file.materials.iter().find(|m| &m.name == mat_name);
        let is_alpha = mtl_data.is_some_and(|m| m.alpha < 1.0);
        let is_cp = mat_name.contains("ckpt");

        // POSITION

        let pos_name = format!("{mat_name}_pos");
        let nrm_name = format!("{mat_name}_nrm");
        let material_name = format!("{mat_name}_mat");
        let clr_name = format!("{mat_name}_clr");
        let mesh_name = format!("{mat_name}_mesh");

        let mut verts: Vec<[f32; 3]> = Vec::new();
        for face in faces {
            verts.push(obj_file.vertices[face[0]]);
            verts.push(obj_file.vertices[face[1]]);
            verts.push(obj_file.vertices[face[2]]);
        }

        // https://wiki.tockdom.com/wiki/MDL0_(File_Format)#Section_2_-_Vertices
        position_buffer.push(VertexPositionBuffer {
            id: i as i32,
            name: pos_name.clone(),
            q_comp: 1, // xyz
            q_type: 4, // float
            q_divisor: 0,
            q_stride: 12,
            data: verts,
            cached_minmax: None,
        });

        // NORMALS

        let mut normals: Vec<[f32; 3]> = Vec::new();
        for face in faces {
            let v0 = obj_file.vertices[face[0]];
            let v1 = obj_file.vertices[face[1]];
            let v2 = obj_file.vertices[face[2]];
            let normal = calculate_normal(v0, v1, v2);
            normals.push(normal);
            normals.push(normal);
            normals.push(normal);
        }

        // https://wiki.tockdom.com/wiki/MDL0_(File_Format)#Section_3_-_Normals
        normal_buffer.push(VertexNormalBuffer {
            id: i as i32,
            name: nrm_name.clone(),
            q_comp: 0, // "normal" normal (XYZ of a normal)
            q_type: 4, // float again ofc
            q_divisor: 0,
            q_stride: 12,
            data: normals,
            cached_minmax: None,
        });

        // COLORS

        // get kd + d from mtl, then multiply for binary format
        let color = mtl_data.map_or([1.0f32; 3], |m| m.color);
        let alpha = mtl_data.map_or(1.0f32, |m| m.alpha);
        let rgba: [u32; 4] = [
            (color[0] * 255.0) as u32,
            (color[1] * 255.0) as u32,
            (color[2] * 255.0) as u32,
            (alpha * 255.0) as u32,
        ];
        let mut colors: Vec<[u32; 4]> = Vec::new();
        for _ in faces {
            colors.push(rgba);
            colors.push(rgba);
            colors.push(rgba);
        }
        color_buffer.push(VertexColorBuffer {
            id: i as i32,
            name: clr_name.clone(),
            q_comp: 1,
            q_divisor: 0,
            q_stride: 4,
            q_type: 5,
            data: colors,
            cached_minmax: None,
        });

        // MATERIALS

        //tev settings
        let mut tev_stage = model_material[0].mStages[0].clone();
        tev_stage.colorStage.a = TevColorArg::rasc;
        tev_stage.colorStage.b = TevColorArg::zero;
        tev_stage.colorStage.c = TevColorArg::zero;
        tev_stage.colorStage.d = TevColorArg::zero;
        tev_stage.colorStage.scale = TevScale::scale_1;

        tev_stage.alphaStage.a = TevAlphaArg::rasa;
        tev_stage.alphaStage.b = TevAlphaArg::zero;
        tev_stage.alphaStage.c = TevAlphaArg::zero;
        tev_stage.alphaStage.d = TevAlphaArg::zero;

        // opaue - alpha settings
        let (alpha_compare, z_mode, blend_mode, xlu, early_z) = if is_alpha {
            translucent_config()
        } else {
            opaque_config()
        };

        material_buffer.push(JSONMaterial {
            flag: 0,
            id: i as u32,
            name: material_name,
            fogIndex: 0,
            lightSetIndex: 0,
            chanData: vec![
                // color chan
                JSONChannelData {
                    matColor: Color {
                        r: 255,
                        g: 255,
                        b: 255,
                        a: 255,
                    },
                    ambColor: Color {
                        r: 255,
                        g: 255,
                        b: 255,
                        a: 255,
                    },
                },
                // alpha chan
                JSONChannelData {
                    matColor: Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                    ambColor: Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                },
            ],
            colorChanControls: vec![
                JSONChannelControl {
                    enabled: true,
                    Ambient: ColorSource::Register,
                    // i use vertex colors
                    Material: ColorSource::Vertex,
                    lightMask: LightID::None,
                    diffuseFn: DiffuseFunction::Clamp,
                    attenuationFn: AttenuationFunction::Spotlight,
                },
                JSONChannelControl {
                    enabled: true,
                    Ambient: ColorSource::Register,
                    Material: ColorSource::Vertex,
                    lightMask: LightID::None,
                    diffuseFn: DiffuseFunction::Clamp,
                    attenuationFn: AttenuationFunction::Spotlight,
                },
            ],
            texMatrices: vec![],
            samplers: vec![],
            texGens: vec![],
            alphaCompare: alpha_compare,
            zMode: z_mode,
            blendMode: blend_mode,
            xlu,
            earlyZComparison: early_z,
            cullMode: {
                if is_cp {
                    CullMode::None
                } else {
                    CullMode::Front
                }
            },
            mStages: vec![tev_stage],
            // put the other stuff
            ..model_material[0].clone()
        });

        // MESHES

        mesh_buffer.push(Mesh {
            name: mesh_name,
            visible: true,
            pos_buffer: pos_name,
            clr_buffer: vec![clr_name, String::new()],
            nrm_buffer: nrm_name,
            uv_buffer: vec![
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ],
            // position, normals and color0 https://github.com/devkitPro/libogc/blob/master/gc/ogc/gx.h
            vcd: (1 << 9) | (1 << 10) | (1 << 11),
            current_matrix: 0,
            mprims: vec![MatrixPrimitive {
                matrices: vec![],
                num_prims: 1,
                vertex_data_buffer: build_vertex_data(faces),
            }],
        });
    }

    brres.models[0].materials = material_buffer;
    brres.models[0].meshes = mesh_buffer;
    brres.models[0].positions = position_buffer;
    brres.models[0].colors = color_buffer;
    brres.models[0].normals = normal_buffer;
    brres.models[0].texcoords = vec![];
    // for each group, update the draw polygon call (needed for the bone to reference new data) with the new data
    brres.models[0].bones[0].draw_calls = (0..obj_file.groups.len())
        .map(|i| JSONDrawCall {
            material: i as u32,
            poly: i as u32,
            prio: 0,
        })
        .collect();

    Ok(())
}

pub fn from_obj_overlay(brres: &mut Archive, obj: &str, mtl: &str) -> Result<(), String> {
    let obj_file = parse_obj(obj)?;
    let mtl_file = parse_mtl(mtl)?;

    let model = &brres.models[0];
    let model_name = model.name.clone();
    let model_info = model.info.clone();
    let model_texcoords = model.texcoords.clone();
    let model_matrices = model.matrices.clone();
    let mut position_buffer: Vec<VertexPositionBuffer> = model.positions.clone();
    let mut normal_buffer: Vec<VertexNormalBuffer> = model.normals.clone();
    let mut material_buffer: Vec<JSONMaterial> = model.materials.clone();
    let mut color_buffer: Vec<VertexColorBuffer> = model.colors.clone();
    let mut mesh_buffer: Vec<Mesh> = model.meshes.clone();

    let mut draw_calls = brres.models[0].bones[0].draw_calls.clone();
    let first_mat_len = material_buffer.len();
    let first_mesh_len = mesh_buffer.len();
    let first_pos_len = position_buffer.len();
    let first_nrm_len = normal_buffer.len();
    let first_clr_len = color_buffer.len();

    for (i, (mat_name, faces)) in obj_file.groups.iter().enumerate() {
        // get mtl data from the file
        let mtl_data = mtl_file.materials.iter().find(|m| &m.name == mat_name);
        let is_alpha = mtl_data.is_some_and(|m| m.alpha < 1.0);
        let is_cp = mat_name.contains("ckpt");

        // POSITIONS
        let pos_name = format!("{mat_name}_pos");
        let nrm_name = format!("{mat_name}_nrm");
        let material_name = format!("{mat_name}_mat");
        let clr_name = format!("{mat_name}_clr");
        let mesh_name = format!("{mat_name}_mesh");

        let mut verts: Vec<[f32; 3]> = Vec::new();
        for face in faces {
            verts.push(obj_file.vertices[face[0]]);
            verts.push(obj_file.vertices[face[1]]);
            verts.push(obj_file.vertices[face[2]]);
        }

        // https://wiki.tockdom.com/wiki/MDL0_(File_Format)#Section_2_-_Vertices
        position_buffer.push(VertexPositionBuffer {
            id: (first_pos_len + i) as i32,
            name: pos_name.clone(),
            q_comp: 1, // xyz
            q_type: 4, // float
            q_divisor: 0,
            q_stride: 12,
            data: verts,
            cached_minmax: None,
        });

        // NORMALS

        let mut normals: Vec<[f32; 3]> = Vec::new();
        for face in faces {
            let v0 = obj_file.vertices[face[0]];
            let v1 = obj_file.vertices[face[1]];
            let v2 = obj_file.vertices[face[2]];
            let normal = calculate_normal(v0, v1, v2);
            normals.push(normal);
            normals.push(normal);
            normals.push(normal);
        }

        // https://wiki.tockdom.com/wiki/MDL0_(File_Format)#Section_3_-_Normals
        normal_buffer.push(VertexNormalBuffer {
            id: (first_nrm_len + i) as i32,
            name: nrm_name.clone(),
            q_comp: 0, // "normal" normal (XYZ of a normal)
            q_type: 4, // float again ofc
            q_divisor: 0,
            q_stride: 12,
            data: normals,
            cached_minmax: None,
        });

        // COLORS

        // get kd + d from mtl, then multiply for binary format
        let color = mtl_data.map_or([1.0f32; 3], |m| m.color);
        let alpha = mtl_data.map_or(1.0f32, |m| m.alpha);
        let rgba: [u32; 4] = [
            (color[0] * 255.0) as u32,
            (color[1] * 255.0) as u32,
            (color[2] * 255.0) as u32,
            (alpha * 255.0) as u32,
        ];
        let mut colors: Vec<[u32; 4]> = Vec::new();
        for _ in faces {
            colors.push(rgba);
            colors.push(rgba);
            colors.push(rgba);
        }
        color_buffer.push(VertexColorBuffer {
            id: (first_clr_len + i) as i32,
            name: clr_name.clone(),
            q_comp: 1,
            q_divisor: 0,
            q_stride: 4,
            q_type: 5,
            data: colors,
            cached_minmax: None,
        });

        // MATERIALS

        //tev settings
        let mut tev_stage = material_buffer[0].mStages[0].clone();
        tev_stage.colorStage.a = TevColorArg::rasc;
        tev_stage.colorStage.b = TevColorArg::zero;
        tev_stage.colorStage.c = TevColorArg::zero;
        tev_stage.colorStage.d = TevColorArg::zero;
        tev_stage.colorStage.scale = TevScale::scale_1;

        tev_stage.alphaStage.a = TevAlphaArg::rasa;
        tev_stage.alphaStage.b = TevAlphaArg::zero;
        tev_stage.alphaStage.c = TevAlphaArg::zero;
        tev_stage.alphaStage.d = TevAlphaArg::zero;

        // opaue - alpha settings
        let (alpha_compare, z_mode, blend_mode, xlu, early_z) = if is_alpha {
            translucent_config()
        } else {
            opaque_config()
        };

        material_buffer.push(JSONMaterial {
            flag: 0,
            id: (first_mat_len + i) as u32,
            name: material_name,
            fogIndex: 0,
            lightSetIndex: 0,
            chanData: vec![
                // color chan
                JSONChannelData {
                    matColor: Color {
                        r: 255,
                        g: 255,
                        b: 255,
                        a: 255,
                    },
                    ambColor: Color {
                        r: 255,
                        g: 255,
                        b: 255,
                        a: 255,
                    },
                },
                // alpha chan
                JSONChannelData {
                    matColor: Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                    ambColor: Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                },
            ],
            colorChanControls: vec![
                JSONChannelControl {
                    enabled: true,
                    Ambient: ColorSource::Register,
                    // i use vertex colors
                    Material: ColorSource::Vertex,
                    lightMask: LightID::None,
                    diffuseFn: DiffuseFunction::Clamp,
                    attenuationFn: AttenuationFunction::Spotlight,
                },
                JSONChannelControl {
                    enabled: true,
                    Ambient: ColorSource::Register,
                    Material: ColorSource::Vertex,
                    lightMask: LightID::None,
                    diffuseFn: DiffuseFunction::Clamp,
                    attenuationFn: AttenuationFunction::Spotlight,
                },
            ],
            texMatrices: vec![],
            samplers: vec![],
            texGens: vec![],
            alphaCompare: alpha_compare,
            zMode: z_mode,
            blendMode: blend_mode,
            xlu,
            earlyZComparison: early_z,
            cullMode: {
                if is_cp {
                    CullMode::None
                } else {
                    CullMode::Front
                }
            },
            mStages: vec![tev_stage],
            // put the other stuff
            ..material_buffer[0].clone()
        });

        // MESHES

        mesh_buffer.push(Mesh {
            name: mesh_name,
            visible: true,
            pos_buffer: pos_name,
            clr_buffer: vec![clr_name, String::new()],
            nrm_buffer: nrm_name,
            uv_buffer: vec![
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ],
            // position, normals and color0 https://github.com/devkitPro/libogc/blob/master/gc/ogc/gx.h
            vcd: (1 << 9) | (1 << 10) | (1 << 11),
            current_matrix: 0,
            mprims: vec![MatrixPrimitive {
                matrices: vec![],
                num_prims: 1,
                vertex_data_buffer: build_vertex_data(faces),
            }],
        });
    }

    let new_draw_calls: Vec<_> = (0..obj_file.groups.len())
        .map(|i| JSONDrawCall {
            material: (first_mat_len + i) as u32,
            poly: (first_mesh_len + i) as u32,
            prio: 0,
        })
        .collect();

    draw_calls.extend(new_draw_calls);
    brres.models[0].bones[0].draw_calls = draw_calls;
    brres.models[0].materials = material_buffer;
    brres.models[0].meshes = mesh_buffer;
    brres.models[0].positions = position_buffer;
    brres.models[0].colors = color_buffer;
    brres.models[0].normals = normal_buffer;

    Ok(())
}

pub fn parse(data: &[u8]) -> anyhow::Result<Archive> {
    brres::Archive::from_memory(data)
}

pub fn parse_from_path(path: &str) -> Result<Archive, String> {
    let data = super::binary::path_to_data(path, "brres")?;
    parse(&data).map_err(|e| e.to_string())
}
