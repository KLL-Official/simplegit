use std::fs;
use std::path::{Path,PathBuf};

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


pub fn list(work_dir:&PathBuf){
    let git_dir=work_dir.join(".mygit");
    assert!(git_dir.exists(),"git does not exist:{}",git_dir.display());

    let heads_dir=git_dir.join("refs").join("heads");

    let mut branches:Vec<String>=Vec::new();
    for p in fs::read_dir(&heads_dir).expect("read file failed"){
        let p1=p.expect("read file failed");
        let p1=p1.path();
        if let Some(name)=p1.file_name(){
            branches.push(name.to_string_lossy().to_string());
        }
    }
    branches.sort();
    for name in branches{
        println!("{name}");
    }
    
    let head_path=git_dir.join("HEAD");
    let head=match fs::read_to_string(&head_path){
        Ok(s)=>s,
        Err(_)=>String::new(),
    };
    let head=head.trim();
    if head.starts_with("ref: "){
        let ph=&head[5..];
        let ph=Path::new(ph);
        if let Some(name)=ph.file_name(){
            let name=name.to_string_lossy();
            println!("current branch: {name}");
        }
    }else{
        println!("detached: {head}");
    }
}

pub fn create(work_dir:&PathBuf,name:&String){
    let git_dir=work_dir.join(".mygit");
    assert!(git_dir.exists(),"git does not exist:{}",git_dir.display());

    let heads_dir=git_dir.join("refs").join("heads");
    let branch_path=heads_dir.join(name);
    if branch_path.exists(){
        panic!("branch already exists: {}",name);
    }

    let head_path=git_dir.join("HEAD");
    let (_hpath,parent)=get_head(&git_dir,&head_path);

    if let Some(hash)=parent{
        fs::write(branch_path,&hash).expect("write branch failed");
    }else{
        panic!("no commit");
    }
}

pub fn delete(work_dir:&PathBuf,name:&String){
    let git_dir=work_dir.join(".mygit");
    assert!(git_dir.exists(),"git does not exist:{}",git_dir.display());

    let heads_dir=git_dir.join("refs").join("heads");
    let branch_path=heads_dir.join(name);
    if !branch_path.exists(){
        panic!("branch not exist: {}",name);
    }

    let head_path=git_dir.join("HEAD");
    let head=match fs::read_to_string(&head_path){
        Ok(s)=>s,
        Err(_)=>String::new(),
    };
    let head=head.trim();
    if head.starts_with("ref: "){
        let ph=&head[5..];
        let ph=Path::new(ph);
        if let Some(cur_b)=ph.file_name(){
            let cur_b=cur_b.to_string_lossy();
            if cur_b==*name{
                panic!("can't delete current branch");
            }
        }
    }else{
        println!("detached: {head}");
    }

    fs::remove_file(&branch_path).expect("delete branch failed");
}