# Phase 3: 完善体验 (Week 5-6)

## 概述

**目标**: 提升用户体验，增加辅助功能

**预计工期**: 2 周

**前置依赖**: Phase 2 完成

---

## 任务清单

### Task 3.1: 撤销/重做系统 - 历史记录

**优先级**: P0

**文件路径**:
- `src-tauri/src/game/history.rs`

**描述**: 实现游戏状态历史记录管理

**实现要点**:

```rust
// src-tauri/src/game/history.rs
use super::state::GameState;
use serde::{Deserialize, Serialize};

/// 历史记录最大容量
const MAX_HISTORY: usize = 100;

/// 历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub state: GameState,
    pub description: String,
    pub timestamp: u64,
}

/// 历史记录管理器
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct History {
    /// 历史状态栈（撤销用）
    undo_stack: Vec<HistoryEntry>,
    /// 重做栈
    redo_stack: Vec<HistoryEntry>,
}

impl History {
    pub fn new() -> Self {
        History {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// 记录一个状态
    pub fn push(&mut self, state: GameState, description: &str) {
        // 清空重做栈（新操作使重做历史失效）
        self.redo_stack.clear();

        // 添加到撤销栈
        self.undo_stack.push(HistoryEntry {
            state,
            description: description.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        // 限制历史大小
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
    }

    /// 撤销：返回上一个状态
    pub fn undo(&mut self, current_state: GameState) -> Option<GameState> {
        if let Some(entry) = self.undo_stack.pop() {
            // 当前状态保存到重做栈
            self.redo_stack.push(HistoryEntry {
                state: current_state,
                description: "Current state".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
            Some(entry.state)
        } else {
            None
        }
    }

    /// 重做：返回下一个状态
    pub fn redo(&mut self, current_state: GameState) -> Option<GameState> {
        if let Some(entry) = self.redo_stack.pop() {
            // 当前状态保存到撤销栈
            self.undo_stack.push(HistoryEntry {
                state: current_state,
                description: "Current state".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
            Some(entry.state)
        } else {
            None
        }
    }

    /// 是否可以撤销
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// 是否可以重做
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// 清空历史
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// 获取撤销历史数量
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// 获取重做历史数量
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::deck::Difficulty;

    #[test]
    fn test_history_push_and_undo() {
        let mut history = History::new();
        let state1 = GameState::new(Difficulty::Easy);
        let state2 = state1.clone();

        history.push(state1, "Move card");
        assert!(history.can_undo());
        assert!(!history.can_redo());

        let undone = history.undo(state2);
        assert!(undone.is_some());
        assert!(history.can_redo());
    }
}
```

**验收标准**:
- [ ] 历史记录正确保存
- [ ] 撤销后可重做
- [ ] 新操作清空重做栈
- [ ] 历史大小限制正常

---

### Task 3.2: 撤销/重做系统 - 集成到游戏状态

**优先级**: P0

**文件路径**:
- `src-tauri/src/game/state.rs` (修改)
- `src-tauri/src/commands/game.rs` (修改)

**描述**: 将历史记录集成到游戏状态管理中

**实现要点**:

```rust
// 在 game/mod.rs 中添加
pub mod history;

// 修改 GameManager
use crate::game::history::History;
use crate::game::state::GameState;

pub struct GameManager {
    pub state: Mutex<Option<GameState>>,
    pub history: Mutex<History>,
}

// 在 commands/game.rs 中添加命令

/// 撤销
#[tauri::command]
pub fn undo(manager: State<GameManager>) -> Result<Option<GameState>, String> {
    let mut state_guard = manager.state.lock().unwrap();
    let mut history_guard = manager.history.lock().unwrap();

    match state_guard.as_ref() {
        Some(current) => {
            let current = current.clone();
            if let Some(previous) = history_guard.undo(current) {
                *state_guard = Some(previous.clone());
                Ok(Some(previous))
            } else {
                Ok(None)
            }
        }
        None => Err("No game in progress".to_string()),
    }
}

/// 重做
#[tauri::command]
pub fn redo(manager: State<GameManager>) -> Result<Option<GameState>, String> {
    let mut state_guard = manager.state.lock().unwrap();
    let mut history_guard = manager.history.lock().unwrap();

    match state_guard.as_ref() {
        Some(current) => {
            let current = current.clone();
            if let Some(next) = history_guard.redo(current) {
                *state_guard = Some(next.clone());
                Ok(Some(next))
            } else {
                Ok(None)
            }
        }
        None => Err("No game in progress".to_string()),
    }
}

/// 获取撤销/重做状态
#[tauri::command]
pub fn get_history_status(manager: State<GameManager>) -> (bool, bool) {
    let history_guard = manager.history.lock().unwrap();
    (history_guard.can_undo(), history_guard.can_redo())
}

// 修改 move_cards 和 deal_cards，在操作前保存历史
// 在 move_cards 方法开始处：
pub fn move_cards(&mut self, history: &mut History, ...) -> MoveResult {
    // 保存当前状态到历史
    history.push(self.clone(), "Move cards");
    // ... 原有逻辑
}
```

