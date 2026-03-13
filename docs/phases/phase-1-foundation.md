# Phase 1: 基础架构 (Week 1-2)

## 概述

**目标**: 搭建项目骨架，实现最小可运行版本（能显示牌面）

**预计工期**: 2 周

**前置依赖**: 无

---

## 任务清单

### Task 1.1: Tauri 项目初始化

**优先级**: P0

**文件路径**:
- `src-tauri/` (Rust 后端目录)
- `src/` (Svelte 前端目录)
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

**描述**: 初始化 Tauri 2.x + Svelte 5 项目，配置开发环境

**实现要点**:
```bash
# 创建项目
npm create tauri-app@latest classic-spider -- --template svelte-ts

# 安装依赖
cd classic-spider
npm install

# 配置 TailwindCSS
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

**tauri.conf.json 关键配置**:
```json
{
  "productName": "Classic Spider",
  "version": "0.1.0",
  "identifier": "com.classic-spider.app",
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "Classic Spider Solitaire",
        "width": 1024,
        "height": 768,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true
      }
    ]
  }
}
```

**验收标准**:
- [ ] `npm run tauri dev` 能启动应用窗口
- [ ] 显示 Svelte 默认页面
- [ ] 热重载正常工作

---

### Task 1.2: Rust 核心数据结构 - Card

**优先级**: P0

**文件路径**:
- `src-tauri/src/game/mod.rs`
- `src-tauri/src/game/card.rs`

**描述**: 定义花色、卡牌核心数据结构

**实现要点**:

```rust
// src-tauri/src/game/mod.rs
pub mod card;
pub mod deck;
pub mod state;

// src-tauri/src/game/card.rs
use serde::{Deserialize, Serialize};

/// 花色（蜘蛛纸牌只用黑桃，但支持多难度）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Suit {
    Spades,   // ♠ 黑桃
    Hearts,   // ♥ 红心
    Diamonds, // ♦ 方块
    Clubs,    // ♣ 梅花
}

impl Suit {
    /// 是否为红色花色
    pub fn is_red(&self) -> bool {
        matches!(self, Suit::Hearts | Suit::Diamonds)
    }

    /// 获取花色符号
    pub fn symbol(&self) -> &'static str {
        match self {
            Suit::Spades => "♠",
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
        }
    }
}

/// 卡牌面值 (1-13, 1=A, 11=J, 12=Q, 13=K)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Rank(pub u8);

impl Rank {
    pub const ACE: Rank = Rank(1);
    pub const JACK: Rank = Rank(11);
    pub const QUEEN: Rank = Rank(12);
    pub const KING: Rank = Rank(13);

    /// 创建面值，返回 None 如果不在 1-13 范围
    pub fn new(value: u8) -> Option<Self> {
        if value >= 1 && value <= 13 {
            Some(Rank(value))
        } else {
            None
        }
    }

    /// 获取显示字符串
    pub fn display(&self) -> String {
        match self.0 {
            1 => "A".to_string(),
            11 => "J".to_string(),
            12 => "Q".to_string(),
            13 => "K".to_string(),
            n => n.to_string(),
        }
    }

    /// 下一个面值（递增）
    pub fn next(&self) -> Option<Rank> {
        Rank::new(self.0 + 1)
    }

    /// 上一个面值（递减）
    pub fn prev(&self) -> Option<Rank> {
        if self.0 > 1 {
            Some(Rank(self.0 - 1))
        } else {
            None
        }
    }
}

/// 单张卡牌
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
    pub face_up: bool, // 是否正面朝上
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Card {
            suit,
            rank,
            face_up: false,
        }
    }

    /// 翻开卡牌
    pub fn flip_up(&mut self) {
        self.face_up = true;
    }

    /// 翻转卡牌
    pub fn flip(&mut self) {
        self.face_up = !self.face_up;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_ordering() {
        assert!(Rank::KING > Rank::QUEEN);
        assert!(Rank::ACE < Rank::KING);
    }

    #[test]
    fn test_suit_color() {
        assert!(!Suit::Spades.is_red());
        assert!(Suit::Hearts.is_red());
    }
}
```

**验收标准**:
- [ ] Card, Suit, Rank 结构定义完整
- [ ] 序列化/反序列化正常
- [ ] 单元测试通过

---

### Task 1.3: Rust 核心数据结构 - Deck

**优先级**: P0

**文件路径**:
- `src-tauri/src/game/deck.rs`

**描述**: 牌组生成、洗牌功能

**实现要点**:

```rust
// src-tauri/src/game/deck.rs
use super::card::{Card, Rank, Suit};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

