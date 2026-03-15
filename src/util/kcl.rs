#![allow(dead_code)]
use crate::Object;

use super::binary::*;
use std::collections::HashMap;

// https://wiki.tockdom.com/wiki/KCL_(File_Format)

pub struct Header {
    pub pos_data_offset: u32,
    pub nrm_data_offset: u32,
    pub prism_data_offset: u32,
    pub block_data_offset: u32,
    pub prism_thickness: f32,
    pub area_min_pos: [f32; 3],
    pub area_x_width_mask: u32,
    pub area_y_width_mask: u32,
    pub area_z_width_mask: u32,
    pub block_width_shift: u32,
    pub area_x_blocks_shift: u32,
    pub area_xy_blocks_shift: u32,
    pub sphere_radius: Option<f32>,
}

#[derive(Debug)]
pub enum Flags {
    Road,
    SlipperyRoad1,
    WeakOffroad,
    Offroad,
    HeavyOffroad,
    SlipperyRoad2,
    BoostPanel,
    BoostRamp,
    JumpPad,
    ItemRoad,
    SolidFall,
    MovingWater,
    Wall,
    InvisibleWall,
    ItemWall,
    Wall2,
    FallBoundary,
    CannonTrigger,
    ForceRecalculation,
    HalfPipeRamp,
    PlayerOnlyWall,
    MovingRoad,
    StickyRoad,
    Road2,
    SoundTrigger,
    WeakWall,
    EffectTrigger,
    ItemStateModifier,
    HalfPipeInvisibleWall,
    RotatingRoad,
    SpecialWall,
    InvisibleWall2,
}

pub struct Prism {
    pub height: f32,
    pub pos_i: u16,
    pub fnrm_i: u16,
    pub enrm1_i: u16,
    pub enrm2_i: u16,
    pub enrm3_i: u16,
    pub flags: Flags,
}

pub struct Sections {
    pub position_vectors: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub prisms: Vec<Prism>,
    pub spatial_index: Vec<u32>,
}

pub struct ParsedKcl {
    pub header: Header,
    pub sections: Sections,
}

impl Header {
    fn parse(data: &[u8]) -> Result<Self, String> {
        let pos_data_offset = read_u32(data, 0x00)?;
        let nrm_data_offset = read_u32(data, 0x04)?;
        let prism_data_offset = read_u32(data, 0x08)?;
        let block_data_offset = read_u32(data, 0x0C)?;
        let prism_thickness = read_f32(data, 0x10)?;
        let area_min_pos = [
            read_f32(data, 0x14)?,
            read_f32(data, 0x18)?,
            read_f32(data, 0x1C)?,
        ];
        let area_x_width_mask = read_u32(data, 0x20)?;
        let area_y_width_mask = read_u32(data, 0x24)?;
        let area_z_width_mask = read_u32(data, 0x28)?;
        let block_width_shift = read_u32(data, 0x2C)?;
        let area_x_blocks_shift = read_u32(data, 0x30)?;
        let area_xy_blocks_shift = read_u32(data, 0x34)?;
        let sphere_radius = if data.len() >= 0x38 + 4 {
            Some(read_f32(data, 0x38)?)
        } else {
            None
        };

        Ok(Header {
            pos_data_offset,
            nrm_data_offset,
            prism_data_offset,
            block_data_offset,
            prism_thickness,
            area_min_pos,
            area_x_width_mask,
            area_y_width_mask,
            area_z_width_mask,
            block_width_shift,
            area_x_blocks_shift,
            area_xy_blocks_shift,
            sphere_radius,
        })
    }
}

