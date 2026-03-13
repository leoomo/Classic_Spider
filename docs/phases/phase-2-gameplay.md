# Phase 2: 核心玩法 (Week 3-4)

## 概述

**目标**: 实现完整的游戏主循环，游戏可玩

**预计工期**: 2 周

**前置依赖**: Phase 1 完成

---

## 任务清单

### Task 2.1: 规则验证 - 基础规则

**优先级**: P0

**文件路径**:
- `src-tauri/src/game/rules.rs`

**描述**: 实现卡牌移动规则验证

**实现要点**:

```rust
// src-tauri/src/game/rules.rs
use super::card::{Card, Rank, Suit};
use super::state::Column;

/// 检查是否可以拖拽一个序列
/// 蜘蛛纸牌规则：只能拖拽同花色、连续递减的序列
pub fn can_drag_sequence(cards: &[Card]) -> bool {
    if cards.is_empty() {
        return false;
    }

    // 第一张牌必须是正面朝上
    if !cards[0].face_up {
        return false;
    }

    // 检查所有牌是否同花色且连续递减
    for i in 0..cards.len() - 1 {
        let current = &cards[i];
        let next = &cards[i + 1];

        // 必须同花色
        if current.suit != next.suit {
            return false;
        }

        // 必须连续递减 (K->Q->J->10...->A)
        if let Some(expected_next) = current.rank.prev() {
            if next.rank != expected_next {
                return false;
            }
        } else {
            return false;
        }

        // 后续牌必须正面朝上
        if !next.face_up {
            return false;
        }
    }

    true
}

/// 检查是否可以将牌放到目标列
/// 规则：只能放在比最上面牌小 1 的牌上，或者放在空列上
pub fn can_place_on(cards: &[Card], target_column: &Column) -> bool {
    // 空列可以放任何牌（但通常只能放 K）
    if target_column.is_empty() {
        return true;
    }

    let top_card = match target_column.top_card() {
        Some(card) => card,
        None => return false,
    };

    // 目标牌必须正面朝上
    if !top_card.face_up {
        return false;
    }

    // 被放置的第一张牌必须比目标牌小 1
    let first_card = &cards[0];
    if let Some(expected) = top_card.rank.prev() {
        first_card.rank == expected
    } else {
        false
    }
}

/// 检查是否是完整的 K-A 序列
pub fn is_complete_sequence(cards: &[Card]) -> bool {
    if cards.len() != 13 {
        return false;
    }

    // 必须从 K 开始
    if cards[0].rank != Rank::KING {
        return false;
    }

    // 必须全部同花色且连续递减到 A
    let suit = cards[0].suit;
    for (i, card) in cards.iter().enumerate() {
        if card.suit != suit {
            return false;
        }
        if card.rank.0 != (13 - i) as u8 {
            return false;
        }
    }

    true
}

/// 从一列中提取可拖拽的序列（从指定索引到末尾）
pub fn extract_draggable(column: &Column, from_index: usize) -> Option<Vec<Card>> {
    if from_index >= column.len() {
        return None;
    }

    let cards: Vec<Card> = column.cards[from_index..].to_vec();

    // 验证是否是有效序列
    if can_drag_sequence(&cards) {
        Some(cards)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_card(suit: Suit, rank: u8, face_up: bool) -> Card {
        Card {
            suit,
            rank: Rank(rank),
            face_up,
        }
    }

    #[test]
    fn test_can_drag_sequence_valid() {
        let cards = vec![
            create_card(Suit::Spades, 13, true), // K
            create_card(Suit::Spades, 12, true), // Q
            create_card(Suit::Spades, 11, true), // J
        ];
        assert!(can_drag_sequence(&cards));
    }

    #[test]
    fn test_can_drag_sequence_different_suit() {
        let cards = vec![
            create_card(Suit::Spades, 13, true),
            create_card(Suit::Hearts, 12, true), // 不同花色
        ];
        assert!(!can_drag_sequence(&cards));
    }

    #[test]
    fn test_can_drag_sequence_not_sequential() {
        let cards = vec![
            create_card(Suit::Spades, 13, true),
            create_card(Suit::Spades, 10, true), // 不连续
        ];
        assert!(!can_drag_sequence(&cards));
    }

    #[test]
    fn test_can_place_on_empty_column() {
        let cards = vec![create_card(Suit::Spades, 5, true)];
        let empty_column = Column::new();
        assert!(can_place_on(&cards, &empty_column));
    }

    #[test]
    fn test_can_place_on_valid() {
        let cards = vec![create_card(Suit::Spades, 11, true)]; // J
        let mut column = Column::new();
        column.push(create_card(Suit::Hearts, 12, true)); // Q (花色不重要)
        assert!(can_place_on(&cards, &column));
    }

    #[test]
    fn test_is_complete_sequence_valid() {
        let mut cards = Vec::new();
        for rank in (1..=13).rev() {
            cards.push(create_card(Suit::Spades, rank, true));
        }
        assert!(is_complete_sequence(&cards));
    }
}
```

