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

[index.ts:36](https://github.com/zorse-lang/drop/blob/7de51ed/src/npm/index.ts#L36)

## Methods

### exec

▸ **exec**(): `number` \| [`Promise`]( https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise )<`number`\>

Execute the command

#### Returns

`number` \| [`Promise`]( https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise )<`number`\>

#### Defined in

[index.ts:38](https://github.com/zorse-lang/drop/blob/7de51ed/src/npm/index.ts#L38)
