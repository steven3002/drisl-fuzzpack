import struct
import base64

def generate_semantic_fingerprint(data):
    if isinstance(data, bool):
        return f"bool:{str(data).lower()}"
    elif isinstance(data, int):
        return f"int:{data}"
    elif isinstance(data, float):
        hex_val = struct.pack('>d', data).hex()
        return f"float:0x{hex_val}"
    elif isinstance(data, str):
        return f"str:{data}"
    elif isinstance(data, bytes):
        return f"bytes:{base64.b64encode(data).decode('utf-8')}"
    elif isinstance(data, list):
        parts = [generate_semantic_fingerprint(item) for item in data]
        return f"[{','.join(parts)}]"
    elif isinstance(data, dict):
        sorted_keys = sorted(data.keys())
        parts = [f"[{generate_semantic_fingerprint(k)},{generate_semantic_fingerprint(data[k])}]" for k in sorted_keys]
        return f"[{','.join(parts)}]"
    else:
        return f"unknown:{type(data).__name__}"

data = {
    "z_string": "hello",
    "a_float": 3.14159,
    "an_int": 42,
    "a_bool": True,
    "nested_map": {"c": "c", "b": "b", "a": "a"},
    "an_array": [1, 2.5, False],
    "some_bytes": b'\xDE\xAD\xBE\xEF'
}

print(generate_semantic_fingerprint(data))