**验收标准**:
- [ ] 同花色连续序列可拖拽
- [ ] 不同花色序列不可拖拽
- [ ] 空列可放任何牌
- [ ] K-A 完整序列检测正确

---

### Task 2.2: 规则验证 - 移动卡牌

**优先级**: P0

**文件路径**:
- `src-tauri/src/game/state.rs` (扩展)

**描述**: 在 GameState 中实现移动卡牌逻辑

**实现要点**:

```rust
// 在 state.rs 中添加

/// 移动结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveResult {
    pub success: bool,
    pub message: String,
    pub completed_sequence: bool, // 是否完成了 K-A 序列
    pub score_change: i32,
}

impl GameState {
    /// 移动卡牌序列
    pub fn move_cards(
        &mut self,
        from_column: usize,
        from_index: usize,
        to_column: usize,
    ) -> MoveResult {
        use super::rules::{can_place_on, extract_draggable, is_complete_sequence};

        // 验证列索引
        if from_column >= COLUMN_COUNT || to_column >= COLUMN_COUNT {
            return MoveResult {
                success: false,
                message: "Invalid column index".to_string(),
                completed_sequence: false,
                score_change: 0,
            };
        }

        // 不能移动到同一列
        if from_column == to_column {
            return MoveResult {
                success: false,
                message: "Cannot move to the same column".to_string(),
                completed_sequence: false,
                score_change: 0,
            };
        }

        // 提取可拖拽序列
        let cards = match extract_draggable(&self.columns[from_column], from_index) {
            Some(cards) => cards,
            None => return MoveResult {
                success: false,
                message: "Cannot drag this sequence".to_string(),
                completed_sequence: false,
                score_change: 0,
            },
        };

        // 验证目标位置
        if !can_place_on(&cards, &self.columns[to_column]) {
            return MoveResult {
                success: false,
                message: "Cannot place cards here".to_string(),
                completed_sequence: false,
                score_change: 0,
            };
        }

        // 执行移动
        self.columns[from_column].pop_n(cards.len());
        self.columns[to_column].extend(cards.clone());

        // 翻开源列顶部的牌
        if let Some(top_card) = self.columns[from_column].cards.last_mut() {
            if !top_card.face_up {
                top_card.flip_up();
            }
        }

        // 检查是否形成完整序列
        let mut completed = false;
        let target_len = self.columns[to_column].len();
        if target_len >= 13 {
            let potential_sequence = &self.columns[to_column].cards[target_len - 13..];
            if is_complete_sequence(potential_sequence) {
                // 移除完成的序列
                self.columns[to_column].pop_n(13);
                // 添加到完成区
                for foundation in &mut self.foundations {
                    if foundation.count < 13 {
                        foundation.add_completed();
                        completed = true;
                        break;
                    }
                }
            }
        }

        // 翻开目标列顶部（如果移除后）
        if completed {
            if let Some(top_card) = self.columns[to_column].cards.last_mut() {
                if !top_card.face_up {
                    top_card.flip_up();
                }
            }
        }

        // 更新分数和移动次数
        let score_change = if completed { 100 } else { -1 };
        self.score += score_change;
        self.moves += 1;

        // 检查胜利
        self.check_win();

        MoveResult {
            success: true,
            message: if completed { "Sequence completed!".to_string() } else { "Move successful".to_string() },
            completed_sequence: completed,
            score_change,
        }
    }

    /// 检查是否胜利
    fn check_win(&mut self) {
        let completed_count: usize = self.foundations.iter().map(|f| f.count).sum();
        if completed_count >= 8 {
            self.won = true;
            self.game_over = true;
        }
    }
}
```

