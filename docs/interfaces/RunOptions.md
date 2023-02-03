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
| `print?` | (`str`: `string`) => `void` | Print to stdout |
| `printErr?` | (`str`: `string`) => `void` | Print to stderr |

#### Defined in

[index.ts:30](https://github.com/zorse-lang/drop/blob/19c1cc1/src/npm/index.ts#L30)

___

### tty

• `Optional` `Readonly` **tty**: `boolean`

Whether to run in a TTY (default: true)

#### Defined in

[index.ts:41](https://github.com/zorse-lang/drop/blob/19c1cc1/src/npm/index.ts#L41)

___

### variant

• `Optional` `Readonly` **variant**: [`ABIVariant`](../modules.md#abivariant)

ABI variant to use

#### Defined in

[index.ts:39](https://github.com/zorse-lang/drop/blob/19c1cc1/src/npm/index.ts#L39)