/// 难度级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    /// 初级：单花色（只使用黑桃）
    Easy,
    /// 中级：双花色（黑桃 + 红心）
    Medium,
    /// 高级：四花色
    Hard,
}

impl Default for Difficulty {
    fn default() -> Self {
        Difficulty::Easy
    }
}

/// 牌组
#[derive(Debug, Clone)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    /// 创建蜘蛛纸牌用牌组（8副牌，104张）
    pub fn new_spider_deck(difficulty: Difficulty) -> Self {
        let suits = match difficulty {
            Difficulty::Easy => vec![Suit::Spades; 8],      // 8副黑桃
            Difficulty::Medium => vec![Suit::Spades, Suit::Hearts; 4], // 各4副
            Difficulty::Hard => vec![Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs; 2], // 各2副
        };

        let mut cards = Vec::with_capacity(104);
        for suit in suits {
            for rank in 1..=13 {
                cards.push(Card::new(suit, Rank(rank)));
            }
        }

        Deck { cards }
    }

    /// 洗牌
    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }

    /// 发出一张牌
    pub fn deal_one(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// 剩余牌数
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_creation() {
        let deck = Deck::new_spider_deck(Difficulty::Easy);
        assert_eq!(deck.len(), 104);
    }

    #[test]
    fn test_shuffle_changes_order() {
        let mut deck1 = Deck::new_spider_deck(Difficulty::Easy);
        let deck2 = Deck::new_spider_deck(Difficulty::Easy);

        deck1.shuffle();

        // 洗牌后顺序大概率不同（极小概率相同）
        // 这里只验证长度不变
        assert_eq!(deck1.len(), deck2.len());
    }
}
```

**验收标准**:
- [ ] 104 张牌生成正确
- [ ] 三种难度牌组正确
- [ ] 洗牌功能正常

---

### Task 1.4: Rust 核心数据结构 - State

**优先级**: P0

**文件路径**:
- `src-tauri/src/game/state.rs`

**描述**: 游戏状态结构定义

**实现要点**:

```rust
// src-tauri/src/game/state.rs
use super::card::Card;
use super::deck::{Deck, Difficulty};
use serde::{Deserialize, Serialize};

/// 最多 10 列牌堆
pub const COLUMN_COUNT: usize = 10;

/// 最多 8 组完成区
pub const FOUNDATION_COUNT: usize = 8;

/// 每次发牌数量
pub const DEAL_COUNT: usize = 10;

/// 牌堆（一列牌）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Column {
    pub cards: Vec<Card>,
}

impl Column {
    pub fn new() -> Self {
        Column { cards: Vec::new() }
    }

    /// 获取可见的牌（从上往下，连续正面朝上的牌）
    pub fn visible_cards(&self) -> &[Card] {
        // 找到第一张正面朝上的牌的位置
        let first_face_up = self.cards.iter().position(|c| c.face_up);
        match first_face_up {
            Some(idx) => &self.cards[idx..],
            None => &[],
        }
    }

    /// 获取最上面的牌
    pub fn top_card(&self) -> Option<&Card> {
        self.cards.last()
    }

    /// 添加牌
    pub fn push(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// 添加多张牌
    pub fn extend(&mut self, cards: Vec<Card>) {
        self.cards.extend(cards);
    }

    /// 移除顶部的 n 张牌
    pub fn pop_n(&mut self, n: usize) -> Vec<Card> {
        let split_at = self.cards.len().saturating_sub(n);
        self.cards.split_off(split_at)
    }

    /// 牌数
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

/// 完成区（已完成的 K-A 序列）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Foundation {
    pub count: usize, // 完成的组数
}

impl Foundation {
    pub fn new() -> Self {
        Foundation { count: 0 }
    }

