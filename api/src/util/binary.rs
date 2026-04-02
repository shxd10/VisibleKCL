#![allow(dead_code)]

use std::fs;
use std::path::Path;

pub fn read_u8(data: &[u8], offset: usize) -> Result<u8, String> {
    if data.len() < offset + 1 {
        return Err("Not enough data to read u8".into());
    }
    Ok(data[offset])
}

pub fn read_u32(data: &[u8], offset: usize) -> Result<u32, String> {
    if data.len() < offset + 4 {
        return Err("Not enough data to read u32".into());
    }
    Ok(u32::from_be_bytes(
        data[offset..offset + 4].try_into().unwrap(),
    ))
}

pub fn read_i32(data: &[u8], offset: usize) -> Result<i32, String> {
    if data.len() < offset + 4 {
        return Err("Not enough data to read i32".into());
    }
    Ok(i32::from_be_bytes(
        data[offset..offset + 4].try_into().unwrap(),
    ))
}

pub fn read_u24(data: &[u8], offset: usize) -> Result<u32, String> {
    if data.len() < offset + 3 {
        return Err("Not enough data to read u24".into());
    }
    let bytes = &data[offset..offset + 3];
    Ok(u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]))
}

pub fn read_u16(data: &[u8], offset: usize) -> Result<u16, String> {
    if data.len() < offset + 2 {
        return Err("Not enough data to read u16".into());
    }
    Ok(u16::from_be_bytes(
        data[offset..offset + 2].try_into().unwrap(),
    ))
}

pub fn read_i16(data: &[u8], offset: usize) -> Result<i16, String> {
    if data.len() < offset + 2 {
        return Err("Not enough data to read i16".into());
    }
    Ok(i16::from_be_bytes(
        data[offset..offset + 2].try_into().unwrap(),
    ))
}

pub fn read_vec_u8(data: &[u8], offset: usize, count: usize) -> Result<Vec<u8>, String> {
    if data.len() < offset + count {
        return Err("Not enough data to read u8 vec".into());
    }
    Ok(data[offset..offset + count].to_vec())
}

pub fn read_vec_u16(data: &[u8], offset: usize, count: usize) -> Result<Vec<u16>, String> {
    if data.len() < offset + count * 2 {
        return Err("Not enough data to read u16 vec".into());
    }
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        result.push(u16::from_be_bytes(
            data[offset + i * 2..offset + i * 2 + 2].try_into().unwrap(),
        ));
    }
    Ok(result)
}

pub fn read_vec_u32(data: &[u8], offset: usize, count: usize) -> Result<Vec<u32>, String> {
    if data.len() < offset + count * 4 {
        return Err("Not enough data to read u32 vec".into());
    }
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        result.push(u32::from_be_bytes(
            data[offset + i * 4..offset + i * 4 + 4].try_into().unwrap(),
        ));
    }
    Ok(result)
}

pub fn read_f32(data: &[u8], offset: usize) -> Result<f32, String> {
    if data.len() < offset + 4 {
        return Err("Not enough data to read f32".into());
    }
    Ok(f32::from_bits(u32::from_be_bytes(
        data[offset..offset + 4].try_into().unwrap(),
    )))
}

pub fn read_vec_f32(data: &[u8], offset: usize, count: usize) -> Result<Vec<f32>, String> {
    if data.len() < offset + count * 4 {
        return Err("Not enough data to read f32 vec".into());
    }
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        result.push(f32::from_bits(u32::from_be_bytes(
            data[offset + i * 4..offset + i * 4 + 4].try_into().unwrap(),
        )));
    }
    Ok(result)
}

// vector algebra
pub fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

pub fn path_to_data(path: &str, filetype: &str) -> Result<Vec<u8>, String> {
    if Path::new(path).extension().and_then(|s| s.to_str()) != Some(filetype) {
        return Err(format!("The file is not a .{} file.", filetype));
    }

    Ok(fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?)
}
