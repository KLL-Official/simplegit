use std::collections::HashMap;
use sha1::{Sha1,Digest};
use std::fs;
use std::io::{BufReader,Read};
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
fn sha1_file(path:&Path)->String{
    let file=fs::File::open(path).expect("read file failed");
    let mut reader=BufReader::new(file);

    let mut hasher=Sha1::new();
    let mut buf=[0u8;8192];
    loop{
        let n=reader.read(&mut buf).expect("read file failed");
        if n==0{break;}
        hasher.update(&buf[..n]);
    }
    hex::encode(hasher.finalize())
}
fn add_file(map:&mut HashMap<String,String>,path:&Path){
    let val=sha1_file(path);
    map.insert(path.to_string_lossy().to_string(),val);
}
fn update_index(map:&mut HashMap<String,String>,path:&Path){
    let meta=fs::metadata(path).expect("read file failed");
    if meta.is_file(){
        add_file(map,path);
    }else if meta.is_dir(){
        for p in fs::read_dir(path).expect("read file failed"){
            let p1=p.expect("read file failed");
            let p1=p1.path();
            update_index(map,&p1);
        }
    }
}
pub fn run(work_dir:&PathBuf,paths:&Vec<PathBuf>){
    let git_dir=work_dir.join(".mygit");
    assert!(git_dir.exists(),"git does not exist:{}",git_dir.display());
    let objects_dir=git_dir.join("objects");
    let index_path=git_dir.join("index");

    let mut index_map=read_index(&index_path);

    for path in paths{
        let true_path=if path.is_absolute(){
            path.clone()
        } else{
            work_dir.join(path)
        };
        update_index(&mut index_map,&true_path);
    }
    for (key,val) in &index_map{
        let path=objects_dir.join(val.as_str());
        if !path.exists(){
            fs::copy(key.as_str(),&path).expect("copy failed");
        }
    }

    write_index(&index_path,&index_map);
}