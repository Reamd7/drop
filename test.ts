import assert from "uvu/assert";
import buffer from "buffer";
import crypto from "crypto";
import events from "events";
import fs from "fs";
import path from "path";
import stream from "stream";
import { suite } from "uvu";
import url from "url";
import util from "util";
import zlib from "zlib";

const urlTest = suite("basics");
urlTest("url module", () => {
	const keys = Object.keys(url);
	assert.ok(keys);
});

const zlibTest = suite("zlib");
zlibTest("zlib module", () => {
	const keys = Object.keys(zlib);
	assert.ok(keys);
});

const utilTest = suite("util");
utilTest("util module", () => {
	const keys = Object.keys(util);
	assert.ok(keys);
});

const streamTest = suite("stream");
streamTest("stream module", () => {
	const keys = Object.keys(stream);
	assert.ok(keys);
});

const pathTest = suite("path");
pathTest("path module", () => {
	const keys = Object.keys(path);
	assert.ok(keys);
});

const fsTest = suite("fs");
fsTest("fs module", () => {
	const keys = Object.keys(fs);
	assert.ok(keys);
});

const eventsTest = suite("events");
eventsTest("events module", () => {
	const keys = Object.keys(events);
	assert.ok(keys);
});

const cryptoTest = suite("crypto");
cryptoTest("crypto module", () => {
	const keys = Object.keys(crypto);
	assert.ok(keys);
	const md5 = crypto.createHash("md5");
	md5.update("hello world");
	assert.equal(md5.digest("hex"), "5eb63bbbe01eeed093cb22bb8f5acdc3");
});

const bufferTest = suite("buffer");
bufferTest("buffer module", () => {
	const keys = Object.keys(buffer);
	assert.ok(keys);
});

const cjsTest = suite("cjs");
cjsTest("cjs module", () => {
	const keys = require("./test.cjs");
	assert.ok(keys);
});
