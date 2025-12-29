use std::fs;
use std::path::{Path,PathBuf};
use sha1::{Sha1,Digest};
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::{BufReader,Read};

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

fn check_index(map:&HashMap<String,String>,path:&Path,git_dir:&Path){
    if path==git_dir{
        return;
    }
    let meta=fs::metadata(path).expect("read file failed");
    if meta.is_file(){
        let p=map.get(&path.to_string_lossy().to_string());
        if p.is_none(){
            println!("untracked {}",path.to_string_lossy().to_string());
        }
    }else if meta.is_dir(){
        for p in fs::read_dir(path).expect("read file failed"){
            let p1=p.expect("read file failed");
            let p1=p1.path();
            check_index(map,&p1,git_dir);
        }
    }
}

pub fn run(work_dir:&PathBuf){
    let git_dir=work_dir.join(".mygit");
    assert!(git_dir.exists(),"git does not exist:{}",git_dir.display());

    let head_path=git_dir.join("HEAD");
    let (_hpath,parent)=get_head(&git_dir,&head_path);
    let objects_dir=git_dir.join("objects");

    let index_map=read_index(&git_dir.join("index"));

    if let Some(hash)=parent{
        let commit_map=get_index_from_commit(&objects_dir,&hash);
        let mut keys:HashSet<String>=HashSet::new();
        for k in commit_map.keys(){
            keys.insert(k.clone());
        }
        for k in index_map.keys(){
            keys.insert(k.clone());
        }
        println!("Changes to be committed:");
        let mut flag:bool=false;
        for k in keys{
            let commit_val:Option<&String>=commit_map.get(&k);
            let index_val:Option<&String>=index_map.get(&k);
            if commit_val!=index_val{
                flag=true;
                if commit_val.is_none(){
                    println!("new file {k}");
                }else if index_val.is_none(){
                    println!("deleted {k}");
                }else{
                    println!("modified {k}");
                }
            }
        }
        if flag==false{
            println!("none");
        }
    }else{
        panic!("no commit");
    }

    println!("Changes not staged:");
    let mut flag:bool=false;

    for (key,val) in &index_map{
        let p=Path::new(&key);
        if p.is_file(){
            let p_hash=sha1_file(&p);
            if p_hash!=*val{
                println!("modified {key}");
                flag=true;
            }
        }else{
            println!("deleted {key}");
            flag=true;
        }
    }
    if flag==false{
        println!("none");
    }

    check_index(&index_map,&work_dir,&git_dir);
}