#![allow(dead_code)]
use super::binary::*;
use crate::util::kcl::*;
use crate::{Object, OverlayOption};

// https://wiki.tockdom.com/wiki/KMP_(File_Format)

// some stuff is also inspired from kmpeek (this is a simpler and more straightforward approach, tho)
// https://github.com/ThomasAlban/kmpeek/blob/main/src/util/kmp_file.rs

pub struct Header {
    pub magic: String,
    pub file_len: u32,
    pub section_count: u16,
    pub header_len: u16,
    pub ver_num: u32,
    pub sections_offset: Vec<u32>,
}

pub struct SectionHeader {
    pub name: String,
    pub entry_num: u16,
    pub other_value: u16,
}

pub struct KTPT {
    pub pos: [f32; 3],
    pub rot: [f32; 3],
    pub player_index: i16,
    pub _padding: u16,
}

pub struct ENPT {
    pub pos: [f32; 3],
    pub enemy_deviation: f32,
    pub set1: u16,
    pub set2: u8,
    pub set3: u8,
}

pub struct ENPH {
    pub point_start: u8,
    pub point_len: u8,
    pub previous_group: [u8; 6],
    pub next_group: [u8; 6],
    pub group_link_flags: u16,
}

pub struct ITPT {
    pub pos: [f32; 3],
    pub bullet_range: f32,
    pub set1: u16,
    pub set2: u16,
}

pub struct ITPH {
    pub point_start: u8,
    pub point_len: u8,
    pub previous_group: [u8; 6],
    pub next_group: [u8; 6],
    pub _padding: u16,
}

#[derive(PartialEq)]
pub enum CheckPointType {
    FinishLine,
    KeyCheckPoint,
    CheckPoint,
}

pub struct CKPT {
    pub left_point: [f32; 2],
    pub right_point: [f32; 2],
    pub respawn_index: u8,
    pub cp_type: i8,
    pub previous_cp: u8,
    pub next_cp: u8,
}

pub struct CKPH {
    pub first_cp: u8,
    pub cp_count: u8,
    pub previous_groups: [u8; 6],
    pub next_groups: [u8; 6],
    pub _padding: u16,
}

pub struct GOBJ {
    pub id: u16,
    pub _padding: u16,
    pub pos: [f32; 3],
    pub rot: [f32; 3],
    pub scale: [f32; 3],
    pub route: u16,
    pub settings: [u16; 8],
    pub presence_flags: u16,
}

pub struct RouteHeader {
    pub points_count: u16,
    pub set1: u8,
    pub set2: u8,
}

pub struct RoutePoints {
    pub pos: [f32; 3],
    pub set: u16,
    pub additional_set: u16,
}

pub struct POTI {
    pub header: RouteHeader,
    pub points: Vec<RoutePoints>,
}

pub struct AREA {
    pub shape: u8,
    pub area_type: u8,
    pub came_index: u8,
    pub priority_value: u8,
    pub pos: [f32; 3],
    pub rot: [f32; 3],
    pub scale: [f32; 3],
    pub set1: u16,
    pub set2: u16,
    pub route_id: u8,
    pub enemy_id: u8,
    pub _padding: u16,
}

pub struct CAME {
    pub camera_type: u8,
    pub next_camera: u8,
    pub camshake: u8,
    pub used_route: u8,
    pub camera_point_velocity: u16,
    pub zooming_velocity: u16,
    pub view_point_velocity: u16,
    pub start_flag: u8,
    pub movie_flag: u8,
    pub pos: [f32; 3],
    pub rot: [f32; 3],
    pub zoom_start: f32,
    pub zoom_end: f32,
    pub start_view_vec: [f32; 3],
    pub dest_view_vec: [f32; 3],
    pub time_active: f32,
}

pub struct JGPT {
    pub pos: [f32; 3],
    pub rot: [f32; 3],
    pub respawn_id: u16,
    pub user_data: i16,
}

pub struct CNPT {
    pub dest_pos: [f32; 3],
    pub release_angle: [f32; 3],
    pub cannon_id: u16,
    pub shoot_effect: i16,
}

pub struct MSPT {
    pub pos: [f32; 3],
    pub rot: [f32; 3],
    pub entry_id: u16,
    pub _unknown: u16,
}

