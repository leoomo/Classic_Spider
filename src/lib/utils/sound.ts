// 经典蜘蛛纸牌音效系统 - 还原 Windows 经典音效
// 核心理念：清脆、干净、有质感的纸牌音效

type SoundType = 'deal' | 'move' | 'complete' | 'click' | 'error' | 'win' | 'shuffle' | 'slide' | 'flip';

class SoundManager {
	private audioContext: AudioContext | null = null;
	private volume: number = 0.6;
	private muted: boolean = false;

	// 初始化音频上下文
	private async initContext(): Promise<AudioContext> {
		if (!this.audioContext) {
			this.audioContext = new AudioContext();
		}
		if (this.audioContext.state === 'suspended') {
			await this.audioContext.resume();
		}
		return this.audioContext;
	}

	// 🃏 经典发牌声 - 清脆的"啪嗒"
	private playDealSound(ctx: AudioContext) {
		const now = ctx.currentTime;

		// 创建噪声缓冲区
		const noiseBuffer = ctx.createBuffer(1, ctx.sampleRate * 0.08, ctx.sampleRate);
		const noiseData = noiseBuffer.getChannelData(0);
		for (let i = 0; i < noiseData.length; i++) {
			noiseData[i] = (Math.random() * 2 - 1) * Math.exp(-i / (ctx.sampleRate * 0.015));
		}

		// 噪声源
		const noise = ctx.createBufferSource();
		noise.buffer = noiseBuffer;

		// 带通滤波 - 让声音更像纸牌
		const bandpass = ctx.createBiquadFilter();
		bandpass.type = 'bandpass';
		bandpass.frequency.value = 2500;
		bandpass.Q.value = 1.5;

		// 高通滤波
		const highpass = ctx.createBiquadFilter();
		highpass.type = 'highpass';
		highpass.frequency.value = 800;

		// 增益包络
		const gain = ctx.createGain();
		gain.gain.setValueAtTime(this.volume * 0.7, now);
		gain.gain.exponentialDecayTo?.(0.001, now + 0.08) || gain.gain.exponentialRampToValueAtTime(0.001, now + 0.08);

		// 连接
		noise.connect(bandpass);
		bandpass.connect(highpass);
		highpass.connect(gain);
		gain.connect(ctx.destination);

		noise.start(now);
		noise.stop(now + 0.08);
	}

	// 🎴 经典落牌声 - 沉稳的"咚"
	private playMoveSound(ctx: AudioContext) {
		const now = ctx.currentTime;

		// 低频主音
		const osc = ctx.createOscillator();
		osc.type = 'sine';
		osc.frequency.setValueAtTime(150, now);
		osc.frequency.exponentialRampToValueAtTime(60, now + 0.1);

		// 高频泛音（纸张质感）
		const osc2 = ctx.createOscillator();
		osc2.type = 'sine';
		osc2.frequency.setValueAtTime(800, now);
		osc2.frequency.exponentialRampToValueAtTime(400, now + 0.05);

		// 噪声层
		const noiseBuffer = ctx.createBuffer(1, ctx.sampleRate * 0.06, ctx.sampleRate);
		const noiseData = noiseBuffer.getChannelData(0);
		for (let i = 0; i < noiseData.length; i++) {
			noiseData[i] = (Math.random() * 2 - 1) * Math.exp(-i / (ctx.sampleRate * 0.01));
		}
		const noise = ctx.createBufferSource();
		noise.buffer = noiseBuffer;

		// 滤波器
		const lowpass = ctx.createBiquadFilter();
		lowpass.type = 'lowpass';
		lowpass.frequency.value = 3000;

		// 增益包络
		const gain1 = ctx.createGain();
		gain1.gain.setValueAtTime(this.volume * 0.5, now);
		gain1.gain.exponentialRampToValueAtTime(0.001, now + 0.1);

		const gain2 = ctx.createGain();
		gain2.gain.setValueAtTime(this.volume * 0.3, now);
		gain2.gain.exponentialRampToValueAtTime(0.001, now + 0.05);

		const gain3 = ctx.createGain();
		gain3.gain.setValueAtTime(this.volume * 0.2, now);
		gain3.gain.exponentialRampToValueAtTime(0.001, now + 0.06);

		// 连接
		osc.connect(gain1);
		gain1.connect(ctx.destination);

		osc2.connect(gain2);
		gain2.connect(ctx.destination);

		noise.connect(lowpass);
		lowpass.connect(gain3);
		gain3.connect(ctx.destination);

		osc.start(now);
		osc.stop(now + 0.1);
		osc2.start(now);
		osc2.stop(now + 0.05);
		noise.start(now);
		noise.stop(now + 0.06);
	}

