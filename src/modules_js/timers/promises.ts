import { setImmediate as _setImmediate, setInterval as _setInterval, setTimeout as _setTimeout } from "timers";

export function setTimeout(callback: (...args: unknown[]) => void, delay: number, ...args: unknown[]): Promise<void> {
	return new Promise((resolve) => {
		_setTimeout(() => {
			callback(...args);
			resolve();
		}, delay);
	});
}

export function setInterval(callback: (...args: unknown[]) => void, delay: number, ...args: unknown[]): Promise<void> {
	return new Promise((resolve) => {
		_setInterval(() => {
			callback(...args);
			resolve();
		}, delay);
	});
}

export function setImmediate(callback: (...args: unknown[]) => void, ...args: unknown[]): Promise<void> {
	return new Promise((resolve) => {
		_setImmediate(() => {
			callback(...args);
			resolve();
		});
	});
}

export default {
	setTimeout,
	setInterval,
	setImmediate,
};