pub struct STGI {
    pub lap_count: u8,
    pub pole_pos: u8,
    pub distance: u8,
    pub lens_flare: u8,
    pub flare_color: u32,
    pub flare_transparency: u8,
    pub padding1: u16,
    pub padding2: u8,
}

pub struct Section<T> {
    pub header: SectionHeader,
    pub entries: Vec<T>,
}

pub struct ParsedKmp {
    pub header: Header,
    pub ktpt: Section<KTPT>,
    pub enpt: Section<ENPT>,
    pub enph: Section<ENPH>,
    pub itpt: Section<ITPT>,
    pub itph: Section<ITPH>,
    pub ckpt: Section<CKPT>,
    pub ckph: Section<CKPH>,
    pub gobj: Section<GOBJ>,
    pub poti: Section<POTI>,
    pub area: Section<AREA>,
    pub came: Section<CAME>,
    pub jgpt: Section<JGPT>,
    pub cnpt: Section<CNPT>,
    pub mspt: Section<MSPT>,
    pub stgi: Section<STGI>,
}

impl Header {
    fn parse(data: &[u8]) -> Result<Self, String> {
        let magic = String::from_utf8_lossy(&data[0x00..0x04]).to_string();
        if magic != "RKMD" {
            return Err("Invalid magic number".into());
        }

        let file_len = read_u32(data, 0x04)?;
        let section_count = read_u16(data, 0x08)?;
        let header_len = read_u16(data, 0x0A)?;

        let ver_num_offset = header_len as usize - section_count as usize * 0x04 - 0x04;
        let ver_num = read_u32(data, ver_num_offset)?;

        let sections_offset_start = header_len as usize - section_count as usize * 0x04;
        let sections_offset = read_vec_u32(data, sections_offset_start, section_count as usize)?;

        Ok(Header {
            magic,
            file_len,
            section_count,
            header_len,
            ver_num,
            sections_offset,
        })
    }
}

impl SectionHeader {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let name = String::from_utf8_lossy(&data[offset..offset + 0x04]).to_string();
        let entry_num = read_u16(data, offset + 0x04)?;
        let other_value = read_u16(data, offset + 0x06)?;
        Ok(SectionHeader {
            name,
            entry_num,
            other_value,
        })
    }
}

impl KTPT {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let pos = read_vec_f32(data, offset, 3)?.try_into().unwrap();
        let rot = read_vec_f32(data, offset + 0x0C, 3)?.try_into().unwrap();
        let player_index = read_i16(data, offset + 0x18)?;
        Ok(KTPT {
            pos,
            rot,
            player_index,
            _padding: 0,
        })
    }
}

impl ENPT {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let pos = read_vec_f32(data, offset, 3)?.try_into().unwrap();
        let enemy_deviation = read_f32(data, offset + 0x0C)?;
        let set1 = read_u16(data, offset + 0x10)?;
        let set2 = read_u8(data, offset + 0x12)?;
        let set3 = read_u8(data, offset + 0x13)?;
        Ok(ENPT {
            pos,
            enemy_deviation,
            set1,
            set2,
            set3,
        })
    }
}

impl ENPH {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let point_start = read_u8(data, offset)?;
        let point_len = read_u8(data, offset + 0x01)?;
        let previous_group = read_vec_u8(data, offset + 0x02, 6)?.try_into().unwrap();
        let next_group = read_vec_u8(data, offset + 0x08, 6)?.try_into().unwrap();
        let group_link_flags = read_u16(data, offset + 0x0E)?;
        Ok(ENPH {
            point_start,
            point_len,
            previous_group,
            next_group,
            group_link_flags,
        })
    }
}

impl ITPT {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let pos = read_vec_f32(data, offset, 3)?.try_into().unwrap();
        let bullet_range = read_f32(data, offset + 0x0C)?;
        let set1 = read_u16(data, offset + 0x10)?;
        let set2 = read_u16(data, offset + 0x12)?;
        Ok(ITPT {
            pos,
            bullet_range,
            set1,
            set2,
        })
    }
}

impl ITPH {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let point_start = read_u8(data, offset)?;
        let point_len = read_u8(data, offset + 0x01)?;
        let previous_group = read_vec_u8(data, offset + 0x02, 6)?.try_into().unwrap();
        let next_group = read_vec_u8(data, offset + 0x08, 6)?.try_into().unwrap();
        Ok(ITPH {
            point_start,
            point_len,
            previous_group,
            next_group,
            _padding: 0,
        })
    }
}

