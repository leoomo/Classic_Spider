# 蜘蛛纸牌开发阶段文档

## 概述

本文档库包含蜘蛛纸牌游戏的详细开发计划，分为 4 个阶段，预计总工期 8 周。

## 阶段总览

| 阶段 | 名称 | 周期 | 核心目标 | 文档 |
|------|------|------|----------|------|
| Phase 1 | 基础架构 | Week 1-2 | 项目初始化、核心数据结构、基础 UI | [phase-1-foundation.md](./phase-1-foundation.md) |
| Phase 2 | 核心玩法 | Week 3-4 | 拖拽系统、规则验证、游戏主循环 | [phase-2-gameplay.md](./phase-2-gameplay.md) |
| Phase 3 | 完善体验 | Week 5-6 | 撤销/重做、提示、动画、音效 | [phase-3-experience.md](./phase-3-experience.md) |
| Phase 4 | 打磨发布 | Week 7-8 | 统计、皮肤、优化、打包 | [phase-4-release.md](./phase-4-release.md) |

## 技术栈

- **后端**: Tauri 2.x + Rust
- **前端**: Svelte 5 + TailwindCSS
- **构建**: Vite

## 依赖关系图

```
Phase 1 (基础架构)
    │
    ├── Rust 数据结构 ──────────────┐
    │                              │
    └── Svelte 基础组件 ────────────┤
                                   │
                                   ▼
Phase 2 (核心玩法) ◄───────────────┘
    │
    ├── 规则验证 (Rust)
    ├── 拖拽交互 (前端)
    ├── 发牌/收牌
    └── 计分/胜利
                                   │
                                   ▼
Phase 3 (完善体验)
    │
    ├── 撤销/重做 (依赖 Phase 2 状态)
    ├── 提示系统 (依赖规则验证)
    ├── 动画效果
    └── 断点续玩
                                   │
                                   ▼
Phase 4 (打磨发布)
    │
    ├── 统计数据
    ├── 皮肤系统
    ├── 性能优化
    └── 打包发布
```

## 快速开始

1. 从 [Phase 1](./phase-1-foundation.md) 开始阅读
2. 每个阶段包含详细的任务清单和验收标准
3. 按顺序完成各阶段任务

## 相关文档

- [PRD 产品需求文档](../prd.md)
- [架构设计文档](../architecture.md)
