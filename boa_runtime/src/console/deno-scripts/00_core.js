// Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.
// https://github.com/denoland/deno/blob/main/ext/node/polyfills/internal/primordials.mjs
// https://github.com/denoland/deno/blob/main/ext/node/polyfills/internal_binding/util.ts

const ALL_PROPERTIES = 0;
const ONLY_WRITABLE = 1;
const ONLY_ENUMERABLE = 2;
const ONLY_CONFIGURABLE = 4;
const ONLY_ENUM_WRITABLE = 6;
const SKIP_STRINGS = 8;
const SKIP_SYMBOLS = 16;
const isNumericLookup = {};
function isArrayIndex(value) {
  switch (typeof value) {
    case "number":
      return value >= 0 && (value | 0) === value;
    case "string": {
      const result = isNumericLookup[value];
      if (result !== void 0) {
        return result;
      }
      const length = value.length;
      if (length === 0) {
        return (isNumericLookup[value] = false);
      }
      let ch = 0;
      let i = 0;
      for (; i < length; ++i) {
        ch = value.charCodeAt(i);
        if ((i === 0 && ch === 48 && length > 1) || ch < 48 || ch > 57) {
          return (isNumericLookup[value] = false);
        }
      }
      return (isNumericLookup[value] = true);
    }
    default:
      return false;
  }
}
function getOwnNonIndexProperties(obj, filter) {
  let allProperties = [
    ...Object.getOwnPropertyNames(obj),
    ...Object.getOwnPropertySymbols(obj),
  ];
  if (Array.isArray(obj)) {
    allProperties = allProperties.filter((k) => !isArrayIndex(k));
  }
  if (filter === ALL_PROPERTIES) {
    return allProperties;
  }
  const result = [];
  for (const key of allProperties) {
    const desc = Object.getOwnPropertyDescriptor(obj, key);
    if (desc === void 0) {
      continue;
    }
    if (filter & ONLY_WRITABLE && !desc.writable) {
      continue;
    }
    if (filter & ONLY_ENUMERABLE && !desc.enumerable) {
      continue;
    }
    if (filter & ONLY_CONFIGURABLE && !desc.configurable) {
      continue;
    }
    if (filter & SKIP_STRINGS && typeof key === "string") {
      continue;
    }
    if (filter & SKIP_SYMBOLS && typeof key === "symbol") {
      continue;
    }
    result.push(key);
  }
  return result;
}

const internals = {};
const primordials = {};

primordials.ArrayBufferPrototypeGetByteLength = (that) => {
  if (!ArrayBuffer.isView(that)) {
    throw new Error();
  }
  that.byteLength;
};
primordials.ArrayPrototypePushApply = (that, ...args) => that.push(...args);
primordials.MapPrototypeGetSize = (that) => that.size;
primordials.RegExpPrototypeSymbolReplace = (that, ...args) =>
  RegExp.prototype[Symbol.replace].call(that, ...args);
