use clap::ArgAction::Set;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::time::Instant;

use vkcl::CourseFiles;
use vkcl::enums::{self, Gobj};
use vkcl::{
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
        #[arg(long, short = 'd')]
        dest: Option<String>,
        #[arg(long, short = 'o', default_value_t = false)]
        overwrite: bool,

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
        #[arg(long, short = 'd')]
        dest: Option<String>,
        #[arg(long, short = 'o', default_value_t = false)]
        overwrite: bool,

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
        #[arg(long, default_value_t = false)]
        stgi: bool,
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
            dest,
            overwrite,
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

            let output = dest.unwrap_or_else(|| {
                let stem = Path::new(&path).file_stem().unwrap().to_str().unwrap();
                if overwrite {
                    path.clone()
                } else {
                    format!("{stem}_kcl.szs")
                }
            });

            vkcl::replace(
                &path,
                &output,
                &highlight,
                &special,
                &overlay_option,
                write_obj,
            )?;

            println!("Took: {:?}", start.elapsed());
        }
        Command::Overlay {
            path,
            dest,
            overwrite,
            write_obj,
            ckpt,
            ckpt_side,
            inv_walls,
            gobj,
        } => {
            let start = Instant::now();

            let overlay = OverlayOption {
                ckpt,
                ckpt_side,
                inv_walls,
                gobj,
            };

            let output = dest.unwrap_or_else(|| {
                let stem = Path::new(&path).file_stem().unwrap().to_str().unwrap();
                if overwrite {
                    path.clone()
                } else {
                    format!("{stem}_overlay.szs")
                }
            });

            vkcl::overlay(&path, &output, &overlay, write_obj);
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
            stgi,
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
                    stgi,
                },
            );
            img.save("output.png").unwrap();
        }
    }

    Ok(())
}
