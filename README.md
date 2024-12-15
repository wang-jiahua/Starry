# StarryOS

## 简介

这里是StarryOS，一个基于ArceOS实现的宏内核。

> Starry意指布满星星的，寓意本OS的开发学习借鉴了许多前辈的思路，并将其汇总归一为这个内核。

在线文档详见：[Starry (azure-stars.github.io)](https://azure-stars.github.io/Starry/)

## 快速开始

### 运行依赖工具

- [aarch64-linux-musl-cross](https://musl.cc/aarch64-linux-musl-cross.tgz)，添加环境变量

```shell
export PATH=$PATH:/path/to/aarch64-linux-musl-cross/bin
export PATH=$PATH:/path/to/aarch64-linux-musl-cross/aarch64-linux-musl/bin
export LD_LIBRARY_PATH=/path/to/aarch64-linux-musl-cross/aarch64-linux-musl/lib:$LD_LIBRARY_PATH
```

输入`aarch64-linux-musl-gcc -v` 正常输出

- Python3.11.11（是否支持 Python3.11 的其他小版本待测试，这里是用 apt 安装 Ubuntu24.04 上最新的 Python3.11，需要先修改软链接忽略自带的 Python3.12），输入 `python3 --version` 正常输出

- QEMU 7.0.0，安装方式参考 [QEMU 模拟器安装](https://rcore-os.cn/arceos-tutorial-book/ch01-03.html)，输入 `qemu-system-aarch64 --version` 正常输出

### 快速开始

1. 编译 Python 和内核。必须挂载动态库到 `/lib`，否则 Python 会找不到动态库

```shell
bash build_pyimg.sh
```

2. 启动 Starry

```shell
bash run.sh
```

## 具体任务内容

详情见：https://github.com/elliott10/python-lab/blob/main/README.md

## 最终目标

通过 Python3 程序完整测试

```shell
cd /opt/python3.11
bin/python3.11 lib/python3.11/test/test___all__.py
```
