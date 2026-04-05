import os


os.makedirs("../corpus/seeds", exist_ok=True)

# The Hex Arsenal: Precisely crafted byte payloads
payloads = {
    # --- CID AND TAG CASES ---
    # Valid Tag 42 + 39-byte CID
    "10_valid_tag42_cid.cbor": "d82a58270001711220fbaef0841804153cf68a18357a7ccba1e3f81e3aab806b77cd5d5ccfb174b1cb",
    # Tag 1 (Epoch Datetime) - Forbidden in DRISL
    "11_forbidden_tag.cbor": "c101",
    # Tag 42 but applied to a String instead of Bytes
    "12_invalid_cid_payload.cbor": "d82a6474657374", 

    # --- FRAMING CASES ---
    # Two empty maps back-to-back (concatenated objects)
    "20_concatenated_objects.cbor": "a0a0",
    # A map declared to hold 1 key/value pair, but it abruptly ends
    "21_truncation.cbor": "a16161",

    # --- FLOAT CASES ---
    # Standard 64-bit float (1.0)
    "30_valid_float64.cbor": "fb3ff0000000000000",
    # Negative Zero (-0.0) - Tricky in strict ATProto
    "31_negative_zero.cbor": "fb8000000000000000",
    # NaN (Not a Number) - Forbidden in DRISL
    "32_nan.cbor": "fb7ff8000000000000",
    # Infinity - Forbidden in DRISL
    "33_infinity.cbor": "fb7ff0000000000000",

    # --- MAP CASES ---
    # {"a": 1, "a": 2} - Duplicate keys
    "40_duplicate_keys.cbor": "a2616101616102",
    # {"b": 1, "a": 2} - Out-of-order keys (DRISL requires strict byte-sorting)
    "41_out_of_order_keys.cbor": "a2616201616102",

    # --- RESOURCE PRESSURE ---
    # An array header claiming to be 18,446,744,073,709,551,615 items long.
    # Causes memory-allocation panics (OOM) in naive parsers.
    "50_large_declared_length.cbor": "9bffffffffffffffff",
    
    # --- INDEFINITE LENGTHS (Strictly Forbidden in DRISL) ---
    "60_indefinite_array.cbor": "9f01ff",
    "61_indefinite_map.cbor": "bf616101ff",
    "62_indefinite_string.cbor": "7f6161ff"
}


for filename, hex_str in payloads.items():
    filepath = f"../corpus/seeds/{filename}"
    with open(filepath, "wb") as f:
        f.write(bytes.fromhex(hex_str))
        print(f"✅ Generated: {filename}")