impl CKPT {
    pub fn checkpoint_type(&self) -> CheckPointType {
        match self.cp_type {
            0 => CheckPointType::FinishLine,
            -1 => CheckPointType::CheckPoint,
            _ => CheckPointType::KeyCheckPoint,
        }
    }
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let left_point = read_vec_f32(data, offset, 2)?.try_into().unwrap();
        let right_point = read_vec_f32(data, offset + 0x08, 2)?.try_into().unwrap();
        let respawn_index = read_u8(data, offset + 0x10)?;
        let cp_type = read_u8(data, offset + 0x11)? as i8;
        let previous_cp = read_u8(data, offset + 0x12)?;
        let next_cp = read_u8(data, offset + 0x13)?;
        Ok(CKPT {
            left_point,
            right_point,
            respawn_index,
            cp_type,
            previous_cp,
            next_cp,
        })
    }
}

impl CKPH {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let first_cp = read_u8(data, offset)?;
        let cp_count = read_u8(data, offset + 0x01)?;
        let previous_groups = read_vec_u8(data, offset + 0x02, 6)?.try_into().unwrap();
        let next_groups = read_vec_u8(data, offset + 0x08, 6)?.try_into().unwrap();
        Ok(CKPH {
            first_cp,
            cp_count,
            previous_groups,
            next_groups,
            _padding: 0,
        })
    }
}

impl GOBJ {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let id = read_u16(data, offset)?;
        let pos = read_vec_f32(data, offset + 0x04, 3)?.try_into().unwrap();
        let rot = read_vec_f32(data, offset + 0x10, 3)?.try_into().unwrap();
        let scale = read_vec_f32(data, offset + 0x1C, 3)?.try_into().unwrap();
        let route = read_u16(data, offset + 0x28)?;
        let settings = read_vec_u16(data, offset + 0x2A, 8)?.try_into().unwrap();
        let presence_flags = read_u16(data, offset + 0x3A)?;
        Ok(GOBJ {
            id,
            _padding: 0,
            pos,
            rot,
            scale,
            route,
            settings,
            presence_flags,
        })
    }
}

impl RouteHeader {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let points_count = read_u16(data, offset)?;
        let set1 = read_u8(data, offset + 0x02)?;
        let set2 = read_u8(data, offset + 0x03)?;
        Ok(RouteHeader {
            points_count,
            set1,
            set2,
        })
    }
}

impl RoutePoints {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let pos = read_vec_f32(data, offset, 3)?.try_into().unwrap();
        let set = read_u16(data, offset + 0x0C)?;
        let additional_set = read_u16(data, offset + 0x0E)?;
        Ok(RoutePoints {
            pos,
            set,
            additional_set,
        })
    }
}

impl POTI {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let header = RouteHeader::parse(data, offset)?;
        let mut points = Vec::with_capacity(header.points_count as usize);
        for i in 0..header.points_count as usize {
            points.push(RoutePoints::parse(data, offset + 0x04 + i * 0x10)?);
        }
        Ok(POTI { header, points })
    }
}

impl AREA {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let shape = read_u8(data, offset)?;
        let area_type = read_u8(data, offset + 0x01)?;
        let came_index = read_u8(data, offset + 0x02)?;
        let priority_value = read_u8(data, offset + 0x03)?;
        let pos = read_vec_f32(data, offset + 0x04, 3)?.try_into().unwrap();
        let rot = read_vec_f32(data, offset + 0x10, 3)?.try_into().unwrap();
        let scale = read_vec_f32(data, offset + 0x1C, 3)?.try_into().unwrap();
        let set1 = read_u16(data, offset + 0x28)?;
        let set2 = read_u16(data, offset + 0x2A)?;
        let route_id = read_u8(data, offset + 0x2C)?;
        let enemy_id = read_u8(data, offset + 0x2D)?;
        Ok(AREA {
            shape,
            area_type,
            came_index,
            priority_value,
            pos,
            rot,
            scale,
            set1,
            set2,
            route_id,
            enemy_id,
            _padding: 0,
        })
    }
}