primordials.SafeArrayIterator = class SafeArrayIterator {
  constructor(array) {
    this.array = [...array];
    this.index = 0;
  }

  next() {
    if (this.index < this.array.length) {
      return { value: this.array[this.index++], done: false };
    } else {
      return { done: true };
    }
  }

  [Symbol.iterator]() {
    return this;
  }
};
primordials.SafeMap = Map;
primordials.SafeMapIterator = class SafeMapIterator {
  get [Symbol.toStringTag]() {
    return "Map Iterator";
  }
  constructor(map) {
    this.map = map;
    this.keys = Array.from(map.keys());
    this.index = 0;
  }

  next() {
    if (this.index < this.keys.length) {
      const key = this.keys[this.index];
      const value = this.map.get(key);
      this.index++;
      return { value: [key, value], done: false };
    } else {
      return { done: true };
    }
  }

  [Symbol.iterator]() {
    return this;
  }
};
primordials.SafeRegExp = RegExp;
primordials.SafeSet = Set;
primordials.SafeSetIterator = class SafeSetIterator {
  get [Symbol.toStringTag]() {
    return "Set Iterator";
  }
  constructor(set) {
    this.set = set;
    this.values = Array.from(set);
    this.index = 0;
  }

  next() {
    if (this.index < this.values.length) {
      const value = this.values[this.index];
      this.index++;
      return { value, done: false };
    } else {
      return { done: true };
    }
  }

  [Symbol.iterator]() {
    return this;
  }
};
primordials.SafeStringIterator = class SafeStringIterator {
  get [Symbol.toStringTag]() {
    return "String Iterator";
  }
  constructor(str) {
    this.str = str;
    this.index = 0;
  }

  next() {
    if (this.index < this.str.length) {
      const char = this.str[this.index];
      this.index++;
      return { value: char, done: false };
    } else {
      return { done: true };
    }
  }

  [Symbol.iterator]() {
    return this;
  }
};
primordials.SetPrototypeGetSize = (that) => that.size;
primordials.SymbolPrototypeGetDescription = (that) => that.description;
primordials.TypedArrayPrototypeGetByteLength = (that) => that.byteLength;
primordials.TypedArrayPrototypeGetLength = (that) => that.length;
primordials.TypedArrayPrototypeGetSymbolToStringTag = (that) => {
  if (ArrayBuffer.isView(that)) {
    return that[Symbol.toStringTag];
  }
};
primordials.ObjectPrototype = Object.prototype;
primordials.ObjectPrototypeIsPrototypeOf = (that, ...args) =>
  Object.prototype.isPrototypeOf.call(that, ...args);
primordials.ObjectPrototypePropertyIsEnumerable = (that, ...args) =>
  Object.prototype.propertyIsEnumerable.call(that, ...args);
primordials.ObjectPrototypeToString = (that, ...args) =>
  Object.prototype.toString.call(that, ...args);
primordials.ObjectAssign = (...args) => Object.assign(...args);
primordials.ObjectGetOwnPropertyDescriptor = (...args) =>
  Object.getOwnPropertyDescriptor(...args);
primordials.ObjectGetOwnPropertyNames = (...args) =>
  Object.getOwnPropertyNames(...args);
primordials.ObjectGetOwnPropertySymbols = (...args) =>
  Object.getOwnPropertySymbols(...args);
primordials.ObjectHasOwn = (...args) => Object.hasOwn(...args);
primordials.ObjectIs = (...args) => Object.is(...args);
primordials.ObjectCreate = (...args) => Object.create(...args);
primordials.ObjectDefineProperty = (...args) => Object.defineProperty(...args);
primordials.ObjectFreeze = (...args) => Object.freeze(...args);
primordials.ObjectGetPrototypeOf = (...args) => Object.getPrototypeOf(...args);
primordials.ObjectSetPrototypeOf = (...args) => Object.setPrototypeOf(...args);
primordials.ObjectKeys = (...args) => Object.keys(...args);
primordials.ObjectFromEntries = (...args) => Object.fromEntries(...args);
primordials.ObjectValues = (...args) => Object.values(...args);
primordials.FunctionPrototypeBind = (that, ...args) =>
  Function.prototype.bind.call(that, ...args);
primordials.FunctionPrototypeCall = (that, ...args) =>
  Function.prototype.call.call(that, ...args);
primordials.FunctionPrototypeToString = (that, ...args) =>
  Function.prototype.toString.call(that, ...args);
primordials.Array = Array;
primordials.ArrayPrototypeFill = (that, ...args) =>
  Array.prototype.fill.call(that, ...args);
primordials.ArrayPrototypeFind = (that, ...args) =>
  Array.prototype.find.call(that, ...args);
primordials.ArrayPrototypePop = (that, ...args) =>
  Array.prototype.pop.call(that, ...args);
primordials.ArrayPrototypePush = (that, ...args) =>
  Array.prototype.push.call(that, ...args);
primordials.ArrayPrototypeShift = (that, ...args) =>
  Array.prototype.shift.call(that, ...args);
primordials.ArrayPrototypeUnshift = (that, ...args) =>
  Array.prototype.unshift.call(that, ...args);
primordials.ArrayPrototypeSlice = (that, ...args) =>
  Array.prototype.slice.call(that, ...args);
primordials.ArrayPrototypeSort = (that, ...args) =>
  Array.prototype.sort.call(that, ...args);
