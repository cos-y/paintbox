export const clamp = (val: number, min: number, max: number) => Math.max(min, Math.min(val, max));

export const similarity = (deltaE: number) => clamp(1 - deltaE, 0, 1) * 100;
