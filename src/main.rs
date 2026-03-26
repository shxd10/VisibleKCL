use clap::{Parser, Subcommand};
use std::time::Instant;
use std::fs;
use std::path::Path;

mod util;
use crate::util::{szs, kmp, kcl, brres, draw};
use kcl::BaseType;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(alias = "x")]
    Extract { path: String },
    Kcl { 
        path: String, 
        #[arg(long, default_value_t = false)]
        write_obj: bool,

        #[arg(long, default_value_t = false)]
        soft_wall: bool,
        #[arg(long, default_value_t = false)]
        horizontal_wall: bool,

        #[arg(long, default_value_t = false)]
        item_road: bool,
        #[arg(long, default_value_t = false)]
        item_wall: bool,
        #[arg(long, default_value_t = false)]
        force_recalc: bool,
        #[arg(long, default_value_t = false)]
        sound_trigger: bool,
        #[arg(long, default_value_t = false)]
        effect_trigger: bool,
        #[arg(long, default_value_t = false)]
        item_state_modifier: bool
    },
    Ckpt { 
        path: String, 
        #[arg(long, default_value_t = false)]
        write_obj: bool
    },
    Draw {
        path: String,
        // kcl options
        #[arg(long, default_value_t = false)]
        wireframe: bool,
        #[arg(long, default_value_t = 0.5)]
        shading: f32,

        // kmp options
        #[arg(long, default_value_t = 4)]
        thickness: u32,
        #[arg(long, default_value_t = false)]
        ktpt: bool,
        #[arg(long, default_value_t = false)]
        enpt: bool,
        #[arg(long, default_value_t = false)]
        itpt: bool,
        #[arg(long, default_value_t = true)]
        ckpt: bool,
        #[arg(long, default_value_t = true)]
        ckpt_side_lines: bool,
        #[arg(long, default_value_t = false)]
        gobj: bool,
        #[arg(long, default_value_t = false)]
        poti: bool,
        #[arg(long, default_value_t = false)]
        area: bool,
        #[arg(long, default_value_t = false)]
        came: bool,
        #[arg(long, default_value_t = true)]
        jgpt: bool,
        #[arg(long, default_value_t = true)]
        jgpt_lines: bool,
        #[arg(long, default_value_t = false)]
        cnpt: bool,
        #[arg(long, default_value_t = false)]
        mspt: bool,
    },
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

pub struct TriggerOption {
    pub soft_wall: bool,
    pub horizontal_wall: bool,
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
    wireframe: bool,
    shading: f32,
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

fn write_obj_file(obj: &str, mtl: &str, name: &str) {
    fs::write(format!("{name}.obj"), &obj).unwrap();
    fs::write(format!("{name}.mtl"), &mtl).unwrap();
}

fn main() -> Result<(), String> {
    let command = Cli::parse().command;

    match command {
        Command::Extract { path } => {
            szs::extract(&path)?;
        }
        Command::Kcl { path, write_obj, soft_wall, horizontal_wall, item_road, item_wall, force_recalc, sound_trigger, effect_trigger, item_state_modifier } => {
            let start = Instant::now();

            let highlight = HighlightOption {
                soft_wall,
                horizontal_wall,
            };

            let special = SpecialPlanesOption {
                item_road,
                item_wall,
                force_recalc,
                sound_trigger,
                effect_trigger,
                item_state_modifier,
            };

            let filename = Path::new(&path).file_stem().unwrap().to_str().unwrap();
            let mut course = szs::parse_course_files(&path)?;
            let object = kcl::to_obj(&course.kcl, filename, &highlight, &special);
            let obj = object.obj;
            let mtl = object.mtl;

            if write_obj {
                write_obj_file(&obj, &mtl, filename);
            }

            brres::from_obj(&mut course.brres, &obj, &mtl)?;

            let buf = course.brres.write_memory().map_err(|e| e.to_string())?;
            course.arc.replace_file("course_model.brres", buf)?;

            let szs_bytes = szs::write_arc_to_szs(&course.arc)?;
            fs::write(format!("{}.szs", filename), szs_bytes).map_err(|e| e.to_string())?;

            println!("Took: {:?}", start.elapsed());
        }
        Command::Ckpt { path, write_obj } => {
            let start = Instant::now();
            kmp::parse_from_path(&path)?;
            println!("Took: {:?}", start.elapsed());
        }
        Command::Draw { 
            path, wireframe, shading,
            thickness, ktpt, enpt, itpt, ckpt, ckpt_side_lines,
            gobj, poti, area, came, jgpt, jgpt_lines, cnpt, mspt,
        } => {
            let img = draw::to_image(
                &path,
                &KclDrawOptions { wireframe, shading },
                &KmpDrawOptions { thickness, ktpt, enpt, itpt, ckpt, ckpt_side_lines, gobj, poti, area, came, jgpt, jgpt_lines, cnpt, mspt, stgi: false },
            );
            img.save("output.png").unwrap();
        }
    }

    Ok(())
}