**验收标准**:
- [ ] 有效移动成功执行
- [ ] 无效移动返回错误信息
- [ ] 移动后自动翻牌
- [ ] K-A 序列自动收牌

---

### Task 2.3: Tauri Commands - 移动命令

**优先级**: P0

**文件路径**:
- `src-tauri/src/commands/game.rs` (扩展)

**描述**: 添加移动卡牌命令

**实现要点**:

```rust
// 在 commands/game.rs 中添加

use crate::game::state::MoveResult;

/// 移动卡牌
#[tauri::command]
pub fn move_cards(
    from_column: usize,
    from_index: usize,
    to_column: usize,
    manager: State<GameManager>,
) -> Result<MoveResult, String> {
    let mut state = manager.state.lock().unwrap();

    match state.as_mut() {
        Some(game_state) => {
            let result = game_state.move_cards(from_column, from_index, to_column);
            Ok(result)
        }
        None => Err("No game in progress".to_string()),
    }
}

/// 获取移动后的完整状态
#[tauri::command]
pub fn move_cards_and_get_state(
    from_column: usize,
    from_index: usize,
    to_column: usize,
    manager: State<GameManager>,
) -> Result<(MoveResult, GameState), String> {
    let mut state = manager.state.lock().unwrap();

    match state.as_mut() {
        Some(game_state) => {
            let result = game_state.move_cards(from_column, from_index, to_column);
            Ok((result, game_state.clone()))
        }
        None => Err("No game in progress".to_string()),
    }
}

// 更新 lib.rs 中的 invoke_handler
.invoke_handler(tauri::generate_handler![
    commands::game::new_game,
    commands::game::get_state,
    commands::game::move_cards,
    commands::game::move_cards_and_get_state,
])
```

**验收标准**:
- [ ] `move_cards` 命令正常工作
- [ ] 返回移动结果和更新后的状态

---

### Task 2.4: 发牌功能

**优先级**: P0

**文件路径**:
- `src-tauri/src/game/state.rs` (扩展)
- `src-tauri/src/commands/game.rs` (扩展)

**描述**: 实现发牌功能

**实现要点**:

```rust
// 在 state.rs 中添加

/// 发牌结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealResult {
    pub success: bool,
    pub message: String,
    pub cards_dealt: usize,
}

impl GameState {
    /// 发牌（给每列发一张）
    pub fn deal(&mut self) -> DealResult {
        // 检查是否可以发牌
        if !self.can_deal() {
            return DealResult {
                success: false,
                message: if self.stock.is_empty() {
                    "No cards left to deal".to_string()
                } else {
                    "All columns must have at least one card before dealing".to_string()
                },
                cards_dealt: 0,
            };
        }

        // 给每列发一张牌
        for column in &mut self.columns {
            if let Some(mut card) = self.stock.pop() {
                card.flip_up();
                column.push(card);
            }
        }

        DealResult {
            success: true,
            message: "Dealt 10 cards".to_string(),
            cards_dealt: 10,
        }
    }
}

// 在 commands/game.rs 中添加

/// 发牌
#[tauri::command]
pub fn deal_cards(manager: State<GameManager>) -> Result<(DealResult, GameState), String> {
    let mut state = manager.state.lock().unwrap();

    match state.as_mut() {
        Some(game_state) => {
            let result = game_state.deal();
            Ok((result, game_state.clone()))
        }
        None => Err("No game in progress".to_string()),
    }
}
```

**验收标准**:
- [ ] 发牌成功给每列发一张
- [ ] 有空列时不能发牌
- [ ] 牌堆用完不能发牌

---

### Task 2.5: 前端拖拽系统 - 基础交互

**优先级**: P0

**文件路径**:
- `src/lib/stores/gameStore.svelte.ts`
- `src/lib/components/GameBoard.svelte` (修改)

**描述**: 实现拖拽交互状态管理

**实现要点**:

```typescript
// src/lib/stores/gameStore.svelte.ts
import { invoke } from '@tauri-apps/api/core';

export interface Card {
  suit: string;
  rank: { display: string; 0: number };
  face_up: boolean;
}

export interface Column {
  cards: Card[];
}

export interface GameState {
  columns: Column[];
  stock: Card[];
  foundations: { count: number }[];
  score: number;
  moves: number;
  difficulty: string;
  game_over: boolean;
  won: boolean;
}

export interface MoveResult {
  success: boolean;
  message: string;
  completed_sequence: boolean;
  score_change: number;
}

class GameStore {
  state = $state<GameState | null>(null);
  selectedCards = $state<{ column: number; startIndex: number } | null>(null);
  isDragging = $state(false);
  dragCards = $state<Card[]>([]);
  dragSource = $state<{ column: number; startIndex: number } | null>(null);

  async newGame(difficulty: string = 'Easy') {
    this.state = await invoke('new_game', { difficulty });
    this.clearSelection();
  }

  async moveCards(fromColumn: number, fromIndex: number, toColumn: number) {
    if (!this.state) return;

    const [result, newState] = await invoke<[MoveResult, GameState]>('move_cards_and_get_state', {
      fromColumn,
      fromIndex,
      toColumn,
    });

    if (result.success) {
      this.state = newState;
      this.clearSelection();

      if (result.completed_sequence) {
        // 可以在这里触发庆祝动画
        console.log('Sequence completed!');
      }
    }

    return result;
  }

  async dealCards() {
    if (!this.state) return;

    try {
      const [result, newState] = await invoke<[DealResult, GameState]>('deal_cards');
      if (result.success) {
        this.state = newState;
      }
      return result;
    } catch (error) {
      console.error('Deal failed:', error);
    }
  }

  selectCards(column: number, startIndex: number) {
    if (!this.state) return;

    const column = this.state.columns[column];
    const cards = column.cards.slice(startIndex);

    // 验证是否是可拖拽序列
    if (this.canDragSequence(cards)) {
      this.selectedCards = { column, startIndex };
      this.dragCards = cards;
    }
  }

  clearSelection() {
    this.selectedCards = null;
    this.dragCards = [];
  }

  startDrag(column: number, startIndex: number) {
    if (!this.state) return;

    const columnCards = this.state.columns[column].cards;
    const cards = columnCards.slice(startIndex);

    if (this.canDragSequence(cards)) {
      this.isDragging = true;
      this.dragCards = cards;
      this.dragSource = { column, startIndex };
    }
  }

  endDrag() {
    this.isDragging = false;
    this.dragCards = [];
    this.dragSource = null;
  }

  private canDragSequence(cards: Card[]): boolean {
    if (cards.length === 0) return false;
    if (!cards[0].face_up) return false;

    for (let i = 0; i < cards.length - 1; i++) {
      const current = cards[i];
      const next = cards[i + 1];

      if (current.suit !== next.suit) return false;
      if (next.rank[0] !== current.rank[0] - 1) return false;
      if (!next.face_up) return false;
    }

    return true;
  }
}

export const gameStore = new GameStore();
```

**验收标准**:
- [ ] 状态管理正常工作
- [ ] 卡牌选择逻辑正确
- [ ] 可拖拽序列验证正确

---

### Task 2.6: 前端拖拽系统 - 拖拽组件

**优先级**: P0

**文件路径**:
- `src/lib/components/Column.svelte` (修改)
- `src/lib/components/DragPreview.svelte`

**描述**: 实现拖拽视觉反馈

**实现要点**:

```svelte
<!-- src/lib/components/DragPreview.svelte -->
<script lang="ts">
  import Card from './Card.svelte';
  import type { Card as CardType } from '../stores/gameStore.svelte';

  interface Props {
    cards: CardType[];
    x: number;
    y: number;
  }

  let { cards, x, y }: Props = $props();
</script>

{#if cards.length > 0}
  <div class="drag-preview" style="left: {x}px; top: {y}px">
    {#each cards as card, i}
      <div class="drag-card" style="top: {i * 20}px">
        <Card {card} />
      </div>
    {/each}
  </div>
{/if}

<style>
  .drag-preview {
    position: fixed;
    pointer-events: none;
    z-index: 1000;
    opacity: 0.9;
    transform: translate(-35px, -50px);
  }

  .drag-card {
    position: absolute;
  }
</style>
```

