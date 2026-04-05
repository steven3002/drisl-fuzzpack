import sys
import json
import traceback
import struct
import base64
import libipld



TARGET_VERSION = "0.1.1"

def generate_semantic_fingerprint(data):
    # Check bool first, because in Python, bool is a subclass of int!
    if isinstance(data, bool):
        return f"bool:{str(data).lower()}"
    elif isinstance(data, int):
        return f"int:{data}"
    elif isinstance(data, float):
        # Pack the float as an 8-byte double ('>d' for big-endian) and convert to hex
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
        # CRITICAL: Sort keys alphabetically to guarantee determinism
        sorted_keys = sorted(data.keys())
        parts = []
        for k in sorted_keys:
            key_str = generate_semantic_fingerprint(k)
            val_str = generate_semantic_fingerprint(data[k])
            parts.append(f"[{key_str},{val_str}]")
        return f"[{','.join(parts)}]"
    else:
        return f"unknown:{type(data).__name__}"

def print_result(status, fingerprint=None, error_reason=None):
    result = {
        "status": status,
        "version": TARGET_VERSION,
        "fingerprint": fingerprint,
        "error_reason": error_reason
    }
    print(json.dumps(result))
    sys.exit(0)

def main():
    try:
        input_bytes = sys.stdin.buffer.read()
        
        if not input_bytes:
            print_result("reject", error_reason="empty input")
            return

      
        # --- STUB LOGIC  ---
       # Attempt decoding using the real libipld parser
        try:
            parsed_data = libipld.decode_dag_cbor(input_bytes)
            fingerprint = generate_semantic_fingerprint(parsed_data)
            print_result("accept", fingerprint=fingerprint)
        except Exception as e:
            # Catch ALL exceptions so we never silently EOF
            print_result("reject", error_reason=str(e))

    except Exception as e:
        # Catch absolutely everything
        print_result("crash", error_reason=f"panic: {str(e)}")

if __name__ == "__main__":
    main()