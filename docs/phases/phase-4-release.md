# Phase 4: 打磨发布 (Week 7-8)

## 概述

**目标**: 完善细节，优化性能，准备发布

**预计工期**: 2 周

**前置依赖**: Phase 3 完成

---

## 任务清单

### Task 4.1: 统计数据系统 - 数据结构

**优先级**: P0

**文件路径**:
- `src-tauri/src/stats/mod.rs`
- `src-tauri/src/stats/models.rs`

**描述**: 实现游戏统计数据模型

**实现要点**:

```rust
// src-tauri/src/stats/mod.rs
pub mod models;

// src-tauri/src/stats/models.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 单局游戏记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRecord {
    pub difficulty: String,
    pub score: i32,
    pub moves: u32,
    pub duration_seconds: u64,
    pub completed: bool,
    pub timestamp: u64,
}

/// 难度统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DifficultyStats {
    pub games_played: u32,
    pub games_won: u32,
    pub best_score: i32,
    pub best_time: Option<u64>,
    pub total_moves: u64,
    pub total_time: u64,
}

impl DifficultyStats {
    pub fn win_rate(&self) -> f64 {
        if self.games_played == 0 {
            0.0
        } else {
            (self.games_won as f64 / self.games_played as f64) * 100.0
        }
    }

    pub fn average_moves(&self) -> f64 {
        if self.games_played == 0 {
            0.0
        } else {
            self.total_moves as f64 / self.games_played as f64
        }
    }

    pub fn average_time(&self) -> f64 {
        if self.games_played == 0 {
            0.0
        } else {
            self.total_time as f64 / self.games_played as f64
        }
    }
}

/// 全局统计数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Statistics {
    pub by_difficulty: HashMap<String, DifficultyStats>,
    pub recent_games: Vec<GameRecord>,
    pub current_streak: u32,
    pub best_streak: u32,
}

impl Statistics {
    pub fn new() -> Self {
        let mut by_difficulty = HashMap::new();
        by_difficulty.insert("Easy".to_string(), DifficultyStats::default());
        by_difficulty.insert("Medium".to_string(), DifficultyStats::default());
        by_difficulty.insert("Hard".to_string(), DifficultyStats::default());

        Statistics {
            by_difficulty,
            recent_games: Vec::new(),
            current_streak: 0,
            best_streak: 0,
        }
    }

    /// 记录一局游戏
    pub fn record_game(&mut self, record: GameRecord) {
        // 更新难度统计
        let stats = self.by_difficulty
            .entry(record.difficulty.clone())
            .or_insert_with(DifficultyStats::default);

        stats.games_played += 1;
        stats.total_moves += record.moves as u64;
        stats.total_time += record.duration_seconds;

        if record.completed {
            stats.games_won += 1;

            if record.score > stats.best_score {
                stats.best_score = record.score;
            }

            if stats.best_time.is_none() || Some(record.duration_seconds) < stats.best_time {
                stats.best_time = Some(record.duration_seconds);
            }

            // 更新连胜
            self.current_streak += 1;
            if self.current_streak > self.best_streak {
                self.best_streak = self.current_streak;
            }
        } else {
            // 重置连胜
            self.current_streak = 0;
        }

        // 添加到最近游戏
        self.recent_games.push(record);

        // 只保留最近 20 局
        if self.recent_games.len() > 20 {
            self.recent_games.remove(0);
        }
    }

    /// 获取总游戏数
    pub fn total_games(&self) -> u32 {
        self.by_difficulty.values().map(|s| s.games_played).sum()
    }

    /// 获取总胜场
    pub fn total_wins(&self) -> u32 {
        self.by_difficulty.values().map(|s| s.games_won).sum()
    }

    /// 获取总胜率
    pub fn overall_win_rate(&self) -> f64 {
        let total = self.total_games();
        if total == 0 {
            0.0
        } else {
            (self.total_wins() as f64 / total as f64) * 100.0
        }
    }
}
```

