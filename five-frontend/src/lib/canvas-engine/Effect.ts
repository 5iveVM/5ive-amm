export interface Effect {
    id: string;

    /**
     * Called once when added to the engine
     */
    init(width: number, height: number): void;

    /**
     * Called when canvas resizes
     */
    resize(width: number, height: number): void;

    /**
     * Called every frame to update state
     * @param dt Delta time in seconds
     */
    update(dt: number): void;

    /**
     * Called every frame to render
     */
    draw(ctx: CanvasRenderingContext2D): void;
}
