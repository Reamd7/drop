import TerserPlugin from "terser-webpack-plugin";
import path from "path";
import webpack from "webpack";

const ALL_PACKAGES = ["crypto", "memfs", "zlib", "chai", "uvu", "sinon", "rc", "nyc"];

const createConfig = (name) => {
	/** @type {webpack.Configuration} */
	const config = {
		mode: "production",
		entry: `./src/modules_js/${name}.js.in`,
		devtool: false,
		target: name === "crypto" ? "web" : "node",
		experiments: {
			outputModule: true,
		},
		output: {
			path: path.resolve("src/modules_js"),
			filename: `${name}.js`,
			chunkFormat: "module",
			library: {
				type: "module",
			},
		},
		externals: {
			"safer-buffer": "buffer",
			"safe-buffer": "buffer",
			...[
				"assert",
				"buffer",
				"events",
				"fs",
				"os",
				"path",
				"process",
				"punycode",
				"querystring",
				"readline",
				"stream",
				"string_decoder",
				"timers",
				"module",
				"tty",
				"url",
				"util",
			]
				.concat(ALL_PACKAGES.filter((p) => p !== name))
				.reduce((acc, curr) => ((acc[curr] = curr), acc), {}),
		},
		performance: {
			hints: false,
		},
		optimization: {
			nodeEnv: false,
			minimize: true,
			minimizer: [
				new TerserPlugin({
					extractComments: false,
					terserOptions: {
						format: {
							comments: false,
						},
					},
				}),
			],
		},
		plugins: [
			new webpack.ProgressPlugin(),
			new webpack.ProvidePlugin({
				Url: ["url", "Url"],
				Buffer: ["buffer", "Buffer"],
				process: "process",
				setTimeout: ["timers", "setTimeout"],
				clearTimeout: ["timers", "clearTimeout"],
				setInterval: ["timers", "setInterval"],
				clearInterval: ["timers", "clearInterval"],
				setImmediate: ["timers", "setImmediate"],
				clearImmediate: ["timers", "clearImmediate"],
			}),
			new webpack.IgnorePlugin({
				contextRegExp: /@babel\/core/,
				resourceRegExp: /import\.cjs/,
			}),
		],
		resolve: {
			extensions: [".js", ".js.in"],
			fallback: {
				"safer-buffer": "buffer",
				"safe-buffer": "buffer",
				child_process: false,
				net: false,
			},
		},
		module: {
			rules: [
				{
					test: /browserslist\/node/,
					loader: "string-replace-loader",
					options: {
						multiple: [
							{
								search: /require\(require\.resolve/g,
								replace: "_non_webpack_require_(_non_webpack_require_.resolve",
								strict: true,
							},
						],
					},
				},
				{
					test: [/@babel\/core\/lib\/config\/files\/configuration\.js/, /@babel\/core\/lib\/config\/files\/plugins\.js/],
					loader: "string-replace-loader",
					options: {
						multiple: [
							{
								search: /require\.resolve/g,
								replace: "_non_webpack_require_.resolve",
								strict: true,
							},
						],
					},
				},
				{
					test: [/@babel\/core\/lib\/config\/files\/module-types\.js/],
					loader: "string-replace-loader",
					options: {
						multiple: [
							{
								search: /\(require\)/g,
								replace: "(_non_webpack_require_)",
								strict: true,
							},
						],
					},
				},
			],
		},
		node: {
			global: true,
			__dirname: true,
			__filename: true,
		},
	};
	return config;
};

export default ALL_PACKAGES.map(createConfig);
