// 顶级纸牌音效系统 - 真实纸牌质感
// 核心理念：模拟真实纸牌的物理特性 - 厚实、有质感、有层次

type SoundType = 'deal' | 'move' | 'complete' | 'click' | 'error' | 'win' | 'shuffle' | 'slide' | 'flip';

class SoundManager {
	private audioContext: AudioContext | null = null;
	private volume: number = 0.7;
	private muted: boolean = false;
	private masterGain: GainNode | null = null;
	private convolver: ConvolverNode | null = null;
	private initialized: boolean = false;

	// 初始化音频上下文（带混响效果）
	private async initContext(): Promise<AudioContext> {
		if (!this.audioContext) {
			this.audioContext = new AudioContext({ sampleRate: 48000 });
		}
		if (this.audioContext.state === 'suspended') {
			await this.audioContext.resume();
		}

		if (!this.initialized) {
			// 创建主增益节点
			this.masterGain = this.audioContext.createGain();
			this.masterGain.gain.value = this.volume;
			this.masterGain.connect(this.audioContext.destination);

			// 创建混响效果（模拟桌面环境）
			this.convolver = this.audioContext.createConvolver();
			this.convolver.buffer = this.createReverbImpulse();
			this.initialized = true;
		}

		return this.audioContext;
	}

	// 创建混响脉冲响应（模拟木质桌面）
	private createReverbImpulse(): AudioBuffer {
		const ctx = this.audioContext!;
		const length = ctx.sampleRate * 0.3; // 0.3秒混响
		const impulse = ctx.createBuffer(2, length, ctx.sampleRate);

		for (let channel = 0; channel < 2; channel++) {
			const data = impulse.getChannelData(channel);
			for (let i = 0; i < length; i++) {
				const t = i / ctx.sampleRate;
				// 指数衰减 + 随机散射
				data[i] = (Math.random() * 2 - 1) * Math.exp(-t * 15) * 0.3;
			}
		}
		return impulse;
	}

	// 创建带混响的增益节点链
	private createDryWetChain(ctx: AudioContext, dryGain: number = 0.7, wetGain: number = 0.3): { dry: GainNode; wet: GainNode } {
		const dry = ctx.createGain();
		dry.gain.value = dryGain * this.volume;

		const wet = ctx.createGain();
		wet.gain.value = wetGain * this.volume;

		dry.connect(this.masterGain!);
		if (this.convolver) {
			wet.connect(this.convolver);
			this.convolver.connect(this.masterGain!);
		}

		return { dry, wet };
	}

	// 🃏 发牌声 - 厚实的"啪嗒"，纸牌落在木桌上
	private playDealSound(ctx: AudioContext) {
		const now = ctx.currentTime;
		const chain = this.createDryWetChain(ctx, 0.8, 0.2);

		// 第一层：低频冲击（桌面振动）
		const impact = ctx.createOscillator();
		impact.type = 'sine';
		impact.frequency.setValueAtTime(180, now);
		impact.frequency.exponentialRampToValueAtTime(50, now + 0.08);

		const impactGain = ctx.createGain();
		impactGain.gain.setValueAtTime(0.6, now);
		impactGain.gain.exponentialRampToValueAtTime(0.001, now + 0.08);

		// 第二层：中频"啪"（纸牌拍打）
		const slap = ctx.createOscillator();
		slap.type = 'triangle';
		slap.frequency.setValueAtTime(600, now);
		slap.frequency.exponentialRampToValueAtTime(200, now + 0.04);

		const slapGain = ctx.createGain();
		slapGain.gain.setValueAtTime(0.4, now);
		slapGain.gain.exponentialRampToValueAtTime(0.001, now + 0.04);

		// 第三层：高频噪声（纸张质感）
		const noiseBuffer = ctx.createBuffer(1, ctx.sampleRate * 0.06, ctx.sampleRate);
		const noiseData = noiseBuffer.getChannelData(0);
		for (let i = 0; i < noiseData.length; i++) {
			const t = i / ctx.sampleRate;
			noiseData[i] = (Math.random() * 2 - 1) * Math.exp(-t * 80);
		}
		const noise = ctx.createBufferSource();
		noise.buffer = noiseBuffer;

		const noiseFilter = ctx.createBiquadFilter();
		noiseFilter.type = 'bandpass';
		noiseFilter.frequency.value = 3000;
		noiseFilter.Q.value = 1;

		const noiseGain = ctx.createGain();
		noiseGain.gain.value = 0.25;

		// 连接
		impact.connect(impactGain);
		impactGain.connect(chain.dry);
		impactGain.connect(chain.wet);

		slap.connect(slapGain);
		slapGain.connect(chain.dry);
		slapGain.connect(chain.wet);

		noise.connect(noiseFilter);
		noiseFilter.connect(noiseGain);
		noiseGain.connect(chain.dry);

		// 播放
		impact.start(now);
		impact.stop(now + 0.08);
		slap.start(now);
		slap.stop(now + 0.04);
		noise.start(now);
		noise.stop(now + 0.06);
	}

