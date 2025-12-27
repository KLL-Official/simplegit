use std::collections::HashMap;
use std::fs;
use std::path::{Path,PathBuf};

fn read_index(index_path:&Path)->HashMap<String,String>{
    let mut map=HashMap::new();

    let text=fs::read_to_string(index_path).expect("read index failed");

    for line in text.lines(){
        let parts:Vec<&str>=line.split_whitespace().collect();
        if parts.len()!=2{
            panic!("index is corrupted");
        }
        map.insert(parts[0].to_string(),parts[1].to_string());
    }
    map
}

fn write_index(index_path:&Path,map:&HashMap<String,String>){
    let mut s=String::new();
    let mut pairs:Vec<(&String,&String)>=map.into_iter().collect();
    pairs.sort_by(|a,b| a.0.cmp(&b.0));
    for (key,val) in pairs{
        s.push_str(key);
        s.push(' ');
        s.push_str(val);
        s.push('\n');
    }
    fs::write(index_path,s).expect("write index failed");
}
fn update_index(map:&mut HashMap<String,String>,path:&Path){
    let mut p=path.to_string_lossy().to_string();
    map.remove(&p);
    if let Ok(meta)=fs::metadata(path){
        if meta.is_file(){
            return;
        }
    }
    if !p.ends_with(std::path::MAIN_SEPARATOR){
        p.push(std::path::MAIN_SEPARATOR);
    }
    let paths:Vec<String>=map.keys().filter(|k| k.starts_with(&p)).cloned().collect();
    for k in paths{
        map.remove(&k);
    }
}
pub fn rm_cache(work_dir:&PathBuf,paths:&Vec<PathBuf>){
    let git_dir=work_dir.join(".mygit");
    assert!(git_dir.exists(),"git does not exist:{}",git_dir.display());
    let index_path=git_dir.join("index");

    let mut index_map=read_index(&index_path);

    for path in paths{
        let true_path=if path.is_absolute(){
            path.clone()
        } else{
            work_dir.join(&path)
        };
        update_index(&mut index_map,&true_path);
    }

    write_index(&index_path,&index_map);
}

pub fn rm(work_dir:&PathBuf,paths:&Vec<PathBuf>){
    rm_cache(work_dir,paths);
    for path in paths{
        let true_path=if path.is_absolute(){
            path.clone()
        }else{
            work_dir.join(&path)
        };
        if true_path.is_file(){
            let _=fs::remove_file(&true_path);
        }else if true_path.is_dir() {
            let _=fs::remove_dir_all(&true_path);
        }
    }
}