```svelte
<!-- 修改 Column.svelte -->
<script lang="ts">
  import Card from './Card.svelte';
  import type { CardData } from './Card.svelte';

  interface Props {
    cards: CardData[];
    columnIndex: number;
    selectedIndex: number | null;
    onCardClick: (index: number) => void;
    onCardDragStart: (index: number, event: DragEvent) => void;
    onDrop: () => void;
  }

  let { cards, columnIndex, selectedIndex = null, onCardClick, onCardDragStart, onDrop }: Props = $props();

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    e.dataTransfer!.dropEffect = 'move';
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    onDrop();
  }
</script>

<div
  class="column"
  role="list"
  ondragover={handleDragOver}
  ondrop={handleDrop}
>
  {#each cards as card, index}
    <div
      class="card-wrapper"
      class:selected={selectedIndex === index}
      draggable={card.face_up}
      ondragstart={(e) => onCardDragStart(index, e)}
      onclick={() => onCardClick(index)}
    >
      <Card {card} selected={selectedIndex === index} />
    </div>
  {/each}

  {#if cards.length === 0}
    <div class="empty-slot">
      <span class="placeholder">K</span>
    </div>
  {/if}
</div>

<style>
  .card-wrapper {
    cursor: grab;
  }

  .card-wrapper:active {
    cursor: grabbing;
  }

  .card-wrapper.selected ::global(.card) {
    box-shadow: 0 0 0 3px #3b82f6;
    transform: translateY(-4px);
  }
</style>
```

**验收标准**:
- [ ] 拖拽预览正确显示
- [ ] 选中状态视觉反馈
- [ ] 拖放目标高亮

---

### Task 2.7: 前端拖拽系统 - 点击移动

**优先级**: P0

**文件路径**:
- `src/lib/components/GameBoard.svelte` (修改)

**描述**: 实现点击选中 + 点击目标的移动方式

**实现要点**:

```svelte
<!-- 在 GameBoard.svelte 中添加 -->
<script lang="ts">
  // ... 之前的 imports
  import { gameStore } from '../stores/gameStore.svelte';
  import DragPreview from './DragPreview.svelte';

  let mouseX = $state(0);
  let mouseY = $state(0);

  function handleCardClick(columnIndex: number, cardIndex: number) {
    const store = gameStore;

    // 如果已有选中的牌
    if (store.selectedCards) {
      // 如果点击的是同一列同一位置，取消选中
      if (store.selectedCards.column === columnIndex &&
          store.selectedCards.startIndex === cardIndex) {
        store.clearSelection();
        return;
      }

      // 尝试移动到目标列
      if (store.selectedCards.column !== columnIndex) {
        attemptMove(store.selectedCards.column, store.selectedCards.startIndex, columnIndex);
      } else {
        // 同一列，重新选择
        store.selectCards(columnIndex, cardIndex);
      }
    } else {
      // 选择新牌
      store.selectCards(columnIndex, cardIndex);
    }
  }

  async function attemptMove(fromColumn: number, fromIndex: number, toColumn: number) {
    const result = await gameStore.moveCards(fromColumn, fromIndex, toColumn);
    if (!result?.success) {
      // 可以显示错误提示
      console.log('Move failed:', result?.message);
    }
  }

  function handleEmptyColumnClick(columnIndex: number) {
    if (gameStore.selectedCards) {
      attemptMove(
        gameStore.selectedCards.column,
        gameStore.selectedCards.startIndex,
        columnIndex
      );
    }
  }

  function handleGlobalMouseMove(e: MouseEvent) {
    mouseX = e.clientX;
    mouseY = e.clientY;
  }

  function handleGlobalKeyUp(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      gameStore.clearSelection();
    }
  }
</script>

<svelte:window onmousemove={handleGlobalMouseMove} onkeyup={handleGlobalKeyUp} />

<div class="game-container">
  <!-- ... 工具栏 ... -->

  <main class="game-board">
    {#if gameStore.state}
      <div class="columns">
        {#each gameStore.state.columns as column, index}
          <Column
            cards={column.cards}
            columnIndex={index}
            selectedIndex={gameStore.selectedCards?.column === index
              ? gameStore.selectedCards.startIndex
              : null}
            onCardClick={(cardIndex) => handleCardClick(index, cardIndex)}
            onCardDragStart={(cardIndex, e) => {
              gameStore.startDrag(index, cardIndex);
              // 设置拖拽数据
              e.dataTransfer!.setData('text/plain', JSON.stringify({
                column: index,
                startIndex: cardIndex
              }));
            }}
            onDrop={() => {
              if (gameStore.dragSource) {
                attemptMove(
                  gameStore.dragSource.column,
                  gameStore.dragSource.startIndex,
                  index
                );
                gameStore.endDrag();
              }
            }}
          />
        {/each}
      </div>

      <!-- 拖拽预览 -->
      {#if gameStore.isDragging && gameStore.dragCards.length > 0}
        <DragPreview cards={gameStore.dragCards} x={mouseX} y={mouseY} />
      {/if}

      <!-- 发牌堆 -->
      <div class="stock-area">
        <button
          class="stock-button"
          onclick={() => gameStore.dealCards()}
          disabled={!gameStore.state || gameStore.state.stock.length === 0}
        >
          <div class="stock-piles">
            {#each Array(Math.ceil((gameStore.state?.stock.length ?? 0) / 10)) as _, i}
              <div class="stock-pile" style="left: {i * 5}px"></div>
            {/each}
          </div>
          <span class="deals-remaining">
            剩余: {Math.floor((gameStore.state?.stock.length ?? 0) / 10)} 次
          </span>
        </button>
      </div>
    {/if}
  </main>
</div>

<style>
  .stock-button {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 10px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }

  .stock-button:hover:not(:disabled) .stock-pile {
    transform: translateY(-2px);
  }

  .stock-button:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .stock-piles {
    position: relative;
    height: 100px;
    width: 80px;
  }

  .stock-pile {
    position: absolute;
    width: 70px;
    height: 100px;
    background: linear-gradient(135deg, #1e40af 0%, #3b82f6 100%);
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    transition: transform 0.1s;
  }

  .deals-remaining {
    color: white;
    font-size: 14px;
  }
</style>
```

**验收标准**:
- [ ] 点击选中牌
- [ ] 点击目标列移动
- [ ] ESC 取消选中
- [ ] 空列可接收牌

---

### Task 2.8: 双击快速移动

**优先级**: P1

**文件路径**:
- `src/lib/components/Column.svelte` (修改)

**描述**: 双击自动找到合适的目标列并移动

**实现要点**:

```typescript
// 在 gameStore.svelte.ts 中添加

findBestTarget(fromColumn: number, fromIndex: number): number | null {
  if (!this.state) return null;

  const sourceCards = this.state.columns[fromColumn].cards.slice(fromIndex);
  if (sourceCards.length === 0 || !this.canDragSequence(sourceCards)) {
    return null;
  }

  const topCard = sourceCards[0];

  // 优先找同花色的列
  for (let i = 0; i < this.state.columns.length; i++) {
    if (i === fromColumn) continue;

    const targetColumn = this.state.columns[i];
    if (targetColumn.cards.length === 0) continue;

    const targetTop = targetColumn.cards[targetColumn.cards.length - 1];
    if (targetTop.suit === topCard.suit && targetTop.rank[0] === topCard.rank[0] + 1) {
      return i;
    }
  }

  // 其次找不同花色但可以放置的列
  for (let i = 0; i < this.state.columns.length; i++) {
    if (i === fromColumn) continue;

    const targetColumn = this.state.columns[i];
    if (targetColumn.cards.length === 0) continue;

    const targetTop = targetColumn.cards[targetColumn.cards.length - 1];
    if (targetTop.rank[0] === topCard.rank[0] + 1) {
      return i;
    }
  }

  // 最后找空列
  for (let i = 0; i < this.state.columns.length; i++) {
    if (i === fromColumn) continue;
    if (this.state.columns[i].cards.length === 0) {
      return i;
    }
  }

  return null;
}

async quickMove(column: number, startIndex: number): Promise<boolean> {
  const target = this.findBestTarget(column, startIndex);
  if (target !== null) {
    const result = await this.moveCards(column, startIndex, target);
    return result?.success ?? false;
  }
  return false;
}
```