impl Flags {
    fn discriminant(&self) -> u8 {
        match self {
            Flags::Road => 0x00,
            Flags::SlipperyRoad1 => 0x01,
            Flags::WeakOffroad => 0x02,
            Flags::Offroad => 0x03,
            Flags::HeavyOffroad => 0x04,
            Flags::SlipperyRoad2 => 0x05,
            Flags::BoostPanel => 0x06,
            Flags::BoostRamp => 0x07,
            Flags::JumpPad => 0x08,
            Flags::ItemRoad => 0x09,
            Flags::SolidFall => 0x0A,
            Flags::MovingWater => 0x0B,
            Flags::Wall => 0x0C,
            Flags::InvisibleWall => 0x0D,
            Flags::ItemWall => 0x0E,
            Flags::Wall2 => 0x0F,
            Flags::FallBoundary => 0x10,
            Flags::CannonTrigger => 0x11,
            Flags::ForceRecalculation => 0x12,
            Flags::HalfPipeRamp => 0x13,
            Flags::PlayerOnlyWall => 0x14,
            Flags::MovingRoad => 0x15,
            Flags::StickyRoad => 0x16,
            Flags::Road2 => 0x17,
            Flags::SoundTrigger => 0x18,
            Flags::WeakWall => 0x19,
            Flags::EffectTrigger => 0x1A,
            Flags::ItemStateModifier => 0x1B,
            Flags::HalfPipeInvisibleWall => 0x1C,
            Flags::RotatingRoad => 0x1D,
            Flags::SpecialWall => 0x1E,
            Flags::InvisibleWall2 => 0x1F,
        }
    }
    fn from_u16(value: u16) -> Result<Self, String> {
        match value & 0x1F {
            0x00 => Ok(Flags::Road),
            0x01 => Ok(Flags::SlipperyRoad1),
            0x02 => Ok(Flags::WeakOffroad),
            0x03 => Ok(Flags::Offroad),
            0x04 => Ok(Flags::HeavyOffroad),
            0x05 => Ok(Flags::SlipperyRoad2),
            0x06 => Ok(Flags::BoostPanel),
            0x07 => Ok(Flags::BoostRamp),
            0x08 => Ok(Flags::JumpPad),
            0x09 => Ok(Flags::ItemRoad),
            0x0A => Ok(Flags::SolidFall),
            0x0B => Ok(Flags::MovingWater),
            0x0C => Ok(Flags::Wall),
            0x0D => Ok(Flags::InvisibleWall),
            0x0E => Ok(Flags::ItemWall),
            0x0F => Ok(Flags::Wall2),
            0x10 => Ok(Flags::FallBoundary),
            0x11 => Ok(Flags::CannonTrigger),
            0x12 => Ok(Flags::ForceRecalculation),
            0x13 => Ok(Flags::HalfPipeRamp),
            0x14 => Ok(Flags::PlayerOnlyWall),
            0x15 => Ok(Flags::MovingRoad),
            0x16 => Ok(Flags::StickyRoad),
            0x17 => Ok(Flags::Road2),
            0x18 => Ok(Flags::SoundTrigger),
            0x19 => Ok(Flags::WeakWall),
            0x1A => Ok(Flags::EffectTrigger),
            0x1B => Ok(Flags::ItemStateModifier),
            0x1C => Ok(Flags::HalfPipeInvisibleWall),
            0x1D => Ok(Flags::RotatingRoad),
            0x1E => Ok(Flags::SpecialWall),
            0x1F => Ok(Flags::InvisibleWall2),
            _ => Err(format!("Unknown KCL flag: {}", value)),
        }
    }

    // lorenzi's colors
    fn color(&self) -> [u8; 4] {
        let (r, g, b, a) = match self {
            Flags::Road => (255, 255, 255, 255),
            Flags::SlipperyRoad1 => (255, 230, 204, 255),
            Flags::WeakOffroad => (0, 204, 0, 255),
            Flags::Offroad => (0, 153, 0, 255),
            Flags::HeavyOffroad => (0, 102, 0, 255),
            Flags::SlipperyRoad2 => (204, 230, 255, 255),
            Flags::BoostPanel => (255, 128, 0, 255),
            Flags::BoostRamp => (255, 153, 0, 255),
            Flags::JumpPad => (255, 204, 0, 255),
            Flags::ItemRoad => (230, 230, 255, 255),
            Flags::SolidFall => (179, 26, 26, 255),
            Flags::MovingWater => (0, 128, 255, 255),
            Flags::Wall => (153, 153, 153, 255),
            Flags::InvisibleWall => (0, 0, 153, 100),
            Flags::ItemWall => (153, 153, 179, 255),
            Flags::Wall2 => (153, 153, 153, 255),
            Flags::FallBoundary => (204, 0, 0, 255),
            Flags::CannonTrigger => (255, 0, 128, 255),
            Flags::ForceRecalculation => (128, 0, 255, 50),
            Flags::HalfPipeRamp => (0, 77, 255, 255),
            Flags::PlayerOnlyWall => (204, 102, 0, 255),
            Flags::MovingRoad => (230, 230, 255, 255),
            Flags::StickyRoad => (230, 179, 255, 255),
            Flags::Road2 => (255, 255, 255, 255),
            Flags::SoundTrigger => (255, 0, 255, 50),
            Flags::WeakWall => (102, 153, 102, 255),
            Flags::EffectTrigger => (204, 0, 255, 50),
            Flags::ItemStateModifier => (255, 0, 255, 50),
            Flags::HalfPipeInvisibleWall => (0, 153, 0, 100),
            Flags::RotatingRoad => (230, 230, 255, 255),
            Flags::SpecialWall => (204, 179, 204, 255),
            Flags::InvisibleWall2 => (0, 0, 153, 100),
        };
        [r, g, b, a]
    }

