import { basename, dirname, join } from "path";
import { cwd, env } from "process";
import { exec, suite, test } from "uvu";
import { mkdirSync, readFileSync, writeFileSync } from "fs";

import { createInstrumenter } from "istanbul-lib-instrument";

env.FORCE_COLOR = "1";
const COVERAGE_VARIABLE = "__coverage__";

function instrumentCodeSync(relPathToCwd: string, codeSnippet: string) {
	const instrumenter = createInstrumenter({
		coverageVariable: COVERAGE_VARIABLE,
		esModules: true,
	});
	const instrumented = instrumenter.instrumentSync(codeSnippet, basename(relPathToCwd));
	return instrumented;
}

function instrumentFileSync(relPathToCwd: string) {
	const indexName = basename(relPathToCwd);
	const outputPath = join(cwd(), "coverage", relPathToCwd);
	const source = readFileSync(relPathToCwd, "utf8");
	const instrumentedSource = indexName === "package.json" ? source : instrumentCodeSync(relPathToCwd, source);
	mkdirSync(dirname(outputPath), { recursive: true });
	writeFileSync(outputPath, instrumentedSource, "utf8");
	return outputPath;
}

function instrumentSync(relPathToCwd: string, maybeCodeSnippet?: string) {
	if (maybeCodeSnippet) {
		return instrumentCodeSync(relPathToCwd, maybeCodeSnippet);
	} else {
		return instrumentFileSync(relPathToCwd);
	}
}

const _eval = (str: string) => new Function(str).call(0);
const _import = (path: string) => _eval(`return import('${path}')`);
const _filename = (): string => _eval("return __filename");
const _record = (): boolean => (_filename().endsWith(".js") ? COVERAGE_VARIABLE in global : true);

let _doneSym = Symbol("done");
let _suites = new Set();
let _suite = (name: string) => {
	if (!_record()) return;
	const s = suite(name);
	let _run = s.run;
	s.run = () => {
		if (_doneSym in s) return;
		Object.defineProperty(s, _doneSym, { value: true });
		_run();
	};
	_suites.add(s);
	return s;
};
let _test = (name: string, fn: any) => {
	if (!_record()) return;
	test(name, fn);
	_suites.add(test);
};

// auto run tests on exit
process.once("beforeExit", async () => {
	const _file = _filename();
	const _name = basename(_file);
	if (_name.endsWith(".js")) {
		const instrumented = instrumentSync(_file);
		await _import(instrumented);
	}
	// todo: this swallows exceptions that are thrown by the harness itself
	_suites.forEach((s: any) => s.run());
	Object.defineProperty(globalThis, "UVU_DEFER", { value: 1 });
	let idx = 0;
	Object.defineProperty(globalThis, "UVU_INDEX", { value: idx++ });
	if ("UVU_QUEUE" in globalThis) {
		// @ts-ignore
		globalThis["UVU_QUEUE"].push(["Test Results:"]);
	}
	await exec(false /* bail */).finally(() => {
		const coverage = COVERAGE_VARIABLE in global ? global[COVERAGE_VARIABLE] : {};
		const coverageDir = "coverage";
		const coverageFile = join(coverageDir, `${_name}.coverage.json`);
		const coverageData = JSON.stringify(coverage);
		writeFileSync(coverageFile, coverageData, "utf-8");
	});
});

export { _suite as suite, _test as test };
