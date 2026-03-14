# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Classic Spider is a Tauri 2.x + Rust + Svelte 5 recreation of the classic Windows Spider Solitaire game. It's a cross-platform desktop application targeting elderly users with accessibility-focused design (large fonts, big buttons, high contrast).

## Common Commands

```bash
# Install dependencies
pnpm install

# Development mode
pnpm tauri dev

# Build for release
pnpm tauri build
```

## Architecture

The project follows a clean separation between Rust backend and Svelte frontend, communicating via Tauri Commands.

### Backend (Rust)

- **Game Logic** (`src-tauri/src/game/`): Core game logic including card definitions, deck operations, rule validation, game state management, and history (undo/redo)
- **Commands** (`src-tauri/src/commands/`): Tauri command handlers exposed to the frontend
- **Storage** (`src-tauri/src/storage/`): Data persistence using Tauri Plugin Store

### Frontend (Svelte 5)

- **Components** (`src/lib/components/`): UI components (Card, Column, StockPile, Foundation, Toolbar, GameBoard, modals)
- **Stores** (`src/lib/stores/`): Reactive state management (game state, settings, statistics)
- **Utils** (`src/lib/utils/`): Drag handling, animations, sound utilities

### Key Patterns

1. **Game rules validation** happens entirely in Rust (`rules.rs`) - frontend sends move requests, backend validates and returns new state
2. **Undo/Redo** implemented via history stack in Rust (`history.rs`)
   - **重要**: 修改游戏状态的命令必须**先执行操作，再将新状态推入历史栈**（见 `move_cards` 和 `deal_cards`）
   - 错误顺序：先 `history.push()` 再执行操作 → 会导致重做失败
3. **State synchronization** - frontend store subscribes to Rust GameState, updates on every command response
4. **Drag interaction** - frontend handles visual drag, validates with backend on drop via `move_cards` command

## Difficulty System

| Level | Suits | Description |
|-------|-------|-------------|
| 1 | Spade only | Beginner |
| 2 | Spade + Heart | Intermediate |
| 3 | All 4 suits | Expert |

## Game Rules Summary

- 10 columns with 54 initial cards (54/104 total, 2 decks worth)
- Move cards: must be same suit, sequential descending (K→A)
- Place cards: first card must be exactly one rank higher than target
- Deal: one card to each column (all columns must be non-empty)
- Win: collect 8 complete K-A sequences of same suit

## Tech Stack

- Tauri 2.x (desktop framework)
- Rust 1.75+ (backend)
- Svelte 5 (frontend)
- TailwindCSS 4.x (styling)