**验收标准**:
- [ ] 撤销命令正常工作
- [ ] 重做命令正常工作
- [ ] 状态正确同步

---

### Task 3.3: 撤销/重做系统 - 前端 UI

**优先级**: P0

**文件路径**:
- `src/lib/stores/gameStore.svelte.ts` (修改)
- `src/lib/components/Toolbar.svelte`

**描述**: 添加撤销/重做按钮和快捷键

**实现要点**:

```svelte
<!-- src/lib/components/Toolbar.svelte -->
<script lang="ts">
  import { gameStore } from '../stores/gameStore.svelte';

  interface Props {
    onNewGame: () => void;
  }

  let { onNewGame }: Props = $props();

  async function handleUndo() {
    await gameStore.undo();
  }

  async function handleRedo() {
    await gameStore.redo();
  }
</script>

<header class="toolbar">
  <div class="left">
    <button class="icon-btn" onclick={onNewGame} title="新游戏 (Ctrl+N)">
      <span class="icon">🎮</span>
    </button>
    <div class="separator"></div>
    <button
      class="icon-btn"
      onclick={handleUndo}
      disabled={!gameStore.canUndo}
      title="撤销 (Ctrl+Z)"
    >
      <span class="icon">↩️</span>
    </button>
    <button
      class="icon-btn"
      onclick={handleRedo}
      disabled={!gameStore.canRedo}
      title="重做 (Ctrl+Y)"
    >
      <span class="icon">↪️</span>
    </button>
  </div>

  <div class="center">
    <div class="stat">
      <span class="label">分数</span>
      <span class="value">{gameStore.state?.score ?? 0}</span>
    </div>
    <div class="stat">
      <span class="label">移动</span>
      <span class="value">{gameStore.state?.moves ?? 0}</span>
    </div>
    <div class="stat">
      <span class="label">剩余</span>
      <span class="value">{Math.floor((gameStore.state?.stock.length ?? 0) / 10)}</span>
    </div>
  </div>

  <div class="right">
    <!-- 保留给其他功能 -->
  </div>
</header>

<style>
  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    background: rgba(0, 0, 0, 0.3);
    color: white;
  }

  .left, .right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .center {
    display: flex;
    gap: 24px;
  }

  .icon-btn {
    width: 36px;
    height: 36px;
    border: none;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s;
  }

  .icon-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.2);
  }

  .icon-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .icon {
    font-size: 18px;
  }

  .separator {
    width: 1px;
    height: 24px;
    background: rgba(255, 255, 255, 0.2);
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  .label {
    font-size: 11px;
    opacity: 0.7;
  }

  .value {
    font-size: 16px;
    font-weight: bold;
  }
</style>
```

```typescript
// 在 gameStore.svelte.ts 中添加
canUndo = $state(false);
canRedo = $state(false);

async undo() {
  const result = await invoke<GameState | null>('undo');
  if (result) {
    this.state = result;
    await this.updateHistoryStatus();
  }
  return result;
}

async redo() {
  const result = await invoke<GameState | null>('redo');
  if (result) {
    this.state = result;
    await this.updateHistoryStatus();
  }
  return result;
}

async updateHistoryStatus() {
  const [canUndo, canRedo] = await invoke<[boolean, boolean]>('get_history_status');
  this.canUndo = canUndo;
  this.canRedo = canRedo;
}
```

```svelte
<!-- 在 App.svelte 或 GameBoard.svelte 中添加快捷键 -->
<script lang="ts">
  import { gameStore } from './lib/stores/gameStore.svelte';

  function handleKeyDown(e: KeyboardEvent) {
    if (e.ctrlKey || e.metaKey) {
      if (e.key === 'z') {
        e.preventDefault();
        if (e.shiftKey) {
          gameStore.redo(); // Ctrl+Shift+Z
        } else {
          gameStore.undo(); // Ctrl+Z
        }
      } else if (e.key === 'y') {
        e.preventDefault();
        gameStore.redo(); // Ctrl+Y
      } else if (e.key === 'n') {
        e.preventDefault();
        // 新游戏
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />
```

