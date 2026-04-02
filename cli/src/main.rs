use clap::ArgAction::Set;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::time::Instant;

use api::CourseFiles;
use api::enums::{self, Gobj};
use api::{
    HighlightOption, KclDrawOptions, KmpDrawOptions, Object, OverlayOption, SpecialPlanesOption,
    brres, draw,
    kcl::{self, BaseType},
    kmp, replace_gobj, szs, write_obj_file, write_szs,
};

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
    Replace {
        path: String,
        #[arg(long, default_value_t = false)]
        write_obj: bool,

        #[arg(long, default_value_t = false)]
        ckpt: bool,
        #[arg(long, default_value_t = false)]
        ckpt_side: bool,
        #[arg(long, default_value_t = false)]
        gobj: bool,

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
        item_state_modifier: bool,
    },
    Overlay {
        path: String,
        #[arg(long, default_value_t = false)]
        write_obj: bool,

        #[arg(long, default_value_t = false)]
        ckpt: bool,
        #[arg(long, default_value_t = false)]
        ckpt_side: bool,
        #[arg(long, default_value_t = false)]
        inv_walls: bool,

        #[arg(long, default_value_t = false)]
        gobj: bool,
    },
    Draw {
        path: String,
        #[arg(long, default_value_t = false)]
        wireframe: bool,
        #[arg(long, default_value_t = 0.5)]
        shading: f32,
        #[arg(long, default_value_t = 4)]
        thickness: u32,
        #[arg(long, default_value_t = false)]
        ktpt: bool,
        #[arg(long, default_value_t = false)]
        enpt: bool,
        #[arg(long, default_value_t = false)]
        itpt: bool,
        #[arg(long, default_value_t = true, action = Set)]
        ckpt: bool,
        #[arg(long, default_value_t = true, action = Set)]
        ckpt_side_lines: bool,
        #[arg(long, default_value_t = false)]
        gobj: bool,
        #[arg(long, default_value_t = false)]
        poti: bool,
        #[arg(long, default_value_t = false)]
        area: bool,
        #[arg(long, default_value_t = false)]
        came: bool,
        #[arg(long, default_value_t = true, action = Set)]
        jgpt: bool,
        #[arg(long, default_value_t = true, action = Set)]
        jgpt_lines: bool,
        #[arg(long, default_value_t = false)]
        cnpt: bool,
        #[arg(long, default_value_t = false)]
        mspt: bool,
    },
}

fn main() -> Result<(), String> {
    let command = Cli::parse().command;

    match command {
        Command::Extract { path } => {
            szs::extract(&path)?;
        }
        Command::Replace {
            path,
            write_obj,
            ckpt,
            ckpt_side,
            gobj,
            soft_wall,
            horizontal_wall,
            item_road,
            item_wall,
            force_recalc,
            sound_trigger,
            effect_trigger,
            item_state_modifier,
        } => {
            let start = Instant::now();

            let overlay_option = OverlayOption {
                ckpt,
                ckpt_side,
                inv_walls: false, // not used in this
                gobj,
            };

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
            let object = kcl::to_obj(
                &course.kcl,
                filename,
                &highlight,
                &special,
                &course.kmp,
                &overlay_option,
            );

            let obj = object.obj;
            let mtl = object.mtl;

            if write_obj {
                write_obj_file(&obj, &mtl, filename);
            }

            brres::from_obj_replace(&mut course.brres, &obj, &mtl)?;

            let buf = course.brres.write_memory().map_err(|e| e.to_string())?;
            course.arc.replace_file("course_model.brres", buf)?;
            if overlay_option.gobj {
                replace_gobj(&mut course)?;
            }

            write_szs(&course, filename)?;
            println!("Took: {:?}", start.elapsed());
        }
        Command::Overlay {
            path,
            write_obj,
            ckpt,
            ckpt_side,
            inv_walls,
            gobj,
        } => {
            let start = Instant::now();

            let overlay_option = OverlayOption {
                ckpt,
                ckpt_side,
                inv_walls,
                gobj,
            };

            let filename = Path::new(&path).file_stem().unwrap().to_str().unwrap();
            let mut course = szs::parse_course_files(&path)?;

            let mut object = Object {
                obj: String::new(),
                mtl: String::new(),
            };

            // if the user enables inv walls on overlay, delete every kcl flag other than inv wall (keep())
            // .replace() is for replacing the object with the one returned by to_obj
            if overlay_option.inv_walls {
                course.kcl = course.kcl.keep(BaseType::InvisibleWall);
                object.replace(kcl::to_obj(
                    &course.kcl,
                    filename,
                    &HighlightOption::default(),
                    &SpecialPlanesOption::default(),
                    &course.kmp,
                    &overlay_option,
                ));
            }

            if overlay_option.ckpt || overlay_option.ckpt_side {
                object.replace(kmp::to_obj(
                    &course.kmp,
                    &course.kcl,
                    filename,
                    &overlay_option,
                ));
            }

            let obj = object.obj;
            let mtl = object.mtl;

            if write_obj {
                write_obj_file(&obj, &mtl, filename);
            }

            brres::from_obj_overlay(&mut course.brres, &obj, &mtl)?;

            let buf = course.brres.write_memory().map_err(|e| e.to_string())?;
            course.arc.replace_file("course_model.brres", buf)?;

            if overlay_option.gobj {
                replace_gobj(&mut course)?;
            }
            write_szs(&course, filename)?;
            println!("Took: {:?}", start.elapsed());
        }
        Command::Draw {
            path,
            wireframe,
            shading,
            thickness,
            ktpt,
            enpt,
            itpt,
            ckpt,
            ckpt_side_lines,
            gobj,
            poti,
            area,
            came,
            jgpt,
            jgpt_lines,
            cnpt,
            mspt,
        } => {
            let img = draw::to_image(
                &path,
                &KclDrawOptions { wireframe, shading },
                &KmpDrawOptions {
                    thickness,
                    ktpt,
                    enpt,
                    itpt,
                    ckpt,
                    ckpt_side_lines,
                    gobj,
                    poti,
                    area,
                    came,
                    jgpt,
                    jgpt_lines,
                    cnpt,
                    mspt,
                    stgi: false,
                },
            );
            img.save("output.png").unwrap();
        }
    }

    Ok(())
}
