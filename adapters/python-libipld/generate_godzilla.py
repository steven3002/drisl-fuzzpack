import sys
sys.path.append('venv/lib/python3.12/site-packages')
import libipld

godzilla = {
    "t": "app.bsky.feed.post",
    "text": "Fuzzing DRISL 💥 \x00 \xFF",
    "createdAt": "2026-04-05T12:00:00Z",
    "facets": [
        {
            "features": [{"$type": "app.bsky.richtext.facet#mention", "did": "did:plc:12345"}],
            "index": {"byteStart": 0, "byteEnd": 65535}
        }
    ],
    "embed": {
        "$type": "app.bsky.embed.images",
        "images": [
            {"alt": "A" * 500, "image": b"\xDE\xAD\xBE\xEF" * 10}
        ]
    },
    "reply": {
        "root": {"uri": "at://did:plc:x/app.bsky.feed.post/1", "cid": "bafyreidf"},
        "parent": {"uri": "at://did:plc:x/app.bsky.feed.post/2", "cid": "bafyreidf"}
    },
    "deeply_nested": [[[[[ 42, -1, 9223372036854775807, -9223372036854775808 ]]]]]
}

with open("../../corpus/seeds/99_godzilla.cbor", "wb") as f:
    f.write(libipld.encode_dag_cbor(godzilla))

print("Godzilla Seed Generated!")
