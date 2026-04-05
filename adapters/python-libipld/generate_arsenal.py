import sys
import os


sys.setrecursionlimit(5000)
sys.path.append('venv/lib/python3.12/site-packages')
import libipld

# The Number Crusher
crusher = {
    "safe_max": 9007199254740991,        # 2^53 - 1
    "unsafe_max": 9007199254740992,      # 2^53 (JS loses precision here)
    "unsafe_plus_one": 9007199254740993, # 2^53 + 1
    "u64_max": 18446744073709551615,     # 2^64 - 1
    "i64_min": -9223372036854775808      # -2^63
}

# The Stack Smasher (1000 nested arrays)
def build_smasher(depth):
    if depth == 0:
        return ["core"]
    return [build_smasher(depth - 1)]

smasher = build_smasher(1000)

# The String Bomber
bomber = {
    "zalgo": "H̷̼͝Ẽ̵̙ L̵͈̚L̶͍͗O̷̠͋",
    "rtl_override": "\u202EThis text is reversed\u202C",
    "zero_width": "a\u200Db\u200Cc\u200Dd",
    "emoji_modifiers": "👨‍👩‍👧‍👦🏳️‍🌈"
}

# Ensure directory exists
os.makedirs("../../corpus/seeds", exist_ok=True)

# Write them to disk
with open("../../corpus/seeds/98_number_crusher.cbor", "wb") as f:
    f.write(libipld.encode_dag_cbor(crusher))

with open("../../corpus/seeds/97_stack_smasher.cbor", "wb") as f:
    f.write(libipld.encode_dag_cbor(smasher))

with open("../../corpus/seeds/96_string_bomber.cbor", "wb") as f:
    f.write(libipld.encode_dag_cbor(bomber))

print("✅ Number Crusher Generated (98_number_crusher.cbor)")
print("✅ Stack Smasher Generated (97_stack_smasher.cbor)")
print("✅ String Bomber Generated (96_string_bomber.cbor)")
