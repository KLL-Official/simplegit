use std::fs;
use std::collections::HashMap;
use std::path::{Path,PathBuf};
use sha1::{Sha1,Digest};

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

fn sha1_string(s:&String)->String{
    let mut hasher=Sha1::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}


fn get_head(git_dir:&Path,path:&Path)->(PathBuf,Option<String>){
    let head=match fs::read_to_string(path){
        Ok(s)=>s,
        Err(_)=>String::new(),
    };
    let head=head.trim();
    if head.starts_with("ref: "){
        let ph=&head[5..];
        let head_path=git_dir.join(ph);
        let parent=match fs::read_to_string(&head_path){
            Ok(s)=>{
                let t=s.trim();
                if t.is_empty(){
                    None
                } else{
                    Some(t.to_string())
                }
            }
            Err(_)=>None,
        };
        return (head_path,parent);
    }
    let parent=if head.is_empty(){
        None
    }else{
        Some(head.to_string())
    };
    return (path.to_path_buf(),parent);
}
pub fn run(work_dir:&PathBuf,message:&Option<String>){
    let git_dir=work_dir.join(".mygit");
    assert!(git_dir.exists(),"git dir does not exist:{}",git_dir.display());


    let objects_dir=git_dir.join("objects");
    let index_path=git_dir.join("index");
    let head_path=git_dir.join("HEAD");


    let index_map=read_index(&index_path);

    if index_map.is_empty(){
        panic!("nothing to commit");
    }

    let mut pairs:Vec<(String,String)>=index_map.into_iter().collect();
    pairs.sort_by(|a,b| a.0.cmp(&b.0));

    let mut tree=String::new();
    for (key,val) in &pairs{
        tree.push_str(key);
        tree.push(' ');
        tree.push_str(val);
        tree.push('\n');
    }

    let tree_hash=sha1_string(&tree);
    fs::write(objects_dir.join(&tree_hash),tree).expect("write object failed");

    
    let(hpath,parent)=get_head(&git_dir,&head_path);


    let mut commit=String::new();
    commit.push_str("tree ");
    commit.push_str(&tree_hash);
    commit.push('\n');
    if let Some(p)=parent{
        commit.push_str("parent ");
        commit.push_str(&p);
        commit.push('\n');
    }
    if let Some(m)=message{
        commit.push_str("message ");
        commit.push_str(m);
        commit.push('\n');
    }
    let commit_hash=sha1_string(&commit);
    fs::write(objects_dir.join(&commit_hash),&commit).expect("write object failed");

    fs::write(hpath,format!("{commit_hash}\n")).expect("write refs/heads failed");
}