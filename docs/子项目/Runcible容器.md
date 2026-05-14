基于我们的讨论，下面为 **Runcible** 项目撰写的完整架构、技术选型与开发功能规划。它被定位为一个跨平台的极致轻量化容器管理工具，核心目标是 **用最少的资源，提供与 Docker 同等的使用体验**。

---

## 1. 项目定位

**Runcible** 取意于精巧、跨界、可承载万物的容器。它不是一个全新的容器引擎，而是对现有顶级轻量组件（`containerd`、`nerdctl`）的深度整合与体验封装，让用户在 macOS、Linux、Windows 上都能获得 **“比 Docker 轻，和 Docker 一样顺手”** 的容器环境，且完全开源、无商业限制。

开发语言： **Go 语言**



**哲学**：

- **只做减法，不做加法**：不重新发明运行时，只剥离所有非必要层。
- **控制权归你**：无守护进程侵入，可选的无根模式，细粒度资源约束。
- **零迁移成本**：100% 兼容 OCI 镜像与 Docker Compose 工作流。

---

## 2. 系统架构

### 2.1 架构总览

Runcible 采用 **微内核式 CLI + 平台适配层 + 原生运行时** 的三层解耦架构。

```
┌──────────────────────────────────────────────────────────┐
│                   Runcible CLI (runcible)                 │
│   - 统一命令接口 (run, build, compose, start/stop VM)     │
│   - 配置文件管理 (~/.runcible/)                           │
│   - 插件系统 (nerdctl, buildkit, compose)                 │
└───────────────────────┬──────────────────────────────────┘
                        │
┌───────────────────────┼──────────────────────────────────┐
│                Platform Adaptation Layer                 │
│  ┌───────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │  macOS    │  │    Linux     │  │     Windows      │   │
│  │ (Colima   │  │ (原生systemd)│  │ (WSL2 + systemd) │   │
│  │  VM管理)  │  │              │  │                  │   │
│  └───────────┘  └──────────────┘  └──────────────────┘   │
└───────────────────────┬──────────────────────────────────┘
                        │
┌───────────────────────┴──────────────────────────────────┐
│              Container Runtime Core                      │
│  ┌─────────────────┐  ┌────────────┐  ┌──────────────┐  │
│  │   containerd    │  │   nerdctl  │  │  BuildKit    │  │
│  └─────────────────┘  └────────────┘  └──────────────┘  │
│  ┌─────────────────┐  ┌────────────────────────────┐    │
│  │    runc/crun    │  │  RootlessKit (可选)        │    │
│  └─────────────────┘  └────────────────────────────┘    │
└──────────────────────────────────────────────────────────┘
```

### 2.2 各层职责

**1. Runcible CLI**
- 提供与 `docker` 高度一致的命令体验：`runcible run/ps/rm/build/compose...`
- 内置智能环境初始化：一键安装 containerd、nerdctl、Colima（若缺失）。
- 管理虚拟机生命周期（macOS）；管理 WSL2 发行版（Windows）；Linux 下管理 systemd 服务。
- 维护用户级配置，如资源限制、镜像加速器、无根模式开关。

**2. 平台适配层**
- **macOS**：通过封装 Colima 来管理一个极简 Linux VM（Alpine），内部仅运行 `containerd`。利用 `virtiofs` 共享文件，Rosetta 2 加速 x86 镜像。
- **Linux**：直接操作本机 `containerd` 服务，通过 systemd 管理；支持一键部署 Debian/Ubuntu/Fedora 等。
- **Windows**：自动化安装/配置 WSL2 发行版（默认 Ubuntu），在其中安装 containerd，并通过 `wsl.exe` 与 `nerdctl` 通信。

**3. 运行时核心**
- **containerd**：业界标准容器运行时，负责镜像管理、容器生命周期。
- **nerdctl**：containerd 的成熟 CLI，兼容 Docker 命令，支持 compose、rootless、镜像加速等。
- **BuildKit**：作为可选项集成，提供现代化镜像构建（`runcible build`）。
- **RootlessKit** + 用户态 containerd：实现无根模式容器，增强隔离，无特权运行。

---

## 3. 技术选型详解

| 组件/领域                    | 选型                                                | 理由                                                         |
| :--------------------------- | :-------------------------------------------------- | :----------------------------------------------------------- |
| **容器运行时**               | containerd (主)，crun (OCI 运行时)                  | 极简、云原生标准，无 Docker 守望者进程开销。crun 以 C 编写，更轻更快。 |
| **CLI 工具**                 | nerdctl                                             | 与 Docker CLI 高度兼容，原生支持 containerd，无守护进程依赖。项目可直接将其作为子命令转发。 |
| **macOS 虚拟化**             | Colima (基于 Lima) + macOS Virtualization.Framework | 极致轻量的 VM，支持 vz 驱动，资源占用低。可直接通过模板启动 containerd。 |
| **Windows 环境**             | WSL2 + Ubuntu 22.04                                 | 微软官方 Linux 子系统，性能接近原生，无需第三方虚拟化。      |
| **文件共享 (macOS)**         | virtiofs                                            | 吞吐量远高于 sshfs，接近原生磁盘性能。                       |
| **x86 加速 (Apple Silicon)** | Rosetta 2 (通过 `vz-rosetta`)                       | 避免模拟损耗，x86 容器性能损失 < 5%。                        |
| **镜像加速**                 | Stargz 快照器 + lazy pulling                        | 按需拉取镜像，减少冷启动延迟与网络占用。                     |
| **构建引擎**                 | BuildKit (可选)                                     | 支持并行构建、缓存挂载、多阶段构建，是 Docker Build 的未来。 |
| **无根容器**                 | RootlessKit + containerd-rootless                   | 用户态运行容器，无特权进程，多租户安全。                     |
| **编排**                     | nerdctl compose (内置)                              | 完全兼容 `docker-compose.yml`，一个文件即可管理多服务。      |
| **语言与打包**               | Go (CLI), Shell (安装脚本)                          | Go 编译为单一二进制文件，无依赖；Shell 脚本处理系统集成。    |

