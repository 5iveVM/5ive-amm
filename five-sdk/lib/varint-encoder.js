/**
 * Varint parameter encoder used by the SDK execution path.
 */
export class VarintEncoder {
    static encodeNumber(value) {
        if (!Number.isFinite(value) || value < 0) {
            throw new Error(`Varint encoding requires a non-negative finite number, got: ${value}`);
        }
        const bytes = [];
        let num = Math.floor(value);
        while (num >= 0x80) {
            bytes.push((num & 0x7f) | 0x80);
            num >>>= 7;
        }
        bytes.push(num & 0x7f);
        return new Uint8Array(bytes);
    }

    static encodeValue(value) {
        if (typeof value === "boolean") {
            return this.encodeNumber(value ? 1 : 0);
        }
        if (typeof value === "number") {
            return this.encodeNumber(value);
        }
        if (typeof value === "bigint") {
            if (value < 0n) {
                throw new Error(`Varint encoding does not support negative bigint: ${value.toString()}`);
            }
            return this.encodeNumber(Number(value));
        }
        if (typeof value === "string") {
            const bytes = new TextEncoder().encode(value);
            return this.concat([this.encodeNumber(bytes.length), bytes]);
        }
        if (value instanceof Uint8Array) {
            return this.concat([this.encodeNumber(value.length), value]);
        }
        throw new Error(`Unsupported parameter type for varint encoding: ${typeof value}`);
    }

    static concat(parts) {
        const total = parts.reduce((sum, p) => sum + p.length, 0);
        const out = new Uint8Array(total);
        let offset = 0;
        for (const part of parts) {
            out.set(part, offset);
            offset += part.length;
        }
        return out;
    }

    /**
     * Encode execution parameters as:
     * [param_count(varint), param_0(varint/blob), ..., param_n(varint/blob)]
     */
    static async encodeExecute(_functionIndex, paramDefs, paramValues) {
        const values = (paramDefs || []).map((def) => paramValues?.[def.name]);
        const encoded = [this.encodeNumber(values.length)];
        for (const value of values) {
            encoded.push(this.encodeValue(value));
        }
        return this.concat(encoded);
    }
}