    fn name(&self) -> &'static str {
        match self {
            Flags::Road => "road",
            Flags::SlipperyRoad1 => "slippery_road1",
            Flags::WeakOffroad => "weak_offroad",
            Flags::Offroad => "offroad",
            Flags::HeavyOffroad => "heavy_offroad",
            Flags::SlipperyRoad2 => "slippery_road2",
            Flags::BoostPanel => "boost_panel",
            Flags::BoostRamp => "boost_ramp",
            Flags::JumpPad => "jump_pad",
            Flags::ItemRoad => "item_road",
            Flags::SolidFall => "solid_fall",
            Flags::MovingWater => "moving_water",
            Flags::Wall => "wall",
            Flags::InvisibleWall => "invisible_wall",
            Flags::ItemWall => "item_wall",
            Flags::Wall2 => "wall2",
            Flags::FallBoundary => "fall_boundary",
            Flags::CannonTrigger => "cannon_trigger",
            Flags::ForceRecalculation => "force_recalc",
            Flags::HalfPipeRamp => "halfpipe_ramp",
            Flags::PlayerOnlyWall => "player_wall",
            Flags::MovingRoad => "moving_road",
            Flags::StickyRoad => "sticky_road",
            Flags::Road2 => "road2",
            Flags::SoundTrigger => "sound_trigger",
            Flags::WeakWall => "weak_wall",
            Flags::EffectTrigger => "effect_trigger",
            Flags::ItemStateModifier => "item_state",
            Flags::HalfPipeInvisibleWall => "halfpipe_invis",
            Flags::RotatingRoad => "rotating_road",
            Flags::SpecialWall => "special_wall",
            Flags::InvisibleWall2 => "invisible_wall2",
        }
    }
}

impl Prism {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let height = read_f32(data, offset)?;
        let pos_i = read_u16(data, offset + 4)?;
        let fnrm_i = read_u16(data, offset + 6)?;
        let enrm1_i = read_u16(data, offset + 8)?;
        let enrm2_i = read_u16(data, offset + 10)?;
        let enrm3_i = read_u16(data, offset + 12)?;
        let flags = Flags::from_u16(read_u16(data, offset + 14)?)?;

        Ok(Prism {
            height,
            pos_i,
            fnrm_i,
            enrm1_i,
            enrm2_i,
            enrm3_i,
            flags,
        })
    }
}

