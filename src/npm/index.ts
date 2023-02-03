// @ts-ignore
import DROP_WASM_BASE64 from "../../target/wasm32-wasi/release/drop.wasm";
import WASI from "wasi-js";
import atob from "atob-lite";
// this says "bindings/node", but we externalize all its dependencies
import bindings from "wasi-js/dist/bindings/node";

/** Drop ABI variation */
export type ABIVariant = "node" | "web";

function getDefaultABIVariant(): ABIVariant {
	if (typeof window !== "undefined" || typeof postMessage !== "undefined") {
		return "web";
	} else {
		return "node";
	}
}

/** Runner lets you decide late execution of of commands */
export interface Runner {
	/** Underlying native instance */
	readonly instance: object;
	/** Execute the command */
	exec(): void | Promise<void>;
}

/** Base options to run a command in Drop/BusyBox */
export interface RunOptions {
	/** Module option accepted by EMCC runtime / Rust runtime */
	readonly Module: {
		/** Print to stdout */
		print?: (str: string) => void;
		/** Print to stderr */
		printErr?: (str: string) => void;
		/** Command line arguments */
		arguments: string[];
	};
	/** ABI variant to use */
	readonly variant?: ABIVariant;
	/** Whether to run in a TTY (default: true) */
	readonly tty?: boolean;
}

function decode(encoded: string) {
	const binaryString = atob(encoded);
	const bytes = new Uint8Array(binaryString.length);
	for (let i = 0; i < binaryString.length; i++) {
		bytes[i] = binaryString.charCodeAt(i);
	}
	return bytes.buffer;
}

/**
 * Run a Drop command (NodeJS subset emulation)
 * @param opts Options to run the command
 * @returns Runner to execute the command
 * @example
 * ```ts
 * const { exec } = await runDrop({ file: "index.ts" });
 * exec();
 * ```
 */
export async function runDrop(opts: RunOptions): Promise<Runner> {
	const variant = opts.variant || getDefaultABIVariant();
	const file = opts.Module.arguments.find((arg) => !arg.startsWith("-"));
	const newArgs = opts.Module.arguments.filter((arg) => !arg.startsWith("-"));
	const sharedOpts = {
		preopens: { [process.cwd()]: ".", ".": "." },
		args: ["drop", file, ...newArgs],
		env: process.env,
	};
	let NodeWASI: typeof import("wasi").WASI;
	const dimport = (x: string) => new Function(`return import(${JSON.stringify(x)})`).call(0);
	if (variant === "node") {
		const wasi = await dimport("wasi");
		NodeWASI = wasi.WASI;
	}
	const wasi =
		variant === "node"
			? new NodeWASI({ returnOnExit: true, ...sharedOpts })
			: new WASI({
					bindings: { ...bindings, isTTY: () => opts.tty ?? true },
					...sharedOpts,
					sendStdout: (buf) => opts.Module?.print?.(buf.toString()),
					sendStderr: (buf) => opts.Module?.printErr?.(buf.toString()),
			  });
	const importObject = { wasi_snapshot_preview1: wasi.wasiImport };
	const buffer = decode(DROP_WASM_BASE64);
	const wasm = await WebAssembly.compile(buffer);
	const instance = await WebAssembly.instantiate(wasm, importObject);
	return {
		instance,
		exec: () => {
			wasi.start(instance);
		},
	};
}

/**
 * Run a BusyBox command (POSIX subset emulation)
 * @param opts Options to run the command
 * @returns Runner to execute the command
 * @example
 * ```ts
 * const { exec } = await runBusy({ args: ["ls", "-la"] });
 * exec();
 * ```
 * @example
 * ```ts
 * const { exec } = await runBusy({ args: ["zip", "archive.zip", "file.txt"] });
 * exec();
 * ```
 */
export async function runBusy(opts: RunOptions): Promise<Runner> {
	const cmd = opts.Module.arguments?.[0] ?? "--help";
	const newArgs = opts.Module.arguments?.slice(1);
	const oldProcArgv = process.argv;
	process.argv = ["drop", cmd === "zip" ? "nanozip" : cmd];
	const variant = opts.variant || getDefaultABIVariant();
	const factory = require("../../out/" + variant + "/busybox.js");
	const instance = await factory(opts.Module);
	return {
		instance,
		exec: () => {
			const result = instance.callMain(newArgs);
			if (result instanceof Promise) {
				result.finally(() => {
					process.argv = oldProcArgv;
				});
			} else {
				process.argv = oldProcArgv;
			}
			return result;
		},
	};
}

/**
 * Run a command (Drop or BusyBox)
 * @param opts Options to run the command
 * @returns Runner to execute the command
 */
export async function run(opts: RunOptions): Promise<Runner> {
	const bin = opts.Module.arguments?.[0];
	if (bin === "drop" || bin === "node") {
		const _optsCopy = { ...opts };
		_optsCopy.Module.arguments = _optsCopy.Module.arguments.slice(1);
		return await runDrop(_optsCopy);
	} else {
		return await runBusy(opts);
	}
}

/** All available commands */
export type ExecCommand =
	| "base64"
	| "basename"
	| "cat"
	| "chmod"
	| "chown"
	| "clear"
	| "cp"
	| "date"
	| "diff"
	| "echo"
	| "egrep"
	| "env"
	| "false"
	| "fgrep"
	| "find"
	| "grep"
	| "head"
	| "link"
	| "ln"
	| "ls"
	| "md5sum"
	| "mkdir"
	| "mktemp"
	| "mv"
	| "nanozip"
	| "patch"
	| "printenv"
	| "printf"
	| "pwd"
	| "readlink"
	| "realpath"
	| "rm"
	| "rmdir"
	| "sed"
	| "sha256sum"
	| "sleep"
	| "sort"
	| "stat"
	| "tail"
	| "tar"
	| "test"
	| "touch"
	| "true"
	| "uniq"
	| "unlink"
	| "unzip"
	| "whoami"
	| "xargs"
	| "drop"
	| "node"
	| "zip"
	| "busybox";

/**
 * Convenience function to run an either a Drop or BusyBox command
 * @param cmd Command to run
 * @param args Arguments to pass to the command
 * @returns Runner to execute the command
 * @example
 * ```ts
 * await exec("node", "index.ts");
 * ```
 * @example
 * ```ts
 * await exec("ls", "-la");
 * ```
 */
export async function exec(cmd: ExecCommand, ...args: string[]): Promise<void> {
	switch (cmd) {
		case "drop":
		case "node": {
			return await (await run({ Module: { arguments: ["node", ...args] } })).exec();
		}
		default: {
			return await (await run({ Module: { arguments: [cmd, ...args] } })).exec();
		}
	}
}
