import type { GameState } from '$lib/types/game';

// Mock Tauri invoke for browser testing
export async function invokeMock<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
	console.log(`[Mock] invoke('${cmd}')`, args);

	switch (cmd) {
		case 'new_game': {
			const difficulty = (args?.difficulty as number) || 1;
			const state = createMockGameState(difficulty);
			console.log(`[Mock] returning state with ${state.columns.length} columns`);
			return Promise.resolve(state as T);
		}
		case 'get_state': {
			return Promise.resolve(null as T);
		}
		case 'move_cards': {
			// TODO: implement move logic
			return Promise.resolve(null as T);
		}
		case 'deal_cards': {
			// TODO: implement deal logic
			return Promise.resolve(null as T);
		}
		case 'get_hint': {
			return Promise.resolve(null as T);
		}
		default:
			return Promise.reject(`Unknown command: ${cmd}`);
	}
}

function createMockGameState(difficulty: number): GameState {
	const suits = difficulty === 1 ? ['spade'] as const : difficulty === 2 ? ['spade', 'heart'] as const : ['spade', 'heart', 'diamond', 'club'] as const;

	// Create 104 cards (2 decks)
	const allCards: Array<{ id: number; suit: string; value: number; face_up: boolean }> = [];
	let cardId = 0;
	for (const suit of suits) {
		for (let value = 1; value <= 13; value++) {
			// Add 8/number_of_suits copies of each card to make 104 total
			const copies = 8 / suits.length;
			for (let i = 0; i < copies; i++) {
				allCards.push({ id: cardId++, suit, value, face_up: false });
			}
		}
	}

	// Shuffle
	for (let i = allCards.length - 1; i > 0; i--) {
		const j = Math.floor(Math.random() * (i + 1));
		[allCards[i], allCards[j]] = [allCards[j], allCards[i]];
	}

	// Deal to 10 columns (54 cards)
	const columns: Array<Array<{ id: number; suit: string; value: number; face_up: boolean }>> = [];
	for (let i = 0; i < 10; i++) {
		const count = i < 4 ? 6 : 5;
		const column: Array<{ id: number; suit: string; value: number; face_up: boolean }> = [];
		for (let j = 0; j < count; j++) {
			const card = allCards.pop()!;
			column.push({
				...card,
				face_up: j === count - 1 // Only last card face up
			});
		}
		columns.push(column);
	}

	// Remaining 50 cards go to stock
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
