export const clamp = (val: number, min: number, max: number) => Math.max(min, Math.min(val, max));

export const similarity = (deltaE: number) => clamp(100 - deltaE * 4, 0, 100);
