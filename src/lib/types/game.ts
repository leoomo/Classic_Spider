export type Suit = 'spade' | 'heart' | 'diamond' | 'club';

export interface Card {
	id: number;
	suit: Suit;
	value: number; // 1-13
	face_up: boolean;
}

export interface GameState {
	columns: Card[][];
	stock: Card[];
	completed: number;
	score: number;
	moves: number;
	difficulty: number;
	game_over: boolean;
	won: boolean;
}