impl CAME {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let camera_type = read_u8(data, offset)?;
        let next_camera = read_u8(data, offset + 0x01)?;
        let camshake = read_u8(data, offset + 0x02)?;
        let used_route = read_u8(data, offset + 0x03)?;
        let camera_point_velocity = read_u16(data, offset + 0x04)?;
        let zooming_velocity = read_u16(data, offset + 0x06)?;
        let view_point_velocity = read_u16(data, offset + 0x08)?;
        let start_flag = read_u8(data, offset + 0x0A)?;
        let movie_flag = read_u8(data, offset + 0x0B)?;
        let pos = read_vec_f32(data, offset + 0x0C, 3)?.try_into().unwrap();
        let rot = read_vec_f32(data, offset + 0x18, 3)?.try_into().unwrap();
        let zoom_start = read_f32(data, offset + 0x24)?;
        let zoom_end = read_f32(data, offset + 0x28)?;
        let start_view_vec = read_vec_f32(data, offset + 0x2C, 3)?.try_into().unwrap();
        let dest_view_vec = read_vec_f32(data, offset + 0x38, 3)?.try_into().unwrap();
        let time_active = read_f32(data, offset + 0x44)?;
        Ok(CAME {
            camera_type,
            next_camera,
            camshake,
            used_route,
            camera_point_velocity,
            zooming_velocity,
            view_point_velocity,
            start_flag,
            movie_flag,
            pos,
            rot,
            zoom_start,
            zoom_end,
            start_view_vec,
            dest_view_vec,
            time_active,
        })
    }
}

impl JGPT {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let pos = read_vec_f32(data, offset, 3)?.try_into().unwrap();
        let rot = read_vec_f32(data, offset + 0x0C, 3)?.try_into().unwrap();
        let respawn_id = read_u16(data, offset + 0x18)?;
        let user_data = read_i16(data, offset + 0x1A)?;
        Ok(JGPT {
            pos,
            rot,
            respawn_id,
            user_data,
        })
    }
}

impl CNPT {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let dest_pos = read_vec_f32(data, offset, 3)?.try_into().unwrap();
        let release_angle = read_vec_f32(data, offset + 0x0C, 3)?.try_into().unwrap();
        let cannon_id = read_u16(data, offset + 0x18)?;
        let shoot_effect = read_i16(data, offset + 0x1A)?;
        Ok(CNPT {
            dest_pos,
            release_angle,
            cannon_id,
            shoot_effect,
        })
    }
}

impl MSPT {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let pos = read_vec_f32(data, offset, 3)?.try_into().unwrap();
        let rot = read_vec_f32(data, offset + 0x0C, 3)?.try_into().unwrap();
        let entry_id = read_u16(data, offset + 0x18)?;
        let _unknown = read_u16(data, offset + 0x1A)?;
        Ok(MSPT {
            pos,
            rot,
            entry_id,
            _unknown,
        })
    }
}

impl STGI {
    fn parse(data: &[u8], offset: usize) -> Result<Self, String> {
        let lap_count = read_u8(data, offset)?;
        let pole_pos = read_u8(data, offset + 0x01)?;
        let distance = read_u8(data, offset + 0x02)?;
        let lens_flare = read_u8(data, offset + 0x03)?;
        let flare_color = read_u32(data, offset + 0x04)?;
        let flare_transparency = read_u8(data, offset + 0x08)?;
        let padding1 = read_u16(data, offset + 0x09)?;
        let padding2 = read_u8(data, offset + 0x0B)?;
        Ok(STGI {
            lap_count,
            pole_pos,
            distance,
            lens_flare,
            flare_color,
            flare_transparency,
            padding1,
            padding2,
        })
    }
}