	// 🎴 移动声 - 沉稳的落牌，比发牌更轻柔
	private playMoveSound(ctx: AudioContext) {
		const now = ctx.currentTime;
		const chain = this.createDryWetChain(ctx, 0.75, 0.25);

		// 低频主体
		const body = ctx.createOscillator();
		body.type = 'sine';
		body.frequency.setValueAtTime(120, now);
		body.frequency.exponentialRampToValueAtTime(60, now + 0.1);

		const bodyGain = ctx.createGain();
		bodyGain.gain.setValueAtTime(0.5, now);
		bodyGain.gain.exponentialRampToValueAtTime(0.001, now + 0.1);

		// 纸牌摩擦噪声
		const frictionBuffer = ctx.createBuffer(1, ctx.sampleRate * 0.08, ctx.sampleRate);
		const frictionData = frictionBuffer.getChannelData(0);
		for (let i = 0; i < frictionData.length; i++) {
			const t = i / ctx.sampleRate;
			frictionData[i] = (Math.random() * 2 - 1) * Math.exp(-t * 50);
		}
		const friction = ctx.createBufferSource();
		friction.buffer = frictionBuffer;

		const frictionFilter = ctx.createBiquadFilter();
		frictionFilter.type = 'highpass';
		frictionFilter.frequency.value = 1000;

		const frictionGain = ctx.createGain();
		frictionGain.gain.value = 0.15;

		// 连接
		body.connect(bodyGain);
		bodyGain.connect(chain.dry);
		bodyGain.connect(chain.wet);

		friction.connect(frictionFilter);
		frictionFilter.connect(frictionGain);
		frictionGain.connect(chain.dry);

		body.start(now);
		body.stop(now + 0.1);
		friction.start(now);
		friction.stop(now + 0.08);
	}

	// 📦 滑牌声 - 多张牌一起移动的"沙沙"声
	private playSlideSound(ctx: AudioContext) {
		const now = ctx.currentTime;
		const chain = this.createDryWetChain(ctx, 0.7, 0.3);

		// 创建多层噪声模拟多张牌
		const duration = 0.2;
		const noiseBuffer = ctx.createBuffer(1, ctx.sampleRate * duration, ctx.sampleRate);
		const noiseData = noiseBuffer.getChannelData(0);

		for (let i = 0; i < noiseData.length; i++) {
			const t = i / ctx.sampleRate;
			// 多层衰减，模拟牌堆滑动
			const envelope = Math.sin(Math.PI * t / duration) * (0.3 + 0.7 * Math.exp(-t * 10));
			// 添加一些"颗粒感"
			const grain = 0.8 + 0.2 * Math.sin(t * 200);
			noiseData[i] = (Math.random() * 2 - 1) * envelope * grain;
		}

		const noise = ctx.createBufferSource();
		noise.buffer = noiseBuffer;

		// 带通滤波，提取纸牌滑动的频段
		const bandpass = ctx.createBiquadFilter();
		bandpass.type = 'bandpass';
		bandpass.frequency.value = 2000;
		bandpass.Q.value = 0.8;

		// 低频补充（桌面共振）
		const lowOsc = ctx.createOscillator();
		lowOsc.type = 'sine';
		lowOsc.frequency.value = 80;

		const lowGain = ctx.createGain();
		lowGain.gain.setValueAtTime(0.2, now);
		lowGain.gain.exponentialRampToValueAtTime(0.001, now + duration);

		// 连接
		noise.connect(bandpass);
		bandpass.connect(chain.dry);
		bandpass.connect(chain.wet);

		lowOsc.connect(lowGain);
		lowGain.connect(chain.dry);

		noise.start(now);
		noise.stop(now + duration);
		lowOsc.start(now);
		lowOsc.stop(now + duration);
	}

