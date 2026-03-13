# 🏗️ 技术架构设计文档

> Classic Spider Solitaire - 技术实现指南
> 版本：1.0.0 | 日期：2026-03-13

---

## 目录

1. [Rust 后端设计](#1-rust-后端设计)
2. [Svelte 前端设计](#2-svelte-前端设计)
3. [通信协议](#3-通信协议)
4. [数据持久化](#4-数据持久化)

---

## 1. Rust 后端设计

### 1.1 模块结构

```
src-tauri/src/
├── main.rs              # 应用入口
├── lib.rs               # 库导出
├── game/
│   ├── mod.rs           # 模块导出
│   ├── card.rs          # 卡牌定义
│   ├── deck.rs          # 牌组操作
│   ├── rules.rs         # 规则验证
│   ├── state.rs         # 游戏状态
│   └── history.rs       # 历史记录
├── commands/
│   ├── mod.rs           # 模块导出
│   └── game.rs          # 游戏命令
└── storage/
    ├── mod.rs           # 模块导出
    └── save.rs          # 存档管理
```

### 1.2 核心类型定义

#### card.rs

```rust
use serde::{Deserialize, Serialize};

/// 花色枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Suit {
    Spade,   // 黑桃 ♠
    Heart,   // 红桃 ♥
    Diamond, // 方块 ♦
    Club,    // 梅花 ♣
}

impl Suit {
    /// 是否为红色花色
    pub fn is_red(self) -> bool {
        matches!(self, Suit::Heart | Suit::Diamond)
    }

    /// 获取花色符号
    pub fn symbol(self) -> &'static str {
        match self {
            Suit::Spade => "♠",
            Suit::Heart => "♥",
            Suit::Diamond => "♦",
            Suit::Club => "♣",
        }
    }

    /// 根据难度获取可用的花色列表
    pub fn for_difficulty(difficulty: u8) -> Vec<Self> {
        match difficulty {
            1 => vec![Suit::Spade; 4],  // 初级：全黑桃
            2 => vec![Suit::Spade, Suit::Heart, Suit::Spade, Suit::Heart], // 中级
            _ => vec![Suit::Spade, Suit::Heart, Suit::Diamond, Suit::Club], // 高级
        }
    }
}

/// 卡牌结构
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    /// 唯一标识符
    pub id: u32,
    /// 花色
    pub suit: Suit,
    /// 点数 (1=A, 11=J, 12=Q, 13=K)
    pub value: u8,
    /// 是否正面朝上
    pub is_face_up: bool,
}

impl Card {
    /// 创建新卡牌
    pub fn new(id: u32, suit: Suit, value: u8) -> Self {
        Self {
            id,
            suit,
            value,
            is_face_up: false,
        }
    }

    /// 获取点数显示文本
    pub fn display_value(self) -> String {
        match self.value {
            1 => "A".to_string(),
            11 => "J".to_string(),
            12 => "Q".to_string(),
            13 => "K".to_string(),
            v => v.to_string(),
        }
    }
}
```

#### deck.rs

```rust
use super::card::{Card, Suit};
use rand::seq::SliceRandom;
use rand::thread_rng;

/// 生成一副完整的蜘蛛纸牌牌组（104张）
pub fn generate_deck(difficulty: u8) -> Vec<Card> {
    let mut deck = Vec::with_capacity(104);
    let suits = Suit::for_difficulty(difficulty);
    let mut id_counter = 0u32;

    // 每种花色配置生成 2 副牌（每种花色 26 张，共 104 张）
    for &suit in &suits {
        for _ in 0..2 {
            for value in 1..=13 {
                deck.push(Card::new(id_counter, suit, value));
                id_counter += 1;
            }
        }
    }

    // 洗牌
    let mut rng = thread_rng();
    deck.shuffle(&mut rng);

    deck
}

/// 将牌组分配到初始布局
pub fn deal_initial_cards(deck: &mut Vec<Card>) -> (Vec<Vec<Card>>, Vec<Card>) {
    let mut columns = vec![Vec::new(); 10];

    // 前 4 列 6 张，后 6 列 5 张，共 54 张
    let column_sizes = [6, 6, 6, 6, 5, 5, 5, 5, 5, 5];
    let mut card_idx = 0;

    for (col_idx, &size) in column_sizes.iter().enumerate() {
        for i in 0..size {
            let mut card = deck.pop().unwrap();
            // 每列最后一张翻开
            if i == size - 1 {
                card.is_face_up = true;
            }
            columns[col_idx].push(card);
            card_idx += 1;
        }
    }

    // 剩余 50 张作为发牌堆
    let stock = deck.drain(..).collect();

    (columns, stock)
}
```

#### rules.rs

```rust
use super::card::Card;

/// 检查序列是否可以整体拖拽
///
/// 条件：
/// 1. 所有牌必须正面朝上
/// 2. 必须是同花色
/// 3. 必须是连续递减序列
pub fn can_drag_sequence(cards: &[Card]) -> bool {
    if cards.is_empty() {
        return false;
    }

    // 检查是否全部翻开
    if !cards.iter().all(|c| c.is_face_up) {
        return false;
    }

    // 单张牌总是可以拖拽
    if cards.len() == 1 {
        return true;
    }

    // 检查同花色且连续递减
    let first_suit = cards[0].suit;
    for window in cards.windows(2) {
        let current = &window[0];
        let next = &window[1];

        if current.suit != first_suit || current.suit != next.suit {
            return false;
        }
        if current.value != next.value + 1 {
            return false;
        }
    }

    true
}

/// 检查是否可以放置到目标列
///
/// 条件：
/// 1. 目标列为空时，任何序列都可以放置
/// 2. 目标列非空时，移动序列第一张必须比目标最后一张小 1
pub fn can_place_on(moving_cards: &[Card], target_column: &[Card]) -> bool {
    if moving_cards.is_empty() {
        return false;
    }

    // 空列可以放任何牌
    if target_column.is_empty() {
        return true;
    }

    let target_card = target_column.last().unwrap();
    let first_moving = &moving_cards[0];

    // 只需要数值比目标小 1，花色不限
    target_card.value == first_moving.value + 1
}

/// 检查是否形成完整的 K-A 序列
///
/// 条件：
/// 1. 必须是 13 张牌
/// 2. 必须是同花色
/// 3. 必须是 K 到 A 的完整序列
pub fn is_complete_sequence(cards: &[Card]) -> bool {
    if cards.len() != 13 {
        return false;
    }

    let suit = cards[0].suit;

    for (i, card) in cards.iter().enumerate() {
        // 检查花色
        if card.suit != suit {
            return false;
        }
        // 检查数值：K=13, Q=12, ..., A=1
        if card.value != (13 - i) as u8 {
            return false;
        }
    }

    true
}

/// 查找可用的提示（合法移动）
pub fn find_hint(columns: &[Vec<Card>]) -> Option<(usize, usize, usize)> {
    // 从每一列寻找可移动的序列
    for (from_col, column) in columns.iter().enumerate() {
        // 从最底部开始尝试最长的序列
        for start_idx in 0..column.len() {
            let sequence = &column[start_idx..];

            if !can_drag_sequence(sequence) {
                continue;
            }

            // 寻找可以放置的目标列
            for (to_col, target) in columns.iter().enumerate() {
                if from_col == to_col {
                    continue;
                }

                if can_place_on(sequence, target) {
                    // 优先返回有意义的移动（避免无意义的移动）
                    // 例如：移动到空列时，确保不是唯一的牌
                    if target.is_empty() && sequence.len() == column.len() {
                        continue; // 跳过将整列移到空列
                    }
                    return Some((from_col, start_idx, to_col));
                }
            }
        }
    }

    None
}
```

#### state.rs

```rust
use super::card::Card;
use super::deck::{deal_initial_cards, generate_deck};
use super::history::History;
use super::rules::{can_drag_sequence, can_place_on, find_hint, is_complete_sequence};
use serde::{Deserialize, Serialize};

/// 游戏状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// 10 列牌堆
    pub columns: Vec<Vec<Card>>,
    /// 发牌堆（剩余 50 张）
    pub stock: Vec<Card>,
    /// 完成区（已收集的完整序列）
    pub completed: Vec<Vec<Card>>,
    /// 当前分数
    pub score: i32,
    /// 移动次数
    pub moves: u32,
    /// 游戏时长（秒）
    pub time_seconds: u32,
    /// 难度 (1-3)
    pub difficulty: u8,
    /// 游戏是否结束
    pub is_game_over: bool,
    /// 是否获胜
    pub is_win: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            columns: vec![Vec::new(); 10],
            stock: Vec::new(),
            completed: Vec::new(),
            score: 500,
            moves: 0,
            time_seconds: 0,
            difficulty: 1,
            is_game_over: false,
            is_win: false,
        }
    }
}

impl GameState {
    /// 创建新游戏
    pub fn new(difficulty: u8) -> Self {
        let mut deck = generate_deck(difficulty);
        let (columns, stock) = deal_initial_cards(&mut deck);

        Self {
            columns,
            stock,
            completed: Vec::new(),
            score: 500,
            moves: 0,
            time_seconds: 0,
            difficulty: difficulty.clamp(1, 3),
            is_game_over: false,
            is_win: false,
        }
    }

    /// 移动卡牌
    pub fn move_cards(
        &mut self,
        from_col: usize,
        start_idx: usize,
        to_col: usize,
    ) -> Result<(), String> {
        // 验证索引
        if from_col >= 10 || to_col >= 10 {
            return Err("Invalid column index".to_string());
        }

        let from_column = &self.columns[from_col];
        if start_idx >= from_column.len() {
            return Err("Invalid start index".to_string());
        }

        // 获取要移动的序列
        let moving_cards = &from_column[start_idx..];

        // 验证拖拽合法性
        if !can_drag_sequence(moving_cards) {
            return Err("Cannot drag this sequence".to_string());
        }

        // 验证目标合法性
        if !can_place_on(moving_cards, &self.columns[to_col]) {
            return Err("Invalid destination".to_string());
        }

        // 执行移动
        let moving = self.columns[from_col].split_off(start_idx);
        self.columns[to_col].extend(moving);

        // 翻开原列最后一张牌
        if let Some(card) = self.columns[from_col].last_mut() {
            card.is_face_up = true;
        }

        // 更新分数和步数
        self.moves += 1;
        self.score -= 1;

        // 检查是否完成序列
        self.check_complete_sequence(to_col);

        // 检查是否获胜
        if self.completed.len() == 8 {
            self.is_win = true;
            self.is_game_over = true;
        }

        Ok(())
    }

    /// 检查并移除完整的 K-A 序列
    fn check_complete_sequence(&mut self, col_idx: usize) {
        let column = &self.columns[col_idx];
        if column.len() < 13 {
            return;
        }

        // 检查最后 13 张是否形成完整序列
        let start_idx = column.len() - 13;
        let sequence = &column[start_idx..];

        if is_complete_sequence(sequence) {
            // 移除并添加到完成区
            let completed_cards = self.columns[col_idx].split_off(start_idx);
            self.completed.push(completed_cards);

            // 翻开新的最后一张牌
            if let Some(card) = self.columns[col_idx].last_mut() {
                card.is_face_up = true;
            }

            // 加分
            self.score += 100;
        }
    }

    /// 发牌
    pub fn deal(&mut self) -> Result<(), String> {
        // 检查是否有牌可发
        if self.stock.is_empty() {
            return Err("No cards left to deal".to_string());
        }

        // 检查所有列是否都有牌（发牌前提条件）
        for (i, col) in self.columns.iter().enumerate() {
            if col.is_empty() {
                return Err(format!("Column {} is empty, cannot deal", i + 1));
            }
        }

        // 向每列发一张牌
        for col in &mut self.columns {
            if let Some(mut card) = self.stock.pop() {
                card.is_face_up = true;
                col.push(card);
            }
        }

        // 检查所有列是否形成完整序列
        for i in 0..10 {
            self.check_complete_sequence(i);
        }

        Ok(())
    }

    /// 获取提示
    pub fn get_hint(&self) -> Option<(usize, usize, usize)> {
        find_hint(&self.columns)
    }

    /// 检查是否可以发牌
    pub fn can_deal(&self) -> bool {
        !self.stock.is_empty() && self.columns.iter().all(|col| !col.is_empty())
    }

    /// 获取剩余发牌次数
    pub fn remaining_deals(&self) -> u8 {
        (self.stock.len() / 10) as u8
    }
}
```

#### history.rs

```rust
use super::state::GameState;

/// 历史记录管理（用于撤销/重做）
#[derive(Debug, Clone)]
pub struct History {
    /// 状态栈
    states: Vec<GameState>,
    /// 当前状态索引
    current_index: usize,
    /// 最大历史记录数
    max_size: usize,
}

impl History {
    pub fn new(initial_state: GameState) -> Self {
        Self {
            states: vec![initial_state],
            current_index: 0,
            max_size: 100,
        }
    }

    /// 保存新状态（会清除重做栈）
    pub fn push(&mut self, state: GameState) {
        // 移除当前位置之后的所有状态
        self.states.truncate(self.current_index + 1);

        // 添加新状态
        self.states.push(state);

        // 如果超过最大大小，移除最早的状态
        if self.states.len() > self.max_size {
            self.states.remove(0);
        } else {
            self.current_index += 1;
        }
    }

    /// 撤销
    pub fn undo(&mut self) -> Option<&GameState> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(&self.states[self.current_index])
        } else {
            None
        }
    }

    /// 重做
    pub fn redo(&mut self) -> Option<&GameState> {
        if self.current_index < self.states.len() - 1 {
            self.current_index += 1;
            Some(&self.states[self.current_index])
        } else {
            None
        }
    }

    /// 获取当前状态
    pub fn current(&self) -> &GameState {
        &self.states[self.current_index]
    }

    /// 是否可以撤销
    pub fn can_undo(&self) -> bool {
        self.current_index > 0
    }

    /// 是否可以重做
    pub fn can_redo(&self) -> bool {
        self.current_index < self.states.len() - 1
    }
}
```

### 1.3 Tauri 命令

#### commands/game.rs

```rust
use crate::game::{GameState, History};
use std::sync::Mutex;
use tauri::State;

/// 应用状态
pub struct AppState {
    pub game: Mutex<GameState>,
    pub history: Mutex<Option<History>>,
}

#[tauri::command]
pub fn new_game(state: State<AppState>, difficulty: u8) -> GameState {
    let game = GameState::new(difficulty);
    let history = History::new(game.clone());

    *state.game.lock().unwrap() = game.clone();
    *state.history.lock().unwrap() = Some(history);

    game
}

#[tauri::command]
pub fn get_state(state: State<AppState>) -> GameState {
    state.game.lock().unwrap().clone()
}

#[tauri::command]
pub fn move_cards(
    state: State<AppState>,
    from_col: usize,
    start_idx: usize,
    to_col: usize,
) -> Result<GameState, String> {
    let mut game = state.game.lock().unwrap();

    // 保存到历史记录
    if let Some(history) = state.history.lock().unwrap().as_mut() {
        history.push(game.clone());
    }

    game.move_cards(from_col, start_idx, to_col)?;
    Ok(game.clone())
}

#[tauri::command]
pub fn deal_cards(state: State<AppState>) -> Result<GameState, String> {
    let mut game = state.game.lock().unwrap();

    // 保存到历史记录
    if let Some(history) = state.history.lock().unwrap().as_mut() {
        history.push(game.clone());
    }

    game.deal()?;
    Ok(game.clone())
}

#[tauri::command]
pub fn undo(state: State<AppState>) -> Result<GameState, String> {
    let mut history_guard = state.history.lock().unwrap();
    let history = history_guard
        .as_mut()
        .ok_or("No game history available")?;

    history
        .undo()
        .cloned()
        .ok_or("Cannot undo: already at oldest state".to_string())
        .map(|game_state| {
            *state.game.lock().unwrap() = game_state.clone();
            game_state
        })
}

#[tauri::command]
pub fn redo(state: State<AppState>) -> Result<GameState, String> {
    let mut history_guard = state.history.lock().unwrap();
    let history = history_guard
        .as_mut()
        .ok_or("No game history available")?;

    history
        .redo()
        .cloned()
        .ok_or("Cannot redo: already at newest state".to_string())
        .map(|game_state| {
            *state.game.lock().unwrap() = game_state.clone();
            game_state
        })
}

#[tauri::command]
pub fn get_hint(state: State<AppState>) -> Option<(usize, usize, usize)> {
    state.game.lock().unwrap().get_hint()
}

#[tauri::command]
pub fn can_undo(state: State<AppState>) -> bool {
    state
        .history
        .lock()
        .unwrap()
        .as_ref()
        .map(|h| h.can_undo())
        .unwrap_or(false)
}

#[tauri::command]
pub fn can_redo(state: State<AppState>) -> bool {
    state
        .history
        .lock()
        .unwrap()
        .as_ref()
        .map(|h| h.can_redo())
        .unwrap_or(false)
}
```

---

## 2. Svelte 前端设计

### 2.1 目录结构

```
src/
├── main.ts                    # 应用入口
├── App.svelte                 # 根组件
├── lib/
│   ├── stores/
│   │   ├── game.ts            # 游戏状态 Store
│   │   ├── settings.ts        # 设置 Store
│   │   └── statistics.ts      # 统计 Store
│   ├── components/
│   │   ├── Card.svelte        # 单张卡牌
│   │   ├── Column.svelte      # 一列牌堆
│   │   ├── StockPile.svelte   # 发牌堆
│   │   ├── Foundation.svelte  # 完成区
│   │   ├── Toolbar.svelte     # 顶部工具栏
│   │   ├── GameBoard.svelte   # 游戏主区域
│   │   ├── DifficultyModal.svelte  # 难度选择弹窗
│   │   ├── WinModal.svelte    # 胜利弹窗
│   │   └── StatsModal.svelte  # 统计弹窗
│   ├── utils/
│   │   ├── drag.ts            # 拖拽工具函数
│   │   ├── animations.ts      # 动画工具函数
│   │   └── sounds.ts          # 音效工具
│   └── types/
│       └── game.ts            # TypeScript 类型定义
└── styles/
    └── globals.css            # 全局样式
```

### 2.2 类型定义

#### lib/types/game.ts

```typescript
export type Suit = 'spade' | 'heart' | 'diamond' | 'club';

export interface Card {
  id: number;
  suit: Suit;
  value: number; // 1-13
  is_face_up: boolean;
}

export interface GameState {
  columns: Card[][];
  stock: Card[];
  completed: Card[][];
  score: number;
  moves: number;
  time_seconds: number;
  difficulty: 1 | 2 | 3;
  is_game_over: boolean;
  is_win: boolean;
}

export interface Hint {
  from_col: number;
  start_idx: number;
  to_col: number;
}

export interface Statistics {
  games_played: number;
  games_won: number;
  best_score: number;
  best_time: number;
  best_moves: number;
  current_streak: number;
  best_streak: number;
}

export interface Settings {
  difficulty: 1 | 2 | 3;
  sound_enabled: boolean;
  animations_enabled: boolean;
  card_back_style: 'classic' | 'modern' | 'minimal';
  table_color: string;
}
```

### 2.3 状态管理

#### lib/stores/game.ts

```typescript
import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { GameState, Hint } from '../types/game';

function createGameStore() {
  const { subscribe, set, update } = writable<GameState | null>(null);
  const selected = writable<{ colIndex: number; startIndex: number } | null>(null);
  const hintHighlight = writable<Hint | null>(null);
  const isAnimating = writable(false);

  return {
    subscribe,
    selected,
    hintHighlight,
    isAnimating,

    async newGame(difficulty: 1 | 2 | 3) {
      const state = await invoke<GameState>('new_game', { difficulty });
      set(state);
      selected.set(null);
      hintHighlight.set(null);
    },

    async moveCards(fromCol: number, startIndex: number, toCol: number) {
      try {
        isAnimating.set(true);
        const state = await invoke<GameState>('move_cards', {
          fromCol,
          startIdx: startIndex,
          toCol,
        });
        set(state);
        selected.set(null);
        return true;
      } catch (e) {
        console.error('Move failed:', e);
        return false;
      } finally {
        isAnimating.set(false);
      }
    },

    async dealCards() {
      try {
        isAnimating.set(true);
        const state = await invoke<GameState>('deal_cards');
        set(state);
      } catch (e) {
        console.error('Deal failed:', e);
      } finally {
        isAnimating.set(false);
      }
    },

    async undo() {
      try {
        const state = await invoke<GameState>('undo');
        set(state);
      } catch (e) {
        console.error('Undo failed:', e);
      }
    },

    async redo() {
      try {
        const state = await invoke<GameState>('redo');
        set(state);
      } catch (e) {
        console.error('Redo failed:', e);
      }
    },

    async getHint() {
      try {
        const hint = await invoke<Hint | null>('get_hint');
        hintHighlight.set(hint);
        // 3 秒后清除高亮
        setTimeout(() => hintHighlight.set(null), 3000);
        return hint;
      } catch (e) {
        console.error('Hint failed:', e);
        return null;
      }
    },

    select(colIndex: number, startIndex: number) {
      selected.set({ colIndex, startIndex });
    },

    clearSelection() {
      selected.set(null);
    },
  };
}

export const game = createGameStore();

// 派生状态
export const canDeal = derived(game, ($game) => {
  if (!$game) return false;
  return $game.stock.length > 0 && $game.columns.every((col) => col.length > 0);
});

export const remainingDeals = derived(game, ($game) => {
  if (!$game) return 0;
  return Math.floor($game.stock.length / 10);
});
```

### 2.4 核心组件

#### Card.svelte

```svelte
<script lang="ts">
  import type { Card as CardType } from '../types/game';

  export let card: CardType;
  export let index: number = 0;
  export let isSelected: boolean = false;
  export let isHinted: boolean = false;
  export let offsetY: number = 25;

  $: suitSymbol = {
    spade: '♠',
    heart: '♥',
    diamond: '♦',
    club: '♣',
  }[card.suit];

  $: displayValue = (() => {
    switch (card.value) {
      case 1: return 'A';
      case 11: return 'J';
      case 12: return 'Q';
      case 13: return 'K';
      default: return card.value.toString();
    }
  })();

  $: isRed = card.suit === 'heart' || card.suit === 'diamond';
</script>

<div
  class="card"
  class:face-up={card.is_face_up}
  class:selected={isSelected}
  class:hinted={isHinted}
  class:red={isRed}
  style="top: {index * offsetY}px; z-index: {index};"
>
  {#if card.is_face_up}
    <div class="card-inner">
      <div class="card-corner top-left">
        <span class="value">{displayValue}</span>
        <span class="suit">{suitSymbol}</span>
      </div>
      <div class="card-center">
        <span class="suit large">{suitSymbol}</span>
      </div>
      <div class="card-corner bottom-right">
        <span class="value">{displayValue}</span>
        <span class="suit">{suitSymbol}</span>
      </div>
    </div>
  {:else}
    <div class="card-back"></div>
  {/if}
</div>

<style>
  .card {
    position: absolute;
    width: 100px;
    height: 140px;
    border-radius: 8px;
    background: #1565c0;
    box-shadow: 2px 2px 5px rgba(0, 0, 0, 0.3);
    cursor: pointer;
    transition: transform 0.15s ease-out, box-shadow 0.15s ease-out;
    user-select: none;
  }

  .card:hover {
    transform: translateY(-3px);
    box-shadow: 3px 5px 10px rgba(0, 0, 0, 0.4);
  }

  .card.face-up {
    background: #fafafa;
  }

  .card.selected {
    box-shadow: 0 0 0 4px #ffd54f, 3px 5px 10px rgba(0, 0, 0, 0.4);
    transform: translateY(-5px);
  }

  .card.hinted {
    animation: hint-pulse 0.5s ease-in-out 3;
  }

  @keyframes hint-pulse {
    0%, 100% { box-shadow: 0 0 0 0 rgba(76, 175, 80, 0); }
    50% { box-shadow: 0 0 0 8px rgba(76, 175, 80, 0.5); }
  }

  .card-inner {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 6px;
  }

  .card-corner {
    display: flex;
    flex-direction: column;
    align-items: center;
    line-height: 1;
  }

  .card-corner.bottom-right {
    transform: rotate(180deg);
  }

  .value {
    font-size: 16px;
    font-weight: bold;
  }

  .suit {
    font-size: 14px;
  }

  .suit.large {
    font-size: 48px;
  }

  .card-center {
    display: flex;
    justify-content: center;
    align-items: center;
    flex: 1;
  }

  .red .value,
  .red .suit {
    color: #d32f2f;
  }

  .card-back {
    width: 100%;
    height: 100%;
    border-radius: 8px;
    background: linear-gradient(135deg, #1565c0 25%, #1976d2 25%, #1976d2 50%, #1565c0 50%, #1565c0 75%, #1976d2 75%);
    background-size: 20px 20px;
    border: 3px solid #0d47a1;
  }
</style>
```

#### Column.svelte

```svelte
<script lang="ts">
  import { getContext } from 'svelte';
  import Card from './Card.svelte';
  import type { Card as CardType } from '../types/game';

  export let columnIndex: number;
  export let cards: CardType[] = [];

  const { onSelect, onDrop, selected, hint } = getContext('game');

  $: selectedInfo = $selected;
  $: currentHint = $hint;

  function handleClick(startIndex: number) {
    if (selectedInfo && selectedInfo.colIndex === columnIndex) {
      // 已选中当前列，取消选中
      onSelect(null);
    } else if (selectedInfo) {
      // 已选中其他牌，尝试移动到当前列
      onDrop(columnIndex);
    } else {
      // 选中当前牌
      onSelect(columnIndex, startIndex);
    }
  }

  $: isTargetValid = selectedInfo && selectedInfo.colIndex !== columnIndex;
</script>

<div
  class="column"
  class:valid-target={isTargetValid}
  class:empty={cards.length === 0}
  role="button"
  tabindex="0"
  on:click={() => cards.length === 0 && isTargetValid && onDrop(columnIndex)}
  on:keydown={(e) => e.key === 'Enter' && cards.length === 0 && isTargetValid && onDrop(columnIndex)}
>
  {#each cards as card, idx (card.id)}
    <Card
      card={card}
      index={idx}
      isSelected={selectedInfo?.colIndex === columnIndex && selectedInfo?.startIndex === idx}
      isHinted={currentHint?.from_col === columnIndex && currentHint?.start_idx === idx}
      on:click={() => handleClick(idx)}
    />
  {/each}
</div>

<style>
  .column {
    position: relative;
    width: 100px;
    min-height: 140px;
    border-radius: 8px;
    transition: background-color 0.2s;
  }

  .column.empty {
    border: 2px dashed rgba(255, 255, 255, 0.3);
    background: rgba(255, 255, 255, 0.05);
  }

  .column.valid-target {
    background: rgba(76, 175, 80, 0.2);
    border-radius: 8px;
  }
</style>
```

---

## 3. 通信协议

### 3.1 Tauri Commands 汇总

| 命令 | 参数 | 返回 | 描述 |
|------|------|------|------|
| `new_game` | `difficulty: u8` | `GameState` | 开始新游戏 |
| `get_state` | - | `GameState` | 获取当前状态 |
| `move_cards` | `from_col, start_idx, to_col` | `Result<GameState, String>` | 移动卡牌 |
| `deal_cards` | - | `Result<GameState, String>` | 发牌 |
| `undo` | - | `Result<GameState, String>` | 撤销 |
| `redo` | - | `Result<GameState, String>` | 重做 |
| `can_undo` | - | `bool` | 是否可撤销 |
| `can_redo` | - | `bool` | 是否可重做 |
| `get_hint` | - | `Option<(usize, usize, usize)>` | 获取提示 |
| `save_game` | - | `Result<(), String>` | 保存游戏 |
| `load_game` | - | `Result<GameState, String>` | 加载游戏 |
| `get_statistics` | - | `Statistics` | 获取统计 |
| `reset_statistics` | - | `()` | 重置统计 |

---

## 4. 数据持久化

### 4.1 使用 Tauri Plugin Store

```rust
use tauri_plugin_store::StoreBuilder;

// 保存游戏
#[tauri::command]
pub async fn save_game(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let game = state.game.lock().unwrap().clone();

    let store = StoreBuilder::new("game.json")
        .build(app)
        .map_err(|e| e.to_string())?;

    store.set("saved_game", serde_json::to_string(&game).unwrap());
    store.save().map_err(|e| e.to_string())?;

    Ok(())
}

// 加载游戏
#[tauri::command]
pub async fn load_game(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<GameState, String> {
    let store = StoreBuilder::new("game.json")
        .build(app)
        .map_err(|e| e.to_string())?;

    let saved = store
        .get("saved_game")
        .and_then(|v| v.as_str())
        .ok_or("No saved game found")?;

    let game: GameState = serde_json::from_str(saved).map_err(|e| e.to_string())?;

    *state.game.lock().unwrap() = game.clone();

    Ok(game)
}
```

---

*文档维护：开发团队*
*最后更新：2026-03-13*
