import os

def write_seed(filename, hex_string):
    with open(filename, "wb") as f:
        f.write(bytes.fromhex(hex_string.replace(" ", "")))
    print(f"Created: {filename}")

os.makedirs("seeds", exist_ok=True)

# 1. Float 16 (ATProto Exception)
# Map: {"a": 1.5}. Standard DRISL might allow this, but ATProto models strictly forbid 16-bit floats.
# A1 (map 1) | 61 61 (string "a") | F9 3E 00 (Float16: 1.5)
write_seed("seeds/01_float16_atproto_exception.cbor", "A1 61 61 F9 3E 00")

# 2. Framing Violation (Trailing Garbage)
# Valid map {"a": 1} followed by two illegal garbage bytes.
# A1 61 61 01 (Valid) | FF FF (Garbage)
write_seed("seeds/02_trailing_garbage.cbor", "A1 61 61 01 FF FF")

# 3. Map Violation (Non-String Key)
# DRISL specifically outlaws map keys that aren't strings.
# A1 (map 1) | 01 (int 1) | 02 (int 2) -> {1: 2}
write_seed("seeds/03_non_string_key.cbor", "A1 01 02")

# 4. CID Tag 42 Violation
# Tag 42 (D8 2A) requires a byte string payload starting with 0x00. 
# This payload gives it an integer (01) instead.
write_seed("seeds/04_malformed_tag42.cbor", "D8 2A 01")

print("Seed corpus successfully generated.")