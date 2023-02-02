export default {
	createRequire: function createRequire() {
		return () => globalThis.require;
	},
};
