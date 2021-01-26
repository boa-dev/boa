const result = encodeURI('foo? bar = 12');
console.log('result', typeof result, decodeURI(result))