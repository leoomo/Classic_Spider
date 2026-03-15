<script lang="ts">
	import { onMount, flushSync } from 'svelte';
	import Column from './Column.svelte';
	import { soundManager } from '$lib/utils/sound';
	import type { GameState, Card, Suit } from '$lib/types/game';

	let gameState = $state<GameState | null>(null);
	let selectedCard = $state<{ colIndex: number; cardIndex: number } | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let isMuted = $state(false);
	let shakeColumn = $state<number | null>(null);
	let showRestorePrompt = $state(false);
	let canUndoState = $state(false);
	let canRedoState = $state(false);
	let showDifficultyModal = $state(false);
	let showVictoryModal = $state(false);
	let showConfetti = $state(true); // 控制庆祝动画
	let lastFocusedElement: HTMLElement | null = null; // 焦点管理 - 记录打开对话框前聚焦的元素
	let hasShownVictory = $state(false); // 每局游戏只显示一次胜利弹窗

	// 拖拽状态
	let dragState = $state<{
		isDragging: boolean;
		fromCol: number;
		startCardIndex: number;
		cards: Card[];
		element: HTMLElement | null;
		offsetX: number;
		offsetY: number;
	} | null>(null);
	let dropTargetCol = $state<number | null>(null);
	let dropValid = $state(true);

	// Tauri 2.x 检测方式
	const isTauri = typeof window !== 'undefined' && ('__TAURI__' in window || '__TAURI_INTERNALS__' in window);

	const difficultyOptions = [
		{ level: 1, name: '🌱 简单', suits: '只有黑桃', description: '最容易赢，适合第一次玩' },
		{ level: 2, name: '🌟 中等', suits: '黑桃+红桃', description: '有点挑战性' },
		{ level: 3, name: '🔥 困难', suits: '四种花色', description: '最难，适合高手' }
	];

	async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
		if (isTauri) {
			const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
			return tauriInvoke<T>(cmd, args);
		} else {
			return invokeMock<T>(cmd, args);
		}
	}

	function invokeMock<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {

		switch (cmd) {
			case 'new_game': {
				const difficulty = (args?.difficulty as number) || 1;
				const state = createMockGameState(difficulty);
				return Promise.resolve(state as T);
			}
			case 'move_cards': {
				const fromCol = args?.from_col as number;
				const startIdx = args?.start_idx as number;
				const toCol = args?.to_col as number;
				const result = mockMoveCards(fromCol, startIdx, toCol);
				return Promise.resolve(result as T);
			}
			case 'deal_cards': {
				const result = mockDealCards();
				if (result === null) {
					return Promise.reject(new Error('没有可发的牌'));
				}
				return Promise.resolve(result as T);
			}
			case 'can_undo':
				return Promise.resolve(false as T);
			case 'can_redo':
				return Promise.resolve(false as T);
			case 'has_saved_game':
				return Promise.resolve(false as T);
		case 'get_hint': {
			// mock hint logic
			if (!gameState) return Promise.resolve(null as T);
			for (let fromCol = 0; fromCol < gameState.columns.length; fromCol++) {
				const column = gameState.columns[fromCol];
				for (let startIdx = 0; startIdx < column.length; startIdx++) {
					const cards = column.slice(startIdx);
					if (!cards.every(c => c.face_up)) continue;
					const firstCard = cards[0];
					for (let toCol = 0; toCol < gameState.columns.length; toCol++) {
						if (fromCol === toCol) continue;
						const target = gameState.columns[toCol];
						if (target.length === 0) {
							return Promise.resolve([fromCol, startIdx, toCol] as T);
						} else {
							const topCard = target[target.length - 1];
							if (topCard.value === firstCard.value + 1) {
								return Promise.resolve([fromCol, startIdx, toCol] as T);
							}
						}
					}
				}
			}
			return Promise.resolve(null as T);
		}
			default:
				return Promise.reject(`Unknown command: ${cmd}`);
		}
	}

	function createMockGameState(difficulty: number): GameState {
		const suits: Suit[] = difficulty === 1
			? ['spade']
			: difficulty === 2
				? ['spade', 'heart']
				: ['spade', 'heart', 'diamond', 'club'];

		const allCards: Card[] = [];
		let cardId = 0;

		for (const suit of suits) {
			for (let value = 1; value <= 13; value++) {
				const copies = 8 / suits.length;
				for (let i = 0; i < copies; i++) {
					allCards.push({ id: cardId++, suit, value, face_up: false });
				}
			}
		}

		for (let i = allCards.length - 1; i > 0; i--) {
			const j = Math.floor(Math.random() * (i + 1));
			[allCards[i], allCards[j]] = [allCards[j], allCards[i]];
		}

		const columns: Card[][] = [];
		for (let i = 0; i < 10; i++) {
			const count = i < 4 ? 6 : 5;
			const column: Card[] = [];
			for (let j = 0; j < count; j++) {
				const card = allCards.pop()!;
				column.push({
					...card,
					face_up: j === count - 1
				});
			}
			columns.push(column);
		}

		const stock = allCards.slice(0, 50);

		return {
			columns,
			stock,
			completed: 0,
			score: 500,
			moves: 0,
			difficulty,
			game_over: false,
			won: false
		};
	}

	function mockMoveCards(fromCol: number, startIdx: number, toCol: number): GameState | null {
		if (!gameState) return null;

		const newColumns = gameState.columns.map(col => [...col]);
		const movingCards = newColumns[fromCol].slice(startIdx);

		const targetColumn = newColumns[toCol];
		if (targetColumn.length === 0) {
			// Can place any sequence on empty column
		} else {
			const topCard = targetColumn[targetColumn.length - 1];
			const firstMovingCard = movingCards[0];
			if (topCard.value !== firstMovingCard.value + 1) {
				return null;
			}
		}

		for (let i = 0; i < movingCards.length - 1; i++) {
			if (movingCards[i].suit !== movingCards[i + 1].suit ||
				movingCards[i].value !== movingCards[i + 1].value + 1) {
				return null;
			}
		}

		newColumns[fromCol] = newColumns[fromCol].slice(0, startIdx);
		newColumns[toCol] = [...newColumns[toCol], ...movingCards];

		if (newColumns[fromCol].length > 0) {
			const topCard = newColumns[fromCol][newColumns[fromCol].length - 1];
			if (!topCard.face_up) {
				newColumns[fromCol][newColumns[fromCol].length - 1] = { ...topCard, face_up: true };
			}
		}

		let completed = gameState.completed;
		const newTargetColumn = newColumns[toCol];
		if (newTargetColumn.length >= 13) {
			const last13 = newTargetColumn.slice(-13);
			let isComplete = true;
			for (let i = 0; i < 13; i++) {
				const card = last13[i];
				if (card.suit !== 'spade' || card.value !== 13 - i) {
					isComplete = false;
					break;
				}
			}
			if (isComplete) {
				newColumns[toCol] = newColumns[toCol].slice(0, -13);
				completed += 1;
			}
		}

		return {
			...gameState,
			columns: newColumns,
			stock: gameState.stock,
			completed,
			score: Math.max(0, gameState.score - 1),
			moves: gameState.moves + 1,
			difficulty: gameState.difficulty,
			game_over: false,
			won: completed === 8
		};
	}

	function mockDealCards(): GameState | null {
		if (!gameState) return null;

		// 检查是否有牌可发
		if (gameState.stock.length === 0) {
			return null;
		}

		// 检查是否有空列
		if (gameState.columns.some(col => col.length === 0)) {
			return null;
		}

		const newStock = [...gameState.stock];
		const newColumns = gameState.columns.map(col => [...col]);

		for (let col of newColumns) {
			if (newStock.length === 0) break;
			const card = newStock.pop()!;
			card.face_up = true;
			col.push(card);
		}

		return {
			...gameState,
			columns: newColumns,
			stock: newStock,
			moves: gameState.moves + 1
		};
	}

	async function initGame(difficulty: number = 1) {
		try {
			isLoading = true;
			error = null;
			showDifficultyModal = false;
			showVictoryModal = false;
			hasShownVictory = false; // 重置胜利弹窗标志
			soundManager.play('shuffle');
			gameState = await invoke<GameState>('new_game', { difficulty });
			selectedCard = null;
			showRestorePrompt = false;
		} catch (e) {
			error = `加载失败: ${e}`;
			console.error('Failed to create game:', e);
		} finally {
			isLoading = false;
		}
	}

	function openNewGameModal() {
		lastFocusedElement = document.activeElement as HTMLElement;
		showVictoryModal = false; // 先关闭胜利弹窗
		showDifficultyModal = true;
		// 延迟聚焦到对话框，确保 DOM 已更新
		setTimeout(() => {
			const modal = document.querySelector('.difficulty-modal') as HTMLElement;
			if (modal) modal.focus();
		}, 0);
	}

	function closeDifficultyModal() {
		showDifficultyModal = false;
		// 恢复焦点到触发元素
		if (lastFocusedElement) {
			setTimeout(() => lastFocusedElement?.focus(), 0);
		}
	}

	function closeVictoryModal() {
		showVictoryModal = false;
		// 恢复焦点到触发元素
		if (lastFocusedElement) {
			setTimeout(() => lastFocusedElement?.focus(), 0);
		}
	}

	function selectDifficulty(level: number) {
		initGame(level);
	}

	async function restoreGame() {
		try {
			isLoading = true;
			error = null;
			hasShownVictory = false; // 重置胜利弹窗标志
			gameState = await invoke<GameState>('load_game');
			selectedCard = null;
			showRestorePrompt = false;
		} catch (e) {
			error = `恢复失败: ${e}`;
			console.error('Failed to load game:', e);
			initGame(1);
		} finally {
			isLoading = false;
		}
	}

	async function handleCardClick(colIndex: number, cardIndex: number) {
		if (!gameState || isLoading || dragState?.isDragging) return;

		const column = gameState.columns[colIndex];
		const card = column[cardIndex];





		if (!card.face_up) return;

		if (selectedCard === null) {
			const cardsFromIndex = column.slice(cardIndex);
			let isValidSequence = true;
			for (let i = 0; i < cardsFromIndex.length - 1; i++) {
				if (cardsFromIndex[i].suit !== cardsFromIndex[i + 1].suit ||
					cardsFromIndex[i].value !== cardsFromIndex[i + 1].value + 1) {
					isValidSequence = false;
					break;
				}
			}

			if (isValidSequence) {
				selectedCard = { colIndex, cardIndex };
				soundManager.play('click');
			} else {
				soundManager.play('error');
				shakeColumn = colIndex;
				setTimeout(() => { shakeColumn = null; }, 500);
			}
		} else if (selectedCard.colIndex === colIndex && selectedCard.cardIndex === cardIndex) {
			selectedCard = null;
		} else {

			await executeMove(selectedCard.colIndex, selectedCard.cardIndex, colIndex);
			selectedCard = null;
		}
	}

	// 执行移动操作
	async function executeMove(fromCol: number, startIdx: number, toCol: number) {
		if (!gameState) return;

		addDebugLog(`move: ${fromCol}[${startIdx}] -> ${toCol}`);
		const movingCardsCount = gameState.columns[fromCol].length - startIdx;
		const previousCompleted = gameState.completed;

		try {
			const result = await invoke<GameState>('move_cards', {
				fromCol: fromCol,
				startIdx: startIdx,
				toCol: toCol
			});

			addDebugLog(`move_cards returned: moves=${result.moves}`);

			gameState = result;

			if (movingCardsCount === 1) {
				soundManager.play('move');
			} else {
				soundManager.play('slide');
			}

			if (result.completed > previousCompleted) {
				soundManager.play('complete');
			}

			if (result.completed === 8) {
				soundManager.play('win');
			}

			// 更新撤销/重做状态
			canUndoState = await invoke<boolean>('can_undo');
			canRedoState = await invoke<boolean>('can_redo');
			addDebugLog(`after move: canUndo=${canUndoState}, canRedo=${canRedoState}`);
		} catch (e) {
			addDebugLog(`move_cards ERROR: ${e}`);
			soundManager.play('error');
			shakeColumn = toCol;
			setTimeout(() => { shakeColumn = null; }, 500);
		}
	}

	// 拖拽开始
	function handleDragStart(colIndex: number, cardIndex: number, event: MouseEvent) {

		if (!gameState || isLoading) {

			return;
		}

		// 清理可能残留的旧拖拽元素
		cleanupDragElements();

		const column = gameState.columns[colIndex];
		const cards = column.slice(cardIndex);


		// 计算鼠标在卡牌上的偏移
		const target = event.target as HTMLElement;
		const rect = target.getBoundingClientRect();
		const offsetX = event.clientX - rect.left;
		const offsetY = event.clientY - rect.top;

		// 创建拖拽视觉元素
		const dragElement = createDragElement(cards);

		dragState = {
			isDragging: true,
			fromCol: colIndex,
			startCardIndex: cardIndex,
			cards,
			element: dragElement,
			offsetX,
			offsetY
		};

		selectedCard = null;
		soundManager.play('click');

		// 添加全局鼠标事件
		document.addEventListener('mousemove', handleDragMove);
		document.addEventListener('mouseup', handleDragEnd);
	}

	// 创建拖拽视觉元素
	function createDragElement(cards: Card[]): HTMLElement {
		const container = document.createElement('div');
		container.className = 'drag-ghost';
		container.id = 'drag-ghost-active'; // 添加 ID 便于调试
		// 简化样式，不使用 filter
		container.style.cssText = `
			position: fixed;
			pointer-events: none;
			z-index: 10000;
			width: 85px;
		`;

		cards.forEach((card, index) => {
			if (!card.face_up) return;

			const cardEl = document.createElement('div');
			cardEl.className = 'drag-card';
			cardEl.style.cssText = `
				position: absolute;
				top: ${index * 22}px;
				left: 0;
				width: 85px;
				height: 124px;
				background: white;
				border-radius: 8px;
				overflow: hidden;
				box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
			`;

			// 渲染卡牌SVG
			const suitMap: Record<string, string> = {
				spade: 'S',
				heart: 'H',
				diamond: 'D',
				club: 'C'
			};

			const getValueChar = (value: number): string => {
				switch (value) {
					case 1: return 'A';
					case 11: return 'J';
					case 12: return 'Q';
					case 13: return 'K';
					default: return String(value);
				}
			};

			const cardFileName = `${getValueChar(card.value)}${suitMap[card.suit]}`;
			const cardUrl = `/cards/${cardFileName}.svg`;

			// 异步加载SVG
			fetch(cardUrl)
				.then(res => res.text())
				.then(svg => {
					cardEl.innerHTML = svg;
					const svgEl = cardEl.querySelector('svg');
					if (svgEl) {
						svgEl.style.width = '100%';
						svgEl.style.height = '100%';
						svgEl.style.display = 'block';
					}
				});

			container.appendChild(cardEl);
		});

		container.style.height = `${(cards.length - 1) * 22 + 124}px`;

		document.body.appendChild(container);
		return container;
	}

	// 拖拽移动 - 使用 DOM 操作直接管理高亮
	function handleDragMove(event: MouseEvent) {
		const state = dragState;
		if (!state?.isDragging || !state.element) return;

		// 更新拖拽元素位置
		state.element.style.left = `${event.clientX - state.offsetX}px`;
		state.element.style.top = `${event.clientY - state.offsetY}px`;

		// 检测目标列
		const targetCol = getColumnAtPosition(event.clientX, event.clientY);

		// 直接使用 DOM 操作更新高亮（绕过 Svelte 响应式）
		const columnsContainer = document.querySelector('.columns');
		if (columnsContainer) {
			const colElements = columnsContainer.querySelectorAll('.column');
			colElements.forEach((col, index) => {
				col.classList.remove('drop-target-valid', 'drop-target-invalid');
				if (index === targetCol && targetCol >= 0) {
					const isValid = isValidDrop(state.cards, targetCol);
					col.classList.add(isValid ? 'drop-target-valid' : 'drop-target-invalid');
				}
			});
		}

		// 同时更新响应式状态
		dropTargetCol = targetCol;
		dropValid = targetCol >= 0 && isValidDrop(state.cards, targetCol);
	}

	// 获取鼠标位置下方的列
	function getColumnAtPosition(x: number, y: number): number {
		// 使用 DOM 查询获取所有列元素
		const columnsContainer = document.querySelector('.columns');
		if (!columnsContainer) return -1;

		const colElements = columnsContainer.querySelectorAll('.column');
		for (let i = 0; i < colElements.length; i++) {
			const col = colElements[i];
			const rect = col.getBoundingClientRect();
			if (x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom) {
				return i;
			}
		}
		return -1;
	}

	// 检查是否可以放置
	function isValidDrop(dragCards: Card[], toCol: number): boolean {
		if (!gameState) return false;
		if (toCol === dragState?.fromCol) return false;

		const targetColumn = gameState.columns[toCol];
		if (targetColumn.length === 0) {
			return true; // 空列可以放任何牌
		}

		const topCard = targetColumn[targetColumn.length - 1];
		const firstDragCard = dragCards[0];

		// 目标牌必须比拖拽的第一张牌大1
		return topCard.value === firstDragCard.value + 1;
	}

	// 清理所有拖拽残留元素
	function cleanupDragElements() {
		// 使用多种选择器确保找到所有可能的残留元素
		const selectors = ['.drag-ghost', '#drag-ghost-active', '.drag-card'];
		let totalRemoved = 0;
		selectors.forEach(selector => {
			document.querySelectorAll(selector).forEach(el => {
				el.remove();
				totalRemoved++;
			});
		});


		// 额外检查 body 直接子元素中的残留
		document.body.querySelectorAll(':scope > .drag-ghost, :scope > #drag-ghost-active').forEach(el => {
			el.remove();

		});

		// 直接清除所有列的高亮样式（绕过 Svelte 响应式）
		document.querySelectorAll('.column.drop-target-valid, .column.drop-target-invalid').forEach(el => {
			el.classList.remove('drop-target-valid', 'drop-target-invalid');
		});
	}

	// 拖拽结束
	async function handleDragEnd(event: MouseEvent) {

		document.removeEventListener('mousemove', handleDragMove);
		document.removeEventListener('mouseup', handleDragEnd);

		// 保存当前状态用于执行移动
		const fromCol = dragState?.fromCol ?? -1;
		const startCardIndex = dragState?.startCardIndex ?? -1;
		const targetCol = dropTargetCol;
		const valid = dropValid;



		// 强制清理所有拖拽元素（确保不残留）
		cleanupDragElements();

		// 重置拖拽状态 - 立即重置，确保 UI 更新
		dragState = null;
		dropTargetCol = null;
		dropValid = true;

		// 强制同步更新视图，确保高亮立即消失
		flushSync();



		// 执行移动
		if (targetCol !== null && targetCol >= 0 && valid && fromCol >= 0) {

			await executeMove(fromCol, startCardIndex, targetCol);
		} else if (targetCol !== null && targetCol >= 0 && !valid) {

			soundManager.play('error');
			shakeColumn = targetCol;
			setTimeout(() => { shakeColumn = null; }, 500);
		} else {

		}

		// 再次清理确保没有残留
		cleanupDragElements();

		// 再次确保 dropTargetCol 是 null
		dropTargetCol = null;


		// 再次清理确保没有残留
		cleanupDragElements();
	}

	async function handleDeal() {
		if (!gameState || isLoading) return;

		if (gameState.stock.length < 10) {
			error = '没有足够的牌可发';
			soundManager.play('error');
			setTimeout(() => { error = null; }, 2000);
			return;
		}

		if (gameState.columns.some(col => col.length === 0)) {
			error = '发牌前，所有列都必须有牌';
			soundManager.play('error');
			shakeColumn = -1;
			setTimeout(() => { shakeColumn = null; }, 500);
			setTimeout(() => { error = null; }, 2000);
			return;
		}

		try {
			const result = await invoke<GameState>('deal_cards');
			if (result) {
				gameState = result;
				soundManager.play('deal');
				// 更新撤销/重做状态
				canUndoState = await invoke<boolean>('can_undo');
				canRedoState = await invoke<boolean>('can_redo');
			}
		} catch (e) {
			error = `发牌失败: ${e}`;
			console.error('Failed to deal:', e);
			setTimeout(() => { error = null; }, 2000);
		}
	}

	function toggleMute() {
		isMuted = soundManager.toggleMute();
	}

	// 提示功能
	let hintCards = $state<{ fromCol: number; startIdx: number; toCol: number } | null>(null);
	let hintTimeout: ReturnType<typeof setTimeout> | null = null;

	async function handleHint() {
		if (!gameState || isLoading) return;

		// 清除之前的提示
		if (hintTimeout) {
			clearTimeout(hintTimeout);
			hintTimeout = null;
		}
		hintCards = null;

		try {
			// Rust 返回的是元组 [fromCol, startIdx, toCol]
			const hintArr = await invoke<[number, number, number] | null>('get_hint');

			if (hintArr && Array.isArray(hintArr) && hintArr.length === 3) {
				hintCards = { fromCol: hintArr[0], startIdx: hintArr[1], toCol: hintArr[2] };

				// 显示提示的详细信息
				const fromCol = gameState.columns[hintArr[0]];
				const card = fromCol[hintArr[1]];

				soundManager.play('click');

				// 3秒后自动清除提示
				hintTimeout = setTimeout(() => {
					hintCards = null;
				}, 3000);
			} else {
				// 没有可用的移动

				soundManager.play('error');
			}
		} catch (e) {
			console.error('Hint failed:', e);
		}
	}

	async function handleUndo() {
		if (!canUndoState || isLoading) return;
		try {
			gameState = await invoke<GameState>('undo');
			selectedCard = null;
			soundManager.play('flip');
			// 更新撤销/重做状态
			canUndoState = await invoke<boolean>('can_undo');
			canRedoState = await invoke<boolean>('can_redo');
		} catch (e) {
			console.error('Undo failed:', e);
		}
	}

	async function handleRedo() {
		if (!canRedoState || isLoading) return;
		try {
			gameState = await invoke<GameState>('redo');
			selectedCard = null;
			soundManager.play('flip');
			// 更新撤销/重做状态
			canUndoState = await invoke<boolean>('can_undo');
			canRedoState = await invoke<boolean>('can_redo');
		} catch (e) {
			console.error('Redo failed:', e);
		}
	}

	onMount(async () => {
		soundManager.preload();
		try {
			const hasSave = await invoke<boolean>('has_saved_game');
			if (hasSave) {
				showRestorePrompt = true;
			} else {
				initGame(1);
			}
		} catch (e) {
			console.error('Failed to check saved game:', e);
			initGame(1);
		}

		// 键盘快捷键
		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.ctrlKey && e.key === 'z') {
				e.preventDefault();
				handleUndo();
			} else if (e.ctrlKey && e.key === 'y') {
				e.preventDefault();
				handleRedo();
			}
		};
		document.addEventListener('keydown', handleKeyDown);

		// 清理函数
		return () => {
			document.removeEventListener('keydown', handleKeyDown);
			if (hintTimeout) {
				clearTimeout(hintTimeout);
			}
			// 清理拖拽相关
			document.removeEventListener('mousemove', handleDragMove);
			document.removeEventListener('mouseup', handleDragEnd);
			cleanupDragElements();
		};
	});

	let remainingDeals = $derived(gameState ? Math.floor(gameState.stock.length / 10) : 0);

	$effect(() => {
		if (gameState && !isLoading) {
			invoke<boolean>('can_undo').then((v) => {
				addDebugLog(`can_undo: ${v}`);
				canUndoState = v;
			}).catch((e) => addDebugLog(`can_undo error: ${e}`));
			invoke<boolean>('can_redo').then((v) => {
				addDebugLog(`can_redo: ${v}`);
				canRedoState = v;
			}).catch((e) => addDebugLog(`can_redo error: ${e}`));
		}
	});

	// Victory detection
	$effect(() => {
		if (gameState?.completed === 8 && !showVictoryModal && !hasShownVictory) {
			hasShownVictory = true;
			showVictoryModal = true;
			showConfetti = true;
			soundManager.play('win');
			// 5秒后停止动画，减少对光敏用户的影响
			setTimeout(() => {
				showConfetti = false;
			}, 5000);
		}
	});

	// 调试模式
	let debugMode = $state(false);
	let debugLogs = $state<string[]>([]);
	function addDebugLog(msg: string) {
		if (!debugMode) return;
		const time = new Date().toLocaleTimeString();
		debugLogs = [...debugLogs.slice(-9), `[${time}] ${msg}`];
	}
