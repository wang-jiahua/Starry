# StarryOS

## 简介

这里是StarryOS，一个基于ArceOS实现的宏内核。

> Starry意指布满星星的，寓意本OS的开发学习借鉴了许多前辈的思路，并将其汇总归一为这个内核。

在线文档详见：[Starry (azure-stars.github.io)](https://azure-stars.github.io/Starry/)

## 快速开始

### 运行依赖工具

 - aarch64-linux-musl-cross 工具链：https://musl.cc/aarch64-linux-musl-cross.tgz，`aarch64-linux-musl-gcc -v`正常输出。
 - Python3.11 注意版本，只能使用这个版本
 - QEMU 7.2.1(是否有版本要求待定) `qemu-system-aarch64 --version`正常输出。

### 快速编译

1.首先运行脚本`./build.sh`编译内核。
``` bash
cd python-lab
bash ./build.sh
```
会在目录下生成编译好的文件

2.裁剪文件
``` bash
bash ./cut.sh
```

3.启动Alpine版本的qemu测试项目
``` bash
bash ./run.sh
```
用户名和密码都是root，ssh端口为127.0.0.1:2222，建议使用ssh连接，在目录下面有萧老师编译好的python版本，可以使用`bin/python3.11 lib/python3.11/test/test___all__.py`测试程序。

## 具体任务内容

详情见：https://github.com/elliott10/python-lab/blob/main/README.md

## 启动Starry

``` bash
cd starry
# 生成fat32文件系统，并复制刚刚生成的opt文件夹到fat32文件系统中
./build_pyimg.sh aarch64 fat32
# 启动qemu
./run.sh
# 最终目标，顺利通过Python3程序完整测试
bin/python3.11 lib/python3.11/test/test___all__.py
```