	// 📦 滑牌声 - 柔和的"沙沙"
	private playSlideSound(ctx: AudioContext) {
		const now = ctx.currentTime;

		// 噪声缓冲区
		const noiseBuffer = ctx.createBuffer(1, ctx.sampleRate * 0.15, ctx.sampleRate);
		const noiseData = noiseBuffer.getChannelData(0);
		for (let i = 0; i < noiseData.length; i++) {
			const t = i / ctx.sampleRate;
			const env = Math.exp(-t * 15);
			noiseData[i] = (Math.random() * 2 - 1) * env;
		}

		const noise = ctx.createBufferSource();
		noise.buffer = noiseBuffer;

		// 带通滤波
		const bandpass = ctx.createBiquadFilter();
		bandpass.type = 'bandpass';
		bandpass.frequency.value = 1200;
		bandpass.Q.value = 2;

		const gain = ctx.createGain();
		gain.gain.value = this.volume * 0.4;

		noise.connect(bandpass);
		bandpass.connect(gain);
		gain.connect(ctx.destination);

		noise.start(now);
		noise.stop(now + 0.15);
	}

	// ✨ 完成序列 - 悦耳的"叮"
	private playCompleteSound(ctx: AudioContext) {
		const now = ctx.currentTime;

		// C大调和弦
		const freqs = [523.25, 659.25, 783.99]; // C5, E5, G5

		freqs.forEach((freq, index) => {
			const osc = ctx.createOscillator();
			osc.type = 'sine';
			osc.frequency.value = freq;

			const gain = ctx.createGain();
			const delay = index * 0.05;
			gain.gain.setValueAtTime(0, now + delay);
			gain.gain.linearRampToValueAtTime(this.volume * 0.25, now + delay + 0.02);
			gain.gain.exponentialRampToValueAtTime(0.001, now + delay + 0.4);

			osc.connect(gain);
			gain.connect(ctx.destination);

			osc.start(now + delay);
			osc.stop(now + delay + 0.4);
		});
	}

	// 🏆 胜利音效 - 欢快的旋律
	private playWinSound(ctx: AudioContext) {
		const now = ctx.currentTime;

		// C大调上行音阶
		const notes = [523.25, 587.33, 659.25, 698.46, 783.99, 880.00, 987.77, 1046.50];

		notes.forEach((freq, index) => {
			const osc = ctx.createOscillator();
			osc.type = 'sine';
			osc.frequency.value = freq;

			const osc2 = ctx.createOscillator();
			osc2.type = 'sine';
			osc2.frequency.value = freq * 2;

			const gain = ctx.createGain();
			const gain2 = ctx.createGain();
			const delay = index * 0.1;

			gain.gain.setValueAtTime(0, now + delay);
			gain.gain.linearRampToValueAtTime(this.volume * 0.2, now + delay + 0.02);
			gain.gain.exponentialRampToValueAtTime(0.001, now + delay + 0.3);

			gain2.gain.setValueAtTime(0, now + delay);
			gain2.gain.linearRampToValueAtTime(this.volume * 0.1, now + delay + 0.02);
			gain2.gain.exponentialRampToValueAtTime(0.001, now + delay + 0.25);

			osc.connect(gain);
			gain.connect(ctx.destination);
			osc2.connect(gain2);
			gain2.connect(ctx.destination);

			osc.start(now + delay);
			osc.stop(now + delay + 0.3);
			osc2.start(now + delay);
			osc2.stop(now + delay + 0.25);
		});
	}

	// 🃏 洗牌声 - 复杂的"刷刷刷"
	private playShuffleSound(ctx: AudioContext) {
		const now = ctx.currentTime;
		const duration = 0.6;

		// 多层噪声
		const noiseBuffer = ctx.createBuffer(1, ctx.sampleRate * duration, ctx.sampleRate);
		const noiseData = noiseBuffer.getChannelData(0);

		for (let i = 0; i < noiseData.length; i++) {
			const t = i / ctx.sampleRate;
			// 洗牌的节奏感
			const rhythm = Math.sin(t * 30) * 0.3 + 0.7;
			const env = Math.sin(Math.PI * t / duration);
			noiseData[i] = (Math.random() * 2 - 1) * rhythm * env;
		}

		const noise = ctx.createBufferSource();
		noise.buffer = noiseBuffer;

		// 低频振动
		const osc = ctx.createOscillator();
		osc.type = 'sine';
		osc.frequency.value = 80;

		const lowpass = ctx.createBiquadFilter();
		lowpass.type = 'lowpass';
		lowpass.frequency.value = 4000;

		const gain1 = ctx.createGain();
		gain1.gain.value = this.volume * 0.5;

		const gain2 = ctx.createGain();
		gain2.gain.setValueAtTime(this.volume * 0.15, now);
		gain2.gain.setValueAtTime(this.volume * 0.15, now + 0.3);
		gain2.gain.linearRampToValueAtTime(0, now + duration);

		noise.connect(lowpass);
		lowpass.connect(gain1);
		gain1.connect(ctx.destination);

		osc.connect(gain2);
		gain2.connect(ctx.destination);

		noise.start(now);
		noise.stop(now + duration);
		osc.start(now);
		osc.stop(now + duration);
	}

