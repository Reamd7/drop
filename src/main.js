(async () => {
	const { require: _require } = await import("commonjs");
	globalThis.require = _require;
})();
(async () => {
	const _process = await import("process");
	globalThis.process = _process;
})();
(async () => {
	const _console = await import("console");
	globalThis.console = _console;
})();
(async () => {
	const _timers = await import("timers");
	// setTimeout and clearTimeout are already defined in the global scope
	globalThis.setImmediate = _timers.setImmediate;
	globalThis.clearImmediate = _timers.clearImmediate;
	globalThis.setInterval = _timers.setInterval;
	globalThis.clearInterval = _timers.clearInterval;
})();
globalThis.global = globalThis;
