#![allow(dead_code)]
use crate::{HighlightOption, KmpOption, Object, SpecialPlanesOption, util::kmp::{CheckPointType, ParsedKmp}};
use super::binary::*;
use super::kmp::add_checkpoint;
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



#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum BaseType {
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

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Variant {
    Value0 = 0,
    Value1 = 1,
    Value2 = 2,
    Value3 = 3,
    Value4 = 4,
    Value5 = 5,
    Value6 = 6,
    Value7 = 7,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Blight {
    Value0 = 0,
    Value1 = 1,
    Value2 = 2,
    Value3 = 3,
    Value4 = 4,
    Value5 = 5,
    Value6 = 6,
    Value7 = 7,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum WheelDepth {
    Value0 = 0,
    Value1 = 1,
    Value2 = 2,
    Value3 = 3,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct CollisionEffect {
    pub trickable: bool,
    pub reject_road: bool,
    pub soft_wall: bool,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct Flag {
    pub base_type: BaseType,
    pub variant: Variant,
    pub blight: Blight,
    pub wheel_depth: WheelDepth,
    pub collision_effect: CollisionEffect,
}

pub struct Prism {
    pub height: f32,
    pub pos_i: u16,
    pub fnrm_i: u16,
    pub enrm1_i: u16,
    pub enrm2_i: u16,
    pub enrm3_i: u16,
    pub flag: Flag,
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

impl BaseType {
    fn discriminant(&self) -> u8 {
        match self {
            Self::Road => 0x00,
            Self::SlipperyRoad1 => 0x01,
            Self::WeakOffroad => 0x02,
            Self::Offroad => 0x03,
            Self::HeavyOffroad => 0x04,
            Self::SlipperyRoad2 => 0x05,
            Self::BoostPanel => 0x06,
            Self::BoostRamp => 0x07,
            Self::JumpPad => 0x08,
            Self::ItemRoad => 0x09,
            Self::SolidFall => 0x0A,
            Self::MovingWater => 0x0B,
            Self::Wall => 0x0C,
            Self::InvisibleWall => 0x0D,
            Self::ItemWall => 0x0E,
            Self::Wall2 => 0x0F,
            Self::FallBoundary => 0x10,
            Self::CannonTrigger => 0x11,
            Self::ForceRecalculation => 0x12,
            Self::HalfPipeRamp => 0x13,
            Self::PlayerOnlyWall => 0x14,
            Self::MovingRoad => 0x15,
            Self::StickyRoad => 0x16,
            Self::Road2 => 0x17,
            Self::SoundTrigger => 0x18,
            Self::WeakWall => 0x19,
            Self::EffectTrigger => 0x1A,
            Self::ItemStateModifier => 0x1B,
            Self::HalfPipeInvisibleWall => 0x1C,
            Self::RotatingRoad => 0x1D,
            Self::SpecialWall => 0x1E,
            Self::InvisibleWall2 => 0x1F,
        }
    }
    fn from_u16(value: u16) -> Result<Self, String> {
        match value & 0x1F {
            0x00 => Ok(Self::Road),
            0x01 => Ok(Self::SlipperyRoad1),
            0x02 => Ok(Self::WeakOffroad),
            0x03 => Ok(Self::Offroad),
            0x04 => Ok(Self::HeavyOffroad),
            0x05 => Ok(Self::SlipperyRoad2),
            0x06 => Ok(Self::BoostPanel),
            0x07 => Ok(Self::BoostRamp),
            0x08 => Ok(Self::JumpPad),
            0x09 => Ok(Self::ItemRoad),
            0x0A => Ok(Self::SolidFall),
            0x0B => Ok(Self::MovingWater),
            0x0C => Ok(Self::Wall),
            0x0D => Ok(Self::InvisibleWall),
            0x0E => Ok(Self::ItemWall),
            0x0F => Ok(Self::Wall2),
            0x10 => Ok(Self::FallBoundary),
            0x11 => Ok(Self::CannonTrigger),
            0x12 => Ok(Self::ForceRecalculation),
            0x13 => Ok(Self::HalfPipeRamp),
            0x14 => Ok(Self::PlayerOnlyWall),
            0x15 => Ok(Self::MovingRoad),
            0x16 => Ok(Self::StickyRoad),
            0x17 => Ok(Self::Road2),
            0x18 => Ok(Self::SoundTrigger),
            0x19 => Ok(Self::WeakWall),
            0x1A => Ok(Self::EffectTrigger),
            0x1B => Ok(Self::ItemStateModifier),
            0x1C => Ok(Self::HalfPipeInvisibleWall),
            0x1D => Ok(Self::RotatingRoad),
            0x1E => Ok(Self::SpecialWall),
            0x1F => Ok(Self::InvisibleWall2),
            _ => Err(format!("Unknown KCL flag: {}", value)),
        }
    }

    // lorenzi's colors
    pub fn color(&self) -> [u8; 4] {
        let (r, g, b, a) = match self {
            Self::Road => (255, 255, 255, 255),
            Self::SlipperyRoad1 => (255, 230, 204, 255),
            Self::WeakOffroad => (0, 204, 0, 255),
            Self::Offroad => (0, 153, 0, 255),
            Self::HeavyOffroad => (0, 102, 0, 255),
            Self::SlipperyRoad2 => (204, 230, 255, 255),
            Self::BoostPanel => (255, 128, 0, 255),
            Self::BoostRamp => (255, 153, 0, 255),
            Self::JumpPad => (255, 204, 0, 255),
            Self::ItemRoad => (230, 230, 255, 255),
            Self::SolidFall => (179, 26, 26, 255),
            Self::MovingWater => (0, 128, 255, 255),
            Self::Wall => (153, 153, 153, 255),
            Self::InvisibleWall => (0, 0, 153, 200),
            Self::ItemWall => (153, 153, 179, 255),
            Self::Wall2 => (153, 153, 153, 255),
            Self::FallBoundary => (204, 0, 0, 255),
            Self::CannonTrigger => (255, 0, 128, 255),
            Self::ForceRecalculation => (128, 0, 255, 50),
            Self::HalfPipeRamp => (0, 77, 255, 255),
            Self::PlayerOnlyWall => (204, 102, 0, 255),
            Self::MovingRoad => (230, 230, 255, 255),
            Self::StickyRoad => (230, 179, 255, 255),
            Self::Road2 => (255, 255, 255, 255),
            Self::SoundTrigger => (255, 0, 255, 50),
            Self::WeakWall => (102, 153, 102, 255),
            Self::EffectTrigger => (204, 0, 255, 50),
            Self::ItemStateModifier => (255, 0, 255, 50),
            Self::HalfPipeInvisibleWall => (0, 153, 0, 200),
            Self::RotatingRoad => (230, 230, 255, 255),
            Self::SpecialWall => (204, 179, 204, 255),
            Self::InvisibleWall2 => (0, 0, 153, 200),
        };
        [r, g, b, a]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Road => "road",
            Self::SlipperyRoad1 => "slippery_road1",
            Self::WeakOffroad => "weak_offroad",
            Self::Offroad => "offroad",
            Self::HeavyOffroad => "heavy_offroad",
            Self::SlipperyRoad2 => "slippery_road2",
            Self::BoostPanel => "boost_panel",
            Self::BoostRamp => "boost_ramp",
            Self::JumpPad => "jump_pad",
            Self::ItemRoad => "item_road",
            Self::SolidFall => "solid_fall",
            Self::MovingWater => "moving_water",
            Self::Wall => "wall",
            Self::InvisibleWall => "invisible_wall",
            Self::ItemWall => "item_wall",
            Self::Wall2 => "wall2",
            Self::FallBoundary => "fall_boundary",
            Self::CannonTrigger => "cannon_trigger",
            Self::ForceRecalculation => "force_recalc",
            Self::HalfPipeRamp => "halfpipe_ramp",
            Self::PlayerOnlyWall => "player_wall",
            Self::MovingRoad => "moving_road",
            Self::StickyRoad => "sticky_road",
            Self::Road2 => "road2",
            Self::SoundTrigger => "sound_trigger",
            Self::WeakWall => "weak_wall",
            Self::EffectTrigger => "effect_trigger",
            Self::ItemStateModifier => "item_state",
            Self::HalfPipeInvisibleWall => "halfpipe_invis",
            Self::RotatingRoad => "rotating_road",
            Self::SpecialWall => "special_wall",
            Self::InvisibleWall2 => "invisible_wall2",
        }
    }
}

impl Flag {
    pub fn from_u16(value: u16) -> Result<Self, String> {
        let base_type = BaseType::from_u16(value)?;
        let variant = Variant::from_u16((value >> 5) & 0x7)?;
        let blight = Blight::from_u16((value >> 8) & 0x7)?;
        let wheel_depth = WheelDepth::from_u16((value >> 11) & 0x3)?;
        let collision_effect = CollisionEffect::from_u16(value);
        Ok(Flag { base_type, variant, blight, wheel_depth, collision_effect })
    }

    pub fn to_u16(&self) -> u16 {
        (self.base_type.discriminant() as u16)
            | ((self.variant as u16) << 5)
            | ((self.blight as u16) << 8)
            | ((self.wheel_depth as u16) << 11)
            | ((self.collision_effect.trickable as u16) << 13)
            | ((self.collision_effect.reject_road as u16) << 14)
            | ((self.collision_effect.soft_wall as u16) << 15)
    }
}

impl Variant {
    pub fn from_u16(value: u16) -> Result<Self, String> {
        match value {
            0 => Ok(Self::Value0),
            1 => Ok(Self::Value1),
            2 => Ok(Self::Value2),
            3 => Ok(Self::Value3),
            4 => Ok(Self::Value4),
            5 => Ok(Self::Value5),
            6 => Ok(Self::Value6),
            7 => Ok(Self::Value7),
            _ => Err(format!("Invalid variant: {}", value)),
        }
    }
}

impl Blight {
    pub fn from_u16(value: u16) -> Result<Self, String> {
        match value {
            0 => Ok(Self::Value0),
            1 => Ok(Self::Value1),
            2 => Ok(Self::Value2),
            3 => Ok(Self::Value3),
            4 => Ok(Self::Value4),
            5 => Ok(Self::Value5),
            6 => Ok(Self::Value6),
            7 => Ok(Self::Value7),
            _ => Err(format!("Invalid blight: {}", value)),
        }
    }
}

impl WheelDepth {
    pub fn from_u16(value: u16) -> Result<Self, String> {
        match value {
            0 => Ok(Self::Value0),
            1 => Ok(Self::Value1),
            2 => Ok(Self::Value2),
            3 => Ok(Self::Value3),
            _ => Err(format!("Invalid wheel depth: {}", value)),
        }
    }
}

impl CollisionEffect {
    pub fn from_u16(value: u16) -> Self {
        Self {
            trickable: (value >> 13) & 1 != 0,
            reject_road: (value >> 14) & 1 != 0,
            soft_wall: (value >> 15) & 1 != 0,
        }
    }
    pub fn color(&self) -> [u8; 4] {
        match (self.trickable, self.reject_road, self.soft_wall) {
            (true, _, _) => [255, 254, 230, 255], // trickable = light yellow
            (_, true, _) => [218, 177, 218, 255], // reject road = pink-ish
            (_, _, true) => [0, 0, 0, 255], // this is here for uh no reason, since i use HighlightOption for br
            _ => [255, 255, 255, 255], // fallback never used
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
        let flag = Flag::from_u16(read_u16(data, offset + 14)?)?;

        Ok(Prism {
            height,
            pos_i,
            fnrm_i,
            enrm1_i,
            enrm2_i,
            enrm3_i,
            flag,
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

pub struct BoundingBox {
    pub x_min: f32, pub y_min: f32, pub z_min: f32,
    pub x_max: f32, pub y_max: f32, pub z_max: f32,
    pub x_size: f32, pub y_size: f32, pub z_size: f32,
    pub x_center: f32, pub y_center: f32, pub z_center: f32,
}

pub fn get_bounding_box(positions: &[[f32; 3]]) -> BoundingBox {
    let mut x_min = f32::MAX; let mut y_min = f32::MAX; let mut z_min = f32::MAX;
    let mut x_max = f32::MIN; let mut y_max = f32::MIN; let mut z_max = f32::MIN;

    for &[x, y, z] in positions {
        x_min = x_min.min(x); y_min = y_min.min(y); z_min = z_min.min(z);
        x_max = x_max.max(x); y_max = y_max.max(y); z_max = z_max.max(z);
    }

    BoundingBox {
        x_min, y_min, z_min,
        x_max, y_max, z_max,
        x_size: x_max - x_min,
        y_size: y_max - y_min,
        z_size: z_max - z_min,
        x_center: (x_min + x_max) / 2.0,
        y_center: (y_min + y_max) / 2.0,
        z_center: (z_min + z_max) / 2.0,
    }
}

fn add_kmp(obj: &mut String, mtl: &mut String, kmp: &ParsedKmp, kmp_option: &KmpOption, bbox: BoundingBox, vertex_offset: &mut usize) {
    if kmp_option.ckpt {
        add_checkpoint(obj, mtl, kmp, bbox, vertex_offset, kmp_option.ckpt_side);
    }
}

fn prism_to_triangle(prism: &Prism, positions: &[[f32; 3]], normals: &[[f32; 3]]) -> [[f32; 3]; 3] {
    let pos = positions[prism.pos_i as usize];
    let fnrm = normals[prism.fnrm_i as usize];
    let enrm1 = normals[prism.enrm1_i as usize];
    let enrm2 = normals[prism.enrm2_i as usize];
    let enrm3 = normals[prism.enrm3_i as usize];

    let cross_a = cross(enrm1, fnrm);
    let cross_b = cross(enrm2, fnrm);
    let v1 = pos;
    let v2 = add(v1, scale(cross_b, prism.height / dot(cross_b, enrm3)));
    let v3 = add(v1, scale(cross_a, prism.height / dot(cross_a, enrm3)));

    [v1, v2, v3]
}

fn is_road(base_type: BaseType) -> bool { 
    matches!(base_type,
        BaseType::Road | BaseType::Road2
    )
}

fn is_wall(base_type: BaseType) -> bool {
    matches!(base_type,
        BaseType::Wall | BaseType::Wall2 | BaseType::InvisibleWall |
        BaseType::InvisibleWall2 | BaseType::SpecialWall | BaseType::WeakWall |
        BaseType::PlayerOnlyWall | BaseType::HalfPipeInvisibleWall
    )
}

// a wall is horizontal if the slope of the wall is greater than arccos(0.85)
// here i already have the face normal vec3, so i just take that instead of v1/v2/v3
fn is_horizontal(fnrm: [f32; 3]) -> bool {
    let value: f32 = 0.85;
    let fixed_angle = value.acos().to_degrees();

    let up: [f32; 3] = [0.0, 1.0, 0.0]; // define the up vector (y)
    let dp = dot(fnrm, up);
    let facing_up = dp > 0.0; // so ceilings aren't highlighted
    let angle = dp.abs().acos().to_degrees();
    if facing_up { angle < fixed_angle } else { false }
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct GroupKey {
    base_type: BaseType,
    collision_effect: CollisionEffect,
    is_horizontal: bool,
}

pub fn to_obj(
    parsed: &ParsedKcl, 
    name: &str, 
    highlight_option: &HighlightOption, 
    special_planes: &SpecialPlanesOption, 
    kmp: &ParsedKmp,
    kmp_option: &KmpOption,
) -> Object {
    let mut obj = String::new();
    let mut mtl = String::new();

    obj.push_str("# Generated by VisibleKCL\n");
    obj.push_str(&format!("mtllib {name}.mtl\n\n"));

    let pos_buf = &parsed.sections.position_vectors;
    let nrm_buf = &parsed.sections.normals;

    
    // OBJ is 1-based
    let mut vertex_offset = 1usize;
    
    // map groups by base type and collision effects
    let mut groups: HashMap<GroupKey, Vec<usize>> = HashMap::new();
    for (i, prism) in parsed.sections.prisms.iter().enumerate() {
        let key = GroupKey {
            base_type: prism.flag.base_type,
            collision_effect: prism.flag.collision_effect,
            is_horizontal: is_horizontal(nrm_buf[prism.fnrm_i as usize]) && is_wall(prism.flag.base_type),
        };
        groups.entry(key).or_default().push(i);
    }

    // sort em
    let mut sorted: Vec<(GroupKey, Vec<usize>)> = groups.into_iter().collect();
    sorted.sort_by_key(|(k, _)| k.base_type.discriminant());

    for (key, prism_indices) in &sorted {
        let current_prism = &parsed.sections.prisms[prism_indices[0]];

        let flag = current_prism.flag;
        let base_type = flag.base_type;
        let collision_effect = flag.collision_effect;

        let trickable = collision_effect.trickable && is_road(base_type);
        let reject_road = collision_effect.reject_road && is_road(base_type);
        let soft_wall = collision_effect.soft_wall && is_wall(base_type);
        let horizontal = key.is_horizontal;
        
        let name = format!("{}{}{}{}{}",
            flag.base_type.name(),
            if trickable { "_trickable" } else { "" },
            if reject_road { "_reject" } else { "" },
            if soft_wall { "_soft" } else { "" },
            if horizontal { "_horizontal" } else { "" },
        );
        
        if special_planes.is_hidden(base_type) { continue }
        
        let highlight = {
            if is_wall(base_type) {
                highlight_option.color(soft_wall, horizontal)
            } else {
                None
            }
        };

        let color = {
            if trickable || reject_road {
                collision_effect.color()
            } else if let Some(color) = highlight {
                color
            } else {
                base_type.color()
            }
        };
        
        // Write MTL entry for this flag (RGB 0.0-1.0).
        mtl.push_str(&format!("newmtl {}\n", name));
        // from int 255 to float 1.0
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

        for &i in prism_indices {
            let [v1, v2, v3] = prism_to_triangle(&parsed.sections.prisms[i], pos_buf, nrm_buf);
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

    // if whatever value is true run kmp
    if kmp_option.any_true() {
        let bbox = get_bounding_box(pos_buf);
        add_kmp(&mut obj, &mut mtl, kmp, kmp_option, bbox, &mut vertex_offset);
    }

    Object {obj, mtl}
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