	// ✨ 完成序列 - 满足感的和弦 + 纸牌收拢声
	private playCompleteSound(ctx: AudioContext) {
		const now = ctx.currentTime;
		const chain = this.createDryWetChain(ctx, 0.6, 0.4);

		// 纸牌收拢声
		const collectBuffer = ctx.createBuffer(1, ctx.sampleRate * 0.15, ctx.sampleRate);
		const collectData = collectBuffer.getChannelData(0);
		for (let i = 0; i < collectData.length; i++) {
			const t = i / ctx.sampleRate;
			collectData[i] = (Math.random() * 2 - 1) * Math.exp(-t * 25);
		}
		const collect = ctx.createBufferSource();
		collect.buffer = collectBuffer;

		const collectFilter = ctx.createBiquadFilter();
		collectFilter.type = 'bandpass';
		collectFilter.frequency.value = 1500;
		collectFilter.Q.value = 1;

		const collectGain = ctx.createGain();
		collectGain.gain.value = 0.3;

		collect.connect(collectFilter);
		collectFilter.connect(collectGain);
		collectGain.connect(chain.dry);

		// 悦耳的和弦（C大调九和弦）
		const freqs = [261.63, 329.63, 392.00, 493.88, 523.25]; // C3, E3, G3, B3, C4

		freqs.forEach((freq, index) => {
			const osc = ctx.createOscillator();
			osc.type = 'sine';
			osc.frequency.value = freq;

			// 第二泛音（更丰富的音色）
			const osc2 = ctx.createOscillator();
			osc2.type = 'triangle';
			osc2.frequency.value = freq * 2;

			const delay = 0.02 + index * 0.03;
			const duration = 0.5;

			const gain1 = ctx.createGain();
			gain1.gain.setValueAtTime(0, now + delay);
			gain1.gain.linearRampToValueAtTime(0.12, now + delay + 0.03);
			gain1.gain.exponentialRampToValueAtTime(0.001, now + delay + duration);

			const gain2 = ctx.createGain();
			gain2.gain.setValueAtTime(0, now + delay);
			gain2.gain.linearRampToValueAtTime(0.04, now + delay + 0.03);
			gain2.gain.exponentialRampToValueAtTime(0.001, now + delay + duration * 0.7);

			osc.connect(gain1);
			gain1.connect(chain.dry);
			gain1.connect(chain.wet);

			osc2.connect(gain2);
			gain2.connect(chain.dry);

			osc.start(now + delay);
			osc.stop(now + delay + duration);
			osc2.start(now + delay);
			osc2.stop(now + delay + duration * 0.7);
		});

		collect.start(now);
		collect.stop(now + 0.15);
	}