primordials.ArrayPrototypeSplice = (that, ...args) =>
  Array.prototype.splice.call(that, ...args);
primordials.ArrayPrototypeIncludes = (that, ...args) =>
  Array.prototype.includes.call(that, ...args);
primordials.ArrayPrototypeJoin = (that, ...args) =>
  Array.prototype.join.call(that, ...args);
primordials.ArrayPrototypeForEach = (that, ...args) =>
  Array.prototype.forEach.call(that, ...args);
primordials.ArrayPrototypeFilter = (that, ...args) =>
  Array.prototype.filter.call(that, ...args);
primordials.ArrayPrototypeMap = (that, ...args) =>
  Array.prototype.map.call(that, ...args);
primordials.ArrayPrototypeReduce = (that, ...args) =>
  Array.prototype.reduce.call(that, ...args);
primordials.ArrayIsArray = (...args) => Array.isArray(...args);
primordials.Number = Number;
primordials.NumberPrototypeToString = (that, ...args) =>
  Number.prototype.toString.call(that, ...args);
primordials.NumberPrototypeValueOf = (that, ...args) =>
  Number.prototype.valueOf.call(that, ...args);
primordials.NumberIsInteger = (...args) => Number.isInteger(...args);
primordials.NumberParseInt = (...args) => Number.parseInt(...args);
primordials.Boolean = Boolean;
primordials.BooleanPrototypeValueOf = (that, ...args) =>
  Boolean.prototype.valueOf.call(that, ...args);
primordials.String = String;
primordials.StringPrototypeCharCodeAt = (that, ...args) =>
  String.prototype.charCodeAt.call(that, ...args);
primordials.StringPrototypeCodePointAt = (that, ...args) =>
  String.prototype.codePointAt.call(that, ...args);
primordials.StringPrototypeEndsWith = (that, ...args) =>
  String.prototype.endsWith.call(that, ...args);
primordials.StringPrototypeIncludes = (that, ...args) =>
  String.prototype.includes.call(that, ...args);
primordials.StringPrototypeIndexOf = (that, ...args) =>
  String.prototype.indexOf.call(that, ...args);
primordials.StringPrototypeLastIndexOf = (that, ...args) =>
  String.prototype.lastIndexOf.call(that, ...args);
primordials.StringPrototypeMatch = (that, ...args) =>
  String.prototype.match.call(that, ...args);
primordials.StringPrototypeNormalize = (that, ...args) =>
  String.prototype.normalize.call(that, ...args);
primordials.StringPrototypePadEnd = (that, ...args) =>
  String.prototype.padEnd.call(that, ...args);
primordials.StringPrototypePadStart = (that, ...args) =>
  String.prototype.padStart.call(that, ...args);
primordials.StringPrototypeRepeat = (that, ...args) =>
  String.prototype.repeat.call(that, ...args);
primordials.StringPrototypeReplace = (that, ...args) =>
  String.prototype.replace.call(that, ...args);
primordials.StringPrototypeReplaceAll = (that, ...args) =>
  String.prototype.replaceAll.call(that, ...args);
primordials.StringPrototypeSlice = (that, ...args) =>
  String.prototype.slice.call(that, ...args);
primordials.StringPrototypeSplit = (that, ...args) =>
  String.prototype.split.call(that, ...args);
primordials.StringPrototypeStartsWith = (that, ...args) =>
  String.prototype.startsWith.call(that, ...args);
primordials.StringPrototypeTrim = (that, ...args) =>
  String.prototype.trim.call(that, ...args);
primordials.StringPrototypeToLowerCase = (that, ...args) =>
  String.prototype.toLowerCase.call(that, ...args);
primordials.StringPrototypeValueOf = (that, ...args) =>
  String.prototype.valueOf.call(that, ...args);
primordials.Symbol = Symbol;
primordials.SymbolPrototypeToString = (that, ...args) =>
  Symbol.prototype.toString.call(that, ...args);
primordials.SymbolPrototypeValueOf = (that, ...args) =>
  Symbol.prototype.valueOf.call(that, ...args);
