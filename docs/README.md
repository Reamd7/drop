[@zorse/drop](README.md) / Exports

# @zorse/drop

## Table of contents

### Interfaces

- [RunOptions](interfaces/RunOptions.md)
- [Runner](interfaces/Runner.md)

### Type Aliases

- [ABIVariant](modules.md#abivariant)
- [ExecCommand](modules.md#execcommand)

### Functions

- [exec](modules.md#exec)
- [run](modules.md#run)
- [runBusy](modules.md#runbusy)
- [runDrop](modules.md#rundrop)

## Type Aliases

### ABIVariant

Ƭ **ABIVariant**: ``"node"`` \| ``"web"``

Drop ABI variation

#### Defined in

[index.ts:9](https://github.com/zorse-lang/drop/blob/19c1cc1/src/npm/index.ts#L9)

___

### ExecCommand

Ƭ **ExecCommand**: ``"base64"`` \| ``"basename"`` \| ``"cat"`` \| ``"chmod"`` \| ``"chown"`` \| ``"clear"`` \| ``"cp"`` \| ``"date"`` \| ``"diff"`` \| ``"echo"`` \| ``"egrep"`` \| ``"env"`` \| ``"false"`` \| ``"fgrep"`` \| ``"find"`` \| ``"grep"`` \| ``"head"`` \| ``"link"`` \| ``"ln"`` \| ``"ls"`` \| ``"md5sum"`` \| ``"mkdir"`` \| ``"mktemp"`` \| ``"mv"`` \| ``"nanozip"`` \| ``"patch"`` \| ``"printenv"`` \| ``"printf"`` \| ``"pwd"`` \| ``"readlink"`` \| ``"realpath"`` \| ``"rm"`` \| ``"rmdir"`` \| ``"sed"`` \| ``"sha256sum"`` \| ``"sleep"`` \| ``"sort"`` \| ``"stat"`` \| ``"tail"`` \| ``"tar"`` \| ``"test"`` \| ``"touch"`` \| ``"true"`` \| ``"uniq"`` \| ``"unlink"`` \| ``"unzip"`` \| ``"whoami"`` \| ``"xargs"`` \| ``"drop"`` \| ``"node"`` \| ``"zip"`` \| ``"busybox"``

All available commands

#### Defined in

[index.ts:155](https://github.com/zorse-lang/drop/blob/19c1cc1/src/npm/index.ts#L155)

## Functions

### exec

▸ **exec**(`cmd`, `...args`): [`Promise`]( https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise )<`void`\>

Convenience function to run an either a Drop or BusyBox command

**`Example`**

```ts
await exec("node", "index.ts");
```

**`Example`**

```ts
await exec("ls", "-la");
```

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `cmd` | [`ExecCommand`](modules.md#execcommand) | Command to run |
| `...args` | `string`[] | Arguments to pass to the command |

#### Returns

[`Promise`]( https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise )<`void`\>

Runner to execute the command

#### Defined in

[index.ts:223](https://github.com/zorse-lang/drop/blob/19c1cc1/src/npm/index.ts#L223)

___

### run

▸ **run**(`opts`): [`Promise`]( https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise )<[`Runner`](interfaces/Runner.md)\>

Run a command (Drop or BusyBox)

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `opts` | [`RunOptions`](interfaces/RunOptions.md) | Options to run the command |

#### Returns

[`Promise`]( https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise )<[`Runner`](interfaces/Runner.md)\>

Runner to execute the command

#### Defined in

[index.ts:143](https://github.com/zorse-lang/drop/blob/19c1cc1/src/npm/index.ts#L143)

___

### runBusy

▸ **runBusy**(`opts`): [`Promise`]( https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise )<[`Runner`](interfaces/Runner.md)\>

Run a BusyBox command (POSIX subset emulation)

**`Example`**

```ts
const { exec } = await runBusy({ args: ["ls", "-la"] });
exec();
```

**`Example`**

```ts
const { exec } = await runBusy({ args: ["zip", "archive.zip", "file.txt"] });
exec();
```

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `opts` | [`RunOptions`](interfaces/RunOptions.md) | Options to run the command |

#### Returns

[`Promise`]( https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise )<[`Runner`](interfaces/Runner.md)\>

Runner to execute the command

#### Defined in

[index.ts:114](https://github.com/zorse-lang/drop/blob/19c1cc1/src/npm/index.ts#L114)

___

### runDrop

▸ **runDrop**(`opts`): [`Promise`]( https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise )<[`Runner`](interfaces/Runner.md)\>

Run a Drop command (NodeJS subset emulation)

**`Example`**

```ts
const { exec } = await runDrop({ file: "index.ts" });
exec();
```

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `opts` | [`RunOptions`](interfaces/RunOptions.md) | Options to run the command |

#### Returns

[`Promise`]( https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise )<[`Runner`](interfaces/Runner.md)\>

Runner to execute the command

#### Defined in

[index.ts:63](https://github.com/zorse-lang/drop/blob/19c1cc1/src/npm/index.ts#L63)
