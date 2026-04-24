# GSD (Get-Shit-Done) 评估报告 — AICrypto 项目适用性分析

> **评估日期**: 2026-04-24
> **GSD 版本**: v1.38.3
> **评估结论**: ✅ 强烈推荐，已嵌入可用

---

## 一、GSD 是什么

GSD 是一个 **AI 编码 agent 的元提示和工作流引擎**，通过结构化的里程碑 → 阶段 → 计划 → 执行体系，让 AI agent 在复杂项目中产出可靠、可追溯的代码。GitHub 56.8k stars。

核心解决的问题是：**AI agent 上下文膨胀后输出质量急剧下降**。GSD 通过将工作拆分为在独立 200K token 上下文窗口中执行的原子任务来保持质量。

---

## 二、AICrypto 项目当前的痛点（GSD 可解决的）

### 2.1 上下文爆炸

**现状**: 前一个 Crush 会话积累了 944 条消息、154K prompt tokens，跨 Phase 1-7 开发，最终卡死。

**GSD 解法**: 每个 phase 的每个 plan 在独立上下文窗口中执行，主上下文始终保持在 30-40% 占用率。执行完毕后产出 SUMMARY.md，下次会话只需读摘要。

### 2.2 无结构化规划

**现状**: 规划文档散落在 `docs/AICrypto-source/docs/` 中的 20 个 md 文件里，没有标准化的状态追踪、进度管理、依赖关系管理。

**GSD 解法**: `.planning/` 目录结构化管理：
- `PROJECT.md` — 项目愿景（每次会话自动加载）
- `ROADMAP.md` — 里程碑和 phase 进度
- `STATE.md` — 决策记录、阻塞项、当前位置
- `phases/{N}/` — 每个 phase 的上下文、研究、计划、验证

### 2.3 会话中断后无法续接

**现状**: 每次会话中断后需要手动回忆"上次做到哪了"、"还差什么"。

**GSD 解法**: 
- `/gsd-pause-work` — 创建上下文交接文档
- `/gsd-resume-work` — 自动恢复上次位置
- `/gsd-progress` — 随时查看"我在哪、下一步做什么"

### 2.4 无法并行推进

**现状**: Phase 5/6/7 需要串行推进，一个会话只能做一件事。

**GSD 解法**: Wave-based 并行执行，独立 plan 并行在不同上下文中运行。

---

## 三、GSD 对 AICrypto 的价值评估

| 维度 | 评分 | 说明 |
|------|------|------|
| 上下文管理 | ⭐⭐⭐⭐⭐ | 直接解决会话卡死问题 |
| 结构化规划 | ⭐⭐⭐⭐⭐ | 20 个规划文档可导入为 GSD 格式 |
| 会话连续性 | ⭐⭐⭐⭐⭐ | pause/resume 机制完美匹配多会话开发 |
| 阶段执行 | ⭐⭐⭐⭐ | Wave 并行执行，但 Rust 编译较慢可能限制并行效果 |
| 质量保证 | ⭐⭐⭐⭐ | 内置 code review、安全审计、UAT 验证 |
| 代码审查 | ⭐⭐⭐⭐ | `/gsd-code-review` + `/gsd-code-review-fix` 自动审查修复 |
| 回滚能力 | ⭐⭐⭐⭐⭐ | 每个 task 原子 git commit，可独立回滚 |

**综合评分**: 4.7/5 — 强烈推荐

---

## 四、当前嵌入状态

### 4.1 已就绪（无需重新安装）

GSD **已经嵌入到 Crush 中**，无需额外操作：

```
安装位置:
├── ~/.claude/skills/gsd-*/     ← 82 个 skill 目录（Crush 已自动发现）
├── ~/.config/opencode/get-shit-done/  ← OpenCode 版本
└── Crush 当前状态: 83 个 GSD skills 已加载（loaded_this_session = 0/83）
```

### 4.2 版本一致性

| Runtime | GSD 版本 | 状态 |
|---------|---------|------|
| Crush (本会话) | v1.38.3 | ✅ 83 skills 已发现 |
| Claude Code | v1.38.3 | ✅ 同一套 skills |
| OpenCode | v1.38.3 | ✅ 独立安装 |

**不存在冲突**: 三者使用不同的 skill 发现路径，不会互相干扰。

### 4.3 已修复的问题

在评估过程中发现并修复了一个 skill 命名问题：

```
问题: gsd-extract_learnings 使用下划线命名
影响: Crush 每次会话输出 WARN 日志（但不影响功能）
修复: 重命名为 gsd-extract-learnings（连字符格式）
```

---

## 五、推荐使用方式

### 5.1 初始化 AICrypto 项目到 GSD

AICrypto 已有大量规划文档和代码，推荐使用 **导入模式**：

```
步骤 1: 在 AICrypto 项目目录下运行
步骤 2: 使用 /gsd-ingest-docs 导入现有 20 个规划文档
步骤 3: GSD 自动分类为 PRD/SPEC/ADR/DOC 并创建 .planning/ 结构
步骤 4: 使用 /gsd-progress 查看整体进度
```

### 5.2 后续开发工作流

```
/gsd-progress              → 查看当前进度和下一步
/gsd-discuss-phase N       → 讨论 phase 实现细节
/gsd-plan-phase N          → 自动研究+规划
/gsd-execute-phase N       → 并行执行
/gsd-verify-work N         → 验证交付
/gsd-next                  → 自动推进到下一步
```

### 5.3 与现有文档的关系

| 现有文档 | GSD 对应位置 |
|---------|-------------|
| `01_项目总纲.md` | `.planning/PROJECT.md` |
| `04_PRD_产品需求文档.md` | `.planning/REQUIREMENTS.md` |
| `06_Phase开发规划.md` | `.planning/ROADMAP.md` |
| `07_开发工作任务书.md` | `.planning/phases/*/` |
| `03_系统架构设计.md` | `.planning/codebase/ARCHITECTURE.md` |

导入后原始文档保留在 `docs/AICrypto-source/`，GSD 在 `.planning/` 创建标准化索引。

---

## 六、不重复安装的原因

1. **Skill 发现机制**: Crush 通过扫描 `~/.claude/skills/` 目录自动发现所有 `gsd-*` skill，无需注册或安装
2. **共享 skills**: Claude Code 和 Crush 读取同一个 `~/.claude/skills/` 目录，GSD 已安装于此
3. **OpenCode 独立**: OpenCode 的 GSD 在 `~/.config/opencode/get-shit-done/`，与 Crush 无冲突
4. **重新安装无意义**: `npx get-shit-done-cc@latest` 会覆盖到同一位置，不增加任何功能

**结论**: GSD 已经在 Crush 中可用，直接使用即可。不需要重新克隆、下载或安装。

---

## 七、立即可用的 GSD 命令

在当前 Crush 会话中可以直接使用的核心命令：

| 命令 | 用途 | 优先级 |
|------|------|--------|
| `/gsd-ingest-docs` | 导入 AICrypto 现有规划文档 | 🔴 立即 |
| `/gsd-progress` | 查看项目进度 | 🔴 立即 |
| `/gsd-map-codebase` | 分析 Rust 代码库结构 | 🟡 尽快 |
| `/gsd-discuss-phase` | 讨论下一阶段实现 | 🟡 尽快 |
| `/gsd-plan-phase` | 自动研究+规划 | 🟢 按需 |
| `/gsd-execute-phase` | 并行执行开发 | 🟢 按需 |
| `/gsd-pause-work` | 暂停并保存上下文 | 🟢 按需 |
| `/gsd-resume-work` | 恢复上次工作 | 🟢 按需 |
