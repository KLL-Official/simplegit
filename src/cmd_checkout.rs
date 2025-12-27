use std::fs;
use std::path::{Path,PathBuf};
use std::collections::HashMap;

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


pub fn switch(work_dir:&PathBuf,name:&String){
    let git_dir=work_dir.join(".mygit");
    assert!(git_dir.exists(),"git does not exist:{}",git_dir.display());
    let objects_dir=git_dir.join("objects");
    let heads_dir=git_dir.join("refs").join("heads");

    let mut commit_hash=(*name).clone();
    let mut head_val=format!("{}\n",commit_hash);

    for p in fs::read_dir(&heads_dir).expect("read file failed"){
        let p1=p.expect("read file failed");
        let p1=p1.path();
        if let Some(fname)=p1.file_name(){
            let fname=fname.to_string_lossy().to_string();
            if fname==*name{
                commit_hash=fs::read_to_string(heads_dir.join(name)).expect("read head failed");
                commit_hash=commit_hash.trim().to_string();
                head_val=format!("ref: refs/heads/{}\n",name);
                break;
            }
        }
    }

    let commit=fs::read_to_string(objects_dir.join(&commit_hash)).expect("read commit failed");

    let mut tree_hash=String::new();
    for line in commit.lines(){
        if line.starts_with("tree "){
            tree_hash=line[5..].trim().to_string();
        }
    }

    let tree=read_index(&objects_dir.join(&tree_hash));
    let index=read_index(&git_dir.join("index"));

    for (key,_) in &index{
        let p=Path::new(key);
        if !tree.contains_key(key){
            if p.is_file(){
                let _=fs::remove_file(&p);
            }else if p.is_dir() {
                let _=fs::remove_dir_all(&p);
            }
        }
    }
    for (key,val) in &tree{
        let p=Path::new(key);
        if let Some(parent)=p.parent(){
            fs::create_dir_all(parent).expect("create dir failed");
        }
        let blob_path=objects_dir.join(val);
        fs::copy(blob_path,p).expect("copy failed");
    }
    write_index(&git_dir.join("index"),&tree);
    fs::write(git_dir.join("HEAD"),head_val).expect("write HEAD failed");
}