	// 🏆 胜利音效 - 华丽的旋律 + 烟花感
	private playWinSound(ctx: AudioContext) {
		const now = ctx.currentTime;
		const chain = this.createDryWetChain(ctx, 0.5, 0.5);

		// 纸牌飞舞的"哗啦"声
		const flourishBuffer = ctx.createBuffer(1, ctx.sampleRate * 0.8, ctx.sampleRate);
		const flourishData = flourishBuffer.getChannelData(0);
		for (let i = 0; i < flourishData.length; i++) {
			const t = i / ctx.sampleRate;
			const env = Math.sin(Math.PI * t / 0.8) * Math.exp(-t * 2);
			const shimmer = 0.5 + 0.5 * Math.sin(t * 50);
			flourishData[i] = (Math.random() * 2 - 1) * env * shimmer;
		}
		const flourish = ctx.createBufferSource();
		flourish.buffer = flourishBuffer;

		const flourishFilter = ctx.createBiquadFilter();
		flourishFilter.type = 'highpass';
		flourishFilter.frequency.value = 800;

		const flourishGain = ctx.createGain();
		flourishGain.gain.value = 0.2;

		flourish.connect(flourishFilter);
		flourishFilter.connect(flourishGain);
		flourishGain.connect(chain.dry);
		flourishGain.connect(chain.wet);

		// 胜利旋律（上行琶音）
		const melody = [523.25, 659.25, 783.99, 1046.50, 1318.51, 1567.98]; // C5, E5, G5, C6, E6, G6

		melody.forEach((freq, index) => {
			// 主音
			const osc = ctx.createOscillator();
			osc.type = 'sine';
			osc.frequency.value = freq;

			// 八度泛音
			const osc2 = ctx.createOscillator();
			osc2.type = 'sine';
			osc2.frequency.value = freq * 0.5;

			// 五度泛音
			const osc3 = ctx.createOscillator();
			osc3.type = 'triangle';
			osc3.frequency.value = freq * 1.5;

			const delay = index * 0.12;
			const duration = 0.6;

			const gain1 = ctx.createGain();
			gain1.gain.setValueAtTime(0, now + delay);
			gain1.gain.linearRampToValueAtTime(0.15, now + delay + 0.02);
			gain1.gain.setValueAtTime(0.15, now + delay + 0.1);
			gain1.gain.exponentialRampToValueAtTime(0.001, now + delay + duration);

			const gain2 = ctx.createGain();
			gain2.gain.setValueAtTime(0, now + delay);
			gain2.gain.linearRampToValueAtTime(0.08, now + delay + 0.02);
			gain2.gain.exponentialRampToValueAtTime(0.001, now + delay + duration * 0.8);

			const gain3 = ctx.createGain();
			gain3.gain.setValueAtTime(0, now + delay);
			gain3.gain.linearRampToValueAtTime(0.03, now + delay + 0.02);
			gain3.gain.exponentialRampToValueAtTime(0.001, now + delay + duration * 0.5);

			osc.connect(gain1);
			gain1.connect(chain.dry);
			gain1.connect(chain.wet);

			osc2.connect(gain2);
			gain2.connect(chain.dry);
			gain2.connect(chain.wet);

			osc3.connect(gain3);
			gain3.connect(chain.dry);

			osc.start(now + delay);
			osc.stop(now + delay + duration);
			osc2.start(now + delay);
			osc2.stop(now + delay + duration * 0.8);
			osc3.start(now + delay);
			osc3.stop(now + delay + duration * 0.5);
		});

		// 最后的和弦
		const finalChord = [523.25, 659.25, 783.99, 1046.50];
		const chordStart = now + 0.8;

		finalChord.forEach((freq, index) => {
			const osc = ctx.createOscillator();
			osc.type = 'sine';
			osc.frequency.value = freq;

			const gain = ctx.createGain();
			gain.gain.setValueAtTime(0, chordStart);
			gain.gain.linearRampToValueAtTime(0.12, chordStart + 0.1);
			gain.gain.setValueAtTime(0.12, chordStart + 0.3);
			gain.gain.exponentialRampToValueAtTime(0.001, chordStart + 1.5);

			osc.connect(gain);
			gain.connect(chain.dry);
			gain.connect(chain.wet);

			osc.start(chordStart);
			osc.stop(chordStart + 1.5);
		});

		flourish.start(now);
		flourish.stop(now + 0.8);
	}

