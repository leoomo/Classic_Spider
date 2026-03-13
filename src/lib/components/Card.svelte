<script lang="ts">
	import type { Card as CardType } from '$lib/types/game';
	import { onMount } from 'svelte';

	interface Props {
		card: CardType;
		offset?: number;
		selected?: boolean;
		onclick?: () => void;
	}

	let { card, offset = 0, selected = false, onclick }: Props = $props();

	// 花色映射: spade→S, heart→H, diamond→D, club→C
	const suitMap: Record<string, string> = {
		spade: 'S',
		heart: 'H',
		diamond: 'D',
		club: 'C'
	};

	// 数值映射: 1→A, 11→J, 12→Q, 13→K
	function getValueChar(value: number): string {
		switch (value) {
		case 1: return 'A';
		case 11: return 'J';
		case 12: return 'Q';
		case 13: return 'K';
		default: return String(value);
		}
	}

	// 生成卡牌 SVG 文件名 (如 "AS" = Ace of Spades)
	let cardFileName = $derived(`${getValueChar(card.value)}${suitMap[card.suit]}`);
	let cardUrl = $derived(`/cards/${cardFileName}.svg`);

	// SVG 内容 (使用 $state 以支持响应式更新)
	let svgContent = $state('');

	// 加载 SVG 并内联渲染
	async function loadSVG() {
		try {
			const response = await fetch(cardUrl);
			if (response.ok) {
				svgContent = await response.text();
			}
		} catch (e) {
			console.warn('Failed to load SVG:', e);
		}
	}

	onMount(() => {
		loadSVG();
	});
</script>

{#if card.face_up}
	<!-- 正面：使用高质量 Vector-Playing-Cards SVG -->
	<div
    class="card"
    class:selected
    style="top: {offset}px;"
    role="button"
    tabindex="0"
    onclick={onclick}
    onkeydown={(e) => e.key === 'Enter' && onclick?.()}
  >
    {#if svgContent}
      {@html svgContent}
    {:else}
      <div class="loading-placeholder"></div>
    {/if}
  </div>
{:else}
  <!-- 牌背：经典蓝色设计 -->
  <div
    class="card back"
    style="top: {offset}px;"
    role="listitem"
  >
    {@html `<svg viewBox="0 0 167.0869141 242.6669922" xmlns="http://www.w3.org/2000/svg">
      <rect x="0" y="0" width="167.0869141" height="242.6669922" rx="8" fill="#1e3a5f"/>
      <rect x="6" y="6" width="155.0869141" height="230.6669922" rx="6" fill="none" stroke="#2e5a8f" stroke-width="3"/>
      <rect x="12" y="12" width="143.0869141" height="218.6669922" rx="4" fill="#153254"/>
      <defs>
        <pattern id="backPattern" x="0" y="0" width="20" height="20" patternUnits="userSpaceOnUse">
          <polygon points="10,0 20,10 10,20 0,10" fill="none" stroke="#2e5a8f" stroke-width="0.8" opacity="0.5"/>
        </pattern>
      </defs>
      <rect x="16" y="16" width="135.0869141" height="210.6669922" fill="url(#backPattern)"/>
      <circle cx="83.543457" cy="121.3334961" r="35" fill="none" stroke="#3d7ab5" stroke-width="2" opacity="0.7"/>
      <circle cx="83.543457" cy="121.3334961" r="25" fill="none" stroke="#3d7ab5" stroke-width="1.5" opacity="0.5"/>
      <text x="83.543457" y="130" text-anchor="middle" font-size="40" fill="#3d7ab5" opacity="0.8">♠</text>
    </svg>`}
  </div>
{/if}

<style>
  .card {
    position: absolute;
    width: 85px;
    height: 124px;
    cursor: pointer;
    transition: transform 0.15s ease-out, box-shadow 0.15s ease-out;
    user-select: none;
    border-radius: 8px;
    overflow: hidden;
    background: white;
  }

  /* 高质量 SVG 渲染 */
  .card :global(svg) {
    width: 100%;
    height: 100%;
    display: block;
  }

  /* 加载占位符 */
  .loading-placeholder {
    width: 100%;
    height: 100%;
    background: linear-gradient(135deg, #f5f5f5 25%, #e0e0e0 50%, #f5f5f5 75%);
    background-size: 20px 20px;
    animation: loading 1s linear infinite;
  }

  @keyframes loading {
    0% { background-position: 0 0; }
    100% { background-position: 20px 20px; }
  }

  /* 选中效果 */
  .card.selected {
    transform: translateY(-8px);
    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.35), 0 0 0 3px rgba(255, 215, 0, 0.7);
    z-index: 100;
  }

  .card:hover:not(.selected) {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
    transform: scale(1.02);
  }

  .card.back {
    cursor: default;
  }

  .card.back:hover {
    transform: none;
    box-shadow: none;
  }
</style>
