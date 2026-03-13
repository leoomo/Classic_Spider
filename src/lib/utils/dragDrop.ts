// 拖拽系统 - 蜘蛛纸牌
// 提供流畅的拖拽移动体验

export interface DragState {
	isDragging: boolean;
	fromCol: number;
	startCardIndex: number;
	cards: Array<{ suit: string; value: number }>;
	element: HTMLElement | null;
	offsetX: number;
	offsetY: number;
	startX: number;
	startY: number;
}

export interface DropTarget {
	colIndex: number;
	isValid: boolean;
}

// 创建拖拽状态
export function createDragState(): DragState {
	return {
		isDragging: false,
		fromCol: -1,
		startCardIndex: -1,
		cards: [],
		element: null,
		offsetX: 0,
		offsetY: 0,
		startX: 0,
		startY: 0
	};
}

// 检查是否为有效的拖拽序列（同花色降序）
export function isValidDragSequence(cards: Array<{ suit: string; value: number }>): boolean {
	if (cards.length === 0) return false;

	for (let i = 0; i < cards.length - 1; i++) {
		if (cards[i].suit !== cards[i + 1].suit) return false;
		if (cards[i].value !== cards[i + 1].value + 1) return false;
	}

	return true;
}

// 检查是否可以放置到目标列
export function canDropToColumn(
	dragCards: Array<{ suit: string; value: number }>,
	targetColumn: Array<{ suit: string; value: number; face_up: boolean }>
): boolean {
	if (targetColumn.length === 0) {
		return true; // 空列可以放任何牌
	}

	const topCard = targetColumn[targetColumn.length - 1];
	const firstDragCard = dragCards[0];

	// 目标牌必须比拖拽的第一张牌大1
	return topCard.value === firstDragCard.value + 1;
}

// 创建拖拽视觉元素
export function createDragElement(
	cards: Array<{ suit: string; value: number; face_up: boolean }>,
	cardWidth: number,
	cardHeight: number,
	cardGap: number
): HTMLElement {
	const container = document.createElement('div');
	container.className = 'drag-ghost';
	container.style.cssText = `
		position: fixed;
		pointer-events: none;
		z-index: 10000;
		width: ${cardWidth}px;
	`;

	cards.forEach((card, index) => {
		if (!card.face_up) return;

		const cardEl = document.createElement('div');
		cardEl.className = 'drag-card';
		cardEl.style.cssText = `
			position: absolute;
			top: ${index * cardGap}px;
			left: 0;
			width: ${cardWidth}px;
			height: ${cardHeight}px;
			background: white;
			border-radius: 8px;
			box-shadow: 0 8px 25px rgba(0, 0, 0, 0.4);
			overflow: hidden;
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

		// 异步加载SVG
		fetch(`/cards/${cardFileName}.svg`)
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

	container.style.height = `${(cards.length - 1) * cardGap + cardHeight}px`;

	return container;
}

// 获取鼠标位置下方的列
export function getColumnAtPosition(
	x: number,
	y: number,
	columnElements: NodeListOf<Element> | Element[]
): number {
	for (let i = 0; i < columnElements.length; i++) {
		const col = columnElements[i] as HTMLElement;
		const rect = col.getBoundingClientRect();

		if (x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom) {
			return i;
		}
	}

	return -1;
}

// 计算拖拽元素位置
export function updateDragPosition(
	element: HTMLElement,
	x: number,
	y: number,
	offsetX: number,
	offsetY: number
): void {
	element.style.left = `${x - offsetX}px`;
	element.style.top = `${y - offsetY}px`;
}

// 添加拖拽高亮样式
export function highlightDropTarget(columnElement: HTMLElement | null, isValid: boolean): void {
	if (!columnElement) return;

	columnElement.classList.remove('drop-target-valid', 'drop-target-invalid');

	if (isValid) {
		columnElement.classList.add('drop-target-valid');
	} else {
		columnElement.classList.add('drop-target-invalid');
	}
}

// 清除所有高亮
export function clearDropHighlights(): void {
	document.querySelectorAll('.drop-target-valid, .drop-target-invalid').forEach(el => {
		el.classList.remove('drop-target-valid', 'drop-target-invalid');
	});
}