pub fn add_checkpoint(
    obj: &mut String,
    mtl: &mut String,
    kmp: &ParsedKmp,
    bbox: BoundingBox,
    vertex_offset: &mut usize,
    side: bool,
) {
    let ckpt = &kmp.ckpt;

    // hardcode the checkpoint groups in the mtl
    let alpha = 50;
    let groups = [
        (
            CheckPointType::FinishLine,
            "ckpt_finish",
            [255u8, 127, 255, alpha],
        ),
        (
            CheckPointType::KeyCheckPoint,
            "ckpt_key",
            [255u8, 0, 255, alpha],
        ),
        (
            CheckPointType::CheckPoint,
            "ckpt_normal",
            [0u8, 0, 255, alpha],
        ),
    ];
    if side {
        let side_color = [0u8, 255, 255, alpha];
        mtl.push_str("newmtl ckpt_side\n");
        mtl.push_str(&format!(
            "Kd {:.4} {:.4} {:.4}\nd {:.4}\n\n",
            side_color[0] as f32 / 255.0,
            side_color[1] as f32 / 255.0,
            side_color[2] as f32 / 255.0,
            side_color[3] as f32 / 255.0,
        ));

        // write the side group once before the main loop
        obj.push_str("g ckpt_side\nusemtl ckpt_side\n");
    }

    for (cp_type, mat_name, color) in &groups {
        mtl.push_str(&format!("newmtl {}\n", mat_name));
        mtl.push_str(&format!(
            "Kd {:.4} {:.4} {:.4}\nd {:.4}\n\n",
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
            color[3] as f32 / 255.0,
        ));

        obj.push_str(&format!("g {}\n", mat_name));
        obj.push_str(&format!("usemtl {}\n", mat_name));

        // use peekable for the ckpt sides
        let mut iter = ckpt.entries.iter().peekable();
        while let Some(checkpoint) = iter.next() {
            if checkpoint.checkpoint_type() != *cp_type {
                continue;
            }

            let (x0, z0) = (checkpoint.left_point[0], checkpoint.left_point[1]);
            let (x1, z1) = (checkpoint.right_point[0], checkpoint.right_point[1]);
            let y = bbox.y_max + 500.0; // some padding
            let v0 = [x0, y, z0]; // top left
            let v1 = [x1, y, z1]; // top right
            let v2 = [x1, 0.0, z1]; // bottom right
            let v3 = [x0, 0.0, z0]; // bottom left
            // vert
            obj.push_str(&format!("v {} {} {}\n", v0[0], v0[1], v0[2]));
            obj.push_str(&format!("v {} {} {}\n", v1[0], v1[1], v1[2]));
            obj.push_str(&format!("v {} {} {}\n", v2[0], v2[1], v2[2]));
            obj.push_str(&format!("v {} {} {}\n", v3[0], v3[1], v3[2]));
            // i pull vertex offset to keep count of the previous vertices count (from the actual kcl)
            let base = *vertex_offset;
            *vertex_offset += 4;
            obj.push_str(&format!("f {} {} {}\n", base, base + 1, base + 2));
            obj.push_str(&format!("f {} {} {}\n", base, base + 2, base + 3));

            if side {
                // circular looping
                // last cp has the first as next, avoiding the last not connecting to the firsr
                let next = iter.peek().copied().unwrap_or(&ckpt.entries[0]);

                let (nx0, nz0) = (next.left_point[0], next.left_point[1]);
                let (nx1, nz1) = (next.right_point[0], next.right_point[1]);

                // left
                let lv0 = [x0, y, z0];
                let lv1 = [nx0, y, nz0];
                let lv2 = [nx0, 0.0, nz0];
                let lv3 = [x0, 0.0, z0];
                obj.push_str(&format!("v {} {} {}\n", lv0[0], lv0[1], lv0[2]));
                obj.push_str(&format!("v {} {} {}\n", lv1[0], lv1[1], lv1[2]));
                obj.push_str(&format!("v {} {} {}\n", lv2[0], lv2[1], lv2[2]));
                obj.push_str(&format!("v {} {} {}\n", lv3[0], lv3[1], lv3[2]));
                let base = *vertex_offset;
                *vertex_offset += 4;
                obj.push_str(&format!("f {} {} {}\n", base, base + 1, base + 2));
                obj.push_str(&format!("f {} {} {}\n", base, base + 2, base + 3));

                // right
                let rv0 = [x1, y, z1];
                let rv1 = [nx1, y, nz1];
                let rv2 = [nx1, 0.0, nz1];
                let rv3 = [x1, 0.0, z1];
                obj.push_str(&format!("v {} {} {}\n", rv0[0], rv0[1], rv0[2]));
                obj.push_str(&format!("v {} {} {}\n", rv1[0], rv1[1], rv1[2]));
                obj.push_str(&format!("v {} {} {}\n", rv2[0], rv2[1], rv2[2]));
                obj.push_str(&format!("v {} {} {}\n", rv3[0], rv3[1], rv3[2]));
                let base = *vertex_offset;
                *vertex_offset += 4;
                obj.push_str(&format!("f {} {} {}\n", base, base + 1, base + 2));
                obj.push_str(&format!("f {} {} {}\n", base, base + 2, base + 3));
            }
        }
        obj.push('\n');
    }
}

