function decimalToHexString(n) {
    var hex = "0123456789ABCDEF";
    n >>>= 0;
    var s = "";
    while (n) {
        s = hex[n & 0xf] + s;
        n >>>= 4;
    }
    while (s.length < 4) {
        s = "0" + s;
    }
    return s;
}

function decimalToPercentHexString(n) {
    var hex = "0123456789ABCDEF";
    return "%" + hex[(n >> 4) & 0xf] + hex[n & 0xf];
}

var errorCount = 0;
var count = 0;
var indexP;
var indexO = 0;

for (var indexB = 0xE0; indexB <= 0xEF; indexB++) {
    count++;
    var hexB = decimalToPercentHexString(indexB);
    var result = true;
    for (var indexC = 0xC0; indexC <= 0xFF; indexC++) {
        var hexC = decimalToPercentHexString(indexC);
        try {
            //   console.log(hexB + "%A0" + hexC)
            decodeURI(hexB + "%A0" + hexC);
        } catch (e) {
            if ((e instanceof URIError) === true) continue;
        }
        result = false;
    }
    if (result !== true) {
        if (indexO === 0) {
            indexO = indexB;
        } else {
            if ((indexB - indexP) !== 1) {
                if ((indexP - indexO) !== 0) {
                    var hexP = decimalToHexString(indexP);
                    var hexO = decimalToHexString(indexO);
                    $ERROR('#' + hexO + '-' + hexP + ' ');
                }
                else {
                    var hexP = decimalToHexString(indexP);
                    $ERROR('#' + hexP + ' ');
                }
                indexO = indexB;
            }
        }
        indexP = indexB;
        errorCount++;
    }
}

if (errorCount > 0) {
    if ((indexP - indexO) !== 0) {
        var hexP = decimalToHexString(indexP);
        var hexO = decimalToHexString(indexO);
        console.log('Error', '#' + hexO + '-' + hexP + ' ')
        //  $ERROR('#' + hexO + '-' + hexP + ' ');
    } else {
        var hexP = decimalToHexString(indexP);
        console.log('Error', '#' + hexP + ' ');
    }
    console.log('Total error: ' + errorCount + ' bad Unicode character in ' + count + ' ');
}
//console.log('decode', decodeURI('%E0%A0%D5'))
