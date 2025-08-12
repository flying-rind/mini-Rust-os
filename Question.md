# rustos-reliability-enhancement

面向RustOS的可靠性增强内核架构设计

## 项目描述

Rust语言的内存安全特性使其在操作系统开发中可以避免内存相关缺陷，但非内存缺陷仍可能影响系统可靠性。传统宏内核架构下，任意内核模块缺陷都可能引发内核崩溃，一种解决方案是改变内核功能模块的运行方式，由传统的函数调用方式转变为独立内核线程运行方式，内核线程拥有独立线程栈，发生失效时可通过微重启的方式恢复服务，不影响内核全局。

## 参考RustOS源码

- rCore: <https://github.com/rcore-os/rCore>
- ArceOS: <https://github.com/oscomp/arceos>
- Starry-next: <https://github.com/oscomp/starry-next>

## 所属赛道

2025全国大学生操作系统比赛的“OS功能设计”赛道

## 参赛要求

- 以小组为单位参赛，最多三人一个小组，且小组成员是来自同一所高校的学生
- 如学生参加了多个项目，参赛学生选择一个自己参加的项目参与评奖
- 请遵循“2025全国大学生操作系统比赛”的章程和技术方案要求

## 项目导师

- 贾周阳
- email <jiazhouyang@nudt.edu.cn>

## 赛题分类

- 新型内核

## 难度

高

## 特征

- 需要熟悉 Rust 编程语言
- 需要熟悉一个 Rust OS 或 能从零编写一个 Rust OS

## 文档

- <https://rcore-os.cn/rCore-Tutorial-Book-v3/>

## License

Apache 2.0(<https://www.apache.org/licenses/LICENSE-2.0.html>)

## 预期目标

1. 基于Rust语言实现基本内核/基于现有Rust OS进行改造
2. 支持不少于2两个内核服务线程（例如文件系统服务、块设备驱动服务、网络栈服务、网卡设备驱动服务等），可使用开源组件
3. 服务可响应来自用户态应用的请求
4. 向运行中的服务注入故障后，服务失效并重启，内核不崩溃
5. 支持典型Linux应用，内核服务崩溃恢复不影响应用正确运行
