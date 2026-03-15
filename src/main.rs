use clap::{Parser, Subcommand};
use std::time::Instant;
use std::fs;
use std::path::Path;

mod util;
use crate::util::{szs, kmp, kcl, brres, draw};

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
        write_obj: bool
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

pub struct KmpOptions {
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

impl Default for KmpOptions {
    fn default() -> Self {
        KmpOptions {
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

pub struct KclOptions {
    wireframe: bool,
    shading: f32,
}

impl Default for KclOptions {
    fn default() -> Self {
        KclOptions {
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
        Command::Kcl { path, write_obj } => {
            let start = Instant::now();

            let filename = Path::new(&path).file_stem().unwrap().to_str().unwrap();

            let arc = szs::parse_course_files(&path)?;

            let object = kcl::to_obj(&arc.kcl, filename);
            let obj = object.obj;
            let mtl = object.mtl;
            
            if write_obj {
                write_obj_file(&obj, &mtl, filename);
            }

            let mut brres = arc.brres;
            brres::from_obj(&mut brres, &obj, &mtl);
            let buf = brres.write_memory().map_err(|e| e.to_string())?;
            fs::write("course_model.brres", buf).map_err(|e| e.to_string())?;

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
                &KclOptions { wireframe, shading },
                &KmpOptions { thickness, ktpt, enpt, itpt, ckpt, ckpt_side_lines, gobj, poti, area, came, jgpt, jgpt_lines, cnpt, mspt, stgi: false },
            );
            img.save("output.png").unwrap();
        }
    }

    Ok(())
}