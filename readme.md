一个用Rust 实现的简易Git，支持最核心的对象存储、分支引用与常用命令

## Features

- 基础命令：
  - `init` 初始化仓库
  - `add <paths>` 添加到暂存区
  - `rm <paths>` 从暂存区和工作区移除
    - `rm --cached <paths>`   从暂存区移除
  - `commit -m <message>` 提交
  - `branch` 列出所有分支
    -  `branch <name>` 创建分支
    -  `branch -d <name>` 删除分支
  - `checkout <branch|commit>` 支持分支切换与分离头
  - `merge <branch>` 支持 fast-forward 与三方合并
  - `log` 提交历史
  - `status` 比较 HEAD / index / workdir 的差异

## Build & Run

**Requirements**

- Rust+Cargo

```rust
clap= "4"
sha1 = "0.10"
hex = "0.4"
```

**Build**

```bash
cargo build --release
```

**Run**

```base
cargo run -- <command> [args]
```

由于本项目通过 `cargo run` 在项目目录启动，但 Git 的工作目录可能是任意路径，因此提供了内置的工作目录管理命令：

- `cd <path>`：将工作目录切换到指定路径（后续所有命令默认作用于该目录）
- `pwd`：输出当前工作目录

工作目录会被持久化记录在配置文件中

## Usage

1. 初始化仓库

```bash
cargo run -- init
```

2. 添加文件到暂存区

```bash
cargo run -- add a.txt d/
```

3. 移除文件

```bash
cargo run -- rm --cached a.txt   # 仅从暂存区移除
cargo run -- rm a.txt            # 从暂存区与工作区移除
```

4. 提交

```bash
cargo run -- commit -m "init"
```

5. 分支与切换

```bash
cargo run -- branch          # 列出分支
cargo run -- branch dev      # 创建分支dev
cargo run -- checkout dev
```

6. 合并

```bash
cargo run -- merge main
```

7. 查看状态/历史

```bash
cargo run -- status      # 查看状态
cargo run -- log         # 查看提交历史
```

