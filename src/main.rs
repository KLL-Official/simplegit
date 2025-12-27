use std::fs;
use std::path::PathBuf;
use std::env;
use clap::{Parser,Subcommand,ValueHint};


mod cmd_init;
mod cmd_add;
mod cmd_rm;
mod cmd_commit;
mod cmd_branch;
mod cmd_checkout;
mod cmd_merge;

fn get_config_path()->PathBuf{
    let cfg_path=PathBuf::from("./config");
    if !cfg_path.exists(){
        let cwd=env::current_dir().expect("get cwd failed");
        fs::write(&cfg_path,cwd.display().to_string()).expect("write ./config failed");
    }
    let s=fs::read_to_string(&cfg_path).expect("read ./config failed");
    PathBuf::from(s.trim())
}

#[derive(Parser)]
#[command(name="mygit",version,about="git tool in Rust")]
struct Cli{
    #[command(subcommand)]
    command:Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Init,
    Add{
        #[arg(value_name="PATH",value_hint=ValueHint::AnyPath)]
        paths:Vec<PathBuf>,
    },
    Rm{
        #[arg(value_name="PATH",value_hint=ValueHint::AnyPath)]
        paths:Vec<PathBuf>,
        #[arg(long="cached")]
        cached:bool,
    },
    Commit{
        #[arg(short='m')]
        message:Option<String>,
        // #[arg(short = 'a', long = "all")]
        // all: bool,
    },
    Branch{
        name:Option<String>,
        // #[arg(long)]
        // list:bool,
        #[arg(short='d')]
        delete:Option<String>,
    },
    Checkout{
        name:Option<String>,
        #[arg(short='b')]
        create:Option<String>,
    },
    Merge{
        branch:String,
    },
    Pwd,
    Cd{
        #[arg(value_name="DIR",value_hint=ValueHint::DirPath)]
        dir:PathBuf,
    },
}
fn main(){
    let work_dir=get_config_path();
    let cli=Cli::parse();
    match cli.command{
        Some(Command::Pwd)=>{
            println!("current work dir: {work_dir:?}");
        }
        Some(Command::Cd{dir})=>{
            assert!(dir.is_dir(),"work dir is not a directory: {}",work_dir.display());
            let cfg_path=PathBuf::from("./config");
            fs::write(&cfg_path,dir.display().to_string()).expect("write ./config failed");
            println!("current work dir: {dir:?}");
        }
        Some(Command::Init)=>{
            cmd_init::run(&work_dir); //git init
            println!("Init successful!");
        }
        Some(Command::Add{paths})=>{
            cmd_add::run(&work_dir,&paths);//git add <name>
            println!("Add {paths:?} successful!");
        }
        Some(Command::Rm{paths,cached})=>{
            if cached{
                cmd_rm::rm_cache(&work_dir,&paths);//git rm --cached <name>
            }else{
                cmd_rm::rm(&work_dir,&paths);//git rm <name>
            }
            println!("Rm {paths:?} {cached} successful!");
        }
        Some(Command::Commit{message})=>{
            cmd_commit::run(&work_dir,&message); //git commit,git commit -m
            println!("Commit successful!");
        }

        Some(Command::Branch{name,delete})=>{
            // println!("Branch {name:?} {delete:?}!");
            if let Some(b)=delete{
                cmd_branch::delete(&work_dir,&b);//git branch -d <name>
            }else if let Some(b)=name{
                cmd_branch::create(&work_dir,&b);//git branch <name>
            }else{
                cmd_branch::list(&work_dir);//git branch
            }
        }
        Some(Command::Checkout{name,create})=>{
            println!("checkout {name:?} {create:?}");
            // if let Some(b)=create{
            //     cmd_checkout::create(&work_dir,&b);//git checkout -b <name>
            // }else 
            if let Some(b)=name{
                cmd_checkout::switch(&work_dir,&b);//git checkout <name>
            }
        }
        Some(Command::Merge{branch})=>{
            println!("Merge {branch}");
            cmd_merge::merge(&work_dir,&branch);
        }
        _=>{
            println!("no command");
        }
    }
}