**验收标准**:
- [ ] 统计数据结构完整
- [ ] 胜率计算正确
- [ ] 连胜记录正确

---

### Task 4.2: 统计数据系统 - 持久化

**优先级**: P0

**文件路径**:
- `src-tauri/src/stats/storage.rs`
- `src-tauri/src/commands/stats.rs`

**描述**: 统计数据持久化和 Tauri 命令

**实现要点**:

```rust
// src-tauri/src/stats/storage.rs
use super::models::{GameRecord, Statistics};
use std::fs;
use std::path::PathBuf;

pub struct StatsStorage {
    path: PathBuf,
}

impl StatsStorage {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let app_data_dir = app_handle.path().app_data_dir().unwrap();
        fs::create_dir_all(&app_data_dir).ok();
        StatsStorage {
            path: app_data_dir.join("statistics.json"),
        }
    }

    pub fn load(&self) -> Statistics {
        if let Ok(json) = fs::read_to_string(&self.path) {
            if let Ok(stats) = serde_json::from_str(&json) {
                return stats;
            }
        }
        Statistics::new()
    }

    pub fn save(&self, stats: &Statistics) -> Result<(), String> {
        let json = serde_json::to_string_pretty(stats)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        fs::write(&self.path, json)
            .map_err(|e| format!("Failed to write: {}", e))
    }
}

// src-tauri/src/commands/stats.rs
use crate::stats::models::{GameRecord, Statistics};
use crate::stats::storage::StatsStorage;
use std::sync::Mutex;
use tauri::State;

pub struct StatsManager {
    pub stats: Mutex<Statistics>,
    pub storage: StatsStorage,
}

#[tauri::command]
pub fn get_statistics(manager: State<StatsManager>) -> Statistics {
    manager.stats.lock().unwrap().clone()
}

#[tauri::command]
pub fn record_game(
    record: GameRecord,
    manager: State<StatsManager>,
) -> Result<Statistics, String> {
    let mut stats = manager.stats.lock().unwrap();
    stats.record_game(record);
    manager.storage.save(&stats)?;
    Ok(stats.clone())
}

#[tauri::command]
pub fn reset_statistics(manager: State<StatsManager>) -> Result<(), String> {
    let mut stats = manager.stats.lock().unwrap();
    *stats = Statistics::new();
    manager.storage.save(&stats)
}
```

**验收标准**:
- [ ] 统计数据持久化
- [ ] 命令正常工作

---

### Task 4.3: 统计数据系统 - 前端 UI

**优先级**: P1

**文件路径**:
- `src/lib/components/StatsModal.svelte`

**描述**: 统计数据展示弹窗

**实现要点**:

```svelte
<!-- src/lib/components/StatsModal.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface DifficultyStats {
    games_played: number;
    games_won: number;
    best_score: number;
    best_time: number | null;
    average_moves: number;
    average_time: number;
    win_rate: number;
  }

  interface Statistics {
    by_difficulty: Record<string, DifficultyStats>;
    current_streak: number;
    best_streak: number;
    total_games: number;
    total_wins: number;
    overall_win_rate: number;
  }

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

  let stats = $state<Statistics | null>(null);

  onMount(async () => {
    stats = await invoke('get_statistics');
  });

  async function resetStats() {
    if (confirm('确定要重置所有统计数据吗？')) {
      stats = await invoke('reset_statistics');
    }
  }

  function formatTime(seconds: number | null): string {
    if (seconds === null) return '-';
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
</script>

<div class="modal-overlay" onclick={onclose}>
  <div class="modal" onclick={(e) => e.stopPropagation()}>
    <header>
      <h2>游戏统计</h2>
      <button class="close-btn" onclick={onclose}>✕</button>
    </header>

    {#if stats}
      <div class="content">
        <!-- 总览 -->
        <section class="overview">
          <div class="stat-card">
            <span class="value">{stats.total_games}</span>
            <span class="label">总游戏数</span>
          </div>
          <div class="stat-card">
            <span class="value">{stats.total_wins}</span>
            <span class="label">胜利场次</span>
          </div>
          <div class="stat-card">
            <span class="value">{stats.overall_win_rate.toFixed(1)}%</span>
            <span class="label">总胜率</span>
          </div>
          <div class="stat-card">
            <span class="value">{stats.best_streak}</span>
            <span class="label">最佳连胜</span>
          </div>
        </section>

        <!-- 各难度统计 -->
        <section class="difficulty-stats">
          <h3>难度统计</h3>
          <table>
            <thead>
              <tr>
                <th>难度</th>
                <th>游戏数</th>
                <th>胜率</th>
                <th>最高分</th>
                <th>最快时间</th>
              </tr>
            </thead>
            <tbody>
              {#each ['Easy', 'Medium', 'Hard'] as diff}
                {@const d = stats.by_difficulty[diff]}
                <tr>
                  <td>
                    {#if diff === 'Easy'}初级
                    {:else if diff === 'Medium'}中级
                    {:else}高级{/if}
                  </td>
                  <td>{d.games_played}</td>
                  <td>{d.win_rate.toFixed(1)}%</td>
                  <td>{d.best_score || '-'}</td>
                  <td>{formatTime(d.best_time)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </section>
      </div>

      <footer>
        <button class="danger" onclick={resetStats}>重置统计</button>
      </footer>
    {:else}
      <div class="loading">加载中...</div>
    {/if}
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
  }

  .modal {
    background: white;
    border-radius: 12px;
    width: 500px;
    max-width: 90vw;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid #e5e7eb;
  }

  h2 {
    margin: 0;
    font-size: 20px;
    color: #1f2937;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 20px;
    cursor: pointer;
    color: #6b7280;
  }

  .content {
    padding: 20px;
    overflow-y: auto;
  }

  .overview {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
    margin-bottom: 24px;
  }

  .stat-card {
    background: #f9fafb;
    border-radius: 8px;
    padding: 12px;
    text-align: center;
  }

  .stat-card .value {
    display: block;
    font-size: 24px;
    font-weight: bold;
    color: #1a472a;
  }

  .stat-card .label {
    display: block;
    font-size: 12px;
    color: #6b7280;
    margin-top: 4px;
  }

  .difficulty-stats h3 {
    margin: 0 0 12px;
    font-size: 16px;
    color: #374151;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th, td {
    padding: 10px;
    text-align: left;
    border-bottom: 1px solid #e5e7eb;
  }

  th {
    background: #f9fafb;
    font-weight: 600;
    font-size: 13px;
    color: #6b7280;
  }

  footer {
    padding: 16px 20px;
    border-top: 1px solid #e5e7eb;
    text-align: right;
  }

  button.danger {
    background: #ef4444;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
  }

  button.danger:hover {
    background: #dc2626;
  }

  .loading {
    padding: 40px;
    text-align: center;
    color: #6b7280;
  }
</style>
```

**验收标准**:
- [ ] 统计数据正确显示
- [ ] 各难度分开统计
- [ ] 重置功能正常

---

### Task 4.4: 难度选择 UI

**优先级**: P0

**文件路径**:
- `src/lib/components/DifficultyModal.svelte`
- `src/lib/components/Toolbar.svelte` (修改)

**描述**: 新游戏时选择难度

**实现要点**:

```svelte
<!-- src/lib/components/DifficultyModal.svelte -->
<script lang="ts">
  interface Props {
    onSelect: (difficulty: string) => void;
    oncancel: () => void;
  }

  let { onSelect, oncancel }: Props = $props();

  const difficulties = [
    {
      id: 'Easy',
      name: '初级',
      description: '单花色（黑桃）',
      icon: '♠',
      suits: 1,
    },
    {
      id: 'Medium',
      name: '中级',
      description: '双花色（黑桃 + 红心）',
      icon: '♠♥',
      suits: 2,
    },
    {
      id: 'Hard',
      name: '高级',
      description: '四花色',
      icon: '♠♥♦♣',
      suits: 4,
    },
  ];
</script>

<div class="modal-overlay" onclick={oncancel}>
  <div class="modal" onclick={(e) => e.stopPropagation()}>
    <h2>选择难度</h2>
    <div class="options">
      {#each difficulties as diff}
        <button
          class="option"
          onclick={() => onSelect(diff.id)}
        >
          <span class="icon">{diff.icon}</span>
          <div class="info">
            <span class="name">{diff.name}</span>
            <span class="desc">{diff.description}</span>
          </div>
          <div class="suits-badge">{diff.suits} 花色</div>
        </button>
      {/each}
    </div>
    <button class="cancel" onclick={oncancel}>取消</button>
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
  }

  .modal {
    background: white;
    border-radius: 12px;
    padding: 24px;
    width: 400px;
  }

  h2 {
    margin: 0 0 20px;
    text-align: center;
    color: #1f2937;
  }

  .options {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 20px;
  }

  .option {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background: #f9fafb;
    border: 2px solid #e5e7eb;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .option:hover {
    border-color: #1a472a;
    background: #f0fdf4;
  }

  .icon {
    font-size: 28px;
    width: 60px;
    text-align: center;
  }

  .info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
    text-align: left;
  }

  .name {
    font-size: 18px;
    font-weight: 600;
    color: #1f2937;
  }

  .desc {
    font-size: 13px;
    color: #6b7280;
  }

  .suits-badge {
    background: #1a472a;
    color: white;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
  }

  .cancel {
    width: 100%;
    padding: 12px;
    background: #f3f4f6;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    color: #6b7280;
  }
</style>
```

**验收标准**:
- [ ] 三种难度可选
- [ ] 选择后开始新游戏
- [ ] 难度说明清晰

---

### Task 4.5: 皮肤系统（可选）

**优先级**: P2

**文件路径**:
- `src/lib/stores/settingsStore.svelte.ts` (修改)
- `src/lib/components/SettingsModal.svelte`

**描述**: 多套卡牌背面和桌面颜色

**实现要点**:

```typescript
// 在 settingsStore.svelte.ts 中添加
cardBackStyle = $state<'classic' | 'blue' | 'red' | 'nature'>('classic');
tableColor = $state<'green' | 'blue' | 'brown'>('green');

const cardBacks = {
  classic: 'linear-gradient(135deg, #1e40af 0%, #3b82f6 100%)',
  blue: 'linear-gradient(135deg, #1e3a8a 0%, #60a5fa 100%)',
  red: 'linear-gradient(135deg, #991b1b 0%, #f87171 100%)',
  nature: 'linear-gradient(135deg, #14532d 0%, #4ade80 100%)',
};

const tableColors = {
  green: '#1a472a',
  blue: '#1e3a5f',
  brown: '#4a3728',
};
```

```svelte
<!-- SettingsModal.svelte -->
<script lang="ts">
  import { settingsStore } from '../stores/settingsStore.svelte';

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();
</script>

<div class="modal-overlay" onclick={onclose}>
  <div class="modal" onclick={(e) => e.stopPropagation()}>
    <h2>设置</h2>

    <div class="setting-group">
      <label>卡牌背面</label>
      <div class="options">
        {#each ['classic', 'blue', 'red', 'nature'] as style}
          <button
            class="card-preview {style}"
            class:selected={settingsStore.cardBackStyle === style}
            onclick={() => settingsStore.setCardBack(style)}
          >
            <div class="card"></div>
          </button>
        {/each}
      </div>
    </div>

    <div class="setting-group">
      <label>桌面颜色</label>
      <div class="options">
        {#each ['green', 'blue', 'brown'] as color}
          <button
            class="color-preview {color}"
            class:selected={settingsStore.tableColor === color}
            onclick={() => settingsStore.setTableColor(color)}
          >
            <div class="swatch"></div>
          </button>
        {/each}
      </div>
    </div>

    <div class="setting-group">
      <label>
        <input
          type="checkbox"
          bind:checked={settingsStore.soundEnabled}
          onchange={() => settingsStore.save()}
        />
        音效
      </label>
      <label>
        <input
          type="checkbox"
          bind:checked={settingsStore.animationEnabled}
          onchange={() => settingsStore.save()}
        />
        动画效果
      </label>
    </div>

    <button class="close" onclick={onclose}>关闭</button>
  </div>
</div>
```

