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

[index.ts:37](https://github.com/zorse-lang/drop/blob/93e72f8/src/npm/index.ts#L37)

## Methods

### exec

▸ **exec**(): [`Promise`]( https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise )<`number`\>

Execute the command

#### Returns

[`Promise`]( https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise )<`number`\>

#### Defined in

[index.ts:39](https://github.com/zorse-lang/drop/blob/93e72f8/src/npm/index.ts#L39)
