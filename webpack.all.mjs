import AfterBuildPlugin from "@fiverr/afterbuild-webpack-plugin";
import CopyPlugin from "copy-webpack-plugin";
import ShebangPlugin from "webpack-shebang-plugin";
import TerserPlugin from "terser-webpack-plugin";
import { execSync } from "child_process";
import path from "path";
import webpack from "webpack";

const OUT_DIR = path.resolve("dist");
execSync(`rm -rf ${OUT_DIR}`);

/** @type {webpack.Configuration} */
const libConfig = {
	devtool: false,
	mode: "production",
	entry: "./src/npm/index.ts",
	output: {
		path: OUT_DIR,
		library: {
			commonjs: "drop",
			amd: "drop",
			root: "DROP",
		},
		libraryTarget: "umd",
		umdNamedDefine: true,
		globalObject: `(typeof self !== 'undefined' ? self : this)`,
		filename: "index.js",
	},
	node: {
		global: false,
		__filename: false,
		__dirname: false,
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
	performance: {
		hints: false,
	},
	externalsPresets: {
		node: true,
	},
	externals: {
		bufferExt: "buffer",
		consoleExt: "console",
		processExt: "process",
		tty: "tty",
	},
	plugins: [
		new AfterBuildPlugin(() => {
			execSync(`cp src/npm/index.d.ts ${OUT_DIR}`);
		}),
		new CopyPlugin({
			patterns: [
				{ from: "drop.wasm", to: OUT_DIR, context: "target/wasm32-wasi/release" },
				{ from: "README.md", to: OUT_DIR },
				{ from: "LICENSE", to: OUT_DIR },
				{
					from: "package.json",
					to: path.join(OUT_DIR, "package.json"),
					transform: (content) => {
						const packageJson = JSON.parse(content);
						packageJson.bin = "./bin.js";
						packageJson.main = "./index.js";
						packageJson.types = "./index.d.ts";
						packageJson.devDependencies = undefined;
						packageJson.scripts = undefined;
						return JSON.stringify(packageJson, null, 2);
					},
				},
			],
		}),
		new webpack.ProvidePlugin({
			process: path.resolve("./src/npm/process.js"),
			console: "consoleExt",
			Buffer: ["bufferExt", "Buffer"],
			Url: ["url", "Url"],
		}),
	],
	module: {
		rules: [
			{
				test: /\.(ts|tsx)$/i,
				loader: "ts-loader",
				exclude: ["/node_modules/"],
			},
			{
				test: /out\/web\/busybox\.js$/,
				loader: "string-replace-loader",
				options: {
					multiple: [
						{
							search: /if\s?\(e instanceof ExitStatus\)\s?return;/g,
							replace:
								"if (e instanceof ExitStatus || (e instanceof WebAssembly.RuntimeError && e.message === 'unreachable')) return;",
						},
					],
				},
			},
			{
				test: /\.wasm$/,
				loader: "url-loader",
				options: {
					mimetype: "delete/me",
					limit: 15 * 1024 * 1024,
					// this removes the "data:<whatever>;base64," from the output bundle
					generator: (content) => content.toString("base64"),
				},
			},
		],
	},
	resolve: {
		extensions: [".tsx", ".ts", ".jsx", ".js"],
		fallback: {
			tty: false
		}
	},
};

const binConfig = { ...libConfig };
binConfig.entry = "./src/npm/bin.ts";
binConfig.plugins = [new ShebangPlugin()];
binConfig.target = "node";
binConfig.output = {
	path: OUT_DIR,
	filename: "bin.js",
};

export default [libConfig, binConfig];