**验收标准**:
- [ ] 多套皮肤可选
- [ ] 设置持久化
- [ ] 实时预览

---

### Task 4.6: 性能优化 - 渲染

**优先级**: P0

**文件路径**:
- `src/lib/components/Card.svelte` (优化)
- `src/lib/components/Column.svelte` (优化)

**描述**: 优化渲染性能

**实现要点**:

```svelte
<!-- 使用 CSS transform 代替 top 定位 -->
<!-- Column.svelte -->
<script lang="ts">
  // 使用虚拟化：只渲染可见卡牌
  $derived(visibleCards = cards.slice(-20)); // 最多显示 20 张
</script>

{#each visibleCards as card, index}
  <div
    class="card-wrapper"
    style="transform: translateY({calculateOffset(cards.length - visibleCards.length + index)}px)"
  >
    <!-- ... -->
  </div>
{/each}

<style>
  .card-wrapper {
    position: absolute;
    will-change: transform; /* GPU 加速 */
  }
</style>
```

```svelte
<!-- 使用 keyed each 提高更新效率 -->
{#each cards as card, index (card.id)}
  <!-- 使用唯一 key -->
{/each}
```

```typescript
// 在 gameStore 中添加防抖更新
import { tick } from 'svelte';

async batchUpdates(callback: () => void) {
  await tick();
  callback();
}
```

**验收标准**:
- [ ] 拖拽时保持 60fps
- [ ] 大量卡牌时不卡顿
- [ ] 内存占用合理

---

### Task 4.7: 性能优化 - 内存

**优先级**: P1

**文件路径**:
- `src-tauri/src/game/state.rs` (优化)
- `src-tauri/src/game/history.rs` (优化)

**描述**: 优化内存使用

**实现要点**:

```rust
// 历史记录使用快照差异而非完整复制
// 或限制历史记录大小

impl History {
    // 使用更紧凑的状态表示
    pub fn push_compact(&mut self, state: &GameState, description: &str) {
        // 只保存必要的状态差异
        // 或者压缩序列化后的数据
    }
}

// 状态使用更紧凑的数据结构
// 例如使用 smallvec 或固定大小数组
use smallvec::SmallVec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    // 最多 104 张牌，使用 SmallVec 避免堆分配
    pub cards: SmallVec<[Card; 20]>,
}
```

**验收标准**:
- [ ] 内存占用 < 50MB
- [ ] 无内存泄漏

---

### Task 4.8: 测试与修复

**优先级**: P0

**文件路径**:
- `src-tauri/tests/`
- `src/lib/__tests__/`

**描述**: 功能测试和边界条件测试

**实现要点**:

```rust
// src-tauri/tests/integration_test.rs
use classic_spider::game::deck::Difficulty;
use classic_spider::game::state::GameState;
use classic_spider::game::rules::*;

#[test]
fn test_complete_game_flow() {
    let mut state = GameState::new(Difficulty::Easy);

    // 验证初始状态
    assert_eq!(state.score, 500);
    assert_eq!(state.stock.len(), 50);

    // 模拟完整游戏流程
    // ...
}

#[test]
fn test_edge_case_empty_column_move() {
    // 测试空列移动
}

#[test]
fn test_edge_case_last_card_flip() {
    // 测试最后一张牌翻转
}

#[test]
fn test_undo_redo_consistency() {
    // 测试撤销重做一致性
}
```

**验收标准**:
- [ ] 核心功能测试通过
- [ ] 边界条件覆盖
- [ ] 无已知 bug

---

### Task 4.9: 打包配置 - Windows

