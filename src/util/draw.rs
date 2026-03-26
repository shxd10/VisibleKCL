use crate::{KclDrawOptions, KmpDrawOptions};
use image::{Rgb, RgbImage};

use super::binary::*;
use super::kcl::*;
use super::kmp::*;
use super::szs::{CourseFiles, parse_course_files};

fn fill_triangle(img: &mut RgbImage, a: (i32, i32), b: (i32, i32), c: (i32, i32), color: Rgb<u8>) {
    // bbox of the triangle
    let min_x = a.0.min(b.0).min(c.0).max(0);
    let max_x = a.0.max(b.0).max(c.0).min(img.width() as i32 - 1);
    let min_y = a.1.min(b.1).min(c.1).max(0);
    let max_y = a.1.max(b.1).max(c.1).min(img.height() as i32 - 1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if point_in_triangle(x, y, a, b, c) {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}

fn point_in_triangle(px: i32, py: i32, a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> bool {
    let sign = |p1: (i32, i32), p2: (i32, i32), p3: (i32, i32)| -> i32 {
        (p1.0 - p3.0) * (p2.1 - p3.1) - (p2.0 - p3.0) * (p1.1 - p3.1)
    };
    let d1 = sign((px, py), a, b);
    let d2 = sign((px, py), b, c);
    let d3 = sign((px, py), c, a);
    let has_neg = (d1 < 0) || (d2 < 0) || (d3 < 0);
    let has_pos = (d1 > 0) || (d2 > 0) || (d3 > 0);
    !(has_neg && has_pos)
}

fn draw_simple_line(img: &mut RgbImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgb<u8>) {
    // Bresenham's line algorithm
    let (mut x, mut y) = (x0, y0);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x >= 0 && x < img.width() as i32 && y >= 0 && y < img.height() as i32 {
            img.put_pixel(x as u32, y as u32, color);
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

// simply drawing 2 lines close to eachother
fn draw_line(img: &mut RgbImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgb<u8>, thickness: u32) {
    let half = (thickness / 2) as i32;
    for i in -half..=half {
        draw_simple_line(img, x0 + i, y0, x1 + i, y1, color);
        draw_simple_line(img, x0, y0 + i, x1, y1 + i, color);
    }
}

fn draw_circle(img: &mut RgbImage, cx: i32, cy: i32, r: i32, color: Rgb<u8>, fill: bool) {
    let mut x = r;
    let mut y = 0;
    let mut err = 0;

    // some bullshit algorithm for circles
    while x >= y {
        if fill {
            for px in (cx - x)..=(cx + x) {
                for py in [cy + y, cy - y] {
                    if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32 {
                        img.put_pixel(px as u32, py as u32, color);
                    }
                }
            }
            for px in (cx - y)..=(cx + y) {
                for py in [cy + x, cy - x] {
                    if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32 {
                        img.put_pixel(px as u32, py as u32, color);
                    }
                }
            }
        } else {
            let points = [
                (cx + x, cy + y),
                (cx - x, cy + y),
                (cx + x, cy - y),
                (cx - x, cy - y),
                (cx + y, cy + x),
                (cx - y, cy + x),
                (cx + y, cy - x),
                (cx - y, cy - x),
            ];
            for (px, py) in points {
                if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32 {
                    img.put_pixel(px as u32, py as u32, color);
                }
            }
        }
        y += 1;
        if err <= 0 {
            err += 2 * y + 1;
        } else {
            x -= 1;
            err += 2 * (y - x) + 1;
        }
    }
}

struct CircleDraw {
    r: i32,
    outline_color: Rgb<u8>,
}

impl Default for CircleDraw {
    fn default() -> Self {
        let thickness = KmpDrawOptions::default().thickness as i32;
        CircleDraw {
            r: thickness,
            outline_color: Rgb([0, 0, 0]),
        }
    }
}

fn draw_point(img: &mut RgbImage, x: i32, y: i32, color: Rgb<u8>, CircleDraw { r, outline_color }: CircleDraw) {
    draw_circle(img, x, y, r, color, true);
    draw_circle(img, x, y, r + 1, outline_color, false);
}

fn draw_point_vector(
    img: &mut RgbImage,
    posx: i32,
    posy: i32,
    roty: f32,
    color: Rgb<u8>,
    thickness: u32,
    circle: CircleDraw,
) {
    let outline_color = circle.outline_color;
    let len = (thickness * 3) as f32;
    draw_point(img, posx, posy, color, circle);
    // for some reason the original rotation is 90deg off?
    let angle = (roty - 90.0).to_radians();
    let ex = posx + (len * angle.cos()) as i32;
    let ey = posy - (len * angle.sin()) as i32;
    draw_simple_line(img, posx, posy, ex, ey, outline_color);
}

fn draw_route(img: &mut RgbImage, points: &[(i32, i32)], color: Rgb<u8>, thickness: u32) {
    for i in 0..points.len() - 1 {
        let (x0, y0) = points[i];
        let (x1, y1) = points[i + 1];
        draw_line(img, x0, y0, x1, y1, color, thickness);
    }
}

fn shade_color(color: Rgb<u8>, brightness: f32) -> Rgb<u8> {
    Rgb([
        (color[0] as f32 * brightness) as u8,
        (color[1] as f32 * brightness) as u8,
        (color[2] as f32 * brightness) as u8,
    ])
}

fn draw_kcl(img: &mut RgbImage, parsed: &ParsedKcl, to_pixel: &dyn Fn(f32, f32) -> (i32, i32), options: &KclDrawOptions) {
    // define draw priority for the more important and less important flags (on top and under)
    let priority = |flag: &Flag| -> u8 {
        match flag.base_type {
            BaseType::FallBoundary | BaseType::SolidFall => 0,
            BaseType::Wall | BaseType::Wall2 | BaseType::InvisibleWall | BaseType::InvisibleWall2 => 1,
            _ => 2,
        }
    };

    let mut prisms: Vec<_> = parsed.sections.prisms.iter().collect();
    prisms.sort_by_key(|p| priority(&p.flag));

    let positions = &parsed.sections.position_vectors;
    for prism in prisms {
        let v1 = positions[prism.pos_i as usize];
        let fnrm = parsed.sections.normals[prism.fnrm_i as usize];
        let enrm1 = parsed.sections.normals[prism.enrm1_i as usize];
        let enrm2 = parsed.sections.normals[prism.enrm2_i as usize];
        let enrm3 = parsed.sections.normals[prism.enrm3_i as usize];
        let cross_a = cross(enrm1, fnrm);
        let cross_b = cross(enrm2, fnrm);
        let v2 = add(v1, scale(cross_b, prism.height / dot(cross_b, enrm3)));
        let v3 = add(v1, scale(cross_a, prism.height / dot(cross_a, enrm3)));

        if !v2[0].is_finite() || !v2[2].is_finite() || !v3[0].is_finite() || !v3[2].is_finite() {
            continue;
        }

        let [r, g, b, _] = prism.flag.base_type.color();
        let color = Rgb([r, g, b]);
        let (ax, az) = to_pixel(v1[0], v1[2]);
        let (bx, bz) = to_pixel(v2[0], v2[2]);
        let (cx, cz) = to_pixel(v3[0], v3[2]);

        if options.wireframe {
            draw_simple_line(img, ax, az, bx, bz, color);
            draw_simple_line(img, bx, bz, cx, cz, color);
            draw_simple_line(img, cx, cz, ax, az, color);
        } else {
            let shading = options.shading;
            let brightness = shading + (fnrm[1].abs()) * shading;
            let shaded_color = shade_color(color, brightness);
            fill_triangle(img, (ax, az), (bx, bz), (cx, cz), shaded_color);
        }
    }
}

// lorenzi's kmp editor colors
fn cp_colors(cp_type: &CheckPointType) -> Rgb<u8> {
    let (r, g, b) = match cp_type {
        CheckPointType::FinishLine => (255, 127, 255),
        CheckPointType::KeyCheckPoint => (255, 0, 255),
        CheckPointType::CheckPoint => (0, 0, 255),
    };
    Rgb([r, g, b])
}

// you guessed it, lorenzi's colors
// (similar atleast, i adjusted a bit)
struct PointColors {
    ktpt: Rgb<u8>,

    enpt: Rgb<u8>,
    first_enpt: Rgb<u8>,
    enpt_line: Rgb<u8>,

    itpt: Rgb<u8>,
    first_itpt: Rgb<u8>,
    itpt_line: Rgb<u8>,

    ckpt_side_lines: Rgb<u8>,
    gobj: Rgb<u8>,

    poti: Rgb<u8>,
    first_poti: Rgb<u8>,
    poti_line: Rgb<u8>,

    area: Rgb<u8>,
    came: Rgb<u8>,
    jgpt: Rgb<u8>,
    cnpt: Rgb<u8>,
    mspt: Rgb<u8>
}

impl PointColors {
    fn new() -> Self {
        PointColors {
            ktpt: Rgb([0, 0, 255]),
            enpt: Rgb([255, 0, 0]),
            first_enpt: Rgb([128, 0, 0]),
            enpt_line: Rgb([255, 128, 0]),

            itpt: Rgb([0, 255, 0]),
            first_itpt: Rgb([0, 128, 0]),
            itpt_line: Rgb([128, 255, 0]),

            ckpt_side_lines: Rgb([0, 255, 255]),
            gobj: Rgb([185, 21, 207]),

            poti: Rgb([0, 150, 222]),
            first_poti: Rgb([0, 50, 222]),
            poti_line: Rgb([0, 200, 222]),

            area: Rgb([250, 150, 0]),
            came: Rgb([128, 26, 179]),
            jgpt: Rgb([255, 255, 0]),
            cnpt: Rgb([255, 0, 0]),
            mspt: Rgb([128, 0, 255])
        }
    }
}

struct CPDraw {
    left: (i32, i32),
    right: (i32, i32),
}

struct ConnectedPointsDraw {
    points: Vec<(i32, i32)>,
    line_color: Rgb<u8>,
    thickness: u32,
}

fn draw_connected_points(img: &mut RgbImage, ConnectedPointsDraw { points, line_color, thickness }: ConnectedPointsDraw) {
    for (i, &(x, z)) in points.iter().enumerate() {
        if let Some(&(nx, nz)) = points.get(i + 1) {
            draw_line(img, x, z, nx, nz, line_color, thickness);
        }
    }
}

fn draw_ktpt(img: &mut RgbImage, ktpt: &Section<KTPT>, to_pixel: &dyn Fn(f32, f32) -> (i32, i32), thickness: u32) {
    let color = PointColors::new().ktpt;
    for entry in ktpt.entries.iter() {
        let (px, pz) = to_pixel(entry.pos[0], entry.pos[2]);
        let ry = entry.rot[1];
        draw_point_vector(img, px, pz, ry, color, thickness, CircleDraw::default());
    }
}

fn draw_enpt(img: &mut RgbImage, enpt: &Section<ENPT>, enph: &Section<ENPH>, to_pixel: &dyn Fn(f32, f32) -> (i32, i32), thickness: u32) {
    let color = PointColors::new().enpt;
    let first_color = PointColors::new().first_enpt;

    let mut points: Vec<(i32, i32)> = vec![];
    for entry in &enpt.entries {
        points.push(to_pixel(entry.pos[0], entry.pos[2]));
    }

    for (i, &(x, z)) in points.iter().enumerate() {
        match i {
            0 => draw_point(img, x, z, first_color, CircleDraw::default()),
            _ => draw_point(img, x, z, color, CircleDraw::default()),
        };
    }

    for (i, group) in enph.entries.iter().enumerate() {
        let start = group.point_start as usize;
        let end = start + group.point_len as usize;
        let group_points = points[start..end].to_vec();

        draw_connected_points(img, ConnectedPointsDraw {
            points: group_points,
            line_color: PointColors::new().enpt_line,
            thickness,
        });

        // connect last point of this group to first point of each next group
        let last = &points[end - 1];
        for &next_gi in group.next_group.iter().filter(|&&i| i != 0xFF) {
            let next_group = &enph.entries[next_gi as usize];
            let next_first = points[next_group.point_start as usize];
            draw_line(img, last.0, last.1, next_first.0, next_first.1, PointColors::new().enpt_line, thickness);
        }
    }
}

fn draw_itpt(img: &mut RgbImage, itpt: &Section<ITPT>, itph: &Section<ITPH>, to_pixel: &dyn Fn(f32, f32) -> (i32, i32), thickness: u32) {
    let color = PointColors::new().itpt;
    let first_color = PointColors::new().first_itpt;

    let mut points: Vec<(i32, i32)> = vec![];
    for entry in &itpt.entries {
        points.push(to_pixel(entry.pos[0], entry.pos[2]));
    }

    for (i, &(x, z)) in points.iter().enumerate() {
        match i {
            0 => draw_point(img, x, z, first_color, CircleDraw::default()),
            _ => draw_point(img, x, z, color, CircleDraw::default()),
        };
    }

    for (i, group) in itph.entries.iter().enumerate() {
        let start = group.point_start as usize;
        let end = start + group.point_len as usize;
        let group_points = points[start..end].to_vec();

        draw_connected_points(img, ConnectedPointsDraw {
            points: group_points,
            line_color: PointColors::new().itpt_line,
            thickness,
        });

        // connect last point of this group to first point of each next group
        let last = &points[end - 1];
        for &next_gi in group.next_group.iter().filter(|&&i| i != 0xFF) {
            let next_group = &itph.entries[next_gi as usize];
            let next_first = points[next_group.point_start as usize];
            draw_line(img, last.0, last.1, next_first.0, next_first.1, PointColors::new().itpt_line, thickness);
        }
    }
}

fn draw_ckpt(img: &mut RgbImage, ckpt: &Section<CKPT>, ckph: &Section<CKPH>, to_pixel: &dyn Fn(f32, f32) -> (i32, i32), thickness: u32, side: bool) {
    let mut checkpoints: Vec<CPDraw> = vec![];
    for checkpoint in &ckpt.entries {
        let color = cp_colors(&checkpoint.checkpoint_type());
        let (x0, z0) = to_pixel(checkpoint.left_point[0], checkpoint.left_point[1]);
        let (x1, z1) = to_pixel(checkpoint.right_point[0], checkpoint.right_point[1]);
        checkpoints.push(CPDraw {
            left: (x0, z0),
            right: (x1, z1),
        });
        draw_line(img, x0, z0, x1, z1, color, thickness);
    }
    if side {
        let color = PointColors::new().ckpt_side_lines;
        let thickness = thickness / 2;
        for group in ckph.entries.iter() {
            let start = group.first_cp as usize;
            let end = start + group.cp_count as usize;

            for i in start..end - 1 {
                let cp = &checkpoints[i];
                let next = &checkpoints[i + 1];
                draw_line(img, cp.left.0, cp.left.1, next.left.0, next.left.1, color, thickness);
                draw_line(img, cp.right.0, cp.right.1, next.right.0, next.right.1, color, thickness);
            }

            let last = &checkpoints[end - 1];
            for &next_gi in group.next_groups.iter().filter(|&&i| i != 0xFF) {
                let next_group = &ckph.entries[next_gi as usize];
                let next_first = &checkpoints[next_group.first_cp as usize];
                draw_line(img, last.left.0, last.left.1, next_first.left.0, next_first.left.1, color, thickness);
                draw_line(img, last.right.0, last.right.1, next_first.right.0, next_first.right.1, color, thickness);
            }
        }
    }
}

fn draw_gobj(img: &mut RgbImage, gobj: &Section<GOBJ>, to_pixel: &dyn Fn(f32, f32) -> (i32, i32), thickness: u32) {
    let color = PointColors::new().gobj;
    for entry in gobj.entries.iter() {
        let (px, pz) = to_pixel(entry.pos[0], entry.pos[2]);
        let ry = entry.rot[1];
        draw_point_vector(img, px, pz, ry, color, thickness, CircleDraw::default());
    }
}

fn draw_poti(img: &mut RgbImage, poti: &Section<POTI>, to_pixel: &dyn Fn(f32, f32) -> (i32, i32), thickness: u32) {
    for route in &poti.entries {
        let mut points: Vec<(i32, i32)> = vec![];
        for entry in &route.points {
            points.push(to_pixel(entry.pos[0], entry.pos[2]));
        }
        for (i, &(x, z)) in points.iter().enumerate() {
            match i {
                0 => draw_point(img, x, z, PointColors::new().first_poti, CircleDraw::default()),
                _ => draw_point(img, x, z, PointColors::new().poti, CircleDraw::default()),
            };
        }
        draw_connected_points(img, ConnectedPointsDraw { 
            points: points,
            line_color: PointColors::new().poti_line,
            thickness: thickness,
        });
    }
}

fn draw_area(img: &mut RgbImage, area: &Section<AREA>, to_pixel: &dyn Fn(f32, f32) -> (i32, i32), thickness: u32) {
    let color = PointColors::new().area;
    for entry in area.entries.iter() {
        let (px, pz) = to_pixel(entry.pos[0], entry.pos[2]);
        let ry = entry.rot[1];
        draw_point_vector(img, px, pz, ry, color, thickness, CircleDraw::default());
    }
}

fn draw_came(img: &mut RgbImage, came: &Section<CAME>, to_pixel: &dyn Fn(f32, f32) -> (i32, i32), thickness: u32) {
    let color = PointColors::new().came;
    for entry in came.entries.iter() {
        let (px, pz) = to_pixel(entry.pos[0], entry.pos[2]);
        let ry = entry.rot[1];
        draw_point_vector(img, px, pz, ry, color, thickness, CircleDraw::default());
    }
}

fn draw_jgpt(img: &mut RgbImage, jgpt: &Section<JGPT>, ckpt: &Section<CKPT>, to_pixel: &dyn Fn(f32, f32) -> (i32, i32), thickness: u32, lines: bool) {
    let color = PointColors::new().jgpt;
    for (i, respawn) in jgpt.entries.iter().enumerate() {
        let (px, pz) = to_pixel(respawn.pos[0], respawn.pos[2]);
        let ry = respawn.rot[1];
        if lines {
            for checkpoint in &ckpt.entries {
                if i as u8 == checkpoint.respawn_index {
                    let (x1, z1) = to_pixel(checkpoint.right_point[0], checkpoint.right_point[1]);
                    draw_line(img, px, pz, x1, z1, color, thickness / 2);
                }
            }
        }
        draw_point_vector(img, px, pz, ry, color, thickness, CircleDraw::default());
    }
}

fn draw_cnpt(img: &mut RgbImage, cnpt: &Section<CNPT>, to_pixel: &dyn Fn(f32, f32) -> (i32, i32), thickness: u32) {
    let color = PointColors::new().cnpt;
    for entry in cnpt.entries.iter() {
        let (px, pz) = to_pixel(entry.dest_pos[0], entry.dest_pos[2]);
        let ry = entry.release_angle[1];
        draw_point_vector(img, px, pz, ry, color, thickness, CircleDraw::default());
    }
}

fn draw_mspt(img: &mut RgbImage, mspt: &Section<MSPT>, to_pixel: &dyn Fn(f32, f32) -> (i32, i32), thickness: u32) {
    let color = PointColors::new().mspt;
    for entry in mspt.entries.iter() {
        let (px, pz) = to_pixel(entry.pos[0], entry.pos[2]);
        let ry = entry.rot[1];
        draw_point_vector(img, px, pz, ry, color, thickness, CircleDraw::default());
    }
}

fn draw_kmp(img: &mut RgbImage, parsed: &ParsedKmp, to_pixel: &dyn Fn(f32, f32) -> (i32, i32), options: &KmpDrawOptions) {
    let ckpt = &parsed.ckpt;
    let thickness = options.thickness;

    if options.ktpt { draw_ktpt(img, &parsed.ktpt, to_pixel, thickness); }
    if options.enpt { draw_enpt(img, &parsed.enpt, &parsed.enph, to_pixel, thickness / 2); }
    if options.itpt { draw_itpt(img, &parsed.itpt, &parsed.itph, to_pixel, thickness / 2); }
    if options.gobj { draw_gobj(img, &parsed.gobj, to_pixel, thickness); }
    if options.poti { draw_poti(img, &parsed.poti, to_pixel, thickness / 2); }
    if options.area { draw_area(img, &parsed.area, to_pixel, thickness); }
    if options.came { draw_came(img, &parsed.came, to_pixel, thickness); }
    if options.jgpt { draw_jgpt(img, &parsed.jgpt, ckpt, to_pixel, thickness, options.jgpt_lines); }
    if options.cnpt { draw_cnpt(img, &parsed.cnpt, to_pixel, thickness); }
    if options.mspt { draw_mspt(img, &parsed.mspt, to_pixel, thickness); }
    // cps drawn later because, well, they're the main point
    if options.ckpt { draw_ckpt(img, ckpt, &parsed.ckph, to_pixel, thickness, options.ckpt_side_lines); }
}

pub fn to_image(szs_path: &str, kcl_options: &KclDrawOptions, kmp_options: &KmpDrawOptions) -> RgbImage {
    let parsed = parse_course_files(szs_path).expect("failed to parse kmp/kcl");
    let kcl = &parsed.kcl;
    let kmp = &parsed.kmp;

    // bbox
    let used_positions: Vec<[f32; 3]> = kcl.sections.prisms.iter()
        // ignore fall boundaries on bounding box for those tracks with extended fall boundaries
        .filter(|p| !matches!(p.flag.base_type, BaseType::FallBoundary | BaseType::SolidFall))
        .map(|p| kcl.sections.position_vectors[p.pos_i as usize])
        .filter(|p| p[0].is_finite() && p[2].is_finite())
        .collect();

    let mut xs: Vec<f32> = used_positions.iter().map(|p| p[0]).collect();
    let mut zs: Vec<f32> = used_positions.iter().map(|p| p[2]).collect();

    // also include kmp points in bbox so nothing gets cut off
    for cp in &kmp.ckpt.entries {
        xs.push(cp.left_point[0]);
        xs.push(cp.right_point[0]);
        zs.push(cp.left_point[1]);
        zs.push(cp.right_point[1]);
    }

    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    zs.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min_x = xs[0];
    let max_x = xs[xs.len() - 1];
    let min_z = zs[0];
    let max_z = zs[zs.len() - 1];

    let padding_x = (max_x - min_x) * 0.05;
    let padding_z = (max_z - min_z) * 0.05;
    let min_x = min_x - padding_x;
    let max_x = max_x + padding_x;
    let min_z = min_z - padding_z;
    let max_z = max_z + padding_z;

    let target_res = 2048.0_f32;
    let track_width = max_x - min_x;
    let track_height = max_z - min_z;
    let scale = (target_res / track_width).min(target_res / track_height);

    let img_width = (track_width * scale) as u32;
    let img_height = (track_height * scale) as u32;

    let to_pixel = |x: f32, z: f32| -> (i32, i32) {
        (((x - min_x) * scale) as i32, ((z - min_z) * scale) as i32)
    };

    let mut img = RgbImage::new(img_width, img_height);
    draw_kcl(&mut img, kcl, &to_pixel, kcl_options);
    draw_kmp(&mut img, kmp, &to_pixel, kmp_options);
    img
}