    /// 添加一组完成的序列
    pub fn add_completed(&mut self) {
        self.count += 1;
    }
}

/// 游戏状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// 10 列牌堆
    pub columns: [Column; COLUMN_COUNT],
    /// 发牌堆（剩余牌）
    pub stock: Vec<Card>,
    /// 完成区
    pub foundations: [Foundation; FOUNDATION_COUNT],
    /// 当前难度
    pub difficulty: Difficulty,
    /// 当前分数
    pub score: i32,
    /// 移动次数
    pub moves: u32,
    /// 游戏是否结束
    pub game_over: bool,
    /// 是否胜利
    pub won: bool,
}

impl GameState {
    /// 创建新游戏
    pub fn new(difficulty: Difficulty) -> Self {
        let mut deck = Deck::new_spider_deck(difficulty);
        deck.shuffle();

        // 初始化 10 列
        let mut columns: [Column; COLUMN_COUNT] = Default::default();

        // 发初始牌：前 4 列 6 张，后 6 列 5 张，共 54 张
        // 只有最上面一张正面朝上
        for (i, col) in columns.iter_mut().enumerate() {
            let card_count = if i < 4 { 6 } else { 5 };
            for j in 0..card_count {
                if let Some(mut card) = deck.deal_one() {
                    // 只有每列最后一张正面朝上
                    if j == card_count - 1 {
                        card.flip_up();
                    }
                    col.push(card);
                }
            }
        }

        // 剩余 50 张放入发牌堆
        let mut stock = Vec::new();
        while let Some(card) = deck.deal_one() {
            stock.push(card);
        }

        GameState {
            columns,
            stock,
            foundations: Default::default(),
            difficulty,
            score: 500,
            moves: 0,
            game_over: false,
            won: false,
        }
    }

    /// 获取发牌堆剩余发牌次数（每次发 10 张）
    pub fn deals_remaining(&self) -> usize {
        self.stock.len() / DEAL_COUNT
    }