</script>

<div class="game-container" oncontextmenu={(e) => e.preventDefault()}>
	<!-- 调试面板已禁用
	{#if debugMode}
		<div class="debug-panel">
			<div class="debug-header">
				<span>🔧 调试模式</span>
				<button class="debug-close" onclick={() => debugMode = false}>×</button>
			</div>
			<div class="debug-content">
				<div class="debug-item">
					<span>isTauri:</span>
					<span class:debug-true={isTauri} class:debug-false={!isTauri}>{isTauri ? 'YES' : 'NO (mock)'}</span>
				</div>
				<div class="debug-item">
					<span>canUndo:</span>
					<span class:debug-true={canUndoState} class:debug-false={!canUndoState}>{canUndoState ? 'YES' : 'NO'}</span>
				</div>
				<div class="debug-item">
					<span>canRedo:</span>
					<span class:debug-true={canRedoState} class:debug-false={!canRedoState}>{canRedoState ? 'YES' : 'NO'}</span>
				</div>
				<div class="debug-item">
					<span>moves:</span>
					<span>{gameState?.moves ?? 0}</span>
				</div>
				<button class="debug-action" onclick={async () => {
					try {
						const info = await invoke<[number, number]>('debug_history');
						addDebugLog(`history: index=${info[0]}, len=${info[1]}`);
					} catch (e) {
						addDebugLog(`debug_history error: ${e}`);
					}
				}}>检查历史</button>
				<div class="debug-logs">
					{#each debugLogs as log}
						<div class="debug-log">{log}</div>
					{/each}
				</div>
			</div>
		</div>
	{/if}
	-->

	<!-- 顶部工具栏 -->
	<header class="toolbar">
		<div class="game-info">
			<span class="info-item">
				<span class="label">分数</span>
				<span class="value">{gameState?.score ?? 500}</span>
			</span>
			<span class="info-item">
				<span class="label">步数</span>
				<span class="value">{gameState?.moves ?? 0}</span>
			</span>
			<span class="info-item">
				<span class="label">完成</span>
				<span class="value">{gameState?.completed ?? 0}/8</span>
			</span>
		</div>
		<div class="actions">
			<button class="btn mute-btn" onclick={toggleMute} title={isMuted ? '开启音效' : '静音'}>
				{#if isMuted}
					<svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/></svg>
					<span>静音</span>
				{:else}
					<svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/></svg>
					<span>音效</span>
				{/if}
			</button>
			<!-- 撤销按钮 -->
			<button class="btn undo-btn" onclick={handleUndo} disabled={!canUndoState || isLoading} title={canUndoState ? '撤销 (Ctrl+Z)' : '没有可撤销的操作'}>
				<svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
					<path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"/>
				</svg>
				<span>撤销</span>
			</button>
			<!-- 重做按钮 -->
			<button class="btn redo-btn" onclick={handleRedo} disabled={!canRedoState || isLoading} title={canRedoState ? '重做 (Ctrl+Y)' : '没有可重做的操作'}>
				<svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
					<path d="M21 7v6h-6"/><path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3l3 2.7"/>
				</svg>
				<span>重做</span>
			</button>
			<!-- 提示按钮 -->
			<button class="btn hint-btn" onclick={handleHint} disabled={isLoading} title="显示提示">
				<svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
					<path d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5"/>
					<path d="M9 18h6"/>
					<path d="M10 22h4"/>
				</svg>
				<span>提示</span>
			</button>
			<!-- 新游戏按钮 -->
			<button class="btn primary" onclick={openNewGameModal}>
				<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
				新游戏
			</button>
			<!-- 调试按钮已禁用
			<button class="btn debug-btn" onclick={() => debugMode = !debugMode} title="切换调试模式">
				🔧
			</button>
			-->
		</div>
	</header>

	<!-- 主游戏区域 -->
	<main class="game-board">
		{#if showRestorePrompt}
			<!-- 📋 存档恢复提示 -->
			<div class="restore-overlay">
				<div class="restore-modal">
					<h2 class="restore-title">发现存档</h2>
					<p class="restore-text">检测到上次未完成的游戏，是否继续？</p>
					<div class="restore-buttons">
						<button class="btn primary" onclick={restoreGame}>
							继续游戏
						</button>
						<button class="btn" onclick={() => showDifficultyModal = true}>
							新游戏
						</button>
					</div>
				</div>
			</div>
		{/if}

		<!-- 难度选择模态框 -->
		{#if showDifficultyModal}
			<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
			<div class="modal-overlay" onclick={closeDifficultyModal} role="button" tabindex="-1" onkeydown={(e) => e.key === 'Escape' && closeDifficultyModal()}>
				<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
				<div class="modal difficulty-modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-labelledby="difficulty-title" tabindex="-1">
					<h2 class="modal-title" id="difficulty-title">选择难度</h2>
					<div class="difficulty-options">
						{#each difficultyOptions as option}
							<button
								class="difficulty-option"
								class:selected={gameState?.difficulty === option.level}
								onclick={() => selectDifficulty(option.level)}
							>
								<span class="difficulty-name">{option.name}</span>
								<span class="difficulty-suits">{option.suits}</span>
								<span class="difficulty-desc">{option.description}</span>
							</button>
						{/each}
					</div>
					<button class="btn cancel-btn" onclick={closeDifficultyModal}>
						取消
					</button>
				</div>
			</div>
		{/if}

		<!-- 胜利庆祝模态框 -->
		{#if showVictoryModal}
			<div class="victory-overlay">
				<div class="confetti-container" class:stopped={!showConfetti}>
					{#each Array(20) as _, i}
						<div class="confetti" style="--delay: {Math.random() * 3}s; --x: {Math.random() * 100}vw; --rotate: {Math.random() * 720 - 360}deg;"></div>
					{/each}
				</div>
				<div class="victory-modal">
					<h2 class="victory-title">🎉 恭喜通关！</h2>
					<div class="victory-stats">
						<div class="stat">
							<span class="stat-label">最终得分</span>
							<span class="stat-value">{gameState?.score}</span>
						</div>
						<div class="stat">
							<span class="stat-label">总步数</span>
							<span class="stat-value">{gameState?.moves}</span>
						</div>
					</div>
					<div class="victory-buttons">
						<button class="btn primary" onclick={openNewGameModal}>
							再来一局
						</button>
					</div>
				</div>
			</div>
		{:else if isLoading}
			<div class="loading">
				<div class="spinner"></div>
				<span>加载中...</span>
			</div>
		{:else if error}
			<div class="error">
				<span>{error}</span>
				<button class="btn" onclick={() => initGame(1)}>重试</button>
			</div>
		{:else if gameState}
			<!-- 10列卡牌 -->
            <div class="columns">
                {#each gameState.columns as column, index}
                    <Column
                        cards={column}
                        columnIndex={index}
                        selectedIndex={selectedCard?.colIndex === index ? selectedCard.cardIndex : null}
                        hintStartIndex={hintCards?.fromCol === index ? hintCards.startIdx : null}
                        isHintTarget={hintCards?.toCol === index}
                        onCardClick={(cardIndex) => handleCardClick(index, cardIndex)}
                        onDragStart={(colIdx, cardIdx, evt) => handleDragStart(colIdx, cardIdx, evt)}
                        shake={shakeColumn === index}
                        isDropTarget={dropTargetCol === index}
                        dropValid={dropValid}
                    />
                {/each}
            </div>

			<!-- 底部区域 - 回收堆在左，发牌堆在右 -->
			<div class="bottom-area">
				<!-- 回收堆 - 左下角 -->
				<div class="foundation-area">
					{#each Array(8) as _, i}
						<div class="foundation" class:filled={i < gameState.completed}>
							{#if i < gameState.completed}
								<div class="completed-stack">
									<div class="stack-card stack-3"></div>
									<div class="stack-card stack-2"></div>
									<div class="stack-card stack-1"></div>
									<div class="stack-card stack-top">
										<span class="card-value">K</span>
										<span class="card-suit">♠</span>
									</div>
								</div>
							{:else}
								<div class="foundation-placeholder"></div>
							{/if}
						</div>
					{/each}
				</div>

				<!-- 发牌堆 - 右下角 -->
				<div class="stock-area">
					<button
						class="stock-pile"
						disabled={gameState.stock.length === 0}
						onclick={handleDeal}
						aria-label="发牌，剩余 {remainingDeals} 次"
						type="button"
					>
						{#if remainingDeals > 0}
							{#each Array(Math.min(remainingDeals, 5)) as _, i}
								<div class="stock-card" style="left: {i * 3}px; top: {i * 2}px; z-index: {i};"></div>
							{/each}
						{/if}
					</button>
					<span class="stock-label">剩余发牌: {remainingDeals} 次</span>
				</div>
			</div>
		{/if}
	</main>
</div>

<style>
	/* 调试面板 */
	.debug-panel {
		position: fixed;
		top: 60px;
		right: 10px;
		width: 280px;
		background: rgba(0, 0, 0, 0.9);
		border: 2px solid #4caf50;
		border-radius: 8px;
		z-index: 10000;
		font-family: monospace;
		font-size: 12px;
		user-select: text !important;
		-webkit-user-select: text !important;
		pointer-events: auto;
	}
	.debug-panel * {
		user-select: text !important;
		-webkit-user-select: text !important;
	}
	.debug-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 12px;
		background: #4caf50;
		color: white;
		font-weight: bold;
	}
	.debug-close {
		background: none;
		border: none;
		color: white;
		font-size: 18px;
		cursor: pointer;
		padding: 0 4px;
	}
	.debug-content {
		padding: 10px;
		color: #fff;
	}
	.debug-item {
		display: flex;
		justify-content: space-between;
		padding: 4px 0;
		border-bottom: 1px solid #333;
	}
	.debug-true {
		color: #4caf50;
		font-weight: bold;
	}
	.debug-false {
		color: #f44336;
	}
	.debug-logs {
		margin-top: 10px;
		max-height: 150px;
		overflow-y: auto;
		border-top: 1px solid #333;
		padding-top: 8px;
	}
	.debug-log {
		color: #aaa;
		font-size: 11px;
		padding: 2px 0;
	}
	.debug-btn {
		background: #333 !important;
		padding: 8px 12px !important;
		min-width: auto !important;
	}
	.debug-action {
		width: 100%;
		margin: 8px 0;
		padding: 6px;
		background: #4caf50;
		border: none;
		border-radius: 4px;
		color: white;
		cursor: pointer;
		font-size: 12px;
	}
	.debug-action:hover {
		background: #45a049;
	}

	.game-container {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: linear-gradient(135deg, #1a472a 0%, #2d5a3f 50%, #1a472a 100%);
		color: white;
		overflow: hidden;
		user-select: none;
		-webkit-user-select: none;
	}

	.toolbar {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 16px;
		background: rgba(0, 0, 0, 0.35);
		backdrop-filter: blur(12px);
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
		flex-shrink: 0;
		gap: 12px;
	}

	.game-info {
		display: flex;
		gap: 24px;
	}

	.info-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1px;
	}

	.label {
		font-size: 14px;
		color: rgba(255, 255, 255, 0.8);
		text-transform: uppercase;
		letter-spacing: 1px;
		font-weight: 600;
	}

	.value {
		font-size: 28px;
		font-weight: 700;
		text-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
	}

	.actions {
		display: flex;
		gap: 8px;
	}

	.btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 12px 24px;
		border: 2px solid transparent;
		border-radius: 10px;
		background: rgba(255, 255, 255, 0.2);
		color: white;
		font-size: 18px;
		font-weight: 700;
		cursor: pointer;
		transition: all 0.2s ease-out;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
		min-width: 64px;
		min-height: 64px;
	}

	.btn:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.32);
		transform: translateY(-3px);
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
		border-color: rgba(255, 255, 255, 0.4);
	}

	.btn:focus {
		outline: none;
		border-color: rgba(255, 255, 255, 0.8);
		box-shadow: 0 0 0 4px rgba(255, 255, 255, 0.3), 0 4px 12px rgba(0, 0, 0, 0.2);
	}

	.btn:active:not(:disabled) {
		transform: translateY(0);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
	}

	.btn.primary {
		background: linear-gradient(135deg, #4caf50 0%, #2e7d32 100%);
		box-shadow: 0 3px 10px rgba(76, 175, 80, 0.4);
	}

	.btn.primary:hover:not(:disabled) {
		background: linear-gradient(135deg, #66bb6a 0%, #388e3c 100%);
		box-shadow: 0 5px 16px rgba(76, 175, 80, 0.5);
	}

	.btn:disabled {
		background: rgba(255, 255, 255, 0.1);
		color: rgba(255, 255, 255, 0.35);
		cursor: not-allowed;
		transform: none;
		box-shadow: none;
		opacity: 0.7;
	}

	.mute-btn, .undo-btn, .redo-btn, .hint-btn {
		padding: 12px;
		min-width: 48px;
		min-height: 48px;
		justify-content: center;
	}

	.mute-btn svg, .undo-btn svg, .redo-btn svg, .hint-btn svg {
		width: 24px;
		height: 24px;
	}

	.game-board {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 20px 8px 8px;
		overflow: hidden;
		min-height: 0;
	}

	.columns {
		display: flex;
		justify-content: center;
		gap: 4px;
		flex-wrap: nowrap;
		flex: 1;
		min-height: 0;
	}

	.bottom-area {
		display: flex;
		justify-content: flex-start;
		align-items: flex-end;
		padding-top: 16px;
		flex-shrink: 0;
		padding-left: 8px;
		padding-right: 8px;
		gap: 16px;
	}

	.foundation-area {
		display: flex;
		justify-content: flex-start;
		gap: 6px;
		flex-wrap: wrap;
	}

	.foundation {
		width: 65px;
		height: 90px;
		border-radius: 8px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
		position: relative;
		overflow: hidden;
	}

	/* 空位样式 - Win7 经典风格 */
	.foundation:not(.filled) {
		background: linear-gradient(180deg, #1a3a1a 0%, #0d1f0d 100%);
		border: 3px solid rgba(255, 255, 255, 0.25);
		box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.4);
	}

	.foundation:not(.filled)::after {
		content: 'K  A';
		font-size: 20px;
		font-weight: 700;
		color: rgba(255, 255, 255, 0.5);
		letter-spacing: 4px;
	}

	.foundation-placeholder {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1px;
		opacity: 0.6;
	}

	/* 完成样式 - 真实的牌堆效果 */
	.foundation.filled {
		background: linear-gradient(145deg, #ffffff 0%, #f5f5f5 100%);
		border: none;
		box-shadow:
			0 2px 4px rgba(0, 0, 0, 0.2),
			0 4px 8px rgba(0, 0, 0, 0.15),
			inset 0 1px 0 rgba(255, 255, 255, 0.8);
		animation: complete-pop 0.5s cubic-bezier(0.34, 1.56, 0.64, 1);
	}

	@keyframes complete-pop {
		0% {
			transform: scale(0.8);
			opacity: 0;
		}
		50% {
			transform: scale(1.1);
		}
		100% {
			transform: scale(1);
			opacity: 1;
		}
	}

	.foundation.filled::before {
		content: '';
		position: absolute;
		top: 3px;
		left: 3px;
		right: 3px;
		bottom: 3px;
		background: linear-gradient(145deg, #f8f8f8 0%, #e8e8e8 100%);
		border-radius: 4px;
		box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.1);
	}

	/* 完成的牌堆 */
	.completed-stack {
		position: relative;
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1;
	}

	.stack-card {
		position: absolute;
		width: 38px;
		height: 52px;
		background: linear-gradient(145deg, #ffffff 0%, #f0f0f0 100%);
		border-radius: 4px;
		box-shadow: 0 1px 2px rgba(0, 0, 0, 0.15);
	}

	.stack-3 {
		transform: translateY(4px);
		opacity: 0.5;
	}

	.stack-2 {
		transform: translateY(2px);
		opacity: 0.7;
	}

	.stack-1 {
		transform: translateY(0);
		opacity: 0.9;
	}

	.stack-top {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		transform: translateY(-2px);
		z-index: 2;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
	}

	.card-value {
		font-size: 18px;
		font-weight: bold;
		color: #1a1a2e;
		line-height: 1;
	}

	.card-suit {
		font-size: 16px;
		color: #1a1a2e;
		line-height: 1;
	}

	.stock-area {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding-bottom: 8px;
		flex-shrink: 0;
		margin-left: auto;
	}

	.stock-pile {
		position: relative;
		width: 85px;
		height: 124px;
		border: none;
		border-radius: 10px;
		background: transparent;
		cursor: pointer;
		padding: 0;
		transition: transform 0.25s ease-out;
	}

	.stock-pile:hover:not(:disabled) {
		transform: scale(1.08);
	}

	.stock-pile:hover:not(:disabled) .stock-card {
		box-shadow:
			inset 0 1px 0 rgba(255, 255, 255, 0.3),
			4px 4px 12px rgba(0, 0, 0, 0.5);
	}

	.stock-pile:disabled {
		cursor: not-allowed;
		opacity: 0.4;
	}

	/* 发牌堆 - 精美扑克牌背 */
	.stock-card {
		position: absolute;
		width: 85px;
		height: 124px;
		border-radius: 10px;
		background: linear-gradient(145deg, #1a5c1a 0%, #2e7d32 25%, #1a5c1a 50%, #2e7d32 75%, #1a5c1a 100%);
		border: 3px solid #0d3d0d;
		box-shadow:
			inset 0 1px 0 rgba(255, 255, 255, 0.15),
			inset 0 -1px 0 rgba(0, 0, 0, 0.2),
			2px 2px 6px rgba(0, 0, 0, 0.4);
		overflow: hidden;
	}

	/* 扑克牌背图案 - 菱形纹理 */
	.stock-card::before {
		content: '';
		position: absolute;
		inset: 8px;
		background-image:
			repeating-linear-gradient(
				45deg,
				transparent,
				transparent 4px,
				rgba(255, 255, 255, 0.03) 4px,
				rgba(255, 255, 255, 0.03) 8px
			),
			repeating-linear-gradient(
				-45deg,
				transparent,
				transparent 4px,
				rgba(255, 255, 255, 0.03) 4px,
				rgba(255, 255, 255, 0.03) 8px
			);
		border-radius: 4px;
	}

	/* 中心花纹 */
	.stock-card::after {
		content: '';
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: 40px;
		height: 40px;
		background: rgba(255, 255, 255, 0.08);
		border-radius: 50%;
		box-shadow:
			0 0 0 3px rgba(255, 255, 255, 0.05),
			0 0 0 6px rgba(255, 255, 255, 0.03);
	}

	.stock-label {
		font-size: 18px;
		color: rgba(255, 255, 255, 0.9);
		font-weight: 500;
		white-space: nowrap;
	}

	.loading, .error {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: 24px;
	}

	.loading span {
		font-size: 26px;
		font-weight: 500;
		color: rgba(255, 255, 255, 0.9);
	}

	.spinner {
		width: 64px;
		height: 64px;
		border: 5px solid rgba(255, 255, 255, 0.25);
		border-top-color: white;
		border-radius: 50%;
		animation: spin 1s linear infinite;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.error {
		color: #ff6b6b;
		background: rgba(0, 0, 0, 0.6);
		padding: 28px 48px;
		border-radius: 20px;
		backdrop-filter: blur(12px);
		border: 2px solid rgba(255, 107, 107, 0.4);
		box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
		max-width: 80%;
		text-align: center;
	}

	.error span {
		font-size: 30px;
		font-weight: 600;
		text-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);
		line-height: 1.4;
	}

	.error .btn {
		margin-top: 20px;
		background: rgba(255, 107, 107, 0.25);
		border: 2px solid rgba(255, 107, 107, 0.6);
		padding: 16px 32px;
		font-size: 22px;
	}

	.error .btn:hover {
		background: rgba(255, 107, 107, 0.35);
		transform: translateY(-2px);
	}

	/* 存档恢复提示框 */
	.restore-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.75);
		display: flex;
		justify-content: center;
		align-items: center;
		z-index: 1000;
		backdrop-filter: blur(8px);
	}

	.restore-modal {
		background: linear-gradient(135deg, #2d5a3f 0%, #1a472a 100%);
		border-radius: 24px;
		padding: 48px 56px;
		text-align: center;
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.5),
			inset 0 1px 0 rgba(255, 255, 255, 0.1);
		max-width: 460px;
		animation: modal-appear 0.35s cubic-bezier(0.34, 1.56, 0.64, 1);
		border: 1px solid rgba(255, 255, 255, 0.1);
	}

	@keyframes modal-appear {
		from {
			opacity: 0;
			transform: scale(0.85) translateY(20px);
		}
		to {
			opacity: 1;
			transform: scale(1) translateY(0);
		}
	}

	.restore-title {
		font-size: 40px;
		font-weight: 700;
		color: white;
		margin-bottom: 20px;
		text-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
	}

	.restore-text {
		font-size: 24px;
		color: rgba(255, 255, 255, 0.9);
		margin-bottom: 36px;
		line-height: 1.5;
	}

	.restore-buttons {
		display: flex;
		gap: 20px;
		justify-content: center;
	}

	/* 难度选择模态框 */
	.modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.75);
		display: flex;
		justify-content: center;
		align-items: center;
		z-index: 1000;
		backdrop-filter: blur(8px);
	}

	.modal {
		background: linear-gradient(135deg, #2d5a3f 0%, #1a472a 100%);
		border-radius: 20px;
		padding: 32px 40px;
		text-align: center;
		box-shadow:
			0 20px 60px rgba(0, 0, 0, 0.5),
			inset 0 1px 0 rgba(255, 255, 255, 0.1);
		animation: modal-appear 0.35s cubic-bezier(0.34, 1.56, 0.64, 1);
		border: 1px solid rgba(255, 255, 255, 0.1);
		min-width: 320px;
		max-width: 90vw;
	}

	.modal-title {
		font-size: 28px;
		font-weight: 700;
		color: white;
		margin-bottom: 24px;
		text-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
	}

	.difficulty-options {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-bottom: 20px;
	}

	.difficulty-option {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 16px 32px;
		border: 2px solid rgba(255, 255, 255, 0.25);
		border-radius: 14px;
		background: rgba(255, 255, 255, 0.08);
		color: white;
		cursor: pointer;
		transition: all 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
		position: relative;
	}

	.difficulty-option:hover {
		background: rgba(255, 255, 255, 0.18);
		border-color: rgba(255, 255, 255, 0.5);
		transform: translateY(-3px) scale(1.02);
		box-shadow: 0 10px 24px rgba(0, 0, 0, 0.35);
	}

	.difficulty-option.selected {
		border-color: #4caf50;
		background: rgba(76, 175, 80, 0.25);
		box-shadow: 0 0 16px rgba(76, 175, 80, 0.3);
		transform: scale(1.02);
	}

	.difficulty-option.selected::after {
		content: '✓';
		position: absolute;
		top: 6px;
		right: 10px;
		font-size: 16px;
		color: #4caf50;
	}

	.difficulty-name {
		font-size: 22px;
		font-weight: 700;
	}

	.difficulty-suits {
		font-size: 16px;
		color: rgba(255, 255, 255, 0.85);
		font-weight: 500;
	}

	.difficulty-desc {
		font-size: 14px;
		color: rgba(255, 255, 255, 0.65);
	}

	.cancel-btn {
		background: rgba(255, 255, 255, 0.15);
		padding: 12px 28px;
		font-size: 16px;
		border-radius: 10px;
		margin-top: 4px;
	}

	.cancel-btn:hover {
		background: rgba(255, 255, 255, 0.25);
	}

	/* 胜利庆祝 */
	.victory-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.8);
		display: flex;
		justify-content: center;
		align-items: center;
		z-index: 2000;
	}

	.confetti-container {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
		overflow: hidden;
		transition: opacity 1s ease-out;
	}

	.confetti-container.stopped {
		opacity: 0;
	}

	.confetti {
		position: absolute;
		width: 12px;
		height: 12px;
		background: var(--color, #f0f);
		left: var(--x);
		top: -20px;
		animation: confetti-fall 3s ease-in-out var(--delay) infinite;
		transform: rotate(var(--rotate));
		will-change: transform, opacity;
	}

	.confetti:nth-child(4n+1) { --color: #ff6b6b; }
	.confetti:nth-child(4n+2) { --color: #4ecdc4; }
	.confetti:nth-child(4n+3) { --color: #ffe66d; }
	.confetti:nth-child(4n+4) { --color: #95e1d3; }

	@keyframes confetti-fall {
		0% {
			transform: translateY(0) rotate(0deg);
			opacity: 1;
		}
		100% {
			transform: translateY(100vh) rotate(720deg);
			opacity: 0;
		}
	}

	.victory-modal {
		background: linear-gradient(135deg, #2d5a3f 0%, #1a472a 100%);
		border-radius: 28px;
		padding: 52px 72px;
		text-align: center;
		box-shadow:
			0 32px 100px rgba(0, 0, 0, 0.6),
			inset 0 1px 0 rgba(255, 255, 255, 0.1);
		animation: victory-appear 0.6s cubic-bezier(0.34, 1.56, 0.64, 1);
		z-index: 2001;
		border: 1px solid rgba(255, 255, 255, 0.1);
	}

	@keyframes victory-appear {
		0% {
			opacity: 0;
			transform: scale(0.6) rotate(-8deg) translateY(30px);
		}
		60% {
			transform: scale(1.05) rotate(1deg) translateY(-5px);
		}
		100% {
			opacity: 1;
			transform: scale(1) rotate(0deg) translateY(0);
		}
	}

	.victory-title {
		font-size: 52px;
		font-weight: 800;
		color: #ffe66d;
		margin-bottom: 36px;
		text-shadow:
			0 4px 20px rgba(255, 230, 109, 0.5),
			0 0 60px rgba(255, 230, 109, 0.3);
		animation: pulse 1.2s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { transform: scale(1); }
		50% { transform: scale(1.05); }
	}

	.victory-stats {
		display: flex;
		justify-content: center;
		gap: 56px;
		margin-bottom: 44px;
	}

	.stat {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 10px;
		padding: 16px 32px;
		background: rgba(0, 0, 0, 0.2);
		border-radius: 16px;
		border: 1px solid rgba(255, 255, 255, 0.1);
	}

	.stat-label {
		font-size: 22px;
		color: rgba(255, 255, 255, 0.75);
		font-weight: 500;
	}

	.stat-value {
		font-size: 52px;
		font-weight: 800;
		color: #4ecdc4;
		text-shadow: 0 2px 8px rgba(78, 205, 196, 0.4);
	}

	.victory-buttons {
		display: flex;
		justify-content: center;
		gap: 16px;
	}

	/* 拖拽视觉元素 - 动态创建的元素需要 :global */
	:global(.drag-ghost) {
		position: fixed;
		pointer-events: none;
		z-index: 10000;
		opacity: 0.95;
	}

	/* 响应式优化 - 适配不同屏幕尺寸 */
	@media (max-width: 1440px) {
		.foundation-area {
			gap: 4px;
		}

		.foundation {
			width: 55px;
			height: 76px;
		}

		.foundation:not(.filled)::after {
			font-size: 13px;
			letter-spacing: 2px;
		}

		.stock-pile {
			width: 72px;
			height: 105px;
		}

		.stock-card {
			width: 72px;
			height: 105px;
		}

		.stock-label {
			font-size: 13px;
		}
	}

	@media (max-width: 1366px) {
		.foundation-area {
			gap: 4px;
		}

		.foundation {
			width: 52px;
			height: 72px;
		}

		.foundation:not(.filled)::after {
			font-size: 12px;
			letter-spacing: 1px;
		}

		.stock-pile {
			width: 68px;
			height: 100px;
		}

		.stock-card {
			width: 68px;
			height: 100px;
		}

		.stock-label {
			font-size: 12px;
		}
	}

	@media (max-width: 1200px) {
		.columns {
			gap: 2px;
		}

		.foundation-area {
			gap: 3px;
		}

		.foundation {
			width: 52px;
			height: 72px;
		}

		.foundation:not(.filled)::after {
			font-size: 12px;
			letter-spacing: 1px;
		}

		.stock-pile {
			width: 65px;
			height: 95px;
		}

		.stock-card {
			width: 65px;
			height: 95px;
		}

		.stock-label {
			font-size: 12px;
		}
	}

	@media (max-width: 1024px) {
		.toolbar {
			padding: 10px 16px;
		}

		.btn {
			padding: 12px 20px;
			font-size: 18px;
		}

		.mute-btn, .undo-btn, .redo-btn, .hint-btn {
			min-width: 48px;
			min-height: 48px;
			padding: 12px;
		}

		.columns {
			gap: 1px;
			padding: 0 4px;
		}
	}

	@media (max-width: 900px) {
		.foundation-area {
			gap: 2px;
		}

		.foundation {
			width: 48px;
			height: 66px;
		}

		.foundation:not(.filled)::after {
			font-size: 10px;
			letter-spacing: 1px;
		}

		.stock-pile {
			width: 62px;
			height: 90px;
		}

		.stock-card {
			width: 62px;
			height: 90px;
		}

		.stock-label {
			font-size: 11px;
		}

		.info-item {
			padding: 6px 10px;
		}

		.info-item .label {
			font-size: 11px;
		}

		.info-item .value {
			font-size: 18px;
		}
	}

	@media (max-width: 768px) {
		.toolbar {
			flex-direction: column;
			gap: 12px;
			padding: 12px;
		}

		.actions {
			gap: 8px;
		}

		.btn {
			padding: 10px 16px;
			font-size: 16px;
		}

		.mute-btn, .undo-btn, .redo-btn, .hint-btn {
			min-width: 44px;
			min-height: 44px;
			padding: 10px;
		}

		.columns {
			gap: 0px;
			transform: scale(0.95);
			transform-origin: top center;
		}

		.bottom-area {
			flex-direction: column;
			align-items: center;
			gap: 16px;
		}

		.stock-area {
			order: -1;
		}

		.foundation-area {
			gap: 2px;
		}

		.foundation {
			width: 45px;
			height: 62px;
		}

		.foundation:not(.filled)::after {
			font-size: 9px;
			letter-spacing: 0;
		}

		.stock-pile {
			width: 58px;
			height: 85px;
		}

		.stock-card {
			width: 58px;
			height: 85px;
		}

		.stock-label {
			font-size: 10px;
		}
	}

	/* 减少动画偏好 - 对光敏用户友好 */
	@media (prefers-reduced-motion: reduce) {
		*, *::before, *::after {
			animation-duration: 0.01ms !important;
			animation-iteration-count: 1 !important;
			transition-duration: 0.01ms !important;
		}

		.confetti {
			display: none;
		}

		.btn {
			transition: none;
		}

		.column {
			transition: none;
		}

		.card {
			transition: none;
		}
	}
</style>
