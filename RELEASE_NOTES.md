# TerraMol v1.0.0 — Core Prototype

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

CI 的实时结果请查看 [GitHub Actions](https://github.com/masterTHou/TerraMol/actions)。