	// ❌ 错误提示 - 温和的"嘟"
	private playErrorSound(ctx: AudioContext) {
		const now = ctx.currentTime;

		const osc = ctx.createOscillator();
		osc.type = 'sine';
		osc.frequency.value = 220;

		const gain = ctx.createGain();
		gain.gain.setValueAtTime(this.volume * 0.3, now);
		gain.gain.setValueAtTime(this.volume * 0.3, now + 0.1);
		gain.gain.exponentialRampToValueAtTime(0.001, now + 0.2);

		osc.connect(gain);
		gain.connect(ctx.destination);

		osc.start(now);
		osc.stop(now + 0.2);
	}

	// 🖱️ 点击声 - 轻柔的"哒"
	private playClickSound(ctx: AudioContext) {
		const now = ctx.currentTime;

		const noiseBuffer = ctx.createBuffer(1, ctx.sampleRate * 0.04, ctx.sampleRate);
		const noiseData = noiseBuffer.getChannelData(0);
		for (let i = 0; i < noiseData.length; i++) {
			noiseData[i] = (Math.random() * 2 - 1) * Math.exp(-i / (ctx.sampleRate * 0.005));
		}

		const noise = ctx.createBufferSource();
		noise.buffer = noiseBuffer;

		const highpass = ctx.createBiquadFilter();
		highpass.type = 'highpass';
		highpass.frequency.value = 1500;

		const gain = ctx.createGain();
		gain.gain.value = this.volume * 0.25;

		noise.connect(highpass);
		highpass.connect(gain);
		gain.connect(ctx.destination);

		noise.start(now);
		noise.stop(now + 0.04);
	}

	// 🔄 翻牌声
	private playFlipSound(ctx: AudioContext) {
		const now = ctx.currentTime;

		// 快速的刷声
		const noiseBuffer = ctx.createBuffer(1, ctx.sampleRate * 0.1, ctx.sampleRate);
		const noiseData = noiseBuffer.getChannelData(0);
		for (let i = 0; i < noiseData.length; i++) {
			const t = i / ctx.sampleRate;
			noiseData[i] = (Math.random() * 2 - 1) * Math.exp(-t * 20);
		}

		const noise = ctx.createBufferSource();
		noise.buffer = noiseBuffer;

		const bandpass = ctx.createBiquadFilter();
		bandpass.type = 'bandpass';
		bandpass.frequency.value = 2000;
		bandpass.Q.value = 2;

		const gain = ctx.createGain();
		gain.gain.value = this.volume * 0.35;

		noise.connect(bandpass);
		bandpass.connect(gain);
		gain.connect(ctx.destination);

		noise.start(now);
		noise.stop(now + 0.1);
	}

	// 预加载（保持兼容性）
	async preload() {
		await this.initContext();
		console.log('🔊 经典纸牌音效系统已就绪');
	}

	// 🎵 播放音效
	async play(type: SoundType) {
		if (this.muted) return;

		try {
			const ctx = await this.initContext();

			switch (type) {
				case 'deal':
					this.playDealSound(ctx);
					break;
				case 'move':
					this.playMoveSound(ctx);
					break;
				case 'slide':
					this.playSlideSound(ctx);
					break;
				case 'complete':
					this.playCompleteSound(ctx);
					break;
				case 'win':
					this.playWinSound(ctx);
					break;
				case 'shuffle':
					this.playShuffleSound(ctx);
					break;
				case 'error':
					this.playErrorSound(ctx);
					break;
				case 'click':
					this.playClickSound(ctx);
					break;
				case 'flip':
					this.playFlipSound(ctx);
					break;
			}
		} catch (e) {
			console.warn('Sound playback failed:', e);
		}
	}

	// 便捷方法
	async playDeal() { await this.play('deal'); }
	async playMove() { await this.play('move'); }
	async playComplete() { await this.play('complete'); }
	async playClick() { await this.play('click'); }
	async playError() { await this.play('error'); }
	async playWin() { await this.play('win'); }
	async playShuffle() { await this.play('shuffle'); }
	async playSlide() { await this.play('slide'); }
	async playFlip() { await this.play('flip'); }

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