```svelte
<!-- 在 Column.svelte 中添加双击处理 -->
<div
  class="card-wrapper"
  ondblclick={() => onCardDoubleClick(index)}
>
  <!-- ... -->
</div>
```

**验收标准**:
- [ ] 双击自动移动到最佳位置
- [ ] 优先同花色目标
- [ ] 无目标时无操作

---

### Task 2.9: 胜利检测与弹窗

**优先级**: P0

**文件路径**:
- `src/lib/components/WinModal.svelte`
- `src/lib/components/GameBoard.svelte` (修改)

**描述**: 胜利时显示庆祝弹窗

**实现要点**:

```svelte
<!-- src/lib/components/WinModal.svelte -->
<script lang="ts">
  interface Props {
    score: number;
    moves: number;
    onNewGame: () => void;
    onClose: () => void;
  }

  let { score, moves, onNewGame, onClose }: Props = $props();
</script>

<div class="modal-overlay" onclick={onClose}>
  <div class="modal" onclick={(e) => e.stopPropagation()}>
    <div class="celebration">
      <span class="trophy">🏆</span>
    </div>
    <h1>恭喜胜利！</h1>
    <div class="stats">
      <div class="stat">
        <span class="label">最终分数</span>
        <span class="value">{score}</span>
      </div>
      <div class="stat">
        <span class="label">移动次数</span>
        <span class="value">{moves}</span>
      </div>
    </div>
    <div class="actions">
      <button class="primary" onclick={onNewGame}>再来一局</button>
      <button onclick={onClose}>关闭</button>
    </div>
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
    animation: fadeIn 0.3s ease;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .modal {
    background: white;
    border-radius: 16px;
    padding: 40px;
    text-align: center;
    max-width: 400px;
    animation: slideUp 0.3s ease;
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .celebration {
    font-size: 64px;
    margin-bottom: 20px;
    animation: bounce 0.5s ease infinite alternate;
  }

  @keyframes bounce {
    from { transform: translateY(0); }
    to { transform: translateY(-10px); }
  }

  h1 {
    color: #1a472a;
    margin-bottom: 30px;
    font-size: 28px;
  }

  .stats {
    display: flex;
    justify-content: center;
    gap: 40px;
    margin-bottom: 30px;
  }

  .stat {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .label {
    color: #666;
    font-size: 14px;
  }

  .value {
    font-size: 32px;
    font-weight: bold;
    color: #1a472a;
  }

  .actions {
    display: flex;
    gap: 15px;
    justify-content: center;
  }

  button {
    padding: 12px 24px;
    border-radius: 8px;
    border: none;
    font-size: 16px;
    cursor: pointer;
    transition: transform 0.1s, box-shadow 0.1s;
  }

  button:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  button.primary {
    background: #1a472a;
    color: white;
  }

  button:not(.primary) {
    background: #f3f4f6;
    color: #333;
  }
</style>
```

**验收标准**:
- [ ] 8 组完成时弹出胜利窗口
- [ ] 显示最终分数和移动次数
- [ ] 可以开始新游戏

---

## 阶段验收清单

- [ ] 能完成一局完整的初级游戏
- [ ] 拖拽交互流畅（点击选中 + 点击目标）
- [ ] 计分正确（移动 -1，完成序列 +100）
- [ ] 发牌功能正常（每次 10 张）
- [ ] 空列不能发牌限制正确
- [ ] 胜利检测正确

## 风险与依赖

### 风险
1. **拖拽性能**: 大量 DOM 操作可能导致卡顿
   - 缓解：使用 CSS transform，避免 reflow

2. **规则理解偏差**: 蜘蛛纸牌有多种变体
   - 缓解：明确使用经典规则，必要时参考 Windows 版本

### 依赖
- Phase 1 所有任务完成
- 前端拖拽 API 兼容性

## 下一阶段

完成本阶段后，进入 [Phase 3: 完善体验](./phase-3-experience.md)
