// Test file for Intl.NumberFormat percent style support

// Test 1: Basic percent formatting with German locale (space before %)
console.log("Test 1: German locale percent");
const de_result = (100n).toLocaleString('de-DE', {style: 'percent'});
console.log(`  Result: ${de_result}`);
console.log(`  Expected: includes ' %' symbol`);
console.log(`  Pass: ${de_result.includes('%')}`);

// Test 2: Percent formatting with English locale (no space)
console.log("\nTest 2: English locale percent");
const en_result = (100n).toLocaleString('en-US', {style: 'percent'});
console.log(`  Result: ${en_result}`);
console.log(`  Expected: includes '%' symbol`);
console.log(`  Pass: ${en_result.includes('%')}`);

// Test 3: Percent with Number (not just BigInt)
console.log("\nTest 3: Number value with percent");
const num_result = (100).toLocaleString('en-US', {style: 'percent'});
console.log(`  Result: ${num_result}`);
console.log(`  Expected: includes '%' symbol`);
console.log(`  Pass: ${num_result.includes('%')}`);

// Test 4: Verify multiplication by 100
console.log("\nTest 4: Multiplication verification");
const test_val = 1n;
const test_result = test_val.toLocaleString('en-US', {style: 'percent'});
console.log(`  Input: 1n`);
console.log(`  Result: ${test_result}`);
console.log(`  Expected: 100%`);
console.log(`  Pass: ${test_result.includes('100')}`);

// Test 5: With significant digits
console.log("\nTest 5: With maximumSignificantDigits");
const sig_result = (88776655n).toLocaleString('de-DE', {style: 'percent', maximumSignificantDigits: 4});
console.log(`  Input: 88776655n`);
console.log(`  Result: ${sig_result}`);
console.log(`  Expected: formatted with 4 significant digits and '%'`);
console.log(`  Pass: ${sig_result.includes('%')}`);

// Test 6: Compare decimal vs percent
console.log("\nTest 6: Decimal vs Percent styling");
const decimal_val = (50n).toLocaleString('de-DE');
const percent_val = (50n).toLocaleString('de-DE', {style: 'percent'});
console.log(`  Decimal: ${decimal_val}`);
console.log(`  Percent: ${percent_val}`);
console.log(`  Percent has '%': ${percent_val.includes('%')}`);
console.log(`  Decimal no '%': ${!decimal_val.includes('%')}`);

// Test 7: Different European locales
console.log("\nTest 7: Multiple European locales with space");
const locales = ['de-DE', 'fr-FR', 'es-ES', 'it-IT', 'pt-PT'];
for (const locale of locales) {
    const result = (50n).toLocaleString(locale, {style: 'percent'});
    console.log(`  ${locale}: ${result} (has space): ${result.includes(' %')}`);
}

console.log("\nAll tests completed!");
