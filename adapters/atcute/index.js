const { readFileSync } = require('fs');
const { Buffer } = require('buffer');
const { decode } = require('@atcute/cbor');

function generateSemanticFingerprint(data) {
    if (data === null || data === undefined) return "unknown:null";
    if (typeof data === 'boolean') return `bool:${data}`;
    if (typeof data === 'number') {
        if (Number.isInteger(data)) {
            return `int:${data}`;
        } else {
            const buf = Buffer.alloc(8);
            buf.writeDoubleBE(data, 0);
            return `float:0x${buf.toString('hex')}`;
        }
    }
    if (typeof data === 'string') return `str:${data}`;
    if (Buffer.isBuffer(data) || data instanceof Uint8Array) {
        const buf = Buffer.isBuffer(data) ? data : Buffer.from(data);
        return `bytes:${buf.toString('base64')}`;
    }
    if (Array.isArray(data)) {
        const parts = data.map(item => generateSemanticFingerprint(item));
        return `[${parts.join(',')}]`;
    }
    if (typeof data === 'object') {
        const keys = Object.keys(data).sort();
        const parts = keys.map(k => `[${generateSemanticFingerprint(k)},${generateSemanticFingerprint(data[k])}]`);
        return `[${parts.join(',')}]`;
    }
    return `unknown:${typeof data}`;
}

function main() {
    try {
        const inputBytes = readFileSync(0); 
        if (inputBytes.length === 0) {
            console.log(JSON.stringify({ status: "reject", version: "0.1.x", fingerprint: null, error_reason: "empty input" }));
            return;
        }

        // Attempt decoding using the real atcute parser
        // readFileSync returns a Node Buffer, which is a Uint8Array, making it compatible with atcute
        const parsedData = decode(inputBytes);
        
        const fingerprint = generateSemanticFingerprint(parsedData);
        console.log(JSON.stringify({ status: "accept", version: "0.1.x", fingerprint: fingerprint, error_reason: null }));
        
    } catch (error) {
        // Catch framing violations and map key violations
        const msg = error instanceof Error ? error.message : String(error);
        console.log(JSON.stringify({ status: "reject", version: "0.1.x", fingerprint: null, error_reason: msg }));
    }
}

main();