primordials.SymbolFor = (...args) => Symbol.for(...args);
primordials.SymbolHasInstance = Symbol.hasInstance;
primordials.SymbolIterator = Symbol.iterator;
primordials.SymbolToStringTag = Symbol.toStringTag;
primordials.DatePrototype = Date.prototype;
primordials.DatePrototypeToISOString = (that, ...args) =>
  Date.prototype.toISOString.call(that, ...args);
primordials.DatePrototypeGetTime = (that, ...args) =>
  Date.prototype.getTime.call(that, ...args);
primordials.DateNow = (...args) => Date.now(...args);
primordials.RegExpPrototypeExec = (that, ...args) =>
  RegExp.prototype.exec.call(that, ...args);
primordials.RegExpPrototypeToString = (that, ...args) =>
  RegExp.prototype.toString.call(that, ...args);
primordials.RegExpPrototypeTest = (that, ...args) =>
  RegExp.prototype.test.call(that, ...args);
primordials.Error = Error;
primordials.ErrorPrototype = Error.prototype;
primordials.ErrorPrototypeToString = (that, ...args) =>
  Error.prototype.toString.call(that, ...args);
primordials.ErrorCaptureStackTrace = (...args) =>
  Error.captureStackTrace(...args);
primordials.AggregateErrorPrototype = AggregateError.prototype;
primordials.MathAbs = (...args) => Math.abs(...args);
primordials.MathFloor = (...args) => Math.floor(...args);
primordials.MathMax = (...args) => Math.max(...args);
primordials.MathMin = (...args) => Math.min(...args);
primordials.MathRound = (...args) => Math.round(...args);
primordials.MathSqrt = (...args) => Math.sqrt(...args);
primordials.ArrayBufferIsView = (...args) => ArrayBuffer.isView(...args);
primordials.Uint8Array = Uint8Array;
primordials.MapPrototype = Map.prototype;
primordials.MapPrototypeGet = (that, ...args) =>
  Map.prototype.get.call(that, ...args);
primordials.MapPrototypeSet = (that, ...args) =>
  Map.prototype.set.call(that, ...args);
primordials.MapPrototypeHas = (that, ...args) =>
  Map.prototype.has.call(that, ...args);
primordials.MapPrototypeDelete = (that, ...args) =>
  Map.prototype.delete.call(that, ...args);
primordials.MapPrototypeEntries = (that, ...args) =>
  Map.prototype.entries.call(that, ...args);
primordials.MapPrototypeForEach = (that, ...args) =>
  Map.prototype.forEach.call(that, ...args);
primordials.BigIntPrototypeValueOf = (that, ...args) =>
  BigInt.prototype.valueOf.call(that, ...args);
primordials.SetPrototype = Set.prototype;
primordials.SetPrototypeHas = (that, ...args) =>
  Set.prototype.has.call(that, ...args);
primordials.SetPrototypeAdd = (that, ...args) =>
  Set.prototype.add.call(that, ...args);
primordials.SetPrototypeValues = (that, ...args) =>
  Set.prototype.values.call(that, ...args);
primordials.WeakMapPrototypeHas = (that, ...args) =>
  WeakMap.prototype.has.call(that, ...args);
primordials.WeakSetPrototypeHas = (that, ...args) =>
  WeakSet.prototype.has.call(that, ...args);
primordials.Proxy = Proxy;
primordials.ReflectGet = (...args) => Reflect.get(...args);
primordials.ReflectGetOwnPropertyDescriptor = (...args) =>
  Reflect.getOwnPropertyDescriptor(...args);
primordials.ReflectGetPrototypeOf = (...args) =>
  Reflect.getPrototypeOf(...args);
primordials.ReflectHas = (...args) => Reflect.has(...args);
primordials.ReflectOwnKeys = (...args) => Reflect.ownKeys(...args);

const ops = {
  op_get_non_index_property_names: getOwnNonIndexProperties,
  op_get_constructor_name(v) {
    return Object.prototype.toString.call(v).slice(8, -1);
  },
};

globalThis.Deno = {
  core: {
    ops,
    getProxyDetails() {
      return null;
    },
  },
};

globalThis.__bootstrap = {
  internals,
  primordials,
};
