const { Buffer } = require('buffer');

function generateSemanticFingerprint(data) {
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
    if (typeof data === 'object' && data !== null) {
        const keys = Object.keys(data).sort();
        const parts = keys.map(k => `[${generateSemanticFingerprint(k)},${generateSemanticFingerprint(data[k])}]`);
        return `[${parts.join(',')}]`;
    }
    return `unknown:${typeof data}`;
}

const data = {
    z_string: "hello",
    a_float: 3.14159,
    an_int: 42,
    a_bool: true,
    nested_map: { c: "c", b: "b", a: "a" },
    an_array: [1, 2.5, false],
    some_bytes: Buffer.from([0xDE, 0xAD, 0xBE, 0xEF])
};

console.log(generateSemanticFingerprint(data));