**验收标准**:
- [ ] 撤销/重做按钮正常工作
- [ ] Ctrl+Z 撤销
- [ ] Ctrl+Y/Ctrl+Shift+Z 重做
- [ ] 按钮禁用状态正确

---

### Task 3.4: 提示系统 - 算法

**优先级**: P1

**文件路径**:
- `src-tauri/src/game/hint.rs`

**描述**: 实现寻找可用移动的算法

**实现要点**:

```rust
// src-tauri/src/game/hint.rs
use super::card::Card;
use super::rules::{can_drag_sequence, can_place_on};
use super::state::{Column, GameState};

/// 提示类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HintType {
    /// 普通移动
    Normal,
    /// 同花色移动（优先）
    SameSuit,
    /// 可以形成更长序列
    ExtendSequence,
    /// 移动到空列
    ToEmpty,
    /// 发牌
    Deal,
}

/// 提示信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hint {
    pub from_column: usize,
    pub from_index: usize,
    pub to_column: usize,
    pub hint_type: HintType,
    pub priority: u8, // 优先级，越高越好
    pub description: String,
}

impl GameState {
    /// 寻找所有可能的移动
    pub fn find_all_moves(&self) -> Vec<Hint> {
        let mut hints = Vec::new();

        // 遍历每一列
        for from_col in 0..self.columns.len() {
            let column = &self.columns[from_col];

            // 从底部向上找可拖拽序列
            for from_idx in 0..column.len() {
                let cards: Vec<Card> = column.cards[from_idx..].to_vec();

                if !can_drag_sequence(&cards) {
                    continue;
                }

                // 尝试移动到每一列
                for to_col in 0..self.columns.len() {
                    if from_col == to_col {
                        continue;
                    }

                    if can_place_on(&cards, &self.columns[to_col]) {
                        let hint = self.evaluate_move(from_col, from_idx, to_col, &cards);
                        hints.push(hint);
                    }
                }
            }
        }

        // 按优先级排序
        hints.sort_by(|a, b| b.priority.cmp(&a.priority));
        hints
    }

    /// 评估一个移动的质量
    fn evaluate_move(
        &self,
        from_col: usize,
        from_idx: usize,
        to_col: usize,
        cards: &[Card],
    ) -> Hint {
        let mut priority = 50; // 基础优先级
        let mut hint_type = HintType::Normal;

        let target_column = &self.columns[to_col];
        let source_column = &self.columns[from_col];

        // 检查是否是同花色移动
        if !target_column.is_empty() {
            let target_top = target_column.top_card().unwrap();
            if target_top.suit == cards[0].suit {
                priority += 30;
                hint_type = HintType::SameSuit;

                // 检查是否可以形成更长序列
                let existing_sequence_len = self.count_same_suit_sequence(target_column);
                if existing_sequence_len + cards.len() >= 13 {
                    priority += 50; // 可能完成序列！
                    hint_type = HintType::ExtendSequence;
                }
            }
        } else {
            // 移动到空列
            priority += 10;
            hint_type = HintType::ToEmpty;
        }

        // 检查移动后是否会翻开新牌
        if from_idx > 0 && !source_column.cards[from_idx - 1].face_up {
            priority += 20;
        }

        // 检查源列是否只剩这些牌（移动后变空）
        if from_idx == 0 {
            priority -= 5; // 轻微惩罚
        }

        let description = format!(
            "Move {} {} from column {} to column {}",
            cards.len(),
            if cards.len() == 1 { "card" } else { "cards" },
            from_col + 1,
            to_col + 1
        );

        Hint {
            from_column: from_col,
            from_index: from_idx,
            to_column: to_col,
            hint_type,
            priority: priority.min(100) as u8,
            description,
        }
    }

    /// 计算一列中从顶部开始的同花色序列长度
    fn count_same_suit_sequence(&self, column: &Column) -> usize {
        let visible = column.visible_cards();
        if visible.is_empty() {
            return 0;
        }

        let suit = visible.last().unwrap().suit;
        let mut count = 0;

        for card in visible.iter().rev() {
            if card.suit == suit && card.face_up {
                count += 1;
            } else {
                break;
            }
        }

        count
    }

    /// 获取最佳提示
    pub fn get_best_hint(&self) -> Option<Hint> {
        self.find_all_moves().into_iter().next()
    }
}

// 添加 Tauri 命令
#[tauri::command]
pub fn get_hint(manager: State<GameManager>) -> Option<Hint> {
    let state = manager.state.lock().unwrap();
    state.as_ref().and_then(|s| s.get_best_hint())
}

#[tauri::command]
pub fn get_all_hints(manager: State<GameManager>) -> Vec<Hint> {
    let state = manager.state.lock().unwrap();
    state.as_ref().map(|s| s.find_all_moves()).unwrap_or_default()
}
```