	// 🃏 洗牌声 - 复杂的"刷刷刷"
	private playShuffleSound(ctx: AudioContext) {
		const now = ctx.currentTime;
		const chain = this.createDryWetChain(ctx, 0.65, 0.35);
		const duration = 1.0;

		// 主噪声层
		const noiseBuffer = ctx.createBuffer(1, ctx.sampleRate * duration, ctx.sampleRate);
		const noiseData = noiseBuffer.getChannelData(0);

		for (let i = 0; i < noiseData.length; i++) {
			const t = i / ctx.sampleRate;
			// 洗牌的节奏感 - 交替的强弱
			const rhythm = 0.5 + 0.5 * Math.abs(Math.sin(t * 8));
			// 整体包络
			const envelope = Math.sin(Math.PI * t / duration);
			// 添加一些随机"卡顿"
			const jitter = 0.7 + 0.3 * (Math.random() > 0.95 ? 0.3 : 1);
			noiseData[i] = (Math.random() * 2 - 1) * rhythm * envelope * jitter;
		}

		const noise = ctx.createBufferSource();
		noise.buffer = noiseBuffer;

		// 多段滤波创造层次感
		const highFilter = ctx.createBiquadFilter();
		highFilter.type = 'highpass';
		highFilter.frequency.value = 600;

		const midFilter = ctx.createBiquadFilter();
		midFilter.type = 'bandpass';
		midFilter.frequency.value = 2000;
		midFilter.Q.value = 0.5;

		const highGain = ctx.createGain();
		highGain.gain.value = 0.3;

		const midGain = ctx.createGain();
		midGain.gain.value = 0.4;

		// 低频振动（桌面）
		const lowOsc = ctx.createOscillator();
		lowOsc.type = 'sine';
		lowOsc.frequency.value = 60;

		const lowGain = ctx.createGain();
		lowGain.gain.setValueAtTime(0.15, now);
		lowGain.gain.setValueAtTime(0.15, now + duration * 0.5);
		lowGain.gain.linearRampToValueAtTime(0, now + duration);

		// 连接
		noise.connect(highFilter);
		highFilter.connect(highGain);
		highGain.connect(chain.dry);

		noise.connect(midFilter);
		midFilter.connect(midGain);
		midGain.connect(chain.dry);
		midGain.connect(chain.wet);

		lowOsc.connect(lowGain);
		lowGain.connect(chain.dry);

		noise.start(now);
		noise.stop(now + duration);
		lowOsc.start(now);
		lowOsc.stop(now + duration);
	}

	// ❌ 错误提示 - 柔和但明确的反馈
	private playErrorSound(ctx: AudioContext) {
		const now = ctx.currentTime;
		const chain = this.createDryWetChain(ctx, 0.8, 0.2);

		// 柔和的双音
		const freqs = [220, 277.18]; // A3, C#3（小三度，略带忧郁）

		freqs.forEach((freq, index) => {
			const osc = ctx.createOscillator();
			osc.type = 'sine';
			osc.frequency.value = freq;

			const gain = ctx.createGain();
			const delay = index * 0.03;
			gain.gain.setValueAtTime(0, now + delay);
			gain.gain.linearRampToValueAtTime(0.2, now + delay + 0.02);
			gain.gain.setValueAtTime(0.2, now + delay + 0.08);
			gain.gain.exponentialRampToValueAtTime(0.001, now + delay + 0.25);

			osc.connect(gain);
			gain.connect(chain.dry);

			osc.start(now + delay);
			osc.stop(now + delay + 0.25);
		});
	}