impl Sections {
    fn parse(data: &[u8], header: &Header) -> Result<Self, String> {
        // prisms first so we can determine sizes of pos/nrm arrays
        let real_prism_offset = header.prism_data_offset as usize + 0x10;
        let max_prisms = (header.block_data_offset as usize - real_prism_offset) / 0x10;
        let mut prisms = Vec::with_capacity(max_prisms);
        for i in 0..max_prisms {
            prisms.push(Prism::parse(data, real_prism_offset + i * 0x10)?);
        }

        let max_pos = prisms.iter().map(|p| p.pos_i).max().unwrap_or(0) as usize + 1;
        let max_nrm = prisms
            .iter()
            .map(|p| p.fnrm_i.max(p.enrm1_i).max(p.enrm2_i).max(p.enrm3_i))
            .max()
            .unwrap_or(0) as usize
            + 1;

        let position_vectors = read_vec_f32(data, header.pos_data_offset as usize, max_pos * 3)?
            .chunks(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();

        let normals = read_vec_f32(data, header.nrm_data_offset as usize, max_nrm * 3)?
            .chunks(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();

        let block_start = header.block_data_offset as usize;
        let spatial_index = read_vec_u32(data, block_start, (data.len() - block_start) / 4)?;

        Ok(Sections {
            position_vectors,
            normals,
            prisms,
            spatial_index,
        })
    }
}

fn prism_to_triangle(prism: &Prism, positions: &[[f32; 3]], normals: &[[f32; 3]]) -> [[f32; 3]; 3] {
    let v1 = positions[prism.pos_i as usize];
    let fnrm = normals[prism.fnrm_i as usize];
    let enrm1 = normals[prism.enrm1_i as usize];
    let enrm2 = normals[prism.enrm2_i as usize];
    let enrm3 = normals[prism.enrm3_i as usize];

    let cross_a = cross(enrm1, fnrm);
    let cross_b = cross(enrm2, fnrm);
    let v2 = add(v1, scale(cross_b, prism.height / dot(cross_b, enrm3)));
    let v3 = add(v1, scale(cross_a, prism.height / dot(cross_a, enrm3)));

    [v1, v2, v3]
}

pub fn to_obj(parsed: &ParsedKcl, name: &str) -> Object {
    let mut obj = String::new();
    let mut mtl = String::new();

    obj.push_str("# Generated by VisibleMKW\n");
    obj.push_str(&format!("mtllib {name}.mtl\n\n"));

    let pos_buf = &parsed.sections.position_vectors;
    let nrm_buf = &parsed.sections.normals;

    // map groups by kcl flags
    let mut groups: HashMap<u8, Vec<usize>> = HashMap::new();
    for (i, prism) in parsed.sections.prisms.iter().enumerate() {
        groups.entry(prism.flags.discriminant()).or_default().push(i);
    }

    // OBJ is 1-based
    let mut vertex_offset = 1usize;

    // additionally sort the group hashmap by hex value
    let mut sorted: Vec<(u8, Vec<usize>)> = groups.into_iter().collect();
    sorted.sort_by_key(|(k, _)| *k);

    for (_key, prism_indices) in &sorted {
        let flag = &parsed.sections.prisms[prism_indices[0]].flags;
        let name  = flag.name();
        let color = flag.color();

        // Write MTL entry for this flag (RGB 0.0-1.0).
        mtl.push_str(&format!("newmtl {}\n", name));
        mtl.push_str(&format!(
            "Kd {:.4} {:.4} {:.4}\nd {:.4}\n\n",
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
            color[3] as f32 / 255.0,
        ));

        // Write OBJ group + material reference.
        obj.push_str(&format!("g {}\n", name));
        obj.push_str(&format!("usemtl {}\n", name));

        let mut faces: Vec<[usize; 3]> = Vec::with_capacity(prism_indices.len());

        for &pi in prism_indices {
            let [v1, v2, v3] = prism_to_triangle(&parsed.sections.prisms[pi], pos_buf, nrm_buf);
            obj.push_str(&format!("v {} {} {}\n", v1[0], v1[1], v1[2]));
            obj.push_str(&format!("v {} {} {}\n", v2[0], v2[1], v2[2]));
            obj.push_str(&format!("v {} {} {}\n", v3[0], v3[1], v3[2]));
            let base = vertex_offset;
            vertex_offset += 3;
            faces.push([base, base + 1, base + 2]);
        }

        for f in &faces {
            obj.push_str(&format!("f {} {} {}\n", f[0], f[1], f[2]));
        }

        obj.push('\n');
    }

    Object {
        obj,
        mtl,
    }
}

pub fn parse(data: &[u8]) -> Result<ParsedKcl, String> {
    let header = Header::parse(data)?;
    let sections = Sections::parse(data, &header)?;
    Ok(ParsedKcl { header, sections })
}

pub fn parse_from_path(path: &str) -> Result<ParsedKcl, String> {
    let data = path_to_data(path, "kcl")?;
    parse(&data)
}