pub fn to_obj(kmp: &ParsedKmp, kcl: &ParsedKcl, name: &str, overlay: &OverlayOption) -> Object {
    let mut obj = String::new();
    let mut mtl = String::new();

    obj.push_str("# Generated by VisibleKCL\n");
    obj.push_str(&format!("mtllib {name}.mtl\n\n"));

    let kcl_pos_buf = &kcl.sections.position_vectors;
    let kcl_nrm_buf = &kcl.sections.normals;
    let kcl_bbox = get_bounding_box(kcl_pos_buf);

    // OBJ is 1-based
    let mut vertex_offset = 1usize;

    if overlay.ckpt {
        add_checkpoint(
            &mut obj,
            &mut mtl,
            kmp,
            kcl_bbox,
            &mut vertex_offset,
            overlay.ckpt_side,
        );
    }

    Object { obj, mtl }
}

fn parse_section<T, F>(
    data: &[u8],
    abs_offset: usize,
    entry_size: usize,
    parse_fn: F,
) -> Result<Section<T>, String>
where
    F: Fn(&[u8], usize) -> Result<T, String>,
{
    let header = SectionHeader::parse(data, abs_offset)?;
    let mut entries = Vec::with_capacity(header.entry_num as usize);
    for i in 0..header.entry_num as usize {
        entries.push(parse_fn(data, abs_offset + 0x08 + i * entry_size)?);
    }
    Ok(Section { header, entries })
}

pub fn parse(data: &[u8]) -> Result<ParsedKmp, String> {
    let header = Header::parse(data)?;
    let offsets: Vec<usize> = header
        .sections_offset
        .iter()
        .map(|&o| header.header_len as usize + o as usize)
        .collect();

    let ktpt = parse_section(data, offsets[0], 0x1C, KTPT::parse)?;
    let enpt = parse_section(data, offsets[1], 0x14, ENPT::parse)?;
    let enph = parse_section(data, offsets[2], 0x10, ENPH::parse)?;
    let itpt = parse_section(data, offsets[3], 0x14, ITPT::parse)?;
    let itph = parse_section(data, offsets[4], 0x10, ITPH::parse)?;
    let ckpt = parse_section(data, offsets[5], 0x14, CKPT::parse)?;
    let ckph = parse_section(data, offsets[6], 0x10, CKPH::parse)?;
    let gobj = parse_section(data, offsets[7], 0x3C, GOBJ::parse)?;
    let area = parse_section(data, offsets[9], 0x30, AREA::parse)?;
    let came = parse_section(data, offsets[10], 0x48, CAME::parse)?;
    let jgpt = parse_section(data, offsets[11], 0x1C, JGPT::parse)?;
    let cnpt = parse_section(data, offsets[12], 0x1C, CNPT::parse)?;
    let mspt = parse_section(data, offsets[13], 0x1C, MSPT::parse)?;
    let stgi = parse_section(data, offsets[14], 0x0C, STGI::parse)?;

    // POTI is special since each route has a variable number of points
    let poti_section_header = SectionHeader::parse(data, offsets[8])?;
    let mut poti_entries = Vec::with_capacity(poti_section_header.entry_num as usize);
    let mut poti_offset = offsets[8] + 0x08;
    for _ in 0..poti_section_header.entry_num {
        let route = POTI::parse(data, poti_offset)?;
        poti_offset += 0x04 + route.header.points_count as usize * 0x10;
        poti_entries.push(route);
    }
    let poti = Section {
        header: poti_section_header,
        entries: poti_entries,
    };

    Ok(ParsedKmp {
        header,
        ktpt,
        enpt,
        enph,
        itpt,
        itph,
        ckpt,
        ckph,
        gobj,
        poti,
        area,
        came,
        jgpt,
        cnpt,
        mspt,
        stgi,
    })
}

pub fn parse_from_path(path: &str) -> Result<ParsedKmp, String> {
    let data = path_to_data(path, "kmp")?;
    parse(&data)
}
