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
globalThis.global = globalThis;
