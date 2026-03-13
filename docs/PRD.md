# 🕷️ 蜘蛛纸牌 - 产品需求文档 (PRD)

> Classic Spider Solitaire - Tauri 复刻版
> 版本：1.0.0 | 日期：2026-03-13

---

## 目录

1. [项目概述](#1-项目概述)
2. [目标用户](#2-目标用户)
3. [核心功能](#3-核心功能)
4. [游戏规则](#4-游戏规则)
5. [技术架构](#5-技术架构)
6. [视觉设计](#6-视觉设计)
7. [交互设计](#7-交互设计)
8. [数据结构](#8-数据结构)
9. [API 设计](#9-api-设计)
10. [非功能性需求](#10-非功能性需求)
11. [里程碑规划](#11-里程碑规划)

---

## 1. 项目概述

### 1.1 项目背景

蜘蛛纸牌（Spider Solitaire）是 Windows 系统中最经典的单人纸牌游戏之一。本项目旨在使用现代技术栈复刻这款经典游戏，在保持原汁原味游戏体验的同时，提供更轻量、更快速、更美观的现代版本。

### 1.2 项目目标

| 指标 | 目标值 |
|------|--------|
| 安装包大小 | < 10 MB |
| 内存占用（空闲） | < 50 MB |
| 冷启动时间 | < 1 秒 |
| 支持平台 | Windows / macOS / Linux |

### 1.3 核心价值主张

- **极致轻量**：利用 Tauri 框架，打造 10MB 以内的安装包
- **跨平台支持**：一次开发，三端运行
- **怀旧体验**：还原经典视觉风格，唤醒童年记忆
- **易用性优先**：针对中老年用户优化，大字体、大按钮、清晰反馈

---

## 2. 目标用户

### 2.1 主要用户画像

**退休人群（55-75岁）**
- 习惯：每天玩几局纸牌放松心情
- 痛点：视力下降、手部动作不够精准
- 需求：大字体、大按钮、清晰的操作反馈

**怀旧玩家（30-50岁）**
- 习惯：追求经典游戏体验
- 痛点：Windows 新版纸牌体验不佳
- 需求：还原经典界面和操作手感

### 2.2 无障碍设计要求

- 支持高对比度模式
- 最小点击区域 44x44 px
- 动画可关闭
- 支持键盘操作

---

## 3. 核心功能

### 3.1 功能清单

#### P0 - 必须实现

| 功能 | 描述 | 优先级 |
|------|------|--------|
| 难度选择 | 初级（单色）/ 中级（双色）/ 高级（四色） | P0 |
| 游戏主循环 | 洗牌、发牌、移动、收牌 | P0 |
| 拖拽系统 | 点击选中、拖拽移动、释放判定 | P0 |
| 撤销/重做 | 无限次步骤回溯 | P0 |
| 计分系统 | 基础 500 分，每步 -1，收牌 +100 | P0 |
| 新游戏 | 重新开始当前难度 | P0 |

#### P1 - 应该实现

| 功能 | 描述 | 优先级 |
|------|------|--------|
| 提示系统 | 智能寻找当前可移动的牌 | P1 |
| 统计数据 | 胜率、最高分、最短用时、最少步数 | P1 |
| 断点续玩 | 关闭后恢复上次棋局 | P1 |
| 音效 | 发牌、收牌、胜利音效（可关闭） | P1 |

#### P2 - 可以实现

| 功能 | 描述 | 优先级 |
|------|------|--------|
| 皮肤系统 | 多套卡牌/桌面皮肤 | P2 |
| 成就系统 | 解锁特殊成就 | P2 |
| 排行榜 | 本地/在线排行榜 | P2 |

### 3.2 功能详细设计

#### 3.2.1 难度系统

```
初级（1 Suit）：  4 副黑桃 = 52 张有效牌
中级（2 Suits）： 2 副黑桃 + 2 副红桃 = 52 张有效牌
高级（4 Suits）： 各 1 副花色 = 52 张有效牌
```

#### 3.2.2 计分规则

| 操作 | 分数变化 |
|------|----------|
| 初始分数 | 500 |
| 每次移动 | -1 |
| 完成一组 K-A | +100 |
| 使用提示 | -5 |
| 撤销操作 | 0（不扣分） |

---

## 4. 游戏规则

### 4.1 初始布局

```
列编号：  1   2   3   4   5   6   7   8   9   10
牌数量：  6   6   6   6   5   5   5   5   5   5  = 54 张
翻牌数：  1   1   1   1   1   1   1   1   1   1  = 10 张正面朝上
剩余牌：                                        = 50 张（5 组发牌堆）
```

### 4.2 移动规则

#### 可拖拽条件
1. 被拖拽的牌必须正面朝上
2. 多张牌拖拽时，必须是**同花色**且**连续递减**的序列
   - 例如：♠7-♠6-♠5 ✓
   - 例如：♠7-♥6-♠5 ✗（花色不同）

#### 可放置条件
1. 目标列为空：任何牌/序列都可以放置
2. 目标列非空：被拖拽序列的第一张牌，必须比目标列最后一张牌**小 1**
   - 花色不限（但只有同花色才能整体移动）

### 4.3 发牌规则

- 点击右下角发牌堆，向每列各发一张牌
- **前提条件**：所有列都不能为空
- 共 5 次发牌机会（50 张牌）

### 4.4 收牌规则

- 当任意列形成 **K-A 的完整同花色序列**（13 张）时，自动收走
- 收走的牌移至左下角完成区
- 完成 8 组即获胜

---

## 5. 技术架构

### 5.1 技术选型

| 层级 | 技术 | 版本 | 职责 |
|------|------|------|------|
| 应用框架 | Tauri | 2.x | 原生窗口、系统菜单、文件 IO |
| 后端逻辑 | Rust | 1.75+ | 游戏逻辑、状态管理、持久化 |
| 前端框架 | Svelte | 5.x | UI 渲染、动画、用户交互 |
| 样式方案 | TailwindCSS | 4.x | 快速样式开发 |
| 状态管理 | Svelte Stores | - | 前端响应式状态 |
| 本地存储 | Tauri Plugin Store | - | 配置和统计数据持久化 |

### 5.2 架构图

```
┌─────────────────────────────────────────────────────────┐
│                     Tauri Application                    │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────┐      JSON-RPC      ┌─────────────┐ │
│  │   Frontend      │ ◄───────────────► │   Backend   │ │
│  │   (Svelte 5)    │    Tauri Commands  │   (Rust)    │ │
│  ├─────────────────┤                    ├─────────────┤ │
│  │ • CardRenderer  │                    │ • GameLogic │ │
│  │ • DragSystem    │                    │ • Validator │ │
│  │ • Animator      │                    │ • Scorer    │ │
│  │ • StoreManager  │                    │ • Saver     │ │
│  └─────────────────┘                    └─────────────┘ │
├─────────────────────────────────────────────────────────┤
│                   Tauri Plugin Store                     │
│              (配置 / 统计 / 存档持久化)                    │
└─────────────────────────────────────────────────────────┘
```

### 5.3 目录结构

```
classic-spider/
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs              # 入口
│   │   ├── lib.rs               # 库导出
│   │   ├── game/
│   │   │   ├── mod.rs           # 游戏模块
│   │   │   ├── card.rs          # 卡牌定义
│   │   │   ├── deck.rs          # 牌组操作
│   │   │   ├── rules.rs         # 规则验证
│   │   │   └── state.rs         # 游戏状态
│   │   ├── commands/
│   │   │   ├── mod.rs           # 命令模块
│   │   │   └── game_commands.rs # 游戏命令
│   │   └── storage/
│   │       ├── mod.rs           # 存储模块
│   │       └── save.rs          # 存档管理
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                          # Svelte 前端
│   ├── main.ts                  # 入口
│   ├── App.svelte               # 根组件
│   ├── lib/
│   │   ├── stores/
│   │   │   ├── game.ts          # 游戏状态
│   │   │   └── settings.ts      # 设置状态
│   │   ├── components/
│   │   │   ├── Card.svelte      # 单张牌
│   │   │   ├── Column.svelte    # 一列牌
│   │   │   ├── Stock.svelte     # 发牌堆
│   │   │   ├── Foundation.svelte # 完成区
│   │   │   ├── Toolbar.svelte   # 工具栏
│   │   │   └── GameBoard.svelte # 游戏主区域
│   │   └── utils/
│   │       ├── drag.ts          # 拖拽工具
│   │       └── animations.ts    # 动画工具
│   ├── assets/
│   │   ├── cards/               # SVG 卡牌资源
│   │   └── sounds/              # 音效文件
│   └── styles/
│       └── globals.css          # 全局样式
│
├── docs/                         # 文档
│   └── PRD.md                   # 本文档
│
├── package.json
├── vite.config.ts
├── svelte.config.js
└── tailwind.config.js
```

---

## 6. 视觉设计

### 6.1 设计原则

针对目标用户（中老年群体），采用**经典拟物化风格**：

1. **清晰度优先**：高对比度、大字体、锐利边缘
2. **辨识度强**：经典的 K/Q/J 人物图案，非抽象符号
3. **操作反馈明显**：选中高亮、悬停效果、动画流畅
4. **怀旧感**：经典绿色桌面、木质边框风格

### 6.2 配色方案

| 元素 | 颜色 | 说明 |
|------|------|------|
| 桌面背景 | `#1B5E20` | 经典深绿色 |
| 卡牌正面 | `#FAFAFA` | 米白色 |
| 卡牌背面 | `#1565C0` | 经典蓝色花纹 |
| 选中边框 | `#FFD54F` | 金黄色高亮 |
| 文字主色 | `#212121` | 深灰色 |
| 黑桃花色 | `#000000` | 纯黑 |
| 红桃花色 | `#D32F2F` | 深红 |
| 方块花色 | `#E65100` | 橙红 |
| 梅花花色 | `#388E3C` | 深绿 |

### 6.3 UI 布局

```
┌────────────────────────────────────────────────────────────┐
│  [新游戏 ▼]  难度: [初级 ▼]   分数: 485   时间: 05:23   步数: 15  │
│  [撤销] [提示] [统计]                                        │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐│
│  │   │ │   │ │   │ │   │ │   │ │   │ │   │ │   │ │   │ │   ││
│  │   │ │   │ │   │ │   │ │   │ │   │ │   │ │   │ │   │ │   ││
│  │ ♠ │ │ ♠ │ │ ♠ │ │ ♠ │ │ ♠ │ │ ♠ │ │ ♠ │ │ ♠ │ │ ♠ │ │ ♠ ││
│  │ 7 │ │ 6 │ │ 5 │ │ 4 │ │ 3 │ │ 2 │ │ A │ │ K │ │ Q │ │ J ││
│  └───┘ └───┘ └───┘ └───┘ └───┘ └───┘ └───┘ └───┘ └───┘ └───┘│
│    Col1  Col2  Col3  Col4  Col5  Col6  Col7  Col8  Col9  Col10 │
│                                                            │
│              （10 列牌堆，可拖拽移动）                          │
│                                                            │
├────────────────────────────────────────────────────────────┤
│  ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐    ┌───┐  │
│  │   │ │   │ │   │ │   │ │   │ │   │ │   │ │   │    │   │  │
│  │ 1 │ │ 2 │ │ 3 │ │ 4 │ │ 5 │ │ 6 │ │ 7 │ │ 8 │    │   │  │
│  └───┘ └───┘ └───┘ └───┘ └───┘ └───┘ └───┘ └───┘    └───┘  │
│     完成区（8 组 K-A）                            发牌堆   │
│                                                  (50 张)   │
└────────────────────────────────────────────────────────────┘
```

### 6.4 卡牌尺寸

| 元素 | 尺寸 | 说明 |
|------|------|------|
| 卡牌宽度 | 100 px | 大尺寸便于点击 |
| 卡牌高度 | 140 px | 标准扑克比例 |
| 卡牌间距（纵向） | 25 px | 层叠显示下层牌 |
| 列间距 | 15 px | 便于区分 |

### 6.5 资源需求

#### 卡牌资源（SVG 格式）

```
assets/cards/
├── spades/
│   ├── A.svg, 2.svg, ..., K.svg    # 黑桃 1-13
├── hearts/
│   ├── A.svg, 2.svg, ..., K.svg    # 红桃 1-13
├── diamonds/
│   ├── A.svg, 2.svg, ..., K.svg    # 方块 1-13
├── clubs/
│   ├── A.svg, 2.svg, ..., K.svg    # 梅花 1-13
└── back.svg                         # 卡牌背面
```

推荐资源：
- [Chris Aguilar's Vector Playing Cards](https://github.com/htdebeer/SVGCards) - 开源免费
- [American Valve Card Deck](https://code.google.com/archive/p/vector-playing-cards/) - 公有领域

---

## 7. 交互设计

### 7.1 拖拽系统

#### 交互流程

```
1. mousedown on card
   ├─ 检查是否可拖拽（can_drag_sequence）
   ├─ 是 → 高亮选中，进入拖拽模式
   └─ 否 → 无反应

2. mousemove (dragging)
   ├─ 被拖拽的牌跟随鼠标
   └─ 经过的有效目标列高亮

3. mouseup
   ├─ 在有效目标上 → 执行移动
   ├─ 在无效位置 → 弹回原位
   └─ 检查是否完成 K-A 序列
```

#### 双击快速移动

```
dblclick on card
├─ 查找第一个合法目标列
├─ 找到 → 自动移动（带动画）
└─ 未找到 → 抖动反馈
```

### 7.2 动画设计

| 动画类型 | 持续时间 | 缓动函数 | 说明 |
|----------|----------|----------|------|
| 卡牌悬停 | 150ms | ease-out | 上浮 5px + 阴影增强 |
| 拖拽选中 | 100ms | ease-out | 轻微放大 1.05x |
| 放置成功 | 200ms | ease-out | 平滑移动到目标位置 |
| 放置失败 | 300ms | ease-in-out | 弹回原位 |
| 发牌动画 | 400ms | ease-out | 从发牌堆飞向各列 |
| 收牌动画 | 600ms | ease-in | 完成序列飞向完成区 |
| 胜利动画 | 1500ms | bounce | 卡牌飞舞庆祝 |

### 7.3 键盘支持

| 快捷键 | 功能 |
|--------|------|
| `Ctrl + N` | 新游戏 |
| `Ctrl + Z` | 撤销 |
| `Ctrl + Y` | 重做 |
| `H` | 提示 |
| `1-4` | 选择难度 |
| `Esc` | 取消当前操作 |

---

## 8. 数据结构

### 8.1 Rust 核心类型

```rust
// 花色枚举
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Suit {
    Spade,   // 黑桃 ♠
    Heart,   // 红桃 ♥
    Diamond, // 方块 ♦
    Club,    // 梅花 ♣
}

// 卡牌结构
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Card {
    pub id: u32,         // 唯一标识
    pub suit: Suit,      // 花色
    pub value: u8,       // 1-13 (A=1, K=13)
    pub is_face_up: bool, // 是否正面朝上
}

// 游戏状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub columns: Vec<Vec<Card>>,    // 10 列牌堆
    pub stock: Vec<Card>,           // 发牌堆
    pub completed: Vec<Vec<Card>>,  // 完成区
    pub score: i32,                 // 当前分数
    pub moves: u32,                 // 移动次数
    pub time_seconds: u32,          // 游戏时长
    pub difficulty: u8,             // 难度 (1-3)
}

// 历史记录（用于撤销）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub states: Vec<GameState>,
    pub current_index: usize,
}

// 统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub games_played: u32,
    pub games_won: u32,
    pub best_score: i32,
    pub best_time: u32,
    pub best_moves: u32,
    pub current_streak: u32,
    pub best_streak: u32,
}
```

### 8.2 前端状态（Svelte Store）

```typescript
// 游戏状态
interface GameState {
  columns: Card[][];       // 10 列牌堆
  stock: Card[];           // 发牌堆
  completed: Card[][];     // 完成区
  score: number;
  moves: number;
  timeSeconds: number;
  difficulty: 1 | 2 | 3;
  isGameOver: boolean;
  isWin: boolean;
}

// UI 状态
interface UIState {
  selectedCards: { colIndex: number; startIndex: number } | null;
  draggingCards: Card[] | null;
  dragPosition: { x: number; y: number } | null;
  hintHighlight: { colIndex: number; startIndex: number } | null;
  isAnimating: boolean;
}

// 设置状态
interface Settings {
  difficulty: 1 | 2 | 3;
  soundEnabled: boolean;
  animationsEnabled: boolean;
  cardBackStyle: 'classic' | 'modern' | 'minimal';
  tableColor: string;
}
```

---

## 9. API 设计

### 9.1 Tauri Commands（Rust → 前端）

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `new_game` | `difficulty: u8` | `GameState` | 开始新游戏 |
| `get_state` | - | `GameState` | 获取当前状态 |
| `move_cards` | `from_col, start_idx, to_col` | `Result<GameState, String>` | 移动卡牌 |
| `deal_cards` | - | `Result<GameState, String>` | 发牌 |
| `undo` | - | `Result<GameState, String>` | 撤销 |
| `redo` | - | `Result<GameState, String>` | 重做 |
| `get_hint` | - | `Option<Hint>` | 获取提示 |
| `save_game` | - | `Result<(), String>` | 保存游戏 |
| `load_game` | - | `Result<GameState, String>` | 加载游戏 |
| `get_statistics` | - | `Statistics` | 获取统计数据 |
| `reset_statistics` | - | `()` | 重置统计 |

### 9.2 命令示例

```rust
#[tauri::command]
fn move_cards(
    state: State<AppState>,
    from_col: usize,
    start_idx: usize,
    to_col: usize,
) -> Result<GameState, String> {
    let mut game = state.0.lock().unwrap();

    // 验证移动合法性
    if !game.can_drag_sequence(from_col, start_idx) {
        return Err("无法拖拽该序列".into());
    }

    let cards = game.columns[from_col][start_idx..].to_vec();
    if !game.is_valid_destination(&cards, to_col) {
        return Err("无效的目标位置".into());
    }

    // 保存历史（用于撤销）
    game.save_history();

    // 执行移动
    let moving = game.columns[from_col].split_off(start_idx);
    game.columns[to_col].extend(moving);

    // 翻开原列最后一张牌
    if let Some(card) = game.columns[from_col].last_mut() {
        card.is_face_up = true;
    }

    // 检查是否完成序列
    game.check_complete_sequence(to_col);

    // 更新分数和步数
    game.moves += 1;
    game.score -= 1;

    Ok(game.clone())
}
```

---

## 10. 非功能性需求

### 10.1 性能要求

| 指标 | 目标值 | 测量方法 |
|------|--------|----------|
| 安装包大小 | < 10 MB | 构建产物大小 |
| 内存占用（空闲） | < 50 MB | 任务管理器 |
| 冷启动时间 | < 1 秒 | 从点击到可交互 |
| 拖拽响应延迟 | < 16ms | 60fps 流畅度 |
| 动画帧率 | ≥ 60 fps | Chrome DevTools |

### 10.2 兼容性要求

| 平台 | 最低版本 |
|------|----------|
| Windows | Windows 10 (1809+) |
| macOS | macOS 10.15 (Catalina) |
| Linux | Ubuntu 20.04+ / Fedora 35+ |

### 10.3 可访问性要求

- [ ] 支持高对比度模式
- [ ] 支持键盘完整操作
- [ ] 最小点击区域 44x44 px
- [ ] 动画可关闭
- [ ] 色盲友好模式（花色使用不同形状）

### 10.4 数据持久化

需要持久化的数据：

1. **游戏存档**
   - 当前棋局状态
   - 历史记录（撤销栈）
   - 自动保存间隔：每次移动后

2. **用户设置**
   - 难度偏好
   - 音效开关
   - 动画开关
   - 皮肤选择

3. **统计数据**
   - 总局数/胜局数
   - 最高分/最短用时/最少步数
   - 连胜记录

---

## 11. 里程碑规划

### Phase 1: 基础架构（Week 1-2）

- [ ] Tauri 项目初始化
- [ ] Rust 核心数据结构
- [ ] 基础游戏逻辑（洗牌、发牌）
- [ ] Svelte 前端脚手架
- [ ] 基础 UI 布局

### Phase 2: 核心玩法（Week 3-4）

- [ ] 拖拽系统实现
- [ ] 移动规则验证
- [ ] 发牌功能
- [ ] 收牌检测
- [ ] 胜利判定
- [ ] 计分系统

### Phase 3: 完善体验（Week 5-6）

- [ ] 撤销/重做功能
- [ ] 提示系统
- [ ] 动画效果
- [ ] 音效集成
- [ ] 断点续玩

### Phase 4: 打磨发布（Week 7-8）

- [ ] 统计数据系统
- [ ] 皮肤系统
- [ ] 多语言支持
- [ ] 性能优化
- [ ] 测试与 Bug 修复
- [ ] 打包发布

---

## 附录

### A. 参考资料

- [Tauri 2.0 官方文档](https://v2.tauri.app/)
- [Svelte 5 文档](https://svelte-5-preview.vercel.app/)
- [蜘蛛纸牌规则 - Wikipedia](https://zh.wikipedia.org/wiki/蜘蛛纸牌)

### B. 变更日志

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| 1.0.0 | 2026-03-13 | 初始版本 |

---

*文档维护：开发团队*
*最后更新：2026-03-13*
