import { Effect } from '../Effect';

interface Beam {
    id: number;
    x: number;
    y: number; // Percentage 0-100 relative to viewport height? Or pixels? Let's use relative for consistency
    width: number;
    speed: number;
    direction: 'left' | 'right';
    color: string;
    delay: number;
    active: boolean;
    complete: boolean;
}

const COLORS = [
    '#eb6f92', // Rose
    '#c4a7e7', // Iris
    '#3e8fb0', // Pine
    '#9ccfd8', // Foam
];

export class BeamSystem implements Effect {
    id = 'beams';
    private beams: Beam[] = [];
    private width: number = 0;
    private height: number = 0;
    private onImpact: (() => void) | null = null;
    private impacted: boolean = false;
    private time: number = 0;

    constructor(onImpactCallback?: () => void) {
        this.onImpact = onImpactCallback || null;
    }

    init(width: number, height: number) {
        this.width = width;
        this.height = height;
        this.generateBeams();
    }

    resize(width: number, height: number) {
        this.width = width;
        this.height = height;
    }

    generateBeams() {
        this.beams = [];
        this.time = 0;
        this.impacted = false;

        // 8 Left Beams
        for (let i = 0; i < 8; i++) {
            this.beams.push({
                id: i,
                x: -500, // Start off screen
                y: 0.40 + (i * 0.02) + (Math.random() * 0.1), // 40% - 60% height
                width: 100 + Math.random() * 200,
                speed: 1500 + Math.random() * 1000, // High speed pixels/sec
                direction: 'left', // Originating from left
                color: COLORS[i % COLORS.length],
                delay: Math.random() * 0.5,
                active: false,
                complete: false
            });
        }

        // 8 Right Beams
        for (let i = 0; i < 8; i++) {
            this.beams.push({
                id: i + 8,
                x: 9999, // Will be set in update
                y: 0.42 + (i * 0.02) + (Math.random() * 0.1),
                width: 100 + Math.random() * 200,
                speed: 1500 + Math.random() * 1000,
                direction: 'right', // Originating from right
                color: COLORS[(i + 2) % COLORS.length],
                delay: Math.random() * 0.5,
                active: false,
                complete: false
            });
        }
    }

    update(dt: number) {
        if (this.impacted) {
            // Even after impact, let trailing beams finish? 
            // Or just stop them? Let's animate them out or stop them.
            // For now, let's keep them running until they fly off
        }

        this.time += dt;

        let activeCount = 0;
        let completeCount = 0;

        for (const beam of this.beams) {
            // Check delay
            if (this.time < beam.delay) continue;

            beam.active = true;

            if (beam.direction === 'left') {
                // Initialize x if just started (though currently x starts at -500)

                beam.x += beam.speed * dt;

                // Check if crossed center
                if (!beam.complete && beam.x > this.width / 2) {
                    this.triggerImpact();
                    beam.complete = true;
                }
            } else {
                // Initialize x for right beams if needed
                if (beam.x > this.width + 1000) beam.x = this.width + beam.width; // Reset to right side

                beam.x -= beam.speed * dt;

                // Check if crossed center
                if (!beam.complete && beam.x < this.width / 2) {
                    this.triggerImpact();
                    beam.complete = true;
                }
            }

            // Visual cleanup if way off screen
            // ...
        }
    }

    triggerImpact() {
        if (!this.impacted && this.onImpact) {
            this.impacted = true;
            this.onImpact();
        }
    }

    draw(ctx: CanvasRenderingContext2D) {
        if (this.beams.length === 0) return;

        // Additive blending
        ctx.globalCompositeOperation = 'lighter';

        for (const beam of this.beams) {
            if (!beam.active) continue;

            const yPos = beam.y * this.height;

            // Draw beam
            // Create gradient
            const grad = ctx.createLinearGradient(beam.x, yPos, beam.x + beam.width, yPos);

            if (beam.direction === 'left') {
                grad.addColorStop(0, 'transparent');
                grad.addColorStop(0.8, beam.color);
                grad.addColorStop(1, 'white');
            } else {
                grad.addColorStop(0, 'white');
                grad.addColorStop(0.2, beam.color);
                grad.addColorStop(1, 'transparent');
            }

            ctx.fillStyle = grad;
            ctx.shadowBlur = 10;
            ctx.shadowColor = beam.color;

            // Draw rounded rect (simplified as rect for speed)
            ctx.beginPath();
            ctx.rect(beam.x, yPos, beam.width, 2); // 2px height
            ctx.fill();
        }

        ctx.globalCompositeOperation = 'source-over';
        ctx.shadowBlur = 0;
    }
}
