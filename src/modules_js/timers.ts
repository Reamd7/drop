// we get setTimeout and clearTimeout from Rust, the rest is JS
export const setTimeout = globalThis.setTimeout;
export const clearTimeout = globalThis.clearTimeout;

// implementation of setInterval and clearInterval on top of setTimeout and clearTimeout
export function setInterval(callback: (...args: unknown[]) => void, delay: number, ...args: unknown[]): number {
	const intervalId = setTimeout(() => {
		callback(...args);
		clearInterval(intervalId as any);
		setInterval(callback, delay, ...args);
	}, delay) as any as number;
	return intervalId;
}

export function clearInterval(intervalId: number): void {
	clearTimeout(intervalId);
}

// implementation of setImmediate and clearImmediate on top of setTimeout and clearTimeout
export function setImmediate(callback: (...args: unknown[]) => void, ...args: unknown[]): number {
	const immediateId = setTimeout(() => {
		callback(...args);
		clearImmediate(immediateId as any);
	}, 0) as any as number;
	return immediateId;
}

export function clearImmediate(immediateId: number): void {
	clearTimeout(immediateId);
}