    /// 是否可以发牌
    pub fn can_deal(&self) -> bool {
        // 发牌堆有牌且所有列都不为空
        !self.stock.is_empty() && self.columns.iter().all(|c| !c.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_state() {
        let state = GameState::new(Difficulty::Easy);

        // 检查总牌数
        let total_cards: usize = state.columns.iter().map(|c| c.len()).sum::<usize>() + state.stock.len();
        assert_eq!(total_cards, 104);

        // 检查发牌堆剩余
        assert_eq!(state.stock.len(), 50);
        assert_eq!(state.deals_remaining(), 5);

        // 检查初始分数
        assert_eq!(state.score, 500);
    }
}
```

**验收标准**:
- [ ] 游戏状态结构完整
- [ ] 新游戏初始化正确
- [ ] 发牌堆计算正确

---

### Task 1.5: Tauri Commands - 基础命令

**优先级**: P0

**文件路径**:
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/commands/game.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/main.rs`

**描述**: 实现 Tauri 命令，连接前后端

**实现要点**:

```rust
// src-tauri/src/commands/mod.rs
pub mod game;

// src-tauri/src/commands/game.rs
use crate::game::deck::Difficulty;
use crate::game::state::GameState;
use std::sync::Mutex;
use tauri::State;

/// 游戏状态管理
pub struct GameManager {
    pub state: Mutex<Option<GameState>>,
}

impl Default for GameManager {
    fn default() -> Self {
        GameManager {
            state: Mutex::new(None),
        }
    }
}

/// 创建新游戏
#[tauri::command]
pub fn new_game(difficulty: Difficulty, manager: State<GameManager>) -> GameState {
    let state = GameState::new(difficulty);
    let mut game_state = manager.state.lock().unwrap();
    *game_state = Some(state.clone());
    state
}

/// 获取当前游戏状态
#[tauri::command]
pub fn get_state(manager: State<GameManager>) -> Option<GameState> {
    let state = manager.state.lock().unwrap();
    state.clone()
}

// src-tauri/src/lib.rs
mod commands;
mod game;

use commands::game::GameManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(GameManager::default())
        .invoke_handler(tauri::generate_handler![
            commands::game::new_game,
            commands::game::get_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// src-tauri/src/main.rs
fn main() {
    classic_spider_lib::run()
}
```

**Cargo.toml 依赖**:
```toml
[dependencies]
tauri = { version = "2", features = ["unstable"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rand = "0.8"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

**验收标准**:
- [ ] `new_game` 命令返回有效状态
- [ ] `get_state` 命令返回当前状态
- [ ] 前端能调用命令并获取响应

---

### Task 1.6: Svelte 组件 - Card

**优先级**: P0

**文件路径**:
- `src/lib/components/Card.svelte`

**描述**: 单张卡牌渲染组件

**实现要点**:

```svelte
<!-- src/lib/components/Card.svelte -->
<script lang="ts">
  interface Card {
    suit: string;
    rank: { display: string; 0: number };
    face_up: boolean;
  }

  interface Props {
    card: Card;
    selected?: boolean;
    onclick?: () => void;
  }

  let { card, selected = false, onclick }: Props = $props();

  // 花色符号映射
  const suitSymbols: Record<string, string> = {
    spades: '♠',
    hearts: '♥',
    diamonds: '♦',
    clubs: '♣',
  };

  // 是否红色
  $derived(isRed = card.suit === 'hearts' || card.suit === 'diamonds');
</script>

<button
  class="card {card.face_up ? 'face-up' : 'face-down'} {isRed ? 'red' : 'black'}"
  class:selected
  onclick={() => onclick?.()}
  disabled={!card.face_up}
>
  {#if card.face_up}
    <div class="card-corner top">
      <span class="rank">{card.rank.display}</span>
      <span class="suit">{suitSymbols[card.suit]}</span>
    </div>
    <div class="card-center">
      <span class="suit-large">{suitSymbols[card.suit]}</span>
    </div>
    <div class="card-corner bottom">
      <span class="rank">{card.rank.display}</span>
      <span class="suit">{suitSymbols[card.suit]}</span>
    </div>
  {:else}
    <div class="card-back">
      <div class="pattern"></div>
    </div>
  {/if}
</button>

<style>
  .card {
    width: 70px;
    height: 100px;
    border-radius: 8px;
    background: white;
    border: 1px solid #ccc;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    position: relative;
    cursor: pointer;
    transition: transform 0.1s, box-shadow 0.1s;
    user-select: none;
    flex-shrink: 0;
  }

  .card:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
  }

  .card.selected {
    box-shadow: 0 0 0 3px #3b82f6;
  }

  .card.face-down {
    cursor: default;
  }

  .card.red {
    color: #dc2626;
  }

  .card.black {
    color: #1f2937;
  }

  .card-corner {
    position: absolute;
    display: flex;
    flex-direction: column;
    align-items: center;
    font-size: 12px;
    line-height: 1;
  }

  .card-corner.top {
    top: 4px;
    left: 4px;
  }

  .card-corner.bottom {
    bottom: 4px;
    right: 4px;
    transform: rotate(180deg);
  }

  .card-center {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
  }

  .suit-large {
    font-size: 32px;
  }

  .card-back {
    width: 100%;
    height: 100%;
    background: linear-gradient(135deg, #1e40af 0%, #3b82f6 100%);
    border-radius: 7px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .pattern {
    width: 50px;
    height: 70px;
    background: repeating-linear-gradient(
      45deg,
      transparent,
      transparent 5px,
      rgba(255, 255, 255, 0.1) 5px,
      rgba(255, 255, 255, 0.1) 10px
    );
    border-radius: 4px;
  }
</style>
```

**验收标准**:
- [ ] 正面卡牌显示花色和面值
- [ ] 背面卡牌显示图案
- [ ] 红色/黑色区分正确
- [ ] 选中状态样式正确

---

### Task 1.7: Svelte 组件 - Column

**优先级**: P0

**文件路径**:
- `src/lib/components/Column.svelte`

**描述**: 一列牌堆组件

**实现要点**:

```svelte
<!-- src/lib/components/Column.svelte -->
<script lang="ts">
  import Card from './Card.svelte';

  interface CardData {
    suit: string;
    rank: { display: string; 0: number };
    face_up: boolean;
  }

  interface Props {
    cards: CardData[];
    columnIndex: number;
    onCardClick?: (cardIndex: number) => void;
  }

  let { cards, columnIndex, onCardClick }: Props = $props();

  // 卡牌堆叠偏移
  const faceDownOffset = 8;  // 背面牌偏移
  const faceUpOffset = 20;   // 正面牌偏移
</script>

<div class="column" role="list" aria-label="Column {columnIndex + 1}">
  {#each cards as card, index}
    <div
      class="card-wrapper"
      style="top: {calculateOffset(index)}px"
      role="listitem"
    >
      <Card
        {card}
        onclick={() => onCardClick?.(index)}
      />
    </div>
  {/each}

  {#if cards.length === 0}
    <div class="empty-slot">
      <span class="placeholder">K</span>
    </div>
  {/if}
</div>

<script context="module">
  // 计算每张牌的偏移位置
  function calculateOffset(cards: CardData[], index: number): number {
    let offset = 0;
    for (let i = 0; i < index; i++) {
      offset += cards[i].face_up ? 20 : 8;
    }
    return offset;
  }
</script>

<style>
  .column {
    position: relative;
    min-height: 400px;
    width: 80px;
    padding: 5px;
  }

  .card-wrapper {
    position: absolute;
    left: 5px;
  }

  .empty-slot {
    width: 70px;
    height: 100px;
    border: 2px dashed rgba(255, 255, 255, 0.3);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 5px;
  }

  .placeholder {
    font-size: 24px;
    color: rgba(255, 255, 255, 0.3);
    font-weight: bold;
  }
</style>
```

**验收标准**:
- [ ] 卡牌正确堆叠显示
- [ ] 空列显示占位符
- [ ] 点击事件正确传递

---

### Task 1.8: Svelte 组件 - GameBoard

**优先级**: P0

**文件路径**:
- `src/lib/components/GameBoard.svelte`

**描述**: 游戏主区域布局组件

**实现要点**:

```svelte
<!-- src/lib/components/GameBoard.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Column from './Column.svelte';

  interface CardData {
    suit: string;
    rank: { display: string; 0: number };
    face_up: boolean;
  }

  interface ColumnData {
    cards: CardData[];
  }

  interface GameState {
    columns: ColumnData[];
    stock: CardData[];
    foundations: { count: number }[];
    score: number;
    moves: number;
    difficulty: string;
  }

  let gameState = $state<GameState | null>(null);

  // 初始化游戏
  async function initGame() {
    try {
      gameState = await invoke('new_game', { difficulty: 'Easy' });
    } catch (error) {
      console.error('Failed to create game:', error);
    }
  }

  onMount(() => {
    initGame();
  });

  function handleCardClick(columnIndex: number, cardIndex: number) {
    console.log(`Clicked card at column ${columnIndex}, index ${cardIndex}`);
    // Phase 2 实现拖拽逻辑
  }
</script>

<div class="game-container">
  <!-- 顶部工具栏 -->
  <header class="toolbar">
    <div class="game-info">
      <span>分数: {gameState?.score ?? 0}</span>
      <span>移动: {gameState?.moves ?? 0}</span>
    </div>
    <div class="actions">
      <button onclick={initGame}>新游戏</button>
    </div>
  </header>

  <!-- 游戏主区域 -->
  <main class="game-board">
    {#if gameState}
      <div class="columns">
        {#each gameState.columns as column, index}
          <Column
            cards={column.cards}
            columnIndex={index}
            onCardClick={(cardIndex) => handleCardClick(index, cardIndex)}
          />
        {/each}
      </div>

      <!-- 发牌堆 -->
      <div class="stock-area">
        <div class="stock">
          {#each Array(gameState.stock.length / 10) as _, i}
            <div class="stock-pile" style="left: {i * 15}px"></div>
          {/each}
        </div>
        <span class="deals-remaining">剩余: {Math.floor(gameState.stock.length / 10)} 次</span>
      </div>

      <!-- 完成区 -->
      <div class="foundation-area">
        {#each gameState.foundations as foundation, i}
          <div class="foundation">
            {#if foundation.count > 0}
              <span class="completed">{foundation.count}</span>
            {/if}
          </div>
        {/each}
      </div>
    {:else}
      <div class="loading">加载中...</div>
    {/if}
  </main>
</div>

<style>
  .game-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #1a472a; /* 经典绿色桌面 */
    color: white;
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 20px;
    background: rgba(0, 0, 0, 0.2);
  }

  .game-info {
    display: flex;
    gap: 20px;
  }

  .game-board {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 20px;
    overflow: auto;
  }

  .columns {
    display: flex;
    justify-content: center;
    gap: 10px;
    margin-bottom: 20px;
  }

  .stock-area {
    display: flex;
    align-items: center;
    gap: 20px;
    margin-top: auto;
  }

  .stock {
    position: relative;
    height: 100px;
    width: 100px;
  }

  .stock-pile {
    position: absolute;
    width: 70px;
    height: 100px;
    background: linear-gradient(135deg, #1e40af 0%, #3b82f6 100%);
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  .foundation-area {
    position: fixed;
    bottom: 20px;
    right: 20px;
    display: flex;
    gap: 5px;
  }

  .foundation {
    width: 50px;
    height: 70px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .completed {
    font-size: 18px;
    font-weight: bold;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    font-size: 18px;
  }
</style>
```

**验收标准**:
- [ ] 10 列牌堆正确显示
- [ ] 发牌堆显示正确
- [ ] 分数/移动次数显示正确

---

### Task 1.9: Svelte 根组件配置

**优先级**: P0

**文件路径**:
- `src/App.svelte`
- `src/main.ts`
- `src/app.css`

**描述**: 配置根组件和全局样式

**实现要点**:

```svelte
<!-- src/App.svelte -->
<script lang="ts">
  import GameBoard from './lib/components/GameBoard.svelte';
</script>

<GameBoard />
```

```css
/* src/app.css */
@tailwind base;
@tailwind components;
@tailwind utilities;

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
  overflow: hidden;
}
```

```typescript
// src/main.ts
import './app.css';
import App from './App.svelte';
import { mount } from 'svelte';

const app = mount(App, {
  target: document.getElementById('app')!,
});

export default app;
```

**验收标准**:
- [ ] 应用正确渲染
- [ ] 全局样式生效

---

### Task 1.10: TailwindCSS 配置

**优先级**: P0

**文件路径**:
- `tailwind.config.js`
- `postcss.config.js`

**描述**: 配置 TailwindCSS

**实现要点**:

```javascript
// tailwind.config.js
/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        'table-green': '#1a472a',
        'card-back': '#1e40af',
      },
    },
  },
  plugins: [],
};
```

```javascript
// postcss.config.js
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
};
```

**验收标准**:
- [ ] Tailwind 类正常工作
- [ ] 自定义颜色可用

---

## 阶段验收清单

- [ ] 应用能启动并显示窗口
- [ ] 能生成随机牌局并渲染 10 列牌堆
- [ ] 卡牌能区分正面/背面
- [ ] 初始分数显示为 500
- [ ] 发牌堆显示 5 次剩余

## 风险与依赖

### 风险
1. **Tauri 2.x API 变化**: Tauri 2 尚在开发中，API 可能变化
   - 缓解：使用稳定版本，关注官方文档

2. **Rust 编译时间**: 首次编译可能较慢
   - 缓解：使用增量编译，配置好 Cargo.toml

### 依赖
- Node.js 18+
- Rust 1.70+
- 系统原生依赖（参考 Tauri 官方文档）

## 下一阶段

完成本阶段后，进入 [Phase 2: 核心玩法](./phase-2-gameplay.md)
