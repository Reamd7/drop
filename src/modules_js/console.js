const _originalConsole = globalThis.console;

export function log(...args) {
	_originalConsole.log(...args);
}

export function error(...args) {
	_originalConsole.error(...args);
}

export function warn(...args) {
	console.log(...args);
}

export function info(...args) {
	console.log(...args);
}

export function debug(...args) {
	console.log(...args);
}

export function trace(...args) {
	console.log(...args);
}

export default {
	log,
	error,
	warn,
	info,
	debug,
	trace,
};