**验收标准**:
- [ ] 能找到有效的移动
- [ ] 同花色移动优先级更高
- [ ] 可形成完整序列的移动最优先

---

### Task 3.5: 提示系统 - 前端 UI

**优先级**: P1

**文件路径**:
- `src/lib/components/Toolbar.svelte` (修改)
- `src/lib/components/GameBoard.svelte` (修改)

**描述**: 显示提示按钮和高亮效果

**实现要点**:

```typescript
// 在 gameStore.svelte.ts 中添加
currentHint = $state<Hint | null>(null);

async getHint() {
  const hint = await invoke<Hint | null>('get_hint');
  this.currentHint = hint;
  return hint;
}

clearHint() {
  this.currentHint = null;
}
```

```svelte
<!-- 在 Toolbar.svelte 中添加提示按钮 -->
<button
  class="icon-btn"
  onclick={() => gameStore.getHint()}
  title="提示 (H)"
>
  <span class="icon">💡</span>
</button>
```

```svelte
<!-- 在 Column.svelte 中添加高亮逻辑 -->
<script lang="ts">
  interface Props {
    // ... 原有 props
    highlightedFrom: boolean;
    highlightedTo: boolean;
  }

  let { highlightedFrom = false, highlightedTo = false, ...rest }: Props = $props();
</script>

<div
  class="column"
  class:highlighted-from={highlightedFrom}
  class:highlighted-to={highlightedTo}
>
  <!-- ... -->
</div>

<style>
  .column.highlighted-from {
    background: rgba(255, 200, 0, 0.2);
    border-radius: 8px;
    animation: pulse 1s ease infinite;
  }

  .column.highlighted-to {
    background: rgba(0, 200, 100, 0.2);
    border-radius: 8px;
    animation: pulse 1s ease infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }
</style>
```

**验收标准**:
- [ ] 点击提示按钮显示高亮
- [ ] 源列和目标列区分明显
- [ ] 执行移动后清除提示

---

### Task 3.6: 动画效果 - 卡牌移动

**优先级**: P1

**文件路径**:
- `src/lib/components/Card.svelte` (修改)
- `src/lib/styles/animations.css`

**描述**: 添加卡牌移动、翻转动画

**实现要点**:

```css
/* src/lib/styles/animations.css */

/* 卡牌移动动画 */
.card-move {
  animation: cardMove 0.3s ease-out;
}

@keyframes cardMove {
  0% {
    transform: translate(var(--from-x), var(--from-y)) scale(1.05);
    opacity: 0.8;
  }
  100% {
    transform: translate(0, 0) scale(1);
    opacity: 1;
  }
}

/* 卡牌翻转动画 */
.card-flip {
  animation: cardFlip 0.4s ease-out;
}

@keyframes cardFlip {
  0% {
    transform: rotateY(180deg);
  }
  100% {
    transform: rotateY(0deg);
  }
}

/* 悬停效果 */
.card-hover {
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.card-hover:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
}

/* 选中效果 */
.card-selected {
  animation: cardSelected 0.5s ease infinite;
}

@keyframes cardSelected {
  0%, 100% {
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.8);
  }
  50% {
    box-shadow: 0 0 0 5px rgba(59, 130, 246, 0.4);
  }
}

/* 完成序列动画 */
.sequence-complete {
  animation: sequenceComplete 0.6s ease-out;
}

@keyframes sequenceComplete {
  0% {
    transform: scale(1);
    opacity: 1;
  }
  50% {
    transform: scale(1.1);
    opacity: 0.8;
  }
  100% {
    transform: scale(0) translateY(-100px);
    opacity: 0;
  }
}

/* 发牌动画 */
.deal-card {
  animation: dealCard 0.3s ease-out;
  animation-delay: var(--delay);
  animation-fill-mode: backwards;
}

@keyframes dealCard {
  0% {
    transform: translateX(-200px) translateY(200px);
    opacity: 0;
  }
  100% {
    transform: translateX(0) translateY(0);
    opacity: 1;
  }
}

/* 胜利庆祝动画 */
.celebration {
  animation: celebrate 0.5s ease infinite alternate;
}

@keyframes celebrate {
  0% {
    transform: scale(1) rotate(-5deg);
  }
  100% {
    transform: scale(1.1) rotate(5deg);
  }
}

/* 淡入 */
.fade-in {
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

/* 滑入 */
.slide-up {
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
```