**优先级**: P0

**文件路径**:
- `src-tauri/tauri.conf.json` (修改)
- `src-tauri/icons/`

**描述**: 配置 Windows 打包

**实现要点**:

```json
// tauri.conf.json
{
  "bundle": {
    "active": true,
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "identifier": "com.classic-spider.app",
    "targets": ["msi", "nsis"],
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": "",
      "wix": {
        "language": "zh-CN"
      },
      "nsis": {
        "languages": ["SimpChinese", "English"],
        "displayLanguageSelector": true
      }
    }
  }
}
```

**验收标准**:
- [ ] MSI 安装包生成成功
- [ ] 安装后可正常启动
- [ ] 包大小 < 10MB

---

### Task 4.10: 打包配置 - macOS

**优先级**: P0

**文件路径**:
- `src-tauri/tauri.conf.json` (修改)

**描述**: 配置 macOS 打包

**实现要点**:

```json
{
  "bundle": {
    "macOS": {
      "minimumSystemVersion": "10.13",
      "entitlements": null,
      "exceptionDomain": "",
      "frameworks": [],
      "providerShortName": null,
      "signingIdentity": null
    },
    "targets": ["dmg", "app"]
  }
}
```

**验收标准**:
- [ ] DMG 生成成功
- [ ] 可正常安装和启动
- [ ] 代码签名正确（如有）

---

### Task 4.11: 打包配置 - Linux

**优先级**: P0

**文件路径**:
- `src-tauri/tauri.conf.json` (修改)

**描述**: 配置 Linux 打包

**实现要点**:

```json
{
  "bundle": {
    "linux": {
      "deb": {
        "depends": []
      },
      "appimage": {
        "bundleMediaFramework": false
      }
    },
    "targets": ["deb", "appimage"]
  }
}
```

**验收标准**:
- [ ] DEB/AppImage 生成成功
- [ ] 可正常运行

---

### Task 4.12: 最终验收

**优先级**: P0

**文件路径**: 无

**描述**: 最终功能验收

**验收清单**:

- [ ] **核心功能**
  - [ ] 新游戏正确初始化
  - [ ] 三种难度正常工作
  - [ ] 拖拽移动正确
  - [ ] 规则验证正确
  - [ ] 发牌功能正常
  - [ ] 胜利检测正确

- [ ] **辅助功能**
  - [ ] 撤销/重做正常
  - [ ] 提示功能有效
  - [ ] 断点续玩正常
  - [ ] 统计数据正确

- [ ] **用户体验**
  - [ ] 动画流畅
  - [ ] 音效正常
  - [ ] 键盘快捷键工作
  - [ ] UI 美观

- [ ] **性能指标**
  - [ ] 安装包 < 10MB
  - [ ] 内存占用 < 50MB
  - [ ] 冷启动 < 1秒
  - [ ] 拖拽 60fps

- [ ] **跨平台**
  - [ ] Windows 打包成功
  - [ ] macOS 打包成功
  - [ ] Linux 打包成功

---

## 阶段验收清单

- [ ] 安装包 < 10MB
- [ ] 内存占用 < 50MB
- [ ] 冷启动 < 1秒
- [ ] 三端打包成功
- [ ] 所有测试通过
- [ ] 无已知 bug

## 风险与依赖

### 风险
1. **打包大小**: 资源文件可能使包过大
   - 缓解：压缩资源，移除不必要的依赖

2. **跨平台问题**: 不同平台可能有独特问题
   - 缓解：在各平台充分测试

3. **签名问题**: macOS/Windows 代码签名需要证书
   - 缓解：准备开发者证书，或先发布未签名版本

### 依赖
- Phase 3 所有任务完成
- 打包环境配置
- 代码签名证书（可选）

## 发布检查清单

- [ ] 版本号更新
- [ ] CHANGELOG 更新
- [ ] 截图/资源准备
- [ ] 发布说明编写
- [ ] GitHub Release 创建
- [ ] 安装包上传
