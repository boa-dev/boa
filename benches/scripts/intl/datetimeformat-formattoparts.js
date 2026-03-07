const kIterationCount = 1000;

function main() {
    const dtf = new Intl.DateTimeFormat('en');
    const date = new Date();
    
    for (let i = 0; i < kIterationCount; i++) {
        dtf.formatToParts(date);
    }
}