	// 🖱️ 点击声 - 轻柔的"哒"
	private playClickSound(ctx: AudioContext) {
		const now = ctx.currentTime;
		const chain = this.createDryWetChain(ctx, 0.85, 0.15);

		// 短促的噪声
		const noiseBuffer = ctx.createBuffer(1, ctx.sampleRate * 0.03, ctx.sampleRate);
		const noiseData = noiseBuffer.getChannelData(0);
		for (let i = 0; i < noiseData.length; i++) {
			noiseData[i] = (Math.random() * 2 - 1) * Math.exp(-i / (ctx.sampleRate * 0.003));
		}

		const noise = ctx.createBufferSource();
		noise.buffer = noiseBuffer;

		const filter = ctx.createBiquadFilter();
		filter.type = 'bandpass';
		filter.frequency.value = 2500;
		filter.Q.value = 2;

		const gain = ctx.createGain();
		gain.gain.value = 0.3;

		// 轻微的低频
		const low = ctx.createOscillator();
		low.type = 'sine';
		low.frequency.value = 100;

		const lowGain = ctx.createGain();
		lowGain.gain.setValueAtTime(0.15, now);
		lowGain.gain.exponentialRampToValueAtTime(0.001, now + 0.03);

		noise.connect(filter);
		filter.connect(gain);
		gain.connect(chain.dry);

		low.connect(lowGain);
		lowGain.connect(chain.dry);

		noise.start(now);
		noise.stop(now + 0.03);
		low.start(now);
		low.stop(now + 0.03);
	}

	// 🔄 翻牌声 - 快速的"刷"
	private playFlipSound(ctx: AudioContext) {
		const now = ctx.currentTime;
		const chain = this.createDryWetChain(ctx, 0.75, 0.25);

		// 快速扫频噪声
		const noiseBuffer = ctx.createBuffer(1, ctx.sampleRate * 0.08, ctx.sampleRate);
		const noiseData = noiseBuffer.getChannelData(0);
		for (let i = 0; i < noiseData.length; i++) {
			const t = i / ctx.sampleRate;
			noiseData[i] = (Math.random() * 2 - 1) * Math.exp(-t * 30);
		}

		const noise = ctx.createBufferSource();
		noise.buffer = noiseBuffer;

		// 扫频滤波器
		const filter = ctx.createBiquadFilter();
		filter.type = 'bandpass';
		filter.frequency.setValueAtTime(4000, now);
		filter.frequency.exponentialRampToValueAtTime(1000, now + 0.08);
		filter.Q.value = 3;

		const gain = ctx.createGain();
		gain.gain.value = 0.35;

		// 低频补充
		const low = ctx.createOscillator();
		low.type = 'sine';
		low.frequency.setValueAtTime(200, now);
		low.frequency.exponentialRampToValueAtTime(80, now + 0.06);

		const lowGain = ctx.createGain();
		lowGain.gain.setValueAtTime(0.2, now);
		lowGain.gain.exponentialRampToValueAtTime(0.001, now + 0.06);

		noise.connect(filter);
		filter.connect(gain);
		gain.connect(chain.dry);

		low.connect(lowGain);
		lowGain.connect(chain.dry);

		noise.start(now);
		noise.stop(now + 0.08);
		low.start(now);
		low.stop(now + 0.06);
	}

	// 预加载
	async preload() {
		await this.initContext();
		console.log('🔊 顶级纸牌音效系统已就绪（真实纸牌质感）');
	}

	// 🎵 播放音效
	async play(type: SoundType) {
		if (this.muted) return;

		try {
			const ctx = await this.initContext();

			switch (type) {
				case 'deal': this.playDealSound(ctx); break;
				case 'move': this.playMoveSound(ctx); break;
				case 'slide': this.playSlideSound(ctx); break;
				case 'complete': this.playCompleteSound(ctx); break;
				case 'win': this.playWinSound(ctx); break;
				case 'shuffle': this.playShuffleSound(ctx); break;
				case 'error': this.playErrorSound(ctx); break;
				case 'click': this.playClickSound(ctx); break;
				case 'flip': this.playFlipSound(ctx); break;
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
		if (this.masterGain) {
			this.masterGain.gain.value = this.volume;
		}
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