```svelte
<!-- 在 Card.svelte 中应用动画 -->
<script lang="ts">
  interface Props {
    card: Card;
    animating?: 'move' | 'flip' | 'deal' | null;
  }

  let { card, animating = null }: Props = $props();
</script>

<button
  class="card {card.face_up ? 'face-up' : 'face-down'} {isRed ? 'red' : 'black'}"
  class:card-move={animating === 'move'}
  class:card-flip={animating === 'flip'}
  class:deal-card={animating === 'deal'}
>
  <!-- ... -->
</button>
```

**验收标准**:
- [ ] 移动动画流畅
- [ ] 翻牌动画正确
- [ ] 发牌动画有延迟效果

---

### Task 3.7: 音效系统

**优先级**: P2

**文件路径**:
- `src/lib/utils/sounds.ts`
- `src/lib/stores/settingsStore.svelte.ts`

**描述**: 添加游戏音效

**实现要点**:

```typescript
// src/lib/utils/sounds.ts
import { settingsStore } from '../stores/settingsStore.svelte';

// 使用 Web Audio API 或简单的 Audio 元素
const sounds = {
  deal: '/sounds/deal.mp3',
  move: '/sounds/move.mp3',
  complete: '/sounds/complete.mp3',
  win: '/sounds/win.mp3',
  error: '/sounds/error.mp3',
};

let audioContext: AudioContext | null = null;
const audioBuffers: Map<string, AudioBuffer> = new Map();

async function initAudioContext() {
  if (!audioContext) {
    audioContext = new AudioContext();
  }
  return audioContext;
}

async function loadSound(name: string): Promise<AudioBuffer | null> {
  if (audioBuffers.has(name)) {
    return audioBuffers.get(name)!;
  }

  try {
    const ctx = await initAudioContext();
    const response = await fetch(sounds[name as keyof typeof sounds]);
    const arrayBuffer = await response.arrayBuffer();
    const audioBuffer = await ctx.decodeAudioData(arrayBuffer);
    audioBuffers.set(name, audioBuffer);
    return audioBuffer;
  } catch (error) {
    console.warn(`Failed to load sound: ${name}`, error);
    return null;
  }
}

export async function playSound(name: keyof typeof sounds) {
  if (!settingsStore.soundEnabled) return;

  try {
    const ctx = await initAudioContext();
    const buffer = await loadSound(name);

    if (buffer) {
      const source = ctx.createBufferSource();
      source.buffer = buffer;
      source.connect(ctx.destination);
      source.start(0);
    }
  } catch (error) {
    console.warn(`Failed to play sound: ${name}`, error);
  }
}

// 预加载所有音效
export async function preloadSounds() {
  await initAudioContext();
  await Promise.all(Object.keys(sounds).map(loadSound));
}
```

```typescript
// src/lib/stores/settingsStore.svelte.ts
class SettingsStore {
  soundEnabled = $state(true);
  animationEnabled = $state(true);

  load() {
    const saved = localStorage.getItem('spider-settings');
    if (saved) {
      const settings = JSON.parse(saved);
      this.soundEnabled = settings.soundEnabled ?? true;
      this.animationEnabled = settings.animationEnabled ?? true;
    }
  }

  save() {
    localStorage.setItem('spider-settings', JSON.stringify({
      soundEnabled: this.soundEnabled,
      animationEnabled: this.animationEnabled,
    }));
  }

  toggleSound() {
    this.soundEnabled = !this.soundEnabled;
    this.save();
  }

  toggleAnimation() {
    this.animationEnabled = !this.animationEnabled;
    this.save();
  }
}

export const settingsStore = new SettingsStore();
```

**验收标准**:
- [ ] 音效正常播放
- [ ] 可关闭音效
- [ ] 设置持久化

