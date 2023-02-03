[@zorse/drop](../README.md) / [Exports](../modules.md) / Runner

# Interface: Runner

Runner lets you decide late execution of of commands

## Table of contents

### Properties

- [instance](Runner.md#instance)

### Methods

- [exec](Runner.md#exec)

## Properties

### instance

• `Readonly` **instance**: `object`

Underlying native instance

#### Defined in

[index.ts:22](https://github.com/zorse-lang/drop/blob/19c1cc1/src/npm/index.ts#L22)

## Methods

### exec

▸ **exec**(): `void` \| [`Promise`]( https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise )<`void`\>

Execute the command

#### Returns

`void` \| [`Promise`]( https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise )<`void`\>

#### Defined in

[index.ts:24](https://github.com/zorse-lang/drop/blob/19c1cc1/src/npm/index.ts#L24)
