(async () => {
	const { require: _require } = await import("commonjs");
	globalThis.require = _require;
})();
(async () => {
	const _process = await import("process");
	globalThis.process = _process;
})();
globalThis.global = globalThis;
