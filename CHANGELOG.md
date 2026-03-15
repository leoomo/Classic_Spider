# Changelog

所有项目的显著变更都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
并且本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

## [0.2.3] - 2026-03-14

### 优化
- 增加工具栏和卡牌区域的间距（8px → 20px）

## [0.2.2] - 2026-03-14

### 修复
- 修复胜利弹窗点击"再来一局"后重复弹出的问题
- 添加 `hasShownVictory` 标志位，确保每局游戏只显示一次胜利弹窗

## [0.2.1] - 2026-03-14

### 修复
- 修复牌组数量：208张 → 104张（符合经典蜘蛛纸牌规则）
- 修复发牌次数：15次 → 5次
- 禁用调试界面

## [0.2.0] - 2026-03-14

### 新增
- 完整的撤销/重做功能 (快捷键: Ctrl+Z / Ctrl+Y)
- 优化难度选择弹窗尺寸，更加紧凑合理
- 更新相关文档和截图

## [0.1.1] - 2026-03-14

### 修复
- 修复小分辨率布局问题
- 修复 MSI 打包问题
- 禁止 Ctrl+A 选中页面文本

## [0.1.0] - 2026-03-13

### 新增
- 经典 Windows 蜘蛛纸牌完整玩法
- 三种难度级别（简单、中等、困难）
- 拖拽和点击两种操作方式
- 自动保存和断点续玩
- 胜利检测和庆祝动画
- 音效系统
- 跨平台支持（Windows、macOS、Linux）

[Unreleased]: https://github.com/leoomo/Classic_Spider/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/leoomo/Classic_Spider/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/leoomo/Classic_Spider/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/leoomo/Classic_Spider/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/leoomo/Classic_Spider/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/leoomo/Classic_Spider/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/leoomo/Classic_Spider/releases/tag/v0.1.0
