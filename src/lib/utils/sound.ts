// 音效管理器 - 为老年用户提供清晰的听觉反馈

type SoundType = 'deal' | 'move' | 'complete' | 'click' | 'error' | 'win';

class SoundManager {
	private audioContext: AudioContext | null = null;
	private sounds: Map<SoundType, AudioBuffer> = new Map();
	private volume: number = 0.5;
	private muted: boolean = false;

	// 初始化音频上下文
	private async initContext() {
		if (!this.audioContext) {
			this.audioContext = new AudioContext();
		}
		if (this.audioContext.state === 'suspended') {
			await this.audioContext.resume();
		}
	}

	// 生成合成音效（无需外部文件）
	private generateSound(type: SoundType): AudioBuffer | null {
		if (!this.audioContext) return null;

		const ctx = this.audioContext;
		let duration: number;
		let frequency: number;
		let oscillatorType: OscillatorType;

		switch (type) {
			case 'deal':
				duration = 0.15;
				frequency = 800;
				oscillatorType = 'sine';
				break;
			case 'move':
				duration = 0.12;
				frequency = 600;
				oscillatorType = 'sine';
				break;
			case 'complete':
				duration = 0.5;
				frequency = 880;
				oscillatorType = 'sine';
				break;
			case 'click':
				duration = 0.08;
				frequency = 1000;
				oscillatorType = 'sine';
				break;
			case 'error':
				duration = 0.25;
				frequency = 200;
				oscillatorType = 'square';
				break;
			case 'win':
				duration = 1.0;
				frequency = 660;
				oscillatorType = 'sine';
				break;
			default:
				return null;
		}

		const buffer = ctx.createBuffer(1, ctx.sampleRate * duration, ctx.sampleRate);
		const data = buffer.getChannelData(0);

		for (let i = 0; i < buffer.length; i++) {
			const t = i / ctx.sampleRate;
			let sample = 0;

			// 根据音效类型生成不同的波形
			if (type === 'complete' || type === 'win') {
				// 上升音调
				const freq = frequency * (1 + t * 0.5);
				sample = Math.sin(2 * Math.PI * freq * t);
			} else if (type === 'error') {
				// 低沉的错误音
				sample = Math.sin(2 * Math.PI * frequency * t) * 0.5;
			} else {
				// 普通音效
				sample = Math.sin(2 * Math.PI * frequency * t);
			}

			// 应用衰减包络
			const envelope = Math.exp(-t * (type === 'win' ? 1 : 5));
			data[i] = sample * envelope * this.volume;
		}

		return buffer;
	}

	// 预加载音效
	async preload() {
		await this.initContext();

		const types: SoundType[] = ['deal', 'move', 'complete', 'click', 'error', 'win'];
		for (const type of types) {
			const buffer = this.generateSound(type);
			if (buffer) {
				this.sounds.set(type, buffer);
			}
		}
	}

	// 播放音效
	async play(type: SoundType) {
		if (this.muted) return;

		try {
			await this.initContext();

			const buffer = this.sounds.get(type);
			if (!buffer || !this.audioContext) {
				// 如果没有预加载，实时生成
				await this.preload();
				const newBuffer = this.sounds.get(type);
				if (!newBuffer || !this.audioContext) return;

				const source = this.audioContext.createBufferSource();
				source.buffer = newBuffer;
				source.connect(this.audioContext.destination);
				source.start();
				return;
			}

			const source = this.audioContext.createBufferSource();
			source.buffer = buffer;
			source.connect(this.audioContext.destination);
			source.start();
		} catch (e) {
			console.warn('Sound playback failed:', e);
		}
	}

	// 设置音量 (0-1)
	setVolume(volume: number) {
		this.volume = Math.max(0, Math.min(1, volume));
	}

	// 获取当前音量
	getVolume(): number {
		return this.volume;
	}

	// 切换静音
	toggleMute(): boolean {
		this.muted = !this.muted;
		return this.muted;
	}

	// 设置静音状态
	setMuted(muted: boolean) {
		this.muted = muted;
	}

	// 获取静音状态
	isMuted(): boolean {
		return this.muted;
	}
}

// 单例导出
export const soundManager = new SoundManager();
