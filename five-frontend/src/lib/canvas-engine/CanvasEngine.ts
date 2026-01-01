import { Effect } from './Effect';

export class CanvasEngine {
    private canvas: HTMLCanvasElement;
    private ctx: CanvasRenderingContext2D;
    private effects: Map<string, Effect> = new Map();
    private animationId: number | null = null;
    private lastTime: number = 0;

    // Performance monitoring
    private fps: number = 0;
    private frames: number = 0;
    private lastFpsTime: number = 0;

    constructor(canvas: HTMLCanvasElement) {
        this.canvas = canvas;
        const ctx = canvas.getContext('2d', {
            alpha: true,
            desynchronized: true // Low latency optimization
        });

        if (!ctx) throw new Error('Could not get 2D context');
        this.ctx = ctx;

        this.resize = this.resize.bind(this);
        this.loop = this.loop.bind(this);

        // Initial sizing
        this.resize();
        window.addEventListener('resize', this.resize);
    }

    public addEffect(effect: Effect) {
        this.effects.set(effect.id, effect);
        const dpr = window.devicePixelRatio || 1;
        effect.init(this.canvas.width / dpr, this.canvas.height / dpr);
    }

    public removeEffect(id: string) {
        this.effects.delete(id);
    }

    public getEffect<T extends Effect>(id: string): T | undefined {
        return this.effects.get(id) as T;
    }

    public start() {
        if (!this.animationId) {
            this.lastTime = performance.now();
            this.loop(this.lastTime);
        }
    }

    public stop() {
        if (this.animationId) {
            cancelAnimationFrame(this.animationId);
            this.animationId = null;
        }
    }

    public destroy() {
        this.stop();
        window.removeEventListener('resize', this.resize);
        this.effects.clear();
    }

    private resize() {
        const dpr = window.devicePixelRatio || 1;
        // Get CSS size
        const rect = this.canvas.getBoundingClientRect();

        // Set actual size in memory (scaled to account for extra pixel density)
        this.canvas.width = rect.width * dpr;
        this.canvas.height = rect.height * dpr;

        // Scale context to ensure correct drawing operations
        this.ctx.scale(dpr, dpr);

        // Notify effects
        // We pass logical CSS dimensions to effects for easier calculations
        const logicalWidth = rect.width;
        const logicalHeight = rect.height;

        this.effects.forEach(effect => effect.resize(logicalWidth, logicalHeight));
    }

    private loop(timestamp: number) {
        // Calculate delta time
        const dt = (timestamp - this.lastTime) / 1000;
        this.lastTime = timestamp;

        // Cap dt to prevent huge jumps if tab is inactive
        const safeDt = Math.min(dt, 0.1);

        // Update effects
        this.effects.forEach(effect => effect.update(safeDt));

        // Clear canvas
        // Note: We clear using logical dimensions because of the context scale
        const rect = this.canvas.getBoundingClientRect();
        this.ctx.clearRect(0, 0, rect.width, rect.height);

        // Draw effects
        // this.ctx.globalCompositeOperation = 'source-over'; // Default
        this.effects.forEach(effect => {
            this.ctx.save();
            effect.draw(this.ctx);
            this.ctx.restore();
        });

        // Debug FPS
        /*
        this.frames++;
        if (timestamp - this.lastFpsTime >= 1000) {
            this.fps = this.frames;
            this.frames = 0;
            this.lastFpsTime = timestamp;
            console.log('FPS:', this.fps);
        }
        */

        this.animationId = requestAnimationFrame(this.loop);
    }
}
