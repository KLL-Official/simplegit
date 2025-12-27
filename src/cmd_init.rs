use std::fs;
use std::path::PathBuf;
//创建.mygit
//包括目录objects,refs/heads
//文件index,HEAD(ref: refs/heads/main),refs/heads/main
fn do_init(git_dir:&PathBuf)->std::io::Result<()>{
    fs::create_dir(git_dir)?;
    fs::create_dir(git_dir.join("objects"))?;
    fs::create_dir(git_dir.join("refs"))?;
    fs::create_dir(git_dir.join("refs").join("heads"))?;

    fs::write(git_dir.join("refs").join("heads").join("main"),"")?;
    fs::write(git_dir.join("HEAD"),"ref: refs/heads/main\n")?;
    fs::write(git_dir.join("index"),"")?;
    Ok(())
}
pub fn run(work_dir:&PathBuf){
    let git_dir=work_dir.join(".mygit");
    assert!(work_dir.exists(),"work dir does not exist:{}",work_dir.display());
    assert!(!git_dir.exists(),"repo already exists: {}",git_dir.display());
    
    let res=do_init(&git_dir);
    if let Err(e)=res{
        let _=fs::remove_dir_all(&git_dir);
        panic!("init failed: {e}");
    }
}