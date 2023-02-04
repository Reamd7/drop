[@zorse/drop](../README.md) / [Exports](../modules.md) / RunOptions

# Interface: RunOptions

Base options to run a command in Drop/BusyBox

## Table of contents

### Properties

- [Module](RunOptions.md#module)
- [tty](RunOptions.md#tty)
- [variant](RunOptions.md#variant)

## Properties

### Module

• `Readonly` **Module**: `Object`

Module option accepted by EMCC runtime / Rust runtime

#### Type declaration

| Name | Type | Description |
| :------ | :------ | :------ |
| `arguments` | `string`[] | Command line arguments |
| `fs?` | `any` | Platform specific pre-constructed FileSystem object |
| `print?` | (`str`: `string`) => `void` | Print to stdout |
| `printErr?` | (`str`: `string`) => `void` | Print to stderr |

#### Defined in

[index.ts:44](https://github.com/zorse-lang/drop/blob/7de51ed/src/npm/index.ts#L44)

___

### tty

• `Optional` `Readonly` **tty**: `boolean`

Whether to run in a TTY (default: true)

#### Defined in

[index.ts:57](https://github.com/zorse-lang/drop/blob/7de51ed/src/npm/index.ts#L57)

___

### variant

• `Optional` `Readonly` **variant**: [`ABIVariant`](../modules.md#abivariant)

ABI variant to use

#### Defined in

[index.ts:55](https://github.com/zorse-lang/drop/blob/7de51ed/src/npm/index.ts#L55)
