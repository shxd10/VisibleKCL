pub mod util;

pub use util::enums;
pub use util::szs::CourseFiles;
pub use util::{
    brres, draw,
    enums::Gobj,
    kcl::{self, BaseType},
    kmp, szs,
};

use std::fs;

pub struct OverlayOption {
    pub ckpt: bool,
    pub ckpt_side: bool,
    pub inv_walls: bool,
    pub gobj: bool,
}

impl OverlayOption {
    pub fn any_true(&self) -> bool {
        [self.ckpt, self.ckpt_side].iter().any(|&option| option)
    }
}

impl Default for OverlayOption {
    fn default() -> Self {
        OverlayOption {
            ckpt: false,
            ckpt_side: false,
            inv_walls: false,
            gobj: false,
        }
    }
}

pub struct SpecialPlanesOption {
    pub item_road: bool,
    pub item_wall: bool,
    pub force_recalc: bool,
    pub sound_trigger: bool,
    pub effect_trigger: bool,
    pub item_state_modifier: bool,
}

impl SpecialPlanesOption {
    pub fn is_hidden(&self, base_type: BaseType) -> bool {
        match base_type {
            BaseType::ItemRoad => !self.item_road,
            BaseType::ItemWall => !self.item_wall,
            BaseType::ForceRecalculation => !self.force_recalc,
            BaseType::SoundTrigger => !self.sound_trigger,
            BaseType::EffectTrigger => !self.effect_trigger,
            BaseType::ItemStateModifier => !self.item_state_modifier,
            _ => false,
        }
    }
}

impl Default for SpecialPlanesOption {
    fn default() -> Self {
        SpecialPlanesOption {
            item_road: false,
            item_wall: false,
            force_recalc: false,
            sound_trigger: false,
            effect_trigger: false,
            item_state_modifier: false,
        }
    }
}

pub struct HighlightOption {
    pub soft_wall: bool,
    pub horizontal_wall: bool,
}

impl HighlightOption {
    pub fn color(&self, is_soft: bool, is_horizontal: bool) -> Option<[u8; 4]> {
        let highlight_sw = self.soft_wall && is_soft;
        let highlight_hw = self.horizontal_wall && is_horizontal;
        match (highlight_sw, highlight_hw) {
            (true, true) => Some([255, 200, 80, 255]), // orange ish: indicates intersection (might be useful for some stuff, it's pretty annoying sometimes when a tri is both BR and HW)
            (true, false) | (false, true) => Some([255, 240, 80, 255]), // yellow ish
            _ => None,
        }
    }
}

impl Default for HighlightOption {
    fn default() -> Self {
        HighlightOption {
            soft_wall: false,
            horizontal_wall: false,
        }
    }
}

pub struct KmpDrawOptions {
    pub thickness: u32,
    pub ktpt: bool,
    pub enpt: bool,
    pub itpt: bool,
    pub ckpt: bool,
    pub ckpt_side_lines: bool,
    pub gobj: bool,
    pub poti: bool,
    pub area: bool,
    pub came: bool,
    pub jgpt: bool,
    pub jgpt_lines: bool,
    pub cnpt: bool,
    pub mspt: bool,
    pub stgi: bool,
}

impl Default for KmpDrawOptions {
    fn default() -> Self {
        KmpDrawOptions {
            thickness: 8,
            ktpt: false,
            enpt: false,
            itpt: false,
            ckpt: true,
            ckpt_side_lines: true,
            gobj: false,
            poti: false,
            area: false,
            came: false,
            jgpt: true,
            jgpt_lines: true,
            cnpt: false,
            mspt: false,
            stgi: false,
        }
    }
}

pub struct KclDrawOptions {
    pub wireframe: bool,
    pub shading: f32,
}

impl Default for KclDrawOptions {
    fn default() -> Self {
        KclDrawOptions {
            wireframe: false,
            shading: 0.5,
        }
    }
}

pub struct Object {
    pub obj: String,
    pub mtl: String,
}

impl Object {
    pub fn replace(&mut self, other_object: Object) {
        self.obj.push_str(&other_object.obj);
        self.mtl.push_str(&other_object.mtl);
    }
}

pub fn write_obj_file(obj: &str, mtl: &str, name: &str) {
    fs::write(format!("{name}.obj"), &obj).unwrap();
    fs::write(format!("{name}.mtl"), &mtl).unwrap();
}

pub fn write_szs(course: &CourseFiles, filename: &str) -> Result<(), String> {
    let szs_buf = szs::write_arc_to_szs(&course.arc)?;
    fs::write(format!("{}.szs", filename), szs_buf).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn replace_gobj(course: &mut CourseFiles) -> Result<(), String> {
    // every .brres file in the szs that is NOT course_model, map_model, or vrcorn_model
    let brres_names: Vec<String> = course
        .arc
        .nodes
        .iter()
        .filter(|n| {
            n.name.ends_with(".brres")
                && !matches!(
                    n.name.as_str(),
                    "course_model.brres" | "map_model.brres" | "vrcorn_model.brres"
                )
        })
        .map(|n| n.name.clone())
        .collect();

    // replace with custom brres from gobj directory (relative to CLI working directory)
    for name in brres_names {
        let gobj_path = format!("gobj/{}", name);
        let Ok(buf) = fs::read(&gobj_path) else {
            eprintln!("Warning: Could not find gobj file: {}", gobj_path);
            continue;
        };
        course.arc.replace_file(&name, buf)?;
    }

    // need the enum to know which objects have kcl and what they're called
    let kcl_names: Vec<&'static str> = course
        .kmp
        .gobj
        .entries
        .iter()
        .filter_map(|entry| Gobj::from_u16(entry.id)?.kcl_name())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // yes im gonna be honest i vibe coded this part
    for name in kcl_names {
        let kcl_filename = format!("{}.kcl", name);
        let brres_filename = format!("{}.brres", name);

        let Some(kcl_node) = course.arc.nodes.iter().find(|n| n.name == kcl_filename) else {
            continue;
        };
        let kcl_data = course.arc.data
            [kcl_node.data_offset as usize..(kcl_node.data_offset + kcl_node.data_size) as usize]
            .to_vec();
        let parsed_kcl = kcl::parse(&kcl_data)?;

        let Some(brres_node) = course.arc.nodes.iter().find(|n| n.name == brres_filename) else {
            continue;
        };
        let brres_data = course.arc.data[brres_node.data_offset as usize
            ..(brres_node.data_offset + brres_node.data_size) as usize]
            .to_vec();
        let mut parsed_brres = brres::parse(&brres_data).map_err(|e| e.to_string())?;

        let object = kcl::to_obj(
            &parsed_kcl,
            name,
            &HighlightOption::default(),
            &SpecialPlanesOption::default(),
            &course.kmp,
            &OverlayOption::default(),
        );
        brres::from_obj_replace(&mut parsed_brres, &object.obj, &object.mtl)?;
        let buf = parsed_brres.write_memory().map_err(|e| e.to_string())?;
        course.arc.replace_file(&brres_filename, buf)?;
    }

    Ok(())
}
