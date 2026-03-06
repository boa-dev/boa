const kIterationCount = 100;
const locales = ["en", "de", "ja", "ar"];

function main() {
    for (let i = 0; i < kIterationCount; i++) {
        for (const locale of locales) {
            new Intl.NumberFormat(locale);
        }
    }
}