---

### Task 3.8: 断点续玩 - 自动保存

**优先级**: P1

**文件路径**:
- `src-tauri/src/game/state.rs` (修改)
- `src-tauri/src/commands/game.rs` (修改)
- `src-tauri/src/storage.rs`

**描述**: 自动保存游戏状态，启动时恢复

**实现要点**:

```rust
// src-tauri/src/storage.rs
use crate::game::state::GameState;
use std::fs;
use std::path::PathBuf;

pub struct Storage {
    save_path: PathBuf,
}

impl Storage {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let app_data_dir = app_handle.path().app_data_dir().unwrap();
        fs::create_dir_all(&app_data_dir).ok();
        Storage {
            save_path: app_data_dir.join("autosave.json"),
        }
    }

    pub fn save(&self, state: &GameState) -> Result<(), String> {
        let json = serde_json::to_string_pretty(state)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        fs::write(&self.save_path, json)
            .map_err(|e| format!("Failed to write file: {}", e))
    }

    pub fn load(&self) -> Option<GameState> {
        let json = fs::read_to_string(&self.save_path).ok()?;
        serde_json::from_str(&json).ok()
    }

    pub fn delete(&self) {
        fs::remove_file(&self.save_path).ok();
    }

    pub fn exists(&self) -> bool {
        self.save_path.exists()
    }
}

// 在 commands/game.rs 中添加
use crate::storage::Storage;

/// 自动保存
#[tauri::command]
pub fn autosave(
    manager: State<GameManager>,
    storage: State<Storage>,
) -> Result<(), String> {
    let state = manager.state.lock().unwrap();
    if let Some(ref game_state) = *state {
        storage.save(game_state)?;
    }
    Ok(())
}

/// 加载存档
#[tauri::command]
pub fn load_save(storage: State<Storage>) -> Option<GameState> {
    storage.load()
}

/// 检查是否有存档
#[tauri::command]
pub fn has_save(storage: State<Storage>) -> bool {
    storage.exists()
}

/// 删除存档
#[tauri::command]
pub fn delete_save(storage: State<Storage>) {
    storage.delete();
}
```

```typescript
// 在 gameStore.svelte.ts 中添加自动保存
import { debounce } from 'lodash-es';

// 每次状态变化后自动保存（防抖 1 秒）
const autosave = debounce(async () => {
  if (this.state) {
    await invoke('autosave');
  }
}, 1000);

// 在状态更新后调用 autosave()
async moveCards(...) {
  // ... 移动逻辑
  autosave();
}
```

```svelte
<!-- 在 App.svelte 启动时检查存档 -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { gameStore } from './lib/stores/gameStore.svelte';

  let showContinueDialog = $state(false);

  onMount(async () => {
    const hasSave = await invoke<boolean>('has_save');
    if (hasSave) {
      showContinueDialog = true;
    } else {
      gameStore.newGame();
    }
  });

  async function continueGame() {
    const state = await invoke<GameState>('load_save');
    if (state) {
      gameStore.state = state;
    }
    showContinueDialog = false;
  }

  function startNewGame() {
    invoke('delete_save');
    gameStore.newGame();
    showContinueDialog = false;
  }
</script>

{#if showContinueDialog}
  <div class="dialog-overlay">
    <div class="dialog">
      <h2>继续游戏？</h2>
      <p>检测到未完成的游戏，是否继续？</p>
      <div class="actions">
        <button class="primary" onclick={continueGame}>继续游戏</button>
        <button onclick={startNewGame}>新游戏</button>
      </div>
    </div>
  </div>
{/if}
```

**验收标准**:
- [ ] 游戏状态自动保存
- [ ] 启动时提示继续
- [ ] 胜利后删除存档

---

## 阶段验收清单

- [ ] 撤销/重做功能正常
- [ ] 键盘快捷键工作
- [ ] 提示能找到有效移动
- [ ] 动画流畅（60fps）
- [ ] 音效可选开关
- [ ] 关闭重开能恢复进度

## 风险与依赖

### 风险
1. **动画性能**: 复杂动画可能影响性能
   - 缓解：使用 CSS transform，避免 reflow

2. **音效资源**: 音效文件增加包体积
   - 缓解：使用压缩格式，控制文件大小

### 依赖
- Phase 2 所有任务完成
- 音效资源文件

## 下一阶段

完成本阶段后，进入 [Phase 4: 打磨发布](./phase-4-release.md)