---

## 4. 功能规划

### 4.1 核心功能 (MVP)

1. **一键环境搭建** (`runcible init`)
   - 自动检测操作系统，安装所需组件（containerd、nerdctl、Colima/WSL2 配置）。
   - 输出一条 `source ~/.runcible/env` 命令即可完成初始化。

2. **容器生命周期管理**
   - 完全兼容 Docker CLI 风格的命令：`run`, `ps`, `logs`, `exec`, `rm`, `stop`, `start`, `restart`。
   - 支持资源限制：`--memory`, `--cpus`，且默认施加安全限制（如不分配特权模式）。

3. **镜像管理**
   - `pull`, `push`, `images`, `rmi`, `tag`，支持多平台拉取（`--platform`）。
   - 内置国内镜像加速器配置（如中科大、阿里云），一键切换。

4. **Compose 支持**
   - `runcible compose up -d` 直接解析 `docker-compose.yml`，通过 `nerdctl compose` 运行。

5. **虚拟机管理 (macOS)**
   - `runcible vm start`：启动 Colima 虚拟机（可选自定义 CPU/内存/磁盘/文件共享类型）。
   - `runcible vm stop`、`runcible vm status`、`runcible vm delete`。
   - 虚拟机内部自动配置 containerd 监听，用户无感知。

6. **跨平台一致性**
   - Linux/Windows/macOS 上 `runcible run nginx` 行为完全一致，卷挂载路径自动转换。

### 4.2 进阶功能 (v2.0+)

7. **无根模式 (Rootless)**
   - `runcible run --rootless` 自动启动用户态 containerd 实例，容器无任何特权，适合共享主机或安全敏感场景。
   - 提供脚本自动化用户命名空间配置（`/etc/subuid`）。

8. **智能资源监视**
   - `runcible stats` 展示所有容器/虚拟机 CPU/内存/IO，轻量无代理（直接通过 cgroup 读取）。

9. **内置轻量 Registry**
   - `runcible registry start --port 5000`，用于本地开发镜像分发，无外部依赖。

10. **GPU 支持 (实验性)**
    - 针对 AI/科学计算场景，Linux 下支持 NVIDIA GPU 穿透（`--gpus all`）。

11. **Runcible Hub（可选）**
    - 一个最小化的 Web UI，用于远程管理容器，仅提供列表、启停、日志查看，保持轻量。

12. **备份与迁移**
    - `runcible snapshot` 将当前所有容器/镜像/卷打包为一个可移植文件，方便跨机器迁移。

---

## 5. 开发路线图

| 阶段                        | 内容                                                         | 交付物                                     |
| :-------------------------- | :----------------------------------------------------------- | :----------------------------------------- |
| **Phase 1: 基础验证**       | 在 Linux/macOS 手动搭建 containerd+nerdctl 原型，验证兼容性。 | 一份可运行的命令参考文档                   |
| **Phase 2: CLI 骨架**       | 用 Go 编写 `runcible` 命令行，内嵌 nerdctl 作为子命令，实现 `init`、`vm` 命令。 | 二进制文件，仅支持 macOS (Colima) 和 Linux |
| **Phase 3: macOS 深度集成** | 优化 Colima 交互，支持 vz、virtiofs、Rosetta，提供一键安装脚本。 | 一行命令启动完整环境                       |
| **Phase 4: Windows 支持**   | 实现 WSL2 自动配置，将 `nerdctl` 通过 `wsl` 调用，添加路径转换。 | 三个平台 CLI 可用                          |
| **Phase 5: 高级特性**       | 无根模式、Stargz 懒加载、镜像加速器、Compose 完善。          | 生产级日常使用                             |
| **Phase 6: GUI (可选)**     | 使用 Wails (Go + Web 前端) 构建一个 <15MB 的桌面应用，仅显示容器状态。 | 轻量 App                                   |

---

## 6. 项目结构与代码组织

```
runcible/
├── cmd/
│   └── runcible/          # 主入口
├── pkg/
│   ├── cli/               # 命令定义 (cobra)
│   ├── platform/          # 平台适配 (macos/linux/windows)
│   │   ├── colima.go
│   │   ├── linux.go
│   │   └── wsl.go
│   ├── runtime/           # containerd 交互封装
│   ├── compose/           # compose 解析与调用
│   └── config/            # 配置管理
├── scripts/               # 一键安装脚本 (install.sh, install.ps1)
├── docs/                  # 文档
└── go.mod
```

---

## 7. 安全与稳定性设计

- **默认非特权**：所有容器运行时默认使用非特权用户（除非显式 `--privileged`）。
- **内核隔离**：依赖 `containerd` 的默认 seccomp/AppArmor/SELinux 策略。
- **文件共享安全**：macOS 下只挂载用户显式许可的目录（如当前项目），而非整个 `/Users`。
- **镜像签名**：支持 `cosign` 验证镜像来源，避免供应链攻击。
- **自动更新检查**：`runcible doctor` 命令检查核心组件版本，确保无已知漏洞。

---

Runcible 的核心承诺是：**用比 Docker 少一半的资源，做完全相同的事，并给予你完全的控制权**。这个设计方案从下层虚拟机到上层命令体验都围绕这一核心打造。你可以根据实际情况随时调整各阶段的优先级。