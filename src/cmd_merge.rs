use std::fs;
use std::path::{Path,PathBuf};
use std::collections::HashMap;
use std::collections::HashSet;
use sha1::{Sha1,Digest};

use crate::cmd_checkout;

fn get_commit_hash(heads_dir:&PathBuf,name:&String)->String{
    let p=heads_dir.join(name);
    assert!(p.exists(),"branch does not exist: {}",name);
    fs::read_to_string(p).expect("read head failed").trim().to_string()
}

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
    for (key,val) in map{
        s.push_str(key);
        s.push(' ');
        s.push_str(val);
        s.push('\n');
    }
    fs::write(index_path,s).expect("write index failed");
}

fn sha1_string(s:&String)->String{
    let mut hasher=Sha1::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}

fn get_index_from_commit(objects_dir:&PathBuf,name:&String)->HashMap<String,String>{
    let commit=fs::read_to_string(objects_dir.join(&name)).expect("read commit failed");

    let mut tree_hash=String::new();
    for line in commit.lines(){
        if line.starts_with("tree "){
            tree_hash=line[5..].trim().to_string();
        }
    }
    read_index(&objects_dir.join(&tree_hash))
}

pub fn merge(work_dir:&PathBuf,name:&String){
    let git_dir=work_dir.join(".mygit");
    assert!(git_dir.exists(),"git does not exist:{}",git_dir.display());
    let objects_dir=git_dir.join("objects");

    let head_path=git_dir.join("HEAD");
    let mut head=match fs::read_to_string(&head_path){
        Ok(s)=>s,
        Err(_)=>String::new(),
    };
    head=head.trim().to_string();
    if head.starts_with("ref: "){
        let ph=&head[5..];
        let ph=Path::new(ph);
        if let Some(name)=ph.file_name(){
            head=name.to_string_lossy().to_string();
        }
    }else{
        panic!("detached: {head}");
    }
    let heads_dir=git_dir.join("refs").join("heads");
    let cur_commit_hash=get_commit_hash(&heads_dir,&head);
    let meg_commit_hash=get_commit_hash(&heads_dir,&name);


    let mut base_commit_hash=String::new();
    let mut fa_set:HashSet<String>=HashSet::new();
    
    let mut jump=cur_commit_hash.clone();
    loop{
        fa_set.insert(jump.clone());
        let commit_text=fs::read_to_string(objects_dir.join(&jump)).expect("read commit failed");
        let mut flag:bool=false;
        for line in commit_text.lines(){
            if line.starts_with("parent "){
                flag=true;
                jump=line[7..].trim().to_string();
                break;
            }
        }
        if flag==false{
            break;
        }
    }

    jump=meg_commit_hash.clone();
    loop{
        if fa_set.contains(&jump){
            base_commit_hash=jump.clone();
            break;
        }
        let commit_text=fs::read_to_string(objects_dir.join(&jump)).expect("read commit failed");
        let mut flag:bool=false;
        for line in commit_text.lines(){
            if line.starts_with("parent "){
                flag=true;
                jump=line[7..].trim().to_string();
                break;
            }
        }
        if flag==false{
            break;
        }
    }

    if base_commit_hash.is_empty() {
        panic!("no common ancestor");
    }

    if base_commit_hash==cur_commit_hash{
        fs::write(&heads_dir.join(&head),meg_commit_hash).expect("write refs/heads failed");
        cmd_checkout::switch(work_dir,&head);
        println!("fast forward!");
        return;
    }else if base_commit_hash==meg_commit_hash{
        println!("Already up to date!");
        return;
    }

    let cur_map=get_index_from_commit(&objects_dir,&cur_commit_hash);
    let meg_map=get_index_from_commit(&objects_dir,&meg_commit_hash);
    let base_map=get_index_from_commit(&objects_dir,&base_commit_hash);

    let mut keys:HashSet<String>=HashSet::new();
    for k in base_map.keys(){
        keys.insert(k.clone());
    }
    for k in cur_map.keys(){
        keys.insert(k.clone());
    }
    for k in meg_map.keys(){
        keys.insert(k.clone());
    }
    let mut tree:HashMap<String,String>=HashMap::new();
    for k in keys{
        let cur_val:Option<&String>=cur_map.get(&k);
        let meg_val:Option<&String>=meg_map.get(&k);
        let base_val:Option<&String>=base_map.get(&k);
        if cur_val==meg_val{
            if let Some(val)=cur_val{
                tree.insert(k.clone(),val.clone());
            }
        }else{
            if cur_val==base_val{
                if let Some(val)=meg_val{
                    tree.insert(k.clone(),val.clone());
                }
            }else if meg_val==base_val{
                if let Some(val)=cur_val{
                    tree.insert(k.clone(),val.clone());
                }
            }else{
                panic!("merge conflict at: {}",k);
            }
        }
    }

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


    let mut tree_string=String::new();
    let mut pairs:Vec<(&String,&String)>=tree.iter().collect();
    pairs.sort_by(|a,b| a.0.cmp(b.0));
    for (key,val) in pairs{
        tree_string.push_str(key);
        tree_string.push(' ');
        tree_string.push_str(val);
        tree_string.push('\n');
    }

    let tree_hash=sha1_string(&tree_string);
    fs::write(objects_dir.join(&tree_hash),tree_string).expect("write object failed");

    let mut commit=String::new();
    commit.push_str("tree ");
    commit.push_str(&tree_hash);
    commit.push('\n');
    commit.push_str("parent ");
    commit.push_str(&cur_commit_hash);
    commit.push('\n');
    commit.push_str("parent1 ");
    commit.push_str(&meg_commit_hash);
    commit.push('\n');
    commit.push_str("message ");
    commit.push_str(&format!("merge {} {}\n",head,name));
    commit.push('\n');
    let commit_hash=sha1_string(&commit);
    fs::write(objects_dir.join(&commit_hash),&commit).expect("write object failed");

    fs::write(heads_dir.join(&head),format!("{commit_hash}\n")).expect("write refs/heads failed");
}