use std::path::{Path,PathBuf};
use std::fs;

fn get_commit_hash(heads_dir:&PathBuf,name:&String)->String{
    let p=heads_dir.join(name);
    assert!(p.exists(),"branch does not exist: {}",name);
    fs::read_to_string(p).expect("read head failed").trim().to_string()
}
pub fn run(work_dir:&PathBuf){
    let git_dir=work_dir.join(".mygit");
    assert!(git_dir.exists(),"git does not exist:{}",git_dir.display());

    let head_path=git_dir.join("HEAD");
    let objects_dir=git_dir.join("objects");
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
    let commit_hash=get_commit_hash(&heads_dir,&head);
    let mut jump=commit_hash.clone();
    println!("commit {commit_hash}");
    loop{
        let commit_text=fs::read_to_string(objects_dir.join(&jump)).expect("read commit failed");
        let mut flag:bool=false;
        for line in commit_text.lines(){
            if line.starts_with("parent "){
                flag=true;
                jump=line[7..].trim().to_string();
                println!("<-     {jump}");
                break;
            }
        }
        if flag==false{
            break;
        }
    }
}