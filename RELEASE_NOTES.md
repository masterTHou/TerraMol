# TerraMol v1.1.0 — One-click Molecular Viewer

**下载 `TerraMol-Viewer-v1.1.0-windows-x64.zip`，全部解压，双击
`启动TerraMol.cmd` 或 `TerraMol.exe`，即可看到运动中的分子。**

无需安装 Unity、Rust、Python、Node.js 或 VC++ 运行库。程序是包含静态 CRT 的
Windows x64 独立 EXE，自动打开默认浏览器；全部网页资源已内置，无 CDN 或联网依赖。

本次新增：

- 离线 WebGL 球棍渲染、模拟盒、鼠标旋转/缩放与全景适配。
- 图形硬件不可用时的 Canvas 兼容投影。
- 默认自动运行 50 个水分子，支持水/甲烷/苯模板切换、混合添加、重置与清空。
- 整分子选择、高亮、Rust 质心质量/速度信息、速度箭头、删除和速度控制。
- Kelvin 热速度采样、暂停/继续、播放速度控制；新分子采样不会改变已有分子的速度。
- 本机 HTTP 传输、固定 1 fs Rust 积分、一致快照和过期响应过滤。
- 正常退出及无页面访问 3 分钟后自动释放资源；不安装后台服务。
- 中文新手指南、真实运行截图、便携包和 SHA-256 校验文件。

已执行本地验证：默认 Rust 测试 15 项通过；包含 viewer 的全部 features 测试
与严格 Clippy 通过。真实 EXE HTTP 测试覆盖增删、索引、暂停、并发、访问边界、
退出和闲置清理。实际浏览器确认 WebGL、Canvas 兼容、整分子点选、速度设置/箭头、
删除与退出；双击入口自动打开默认 Chrome。静态依赖检查只列出 Windows 系统 DLL。

这是 Native 刚体近似可视化，**不是完整原子级 MD**。水、甲烷、苯的真实动力学
仍需适当的力场参数化；本次没有伪造构象搜索、能量优化或 RL 训练功能。
原有 Rust API、C ABI v1、Unity 源码、Python 和可选 LAMMPS 后端继续保留。

详细说明：[新手开始](https://github.com/masterTHou/TerraMol/blob/main/新手开始.md)、
[Viewer 架构与验证](https://github.com/masterTHou/TerraMol/blob/main/docs/viewer.md)。
CI 的实时结果请查看 [GitHub Actions](https://github.com/masterTHou/TerraMol/actions)。

## v1.0 核心基础

首个核心原型预发布：Rust 拥有唯一分子状态，Unity 通过版本化 C ABI 渲染和交互，
Python/CLI 可无头运行。五个阶段保留独立提交，README、架构决策、单位、ABI、
阶段验证记录和长期路线图一并交付。

包含：

- 八种元素、水/甲烷/苯模板、XYZ 候选键推断、共享定义与稳定分子 ID。
- 随机旋转、防重叠批量放置、质量相关热速度、刚体平动、网格 broad phase 与弹性反射/碰撞。
- Opaque handle、错误码、panic 边界、调用方缓冲区、版本化 Render Snapshot。
- Unity 五个运行脚本、对象池、CPK、高亮、质心速度箭头和 OnGUI 控制面板。
- 可选 LAMMPS C-API、显式 LJ 参数示例、拓扑重建与状态同步。
- ConformerGenerator 契约、实际二面角几何编辑、统一 Action/Observation 与确定性 reset。

本地验证：默认 Rust 测试 14 项通过；全部 features 普通测试 15 项通过；另外
3 项 LAMMPS 联调测试已显式执行通过。实际 DLL、Python 与 C# P/Invoke 测试通过。
`cargo fmt --check`、严格 Clippy 和 release 构建通过。

Windows x64 发行包包含 Native DLL、C 头文件、Python 客户端与使用说明，另附
SHA-256 校验和。完整 Rust/Unity/测试源码由本标签的 Source code 提供。
LAMMPS 二进制未捆绑，需用 `--features lammps` 编译并提供兼容的外部共享库。

已知边界：**Native 不是原子级 MD**；LJ 参数不代表水/甲烷/苯的有效力场。
没有完整构象搜索、力场优化或 RL 训练。Unity Editor、Play mode、玩家构建和
IL2CPP 尚未验证，不能以 C# 桥接测试替代。详见 `docs/phases.md`、
`docs/lammps.md`、`unity/README.md`。

跨平台 CI [运行 39030938630](https://github.com/masterTHou/TerraMol/actions/runs/39030938630)
已全部通过：Windows/Linux/macOS 核心与 ABI、Windows C# 桥接，以及 Ubuntu
LAMMPS 物理联调。CI 验证 Rust 1.99.1，本地验证 Rust 1.95.0。
