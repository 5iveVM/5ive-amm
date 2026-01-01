import { Effect } from '../Effect';

interface Particle {
    x: number;
    y: number;
    vx: number;
    vy: number;
    size: number;
    color: string;
    life: number;
    maxLife: number;
    drag: number;
}

const COLORS = [
    '#eb6f92', // Rose
    '#c4a7e7', // Iris
    '#3e8fb0', // Pine
    '#9ccfd8', // Foam
    '#f6c177', // Gold
    '#ea9a97', // Love
];

export class ParticleSystem implements Effect {
    id = 'particles';
    private particles: Particle[] = [];
    private width: number = 0;
    private height: number = 0;
    private exploded: boolean = false;

    init(width: number, height: number) {
        this.width = width;
        this.height = height;
    }

    resize(width: number, height: number) {
        this.width = width;
        this.height = height;
    }

    triggerExplosion() {
        if (this.exploded) return;
        this.exploded = true;

        // Spawn 500 particles
        for (let i = 0; i < 500; i++) {
            const angle = (Math.PI * 2 * i) / 500;
            // Higher speed for more impact
            const speed = Math.random() * 800 + 200;

            this.particles.push({
                x: this.width / 2,
                y: this.height / 2,
                vx: Math.cos(angle) * speed,
                vy: Math.sin(angle) * speed,
                size: Math.random() > 0.8 ? 3 : 1.5,
                color: COLORS[i % COLORS.length],
                life: 1,
                maxLife: 1,
                // Varied drag for depth
                drag: 0.92 + Math.random() * 0.04
            });
        }
    }

    reset() {
        this.particles = [];
        this.exploded = false;
    }

    update(dt: number) {
        if (!this.exploded) return;

        // Update loop backwards for easy removal
        for (let i = this.particles.length - 1; i >= 0; i--) {
            const p = this.particles[i];

            // Physics
            p.x += p.vx * dt;
            p.y += p.vy * dt;

            // Drag
            p.vx *= p.drag;
            p.vy *= p.drag;

            // Life decay (simulated easing)
            // Slower decay at start (physically correct? no, but looks good)
            p.life -= dt * 0.3; // ~3 seconds duration

            if (p.life <= 0) {
                this.particles.splice(i, 1);
            }
        }
    }

    draw(ctx: CanvasRenderingContext2D) {
        if (this.particles.length === 0) return;

        // Additive blending for neon glow
        ctx.globalCompositeOperation = 'lighter';

        for (const p of this.particles) {
            ctx.globalAlpha = Math.max(0, p.life); // Fade out
            ctx.fillStyle = p.color;
            ctx.shadowBlur = 15;
            ctx.shadowColor = p.color;

            ctx.beginPath();
            ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
            ctx.fill();
        }

        // Reset context
        ctx.globalCompositeOperation = 'source-over';
        ctx.shadowBlur = 0;
        ctx.globalAlpha = 1;
    }
}
