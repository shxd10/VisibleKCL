use std::fs;
use std::path::{Path, PathBuf};
use super::{kcl, kmp, binary::*};
use anyhow::Result;

// https://wiki.tockdom.com/wiki/U8_(File_Format)

pub struct Header {
    pub magic: u32,
    pub first_node: i32,
    pub node_size: i32,
    pub data_offset: i32,
    _reserved: [i32; 4],
}

pub struct Node {
    pub node_type: u8,
    pub string_offset: u32, // this is actually u24
    pub data_offset: u32, // Index of parent directory
    pub data_size: u32, // Index of first node that is not part of this directory
}

pub struct ParsedNode {
    pub node_type: u8,
    pub name: String,
    pub data_offset: u32,
    pub data_size: u32,
}

pub struct ParsedArc {
    pub header: Header,
    pub nodes: Vec<ParsedNode>,
    data: Vec<u8>,
}

impl Header {
    fn parse(data: &[u8]) -> Result<Self, String> {
        let magic = read_u32(data, 0x00)?;
        if magic != 0x55AA382D {
            return Err("Invalid magic number".into());
        }
        
        let first_node = read_i32(data, 0x04)?;
        let node_size = read_i32(data, 0x08)?;
        let data_offset = read_i32(data, 0x0C)?;
        let _reserved = [
            read_i32(data, 0x10)?,
            read_i32(data, 0x14)?,
            read_i32(data, 0x18)?,
            read_i32(data, 0x1C)?,
        ];

        Ok(Header { magic, first_node, node_size, data_offset, _reserved })
    }
}

impl Node {
    fn parse(node_offset: &[u8]) -> Result<Self, String> {
        let node_type = node_offset[0];
        let string_offset = read_u24(node_offset, 0x01)?;
        let data_offset = read_u32(node_offset, 0x04)?;
        let data_size = read_u32(node_offset, 0x08)?;

        Ok(Node { node_type, string_offset, data_offset, data_size })
    }
}

fn decode_yaz0(data: &[u8]) -> Result<Vec<u8>, String> {
    szs::decode(data).map_err(|e| format!("Failed to decode: {}", e))
}

fn parse_arc(data: &[u8]) -> Result<ParsedArc, String> {
    let header = Header::parse(data)?;

    let first_node = Node::parse(&data[header.first_node as usize..])?;
    let total_nodes = first_node.data_size as usize;

    let string_pool_start = header.first_node as usize + total_nodes * 0x0C;

    let mut parsed_nodes: Vec<ParsedNode> = Vec::new();

    for i in 0..total_nodes {
        let offset = header.first_node as usize + i * 0x0C;
        let node = Node::parse(&data[offset..])?;
        let string = &data[string_pool_start + node.string_offset as usize..];
        let name = String::from_utf8_lossy(string.split(|&b| b == 0).next().unwrap_or(&[])).to_string();

        let parsed_node = ParsedNode {
            node_type: node.node_type,
            name,
            data_offset: node.data_offset,
            data_size: node.data_size,
        };

        parsed_nodes.push(parsed_node);
    }
    Ok(ParsedArc { header, nodes: parsed_nodes, data: data.to_vec() })
}

pub fn parse(data: &[u8]) -> Result<ParsedArc, String> {
    let decoded = decode_yaz0(data)?;
    parse_arc(&decoded)
}

pub fn parse_from_path(path: &str) -> Result<ParsedArc, String> {
    let data = path_to_data(path, "szs")?;
    parse(&data)
}

pub fn extract(path: &str) -> Result<String, String> {
    let parsed_szs = parse_from_path(path)?;

    let folder = Path::new(path).with_extension("d");
    fs::create_dir_all(&folder)
        .map_err(|e| format!("Failed to create dir: {}", e))?;

    let mut dir_stack: Vec<(usize, PathBuf)> = Vec::new();

    for (i, node) in parsed_szs.nodes.iter().enumerate() {
        // remove previous current dirs
        while let Some((end, _)) = dir_stack.last() {
            if i >= *end {
                dir_stack.pop();
            } else {
                break;
            }
        }
        
        // some wacky workaround for subfolders and files
        let current_dir = dir_stack.last().map(|(_, p)| p.as_path()).unwrap_or(folder.as_path());
        let node_path = current_dir.join(&node.name);

        match node.node_type {
            0 => {
                let file_data = &parsed_szs.data[node.data_offset as usize..(node.data_offset + node.data_size) as usize];
                fs::write(&node_path, file_data)
                    .map_err(|e| format!("Failed to write file: {}", e))?;
            }
            1 => {
                if node.name == "" || node.name == "." {
                    continue;
                }
                fs::create_dir_all(&node_path)
                    .map_err(|e| format!("Failed to create dir: {}", e))?;
                dir_stack.push((node.data_size as usize, node_path));
            }
            _ => return Err(format!("Unknown node type: {}", node.node_type)),
        }
    }

    Ok(folder.to_str().unwrap_or("Unknown folder").to_string())
}

pub struct CourseFiles {
    pub kmp: kmp::ParsedKmp,
    pub kcl: kcl::ParsedKcl,
    pub brres: brres::Archive,
}

pub fn parse_course_files(path: &str) -> Result<CourseFiles, String> {
    let parsed = parse_from_path(path)?;

    let kmp = parsed.nodes.iter()
        .find(|n| n.name == "course.kmp")
        .map(|n| parsed.data[n.data_offset as usize..(n.data_offset + n.data_size) as usize].to_vec())
        .ok_or("course.kmp not found")?;

    let kcl = parsed.nodes.iter()
        .find(|n| n.name == "course.kcl")
        .map(|n| parsed.data[n.data_offset as usize..(n.data_offset + n.data_size) as usize].to_vec())
        .ok_or("course.kcl not found")?;

    let brres = parsed.nodes.iter()
        .find(|n| n.name == "course_model.brres")
        .map(|n| parsed.data[n.data_offset as usize..(n.data_offset + n.data_size) as usize].to_vec())
        .ok_or("course_model.brres not found")?;

    Ok(CourseFiles {
        kmp: kmp::parse(&kmp)?,
        kcl: kcl::parse(&kcl)?,
        brres: super::brres::parse(&brres).map_err(|e| e.to_string())?,
    })
}